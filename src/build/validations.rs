use std::collections::{HashMap, HashSet};

use std::borrow::Cow;

use crate::build::LangBundle;
use crate::build::typed::{ElementKind, Id, Message, Ref, RefKind, VarType};
use crate::contract::{RefGraph, Resolver, render_elements};
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

/// A message's contract in the form [`check_refs`] takes: its argument names,
/// which of those are `(Bool)`, and its `(Element)` marker sequence.
struct Contract<'m> {
    vars: Vec<&'m str>,
    bool_vars: Vec<&'m str>,
    elements: Vec<(&'m str, RefKind)>,
}

impl<'m> Contract<'m> {
    fn of(msg: &'m Message) -> Self {
        Self {
            vars: msg.variables.iter().map(|v| v.id.as_str()).collect(),
            bool_vars: msg
                .variables
                .iter()
                .filter(|v| v.typ == VarType::Bool)
                .map(|v| v.id.as_str())
                .collect(),
            elements: msg
                .elements
                .iter()
                .map(|e| {
                    let kind = match e.kind {
                        ElementKind::Variable => RefKind::Variable,
                        ElementKind::Term => RefKind::Term,
                    };
                    (e.name.as_str(), kind)
                })
                .collect(),
        }
    }

    /// Check `other`'s pattern against this contract. The check is
    /// comment-independent: it uses `pattern_refs`, the raw `$variable`/`-term`
    /// references, never per-locale comment annotations. It is [`check_refs`],
    /// the same check `validate_ftl` applies to external translations at
    /// runtime, so the two can never drift apart.
    fn check(&self, other: &Message) -> Result<(), Vec<RefsIncompat>> {
        check_refs(
            &self.vars,
            &self.bool_vars,
            &self.elements,
            &other.pattern_refs,
            &other.selectors,
        )
    }
}

/// The result of comparing every locale against the default-locale contract.
#[derive(Debug)]
pub struct Analyzed {
    /// The message ids that will be generated: defined in the default locale,
    /// present in every other locale, and structurally compatible everywhere.
    pub common: HashSet<Id>,
    /// Human-readable warnings for messages that could *not* be generated.
    /// Each starts with the `file:line` to fix. Sorted for determinism.
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

        for msg in &default.messages {
            let id = &msg.id;
            let contract = Contract::of(msg);
            let mut missing_in: Vec<&str> = Vec::new();
            let mut compatible = true;

            for graph in &others {
                let lang = graph.lang.language_id.as_str();
                let Some(translation) = graph.by_id.get(&key(id)) else {
                    missing_in.push(lang);
                    continue;
                };
                if let Err(incompatibilities) = contract.check(translation) {
                    compatible = false;
                    let at = format!("{}:{}", msg.file, msg.line);
                    for incompatibility in incompatibilities {
                        warnings.push(format!(
                            "{}:{}: {id} is not generated — the '{lang}' translation {}",
                            translation.file,
                            translation.line,
                            describe(&incompatibility, &at),
                        ));
                    }
                }
            }

            if !missing_in.is_empty() {
                warnings.push(format!(
                    "{}:{}: {id} is not generated — it is missing from locale(s): {}",
                    msg.file,
                    msg.line,
                    missing_in.join(", "),
                ));
            } else if compatible {
                let unresolved = unresolved_references(msg, &graphs);
                if unresolved.is_empty() {
                    common.insert(id.clone());
                }
                warnings.extend(unresolved);
            }
        }

        warnings.extend(orphan_warnings(&others, default));
        warnings.sort();

        Self { common, warnings }
    }
}

/// Why a translation doesn't fit its contract, as the end of a sentence that
/// starts with "the 'fr' translation". `at` is where the contract is defined.
fn describe(incompatibility: &RefsIncompat, at: &str) -> String {
    match incompatibility {
        RefsIncompat::UnknownVariable { variable } => format!(
            "uses ${variable}, which the default locale's message ({at}) doesn't have, \
             so the accessor never passes it"
        ),
        RefsIncompat::ElementMismatch { expected, found } => format!(
            "has the (Element) markers [{}], but the default locale's message ({at}) has \
             [{}] in that order",
            render_elements(found).join(", "),
            render_elements(expected).join(", "),
        ),
        RefsIncompat::BoolSelectorMismatch { variable, found } => format!(
            "selects on the Boolean ${variable} with the keys [{}] — expected [true] and \
             [false]",
            found.join(", "),
        ),
        RefsIncompat::BoolReferenceOutsideSelector { variable } => format!(
            "uses the Boolean ${variable} outside a [true]/[false] selector, where it would \
             render as the text \"true\" or \"false\""
        ),
    }
}

/// Check, in every locale, that everything `msg`'s pattern references
/// resolves — the same check [`validate_ftl`](crate::validate_ftl) runs on
/// external translations. Returns one warning per locale with a problem,
/// starting with that locale's `file:line` and naming every problem found.
fn unresolved_references(msg: &Message, graphs: &[BuildGraph]) -> Vec<String> {
    // The accessor's arguments. `(Element)` variables aren't among them: they
    // are never passed.
    let allowed: Vec<&str> = msg.variables.iter().map(|v| v.id.as_str()).collect();
    let id = &msg.id;

    let mut warnings = Vec::new();
    for graph in graphs {
        let Some(translation) = graph.by_id.get(&key(id)) else {
            continue; // missing messages are reported by the caller
        };
        let mut resolver = Resolver::new(graph);
        resolver.check(
            &id.message,
            id.attribute.as_deref(),
            &translation.pattern_refs,
            &allowed,
        );
        let violations = resolver.into_violations();
        if violations.is_empty() {
            continue;
        }
        let violations: Vec<String> = violations.iter().map(ToString::to_string).collect();
        warnings.push(format!(
            "{}:{}: {id} is not generated — the '{}' translation would fail to format: {}",
            translation.file,
            translation.line,
            graph.lang.language_id,
            violations.join("; "),
        ));
    }
    warnings
}

/// Warn about messages that exist in non-default locales but are absent from
/// the default locale — they have no contract, so no accessor is generated.
fn orphan_warnings(others: &[&BuildGraph], default: &LangBundle) -> Vec<String> {
    let default_ids: HashSet<&Id> = default.messages.iter().map(|m| &m.id).collect();
    others
        .iter()
        .flat_map(|graph| &graph.lang.messages)
        .filter(|msg| !default_ids.contains(&msg.id))
        .map(|msg| {
            format!(
                "{}:{}: {} is not generated — it is missing from the default locale '{}'",
                msg.file, msg.line, msg.id, default.language_id,
            )
        })
        .collect()
}

/// Structural errors in default-locale Boolean contracts. These must fail the
/// build at every lint level: generated accessors would otherwise return
/// silently wrong output.
pub(crate) fn default_contract_errors(default: &LangBundle) -> Vec<String> {
    let mut errors = Vec::new();

    for msg in &default.messages {
        let Err(incompatibilities) = Contract::of(msg).check(msg) else {
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
