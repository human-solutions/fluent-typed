//! Lints over the parsed `.ftl` locales.
//!
//! fluent-typed reads argument types from message comments. A mistake in a
//! comment — a typo'd keyword, a misnamed variable, a comment detached from its
//! message — otherwise fails silently. These lints surface them, with the file
//! and line of every problem. See [`crate::LintLevel`] for how they are
//! reported.

use std::collections::{HashMap, HashSet};

use crate::build::LangBundle;
use crate::build::typed::{
    Annotation, ElementKind, Id, Message, Ref, RefKind, VarType, annotation,
};

/// Every lint diagnostic produced from the parsed locales.
pub struct Lints {
    /// L1/L2/L3 — comment mistakes in the default locale. Reported as warnings
    /// under [`crate::LintLevel::Warn`], as hard errors under
    /// [`crate::LintLevel::Strict`].
    pub mistakes: Vec<String>,
    /// L5 — type annotations in non-default locales, which never affect output.
    /// Always a warning, even in strict mode (it concerns translator files).
    pub ineffective: Vec<String>,
    /// Variables of generated messages that have no concrete type. Used only
    /// under [`crate::LintLevel::Strict`].
    pub untyped: Vec<String>,
}

/// Run every lint. `default` is the default-language bundle (whose comments are
/// the type contract); `common` is the set of message ids that get generated.
pub fn check(langs: &[LangBundle], default: &LangBundle, common: &HashSet<Id>) -> Lints {
    // Index every message's pattern references by message name once, so the
    // per-message comment check is a hash lookup rather than a rescan of all
    // messages — keeping the comment lints O(messages) rather than O(messages²).
    let mut refs_by_message: HashMap<&str, Vec<&Ref>> = HashMap::new();
    for m in &default.messages {
        refs_by_message
            .entry(m.id.message.as_str())
            .or_default()
            .extend(&m.pattern_refs);
    }

    let mut mistakes = Vec::new();
    for msg in &default.messages {
        mistakes.extend(comment_mistakes(msg, &refs_by_message));
        mistakes.extend(bool_selector_mistakes(msg));
    }
    mistakes.extend(detached_comments(default));
    mistakes.sort();

    let mut ineffective = Vec::new();
    for lang in langs {
        if lang.language_id != default.language_id {
            ineffective.extend(ineffective_annotations(lang));
        }
    }
    ineffective.sort();

    // Variables a message entry deliberately annotated `(Element)`. They are
    // intentional positional gaps, not "untyped by neglect", so the strict
    // untyped check must not flag them — including where an attribute reuses
    // the same name (attributes keep the variable as a runtime argument).
    let element_vars: HashSet<(&str, &str)> = default
        .messages
        .iter()
        .flat_map(|m| {
            m.elements
                .iter()
                .filter(|e| e.kind == ElementKind::Variable)
                .map(move |e| (m.id.message.as_str(), e.name.as_str()))
        })
        .collect();

    let mut untyped = Vec::new();
    for msg in &default.messages {
        if common.contains(&msg.id) {
            untyped.extend(untyped_variables(msg, &element_vars));
        }
    }
    untyped.sort();

    Lints {
        mistakes,
        ineffective,
        untyped,
    }
}

/// A `(Bool)` variable is encoded as the strings `"true"` and `"false"`.
/// Every selector it drives must therefore expose exactly those two keys.
fn bool_selector_mistakes(msg: &Message) -> Vec<String> {
    let bool_vars: HashSet<&str> = msg
        .variables
        .iter()
        .filter(|v| v.typ == VarType::Bool)
        .map(|v| v.id.as_str())
        .collect();

    msg.selectors
        .iter()
        .filter(|selector| bool_vars.contains(selector.variable.as_str()))
        .filter(|selector| !selector.has_bool_keys())
        .map(|selector| {
            format!(
                "{}:{}: Boolean selector ${} in {} has keys [{}] — expected \
                 [true] and [false]",
                msg.file,
                msg.line,
                selector.variable,
                msg.id,
                selector.keys.join(", "),
            )
        })
        .collect()
}

/// L1 (typo'd keyword) and L2 (annotation of a non-existent variable/term),
/// scanning one message's comment. `refs_by_message` maps each message name to
/// every pattern reference of its value *and* its attributes, so an annotation
/// pointing at an *attribute*'s variable is seen too.
fn comment_mistakes(msg: &Message, refs_by_message: &HashMap<&str, Vec<&Ref>>) -> Vec<String> {
    if msg.comment.is_empty() {
        return Vec::new();
    }
    let no_refs = Vec::new();
    let refs = refs_by_message
        .get(msg.id.message.as_str())
        .unwrap_or(&no_refs);

    let mut out = Vec::new();
    for (i, line) in msg.comment.iter().enumerate() {
        let Some(a) = annotation(line) else { continue };
        let at = format!("{}:{}", msg.file, msg.comment_line + i);
        if a.is_recognized() {
            if !target_exists(&a, refs) {
                out.push(format!(
                    "{at}: comment annotates {}{} but {} references no such {}",
                    a.sigil(),
                    a.name,
                    msg.id,
                    if a.is_term { "term" } else { "variable" },
                ));
            }
        } else if a.is_type_annotation() {
            // Shaped like a type annotation, keyword is a near-miss typo.
            out.push(format!(
                "{at}: unrecognized type annotation '({})' for {}{} — expected \
                 (String), (Number), (Bool) or (Element)",
                a.keyword,
                a.sigil(),
                a.name,
            ));
        }
        // Otherwise the parentheses are ordinary prose — not an annotation.
    }
    out
}

fn target_exists(a: &Annotation, refs: &[&Ref]) -> bool {
    let kind = if a.is_term {
        RefKind::Term
    } else {
        RefKind::Variable
    };
    refs.iter().any(|r| r.kind == kind && r.name == a.name)
}

/// L3 — a standalone comment in the default locale shaped like a type
/// annotation: a blank line has detached it from its message, so it is inert.
fn detached_comments(default: &LangBundle) -> Vec<String> {
    default
        .standalone_comments
        .iter()
        .filter_map(|c| {
            let a = annotation(&c.text)?;
            if !a.is_type_annotation() {
                return None;
            }
            Some(format!(
                "{}:{}: type annotation '{}{} ({})' is detached from its message \
                 by a blank line and has no effect",
                c.file,
                c.line,
                a.sigil(),
                a.name,
                a.keyword,
            ))
        })
        .collect()
}

/// L5 — type annotations in a non-default locale, which never affect output.
fn ineffective_annotations(lang: &LangBundle) -> Vec<String> {
    let report = |file: &str, line: usize, a: &Annotation| {
        format!(
            "{file}:{line}: type annotation '{}{} ({})' in non-default locale \
             '{}' has no effect — type comments only apply to the default locale",
            a.sigil(),
            a.name,
            a.keyword,
            lang.language_id,
        )
    };

    let mut out = Vec::new();
    for msg in &lang.messages {
        for (i, line) in msg.comment.iter().enumerate() {
            if let Some(a) = annotation(line).filter(|a| a.is_type_annotation()) {
                out.push(report(&msg.file, msg.comment_line + i, &a));
            }
        }
    }
    for c in &lang.standalone_comments {
        if let Some(a) = annotation(&c.text).filter(|a| a.is_type_annotation()) {
            out.push(report(&c.file, c.line, &a));
        }
    }
    out
}

/// Strict-only — every variable of a generated message must resolve to a
/// concrete type. An attribute's variables are typed via the parent message's
/// comment, so they are checked here too. Variables annotated `(Element)`
/// (listed in `element_vars` as `(message, variable)` pairs) are skipped.
fn untyped_variables(msg: &Message, element_vars: &HashSet<(&str, &str)>) -> Vec<String> {
    msg.variables
        .iter()
        .filter(|v| v.typ == VarType::Any)
        .filter(|v| !element_vars.contains(&(msg.id.message.as_str(), v.id.as_str())))
        .map(|v| {
            format!(
                "{}:{}: variable ${} in {} has no type — add a \
                 '# ${} (String)', '# ${} (Number)' or '# ${} (Bool)' comment",
                msg.file, msg.line, v.id, msg.id, v.id, v.id, v.id,
            )
        })
        .collect()
}
