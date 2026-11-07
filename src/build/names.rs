//! Checks that the Rust names generated from `.ftl` ids don't clash. Fluent
//! ids that differ only in `-`, `_` or case (`foo-bar`, `foo_bar`, `fooBar`)
//! map to the same Rust name, and without these checks the clash would only
//! show up as a compile error inside the generated file.

use std::collections::HashMap;

use super::r#gen::{StrExt, param_name};
use super::{LangBundle, Message};

/// The methods `L10nLanguage` has besides the message accessors.
const LANGUAGE_METHODS: &[&str] = &["new", "new_external"];

/// Type and trait names the generated module defines, imports or relies on
/// from the prelude. A structured message's struct can't take one of these
/// names: it would clash with, or shadow, the original.
const RESERVED_TYPES: &[&str] = &[
    "AsRef",
    "Box",
    "ContractViolation",
    "Default",
    "Deref",
    "Display",
    "ElementContract",
    "ElementGap",
    "FluentArgs",
    "FluentNumber",
    "FluentValue",
    "Fn",
    "FromStr",
    "Into",
    "Iter",
    "L10n",
    "L10nBundle",
    "L10nError",
    "L10nLanguage",
    "L10nLanguageVec",
    "LanguageIdentifier",
    "MessageContract",
    "Option",
    "Result",
    "Segment",
    "String",
    "Vec",
];

/// Every clash among the generated names of `messages` (their accessors,
/// structs and parameters) and of the `langs` (their `L10n` variants), in
/// source order. Each entry starts with the `file:line` (or folder) to fix.
pub(crate) fn name_collisions(
    prefix: &str,
    locales_folder: &str,
    langs: &[LangBundle],
    messages: &[&Message],
) -> Vec<String> {
    let mut errors = Vec::new();

    let mut variants: HashMap<String, &str> = HashMap::new();
    for lang in langs {
        let id = lang.language_id.as_str();
        let variant = id.rust_var_name();
        if let Some(first) = variants.get(&variant) {
            errors.push(format!(
                "{locales_folder}/{id}: the locale folders '{first}' and '{id}' both \
                 become `L10n::{variant}`. Remove or rename one."
            ));
        } else {
            variants.insert(variant, id);
        }
    }

    let mut methods: HashMap<String, &Message> = HashMap::new();
    let mut structs: HashMap<String, &Message> = HashMap::new();
    for &msg in messages {
        let at = location(msg);
        let id = &msg.id;

        let method = msg.accessor_name(prefix);
        if LANGUAGE_METHODS.contains(&method.as_str()) {
            errors.push(format!(
                "{at}: {id} would generate the method `{method}`, which `L10nLanguage` \
                 already has. Rename the message, or set a prefix with \
                 `BuildOptions::with_prefix`."
            ));
        } else if let Some(first) = methods.get(&method) {
            errors.push(format!(
                "{at}: {id} and {} ({}) both generate the method `{method}`. Rename one.",
                first.id,
                location(first),
            ));
        } else {
            methods.insert(method, msg);
        }

        if !msg.elements.is_empty() {
            let name = msg.struct_name();
            if RESERVED_TYPES.contains(&name.as_str()) {
                errors.push(format!(
                    "{at}: {id} would generate the struct `{name}`, a name the generated \
                     code already uses. Rename the message."
                ));
            } else if let Some(first) = structs.get(&name) {
                errors.push(format!(
                    "{at}: {id} and {} ({}) both generate the struct `{name}`. Rename one.",
                    first.id,
                    location(first),
                ));
            } else {
                structs.insert(name, msg);
            }
        }

        let mut params: HashMap<String, &str> = HashMap::new();
        for var in &msg.variables {
            let name = param_name(var);
            if let Some(first) = params.get(&name) {
                errors.push(format!(
                    "{at}: in {id}, ${first} and ${} both become the parameter `{name}`. \
                     Rename one.",
                    var.id,
                ));
            } else {
                params.insert(name, &var.id);
            }
        }
    }

    errors
}

fn location(msg: &Message) -> String {
    format!("{}:{}", msg.file, msg.line)
}
