use std::collections::{HashMap, HashSet};

use crate::build::LangBundle;
use crate::build::typed::{ElementKind, Id, Message, RefKind};

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
        // Each non-default locale, paired with its messages indexed by id, so
        // the per-message contract check below is a hash lookup rather than a
        // linear scan — keeping the whole analysis O(messages · locales).
        let others: Vec<(&LangBundle, HashMap<&Id, &Message>)> = langs
            .iter()
            .filter(|l| l.language_id != default.language_id)
            .map(|l| {
                let mut by_id: HashMap<&Id, &Message> = HashMap::new();
                for m in &l.messages {
                    // Keep the first occurrence, matching the previous
                    // `.find()` — significant when duplicate keys are allowed.
                    by_id.entry(&m.id).or_insert(m);
                }
                (l, by_id)
            })
            .collect();

        let mut common = HashSet::new();
        let mut warnings = Vec::new();

        for contract in &default.messages {
            let id = &contract.id;
            let mut missing_in: Vec<String> = Vec::new();
            let mut incompatible_in: Vec<String> = Vec::new();

            for (lang, by_id) in &others {
                match by_id.get(id) {
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
                    "{}:{}: {id} is not generated — incompatible variables or \
                     elements in locale(s): {}",
                    contract.file,
                    contract.line,
                    incompatible_in.join(", "),
                ));
            } else {
                common.insert(id.clone());
            }
        }

        warnings.extend(orphan_warnings(&others, default));
        warnings.sort();

        Self { common, warnings }
    }
}

/// Warn about messages that exist in non-default locales but are absent from
/// the default locale — they have no contract, so no accessor is generated.
fn orphan_warnings(
    others: &[(&LangBundle, HashMap<&Id, &Message>)],
    default: &LangBundle,
) -> Vec<String> {
    let default_ids: HashSet<&Id> = default.messages.iter().map(|m| &m.id).collect();
    let mut orphans: HashMap<Id, Vec<String>> = HashMap::new();

    for (lang, _) in others {
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
fn compatible(contract: &Message, other: &Message) -> bool {
    let args: HashSet<&str> = contract.variables.iter().map(|v| v.id.as_str()).collect();

    if contract.elements.is_empty() {
        // Plain message: every variable the other locale references must be a
        // known contract argument. Extra args would be unfilled at runtime.
        other
            .pattern_refs
            .iter()
            .filter(|r| r.kind == RefKind::Variable)
            .all(|r| args.contains(r.name.as_str()))
    } else {
        // Element message: the element markers must line up exactly so that
        // `msg_segments` splits the other locale's pattern into the same slots.
        let element_names: HashSet<&str> =
            contract.elements.iter().map(|e| e.name.as_str()).collect();

        let contract_elems: Vec<(&str, ElementKind)> = contract
            .elements
            .iter()
            .map(|e| (e.name.as_str(), e.kind))
            .collect();
        let other_elems: Vec<(&str, ElementKind)> = other
            .pattern_refs
            .iter()
            .filter(|r| element_names.contains(r.name.as_str()))
            .map(|r| (r.name.as_str(), element_kind(r.kind)))
            .collect();
        if contract_elems != other_elems {
            return false;
        }

        // Non-element variables must still be known contract arguments.
        other
            .pattern_refs
            .iter()
            .filter(|r| r.kind == RefKind::Variable && !element_names.contains(r.name.as_str()))
            .all(|r| args.contains(r.name.as_str()))
    }
}

fn element_kind(kind: RefKind) -> ElementKind {
    match kind {
        RefKind::Variable => ElementKind::Variable,
        RefKind::Term => ElementKind::Term,
    }
}
