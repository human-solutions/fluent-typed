use std::collections::{HashMap, HashSet};

use std::borrow::Cow;

use crate::build::LangBundle;
use crate::build::typed::{ElementKind, Id, Message, Ref, RefKind, VarType};
use crate::contract::{RefGraph, Resolver};
use crate::ftl_refs::{RefsIncompat, check_refs};

/// A locale's messages by `(message, attribute)`, which is also how the
/// reference check looks them up.
type ById<'b> = HashMap<(&'b str, Option<&'b str>), &'b Message>;

fn key(id: &Id) -> (&str, Option<&str>) {
    (id.message.as_str(), id.attribute.as_deref())
}

/// The build's [`RefGraph`] for one locale: the references collected while
/// parsing, so the check needs no second parse.
struct BuildGraph<'b> {
    lang: &'b LangBundle,
    by_id: ById<'b>,
}

impl<'b> BuildGraph<'b> {
    fn new(lang: &'b LangBundle) -> Self {
        let mut by_id: ById = HashMap::with_capacity(lang.messages.len());
        for m in &lang.messages {
            // Keep the first occurrence — significant when duplicate keys are
            // allowed.
            by_id.entry(key(&m.id)).or_insert(m);
        }
        Self { lang, by_id }
    }
}

impl RefGraph for BuildGraph<'_> {
    fn message_refs(&self, message: &str, attribute: Option<&str>) -> Option<Cow<'_, [Ref]>> {
        let m = self.by_id.get(&(message, attribute))?;
        Some(Cow::Borrowed(m.pattern_refs.as_slice()))
    }

    fn message_exists(&self, message: &str) -> bool {
        // Only asked when a reference doesn't resolve, so a scan is fine.
        self.lang.messages.iter().any(|m| m.id.message == message)
    }

    fn term_refs(&self, name: &str, attribute: Option<&str>) -> Option<Cow<'_, [Ref]>> {
        let refs = match attribute {
            Some(a) => self.lang.term_refs.get(&format!("{name}.{a}"))?,
            None => self.lang.term_refs.get(name)?,
        };
        Some(Cow::Borrowed(refs.as_slice()))
    }
}

/// The result of comparing every locale against the default-locale contract.
#[derive(Debug)]
pub struct Analyzed {
    /// The message ids that will be generated: defined in the default locale,
    /// present in every other locale, and structurally compatible everywhere.
    pub common: HashSet<Id>,
    /// Human-readable warnings for messages that could *not* be generated.
    /// Each names the `.ftl` file and line involved. Sorted for determinism.
    pub warnings: Vec<String>,
}

impl Analyzed {
    /// Analyze the locales against `default`, the default-language bundle.
    ///
    /// The default locale defines each message's contract (its variables and
    /// `(Element)` layout). A message is generated only when every other locale
    /// defines it too, with a structurally compatible pattern.
    pub fn from(langs: &[LangBundle], default: &LangBundle) -> Self {
        // Every locale with its messages indexed by id, so the per-message
        // checks below are hash lookups rather than linear scans — keeping the
        // whole analysis O(messages · locales).
        let graphs: Vec<BuildGraph> = langs.iter().map(BuildGraph::new).collect();
        let others: Vec<&BuildGraph> = graphs
            .iter()
            .filter(|g| g.lang.language_id != default.language_id)
            .collect();

        let mut common = HashSet::new();
        let mut warnings = Vec::new();

        for contract in &default.messages {
            let id = &contract.id;
            let mut missing_in: Vec<String> = Vec::new();
            let mut incompatible_in: Vec<String> = Vec::new();

            for graph in &others {
                let lang = graph.lang;
                match graph.by_id.get(&key(id)) {
                    None => missing_in.push(lang.language_id.clone()),
                    Some(msg) if !compatible(contract, msg) => {
                        incompatible_in
                            .push(format!("{} ({}:{})", lang.language_id, msg.file, msg.line));
                    }
                    Some(_) => {}
                }
            }

            if !missing_in.is_empty() {
                warnings.push(format!(
                    "{}:{}: {id} is not generated — missing from locale(s): {}",
                    contract.file,
                    contract.line,
                    missing_in.join(", "),
                ));
            } else if !incompatible_in.is_empty() {
                warnings.push(format!(
                    "{}:{}: {id} is not generated — incompatible variables, \
                     Boolean selectors or elements in locale(s): {}",
                    contract.file,
                    contract.line,
                    incompatible_in.join(", "),
                ));
            } else {
                let unresolved_in = unresolved_references(contract, &graphs);
                if unresolved_in.is_empty() {
                    common.insert(id.clone());
                } else {
                    warnings.push(format!(
                        "{}:{}: {id} is not generated — it would fail to format: {}",
                        contract.file,
                        contract.line,
                        unresolved_in.join("; "),
                    ));
                }
            }
        }

        warnings.extend(orphan_warnings(&others, default));
        warnings.sort();

        Self { common, warnings }
    }
}

/// Check, in every locale, that everything `contract`'s pattern references
/// resolves — the same check [`validate_ftl`](crate::validate_ftl) runs on
/// external translations. Returns one entry per locale with a problem, naming
/// the locale, the file and line of its message, and every problem found.
fn unresolved_references(contract: &Message, graphs: &[BuildGraph]) -> Vec<String> {
    // The accessor's arguments. `(Element)` variables aren't among them: they
    // are never passed.
    let allowed: Vec<&str> = contract.variables.iter().map(|v| v.id.as_str()).collect();
    let id = &contract.id;

    let mut problems = Vec::new();
    for graph in graphs {
        let Some(msg) = graph.by_id.get(&key(id)) else {
            continue; // missing messages are reported by the caller
        };
        let mut resolver = Resolver::new(graph);
        resolver.check(
            &id.message,
            id.attribute.as_deref(),
            &msg.pattern_refs,
            &allowed,
        );
        let violations = resolver.into_violations();
        if violations.is_empty() {
            continue;
        }
        let violations: Vec<String> = violations.iter().map(ToString::to_string).collect();
        problems.push(format!(
            "{} ({}:{}): {}",
            graph.lang.language_id,
            msg.file,
            msg.line,
            violations.join(", "),
        ));
    }
    problems
}

/// Warn about messages that exist in non-default locales but are absent from
/// the default locale — they have no contract, so no accessor is generated.
fn orphan_warnings(others: &[&BuildGraph], default: &LangBundle) -> Vec<String> {
    let default_ids: HashSet<&Id> = default.messages.iter().map(|m| &m.id).collect();
    let mut orphans: HashMap<Id, Vec<String>> = HashMap::new();

    for graph in others {
        let lang = graph.lang;
        for msg in &lang.messages {
            if !default_ids.contains(&msg.id) {
                orphans
                    .entry(msg.id.clone())
                    .or_default()
                    .push(lang.language_id.clone());
            }
        }
    }

    orphans
        .into_iter()
        .map(|(id, mut langs)| {
            langs.sort();
            format!(
                "{id} is not generated — present in locale(s) {} but missing from \
                 the default locale '{}'",
                langs.join(", "),
                default.language_id,
            )
        })
        .collect()
}

/// Whether `other` (a non-default locale's message) is structurally compatible
/// with the `contract` (the default locale's message of the same id).
///
/// The check is comment-independent: it uses `pattern_refs`, the raw
/// `$variable`/`-term` references, never the per-locale comment annotations.
/// It delegates to [`check_refs`], the same check `validate_ftl` applies to
/// external translations at runtime, so the two can never drift apart.
fn compatible(contract: &Message, other: &Message) -> bool {
    let vars: Vec<&str> = contract.variables.iter().map(|v| v.id.as_str()).collect();
    let bool_vars: Vec<&str> = contract
        .variables
        .iter()
        .filter(|v| v.typ == VarType::Bool)
        .map(|v| v.id.as_str())
        .collect();
    let elements: Vec<(&str, RefKind)> = contract
        .elements
        .iter()
        .map(|e| {
            let kind = match e.kind {
                ElementKind::Variable => RefKind::Variable,
                ElementKind::Term => RefKind::Term,
            };
            (e.name.as_str(), kind)
        })
        .collect();
    check_refs(
        &vars,
        &bool_vars,
        &elements,
        &other.pattern_refs,
        &other.selectors,
    )
    .is_ok()
}

/// Structural errors in default-locale Boolean contracts. These must fail the
/// build at every lint level: generated accessors would otherwise return
/// silently wrong output.
pub(crate) fn default_contract_errors(default: &LangBundle) -> Vec<String> {
    let mut errors = Vec::new();

    for msg in &default.messages {
        let vars: Vec<&str> = msg.variables.iter().map(|v| v.id.as_str()).collect();
        let bool_vars: Vec<&str> = msg
            .variables
            .iter()
            .filter(|v| v.typ == VarType::Bool)
            .map(|v| v.id.as_str())
            .collect();
        let elements: Vec<(&str, RefKind)> = msg
            .elements
            .iter()
            .map(|e| {
                let kind = match e.kind {
                    ElementKind::Variable => RefKind::Variable,
                    ElementKind::Term => RefKind::Term,
                };
                (e.name.as_str(), kind)
            })
            .collect();

        let Err(incompatibilities) = check_refs(
            &vars,
            &bool_vars,
            &elements,
            &msg.pattern_refs,
            &msg.selectors,
        ) else {
            continue;
        };

        for incompatibility in incompatibilities {
            match incompatibility {
                RefsIncompat::BoolSelectorMismatch { variable, found } => errors.push(format!(
                    "{}:{}: Boolean selector ${variable} in {} has keys [{}] — expected [true] and [false]",
                    msg.file,
                    msg.line,
                    msg.id,
                    found.join(", "),
                )),
                RefsIncompat::BoolReferenceOutsideSelector { variable } => errors.push(format!(
                    "{}:{}: Boolean variable ${variable} in {} is referenced outside a [true]/[false] selector",
                    msg.file, msg.line, msg.id,
                )),
                // A message checked against its own parsed contract cannot
                // have unknown variables or mismatched element markers.
                RefsIncompat::UnknownVariable { .. } | RefsIncompat::ElementMismatch { .. } => {}
            }
        }
    }

    errors.sort();
    errors
}
