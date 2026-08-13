//! Runtime validation of external `.ftl` translations.
//!
//! The build step compiles a [`MessageContract`] for every generated accessor
//! into the generated file (the `MESSAGE_CONTRACTS` static). [`validate_ftl`]
//! checks external `.ftl` bytes — a translation read from disk or downloaded
//! at runtime — against those contracts, using the same compatibility rules
//! the build applies to every locale. On success, every generated accessor is
//! safe to call on a bundle loaded from those bytes.

use std::collections::{HashMap, HashSet};
use std::fmt;

use fluent_syntax::{ast, parser};

use crate::error::L10nError;
use crate::ftl_refs::{Ref, RefKind, RefsIncompat, check_refs, find_refs, find_selectors};

/// The compiled contract of one generated message accessor: which message (or
/// attribute) it resolves, which arguments it fills, and — for structured
/// messages — the exact `(Element)` marker sequence it splits on.
///
/// Instances are emitted by the build step into the generated file's
/// `MESSAGE_CONTRACTS` static; there is no reason to construct one by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageContract {
    /// The Fluent message id.
    pub message: &'static str,
    /// The attribute of the message, for an attribute accessor.
    pub attribute: Option<&'static str>,
    /// The argument names the accessor fills (`$variable` names).
    pub vars: &'static [&'static str],
    /// Variables encoded from Rust `bool` as Fluent `"true"` / `"false"`.
    pub bool_vars: &'static [&'static str],
    /// The `(Element)` markers, in pattern order. Empty for plain messages.
    pub elements: &'static [ElementContract],
}

/// One `(Element)` marker of a structured message contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementContract {
    /// The variable or term name (without the `$` / `-` sigil).
    pub name: &'static str,
    /// `true` for a term element (`-name`), `false` for a variable element.
    pub is_term: bool,
}

/// One way an external `.ftl` failed to satisfy a [`MessageContract`].
///
/// Carried by [`L10nError::Validation`]. Each variant describes a mistake
/// that would otherwise surface later as a panic or formatting error in a
/// generated accessor.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContractViolation {
    /// A contract message is missing from the `.ftl` entirely.
    MissingMessage {
        /// The message id.
        message: String,
    },
    /// The message exists but has no value of its own, and the contract
    /// expects one (the message defines only attributes).
    MissingValue {
        /// The message id.
        message: String,
    },
    /// The message exists but lacks a required attribute.
    MissingAttribute {
        /// The message id.
        message: String,
        /// The missing attribute.
        attribute: String,
    },
    /// The pattern references a variable that is not a contract argument.
    /// The generated accessor would leave it unfilled, which fails to format.
    UnknownVariable {
        /// The message id.
        message: String,
        /// The attribute, for an attribute accessor.
        attribute: Option<String>,
        /// The unknown `$variable` name.
        variable: String,
    },
    /// The `(Element)` marker sequence does not match the contract, so the
    /// structured-message split would not produce the expected slots.
    /// The entries are rendered as `$variable` / `-term`.
    ElementMismatch {
        /// The message id.
        message: String,
        /// The contract's marker sequence.
        expected: Vec<String>,
        /// The sequence found in the `.ftl`.
        found: Vec<String>,
    },
    /// A Boolean variable's selector does not contain exactly `[true]` and
    /// `[false]` branches.
    BoolSelectorMismatch {
        /// The message id.
        message: String,
        /// The attribute, for an attribute accessor.
        attribute: Option<String>,
        /// The Boolean variable driving the selector.
        variable: String,
        /// Variant keys found in the translation.
        found: Vec<String>,
    },
    /// A Boolean variable is referenced outside its selector, which would
    /// render its encoded `true` / `false` string directly.
    BoolReferenceOutsideSelector {
        /// The message id.
        message: String,
        /// The attribute, for an attribute accessor.
        attribute: Option<String>,
        /// The Boolean variable referenced directly.
        variable: String,
    },
    /// A referenced term is not defined in the `.ftl`. Resolving it would
    /// fail to format at runtime.
    UnknownTerm {
        /// The referenced `-term` name.
        term: String,
        /// Where the reference was found: a message id, `message.attribute`,
        /// or another term rendered as `-term`.
        referenced_from: String,
    },
}

impl fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMessage { message } => {
                write!(f, "message '{message}' is missing")
            }
            Self::MissingValue { message } => {
                write!(f, "message '{message}' has no value of its own")
            }
            Self::MissingAttribute { message, attribute } => {
                write!(f, "message '{message}' has no attribute '{attribute}'")
            }
            Self::UnknownVariable {
                message,
                attribute,
                variable,
            } => {
                match attribute {
                    Some(a) => write!(f, "attribute '{a}' of message '{message}'")?,
                    None => write!(f, "message '{message}'")?,
                }
                write!(f, " references unknown variable '${variable}'")
            }
            Self::ElementMismatch {
                message,
                expected,
                found,
            } => write!(
                f,
                "message '{message}' has element markers [{}] but the contract expects [{}]",
                found.join(", "),
                expected.join(", "),
            ),
            Self::BoolSelectorMismatch {
                message,
                attribute,
                variable,
                found,
            } => {
                match attribute {
                    Some(a) => write!(f, "attribute '{a}' of message '{message}'")?,
                    None => write!(f, "message '{message}'")?,
                }
                write!(
                    f,
                    " has Boolean selector ${variable} with keys [{}]; expected [true, false]",
                    found.join(", "),
                )
            }
            Self::BoolReferenceOutsideSelector {
                message,
                attribute,
                variable,
            } => {
                match attribute {
                    Some(a) => write!(f, "attribute '{a}' of message '{message}'")?,
                    None => write!(f, "message '{message}'")?,
                }
                write!(
                    f,
                    " references Boolean variable '${variable}' outside a [true]/[false] selector",
                )
            }
            Self::UnknownTerm {
                term,
                referenced_from,
            } => write!(
                f,
                "'{referenced_from}' references term '-{term}', which is not defined",
            ),
        }
    }
}

/// Validate external `.ftl` bytes against the compiled message contracts.
///
/// This is the check behind the generated `L10nLanguage::new_external`. It
/// verifies that:
///
/// - the bytes are valid UTF-8 and parse as Fluent
///   ([`L10nError::InvalidUtf8`] / [`L10nError::ResourceParse`] otherwise);
/// - every contract message (and attribute) is defined;
/// - no validated pattern references a variable outside its contract
///   arguments;
/// - selectors driven by Boolean arguments keep exactly `[true]` and `[false]`
///   variants;
/// - structured messages keep the exact `(Element)` marker sequence;
/// - every term referenced from a validated pattern (transitively, through
///   other terms) is defined in the file.
///
/// Contract violations are *collected* — the returned
/// [`L10nError::Validation`] lists every problem, not just the first — so a
/// translator can fix a rejected file in one pass.
pub fn validate_ftl(bytes: &[u8], contracts: &[MessageContract]) -> Result<(), L10nError> {
    let ftl = String::from_utf8(bytes.to_vec()).map_err(L10nError::InvalidUtf8)?;
    let resource = match parser::parse(ftl.as_str()) {
        Ok(resource) => resource,
        Err((_, errors)) => {
            return Err(L10nError::ResourceParse(
                errors.iter().map(|e| format!("{e:?}")).collect(),
            ));
        }
    };

    let mut messages: HashMap<&str, &ast::Message<&str>> = HashMap::new();
    let mut terms: HashMap<&str, &ast::Term<&str>> = HashMap::new();
    for entry in &resource.body {
        match entry {
            ast::Entry::Message(m) => {
                messages.entry(m.id.name).or_insert(m);
            }
            ast::Entry::Term(t) => {
                terms.entry(t.id.name).or_insert(t);
            }
            _ => {}
        }
    }

    let mut violations = Vec::new();
    // Terms already walked for undefined references, shared across contracts
    // so a term reached from several messages is reported once.
    let mut walked_terms: HashSet<String> = HashSet::new();

    for contract in contracts {
        let Some(message) = messages.get(contract.message) else {
            violations.push(ContractViolation::MissingMessage {
                message: contract.message.to_string(),
            });
            continue;
        };

        let pattern = if let Some(attribute) = contract.attribute {
            match message.attributes.iter().find(|a| a.id.name == attribute) {
                Some(attr) => &attr.value,
                None => {
                    violations.push(ContractViolation::MissingAttribute {
                        message: contract.message.to_string(),
                        attribute: attribute.to_string(),
                    });
                    continue;
                }
            }
        } else {
            match message.value.as_ref() {
                Some(value) => value,
                None => {
                    violations.push(ContractViolation::MissingValue {
                        message: contract.message.to_string(),
                    });
                    continue;
                }
            }
        };

        let refs = find_refs(pattern);
        let elements: Vec<(&str, RefKind)> = contract
            .elements
            .iter()
            .map(|e| {
                let kind = if e.is_term {
                    RefKind::Term
                } else {
                    RefKind::Variable
                };
                (e.name, kind)
            })
            .collect();
        let selectors = find_selectors(pattern);
        if let Err(incompatibilities) = check_refs(
            contract.vars,
            contract.bool_vars,
            &elements,
            &refs,
            &selectors,
        ) {
            violations.extend(
                incompatibilities
                    .into_iter()
                    .map(|incompat| match incompat {
                        RefsIncompat::UnknownVariable { variable } => {
                            ContractViolation::UnknownVariable {
                                message: contract.message.to_string(),
                                attribute: contract.attribute.map(str::to_string),
                                variable,
                            }
                        }
                        RefsIncompat::ElementMismatch { expected, found } => {
                            ContractViolation::ElementMismatch {
                                message: contract.message.to_string(),
                                expected: render_elements(&expected),
                                found: render_elements(&found),
                            }
                        }
                        RefsIncompat::BoolSelectorMismatch { variable, found } => {
                            ContractViolation::BoolSelectorMismatch {
                                message: contract.message.to_string(),
                                attribute: contract.attribute.map(str::to_string),
                                variable,
                                found,
                            }
                        }
                        RefsIncompat::BoolReferenceOutsideSelector { variable } => {
                            ContractViolation::BoolReferenceOutsideSelector {
                                message: contract.message.to_string(),
                                attribute: contract.attribute.map(str::to_string),
                                variable,
                            }
                        }
                    }),
            );
        }

        let origin = match contract.attribute {
            Some(a) => format!("{}.{a}", contract.message),
            None => contract.message.to_string(),
        };
        check_term_refs(&refs, &origin, &terms, &mut walked_terms, &mut violations);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(L10nError::Validation { violations })
    }
}

/// Flag references to undefined terms, walking transitively into the terms a
/// pattern references (a term's own pattern may reference further terms).
/// Only terms reachable from validated patterns are walked, so an unused,
/// broken term does not reject the file.
fn check_term_refs(
    refs: &[Ref],
    origin: &str,
    terms: &HashMap<&str, &ast::Term<&str>>,
    walked: &mut HashSet<String>,
    violations: &mut Vec<ContractViolation>,
) {
    for r in refs {
        if r.kind != RefKind::Term {
            continue;
        }
        match terms.get(r.name.as_str()) {
            None => violations.push(ContractViolation::UnknownTerm {
                term: r.name.clone(),
                referenced_from: origin.to_string(),
            }),
            Some(term) => {
                if walked.insert(r.name.clone()) {
                    let term_refs = find_refs(&term.value);
                    let term_origin = format!("-{}", r.name);
                    check_term_refs(&term_refs, &term_origin, terms, walked, violations);
                }
            }
        }
    }
}

/// Render an element marker sequence as `$variable` / `-term` names.
fn render_elements(elements: &[(String, RefKind)]) -> Vec<String> {
    elements
        .iter()
        .map(|(name, kind)| match kind {
            RefKind::Variable => format!("${name}"),
            RefKind::Term => format!("-{name}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACTS: &[MessageContract] = &[
        MessageContract {
            message: "hello",
            attribute: None,
            vars: &["name"],
            bool_vars: &[],
            elements: &[],
        },
        MessageContract {
            message: "login",
            attribute: Some("placeholder"),
            vars: &[],
            bool_vars: &[],
            elements: &[],
        },
        MessageContract {
            message: "notice",
            attribute: None,
            vars: &["count"],
            bool_vars: &[],
            elements: &[
                ElementContract {
                    name: "icon",
                    is_term: false,
                },
                ElementContract {
                    name: "privacy-link",
                    is_term: true,
                },
            ],
        },
    ];

    const VALID: &str = r#"
hello = Hej { $name }
login = Logga in
    .placeholder = Ange din e-post
notice = { $icon } Du har { $count } olästa, se { -privacy-link }.
-privacy-link = vår integritetspolicy
"#;

    fn violations(ftl: &str) -> Vec<ContractViolation> {
        match validate_ftl(ftl.as_bytes(), CONTRACTS) {
            Err(L10nError::Validation { violations }) => violations,
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn a_complete_translation_passes() {
        validate_ftl(VALID.as_bytes(), CONTRACTS).unwrap();
    }

    #[test]
    fn fewer_variables_than_the_contract_is_allowed() {
        // A locale that does not need an argument is fine — build-time
        // locales are held to the same rule.
        let ftl = VALID.replace("Hej { $name }", "Hej!");
        validate_ftl(ftl.as_bytes(), CONTRACTS).unwrap();
    }

    #[test]
    fn a_missing_message_is_a_violation() {
        let ftl = VALID.replace("hello", "helo");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::MissingMessage {
                message: "hello".to_string()
            }]
        );
    }

    #[test]
    fn a_missing_attribute_is_a_violation() {
        let ftl = VALID.replace(".placeholder", ".placeholdr");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::MissingAttribute {
                message: "login".to_string(),
                attribute: "placeholder".to_string(),
            }]
        );
    }

    #[test]
    fn a_message_without_a_value_is_a_violation() {
        let ftl = VALID.replace(
            "hello = Hej { $name }",
            "hello =\n    .other = Hej { $name }",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::MissingValue {
                message: "hello".to_string()
            }]
        );
    }

    #[test]
    fn an_unknown_variable_is_a_violation() {
        // `$nom` is not a contract argument: the accessor would leave it
        // unfilled. This is the typo that used to panic at accessor time.
        let ftl = VALID.replace("{ $name }", "{ $nom }");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownVariable {
                message: "hello".to_string(),
                attribute: None,
                variable: "nom".to_string(),
            }]
        );
    }

    #[test]
    fn an_unknown_variable_in_a_selector_is_a_violation() {
        let ftl = VALID.replace(
            "hello = Hej { $name }",
            "hello = { $other ->\n    [one] En\n   *[other] Hej\n}",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownVariable {
                message: "hello".to_string(),
                attribute: None,
                variable: "other".to_string(),
            }]
        );
    }

    #[test]
    fn boolean_selector_keys_are_validated() {
        const BOOL_CONTRACT: &[MessageContract] = &[MessageContract {
            message: "feature",
            attribute: None,
            vars: &["enabled"],
            bool_vars: &["enabled"],
            elements: &[],
        }];
        let valid = "feature = { $enabled ->\n    [true] On\n   *[false] Off\n}\n";
        validate_ftl(valid.as_bytes(), BOOL_CONTRACT).unwrap();

        let invalid = "feature = { $enabled ->\n    [yes] On\n   *[no] Off\n}\n";
        match validate_ftl(invalid.as_bytes(), BOOL_CONTRACT) {
            Err(L10nError::Validation { violations }) => assert_eq!(
                violations,
                vec![ContractViolation::BoolSelectorMismatch {
                    message: "feature".to_string(),
                    attribute: None,
                    variable: "enabled".to_string(),
                    found: vec!["yes".to_string(), "no".to_string()],
                }]
            ),
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn boolean_variable_must_not_be_interpolated_directly() {
        const BOOL_CONTRACT: &[MessageContract] = &[MessageContract {
            message: "feature",
            attribute: None,
            vars: &["enabled"],
            bool_vars: &["enabled"],
            elements: &[],
        }];
        let invalid = "feature = Feature: { $enabled }\n";

        match validate_ftl(invalid.as_bytes(), BOOL_CONTRACT) {
            Err(L10nError::Validation { violations }) => assert_eq!(
                violations,
                vec![ContractViolation::BoolReferenceOutsideSelector {
                    message: "feature".to_string(),
                    attribute: None,
                    variable: "enabled".to_string(),
                }]
            ),
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn independent_reference_violations_are_collected() {
        const BOOL_CONTRACT: &[MessageContract] = &[MessageContract {
            message: "feature",
            attribute: None,
            vars: &["enabled"],
            bool_vars: &["enabled"],
            elements: &[],
        }];
        let invalid = "feature = { $enabled ->\n    [yes] { $extra }\n   *[no] Off\n}\n";

        match validate_ftl(invalid.as_bytes(), BOOL_CONTRACT) {
            Err(L10nError::Validation { violations }) => assert_eq!(
                violations,
                vec![
                    ContractViolation::UnknownVariable {
                        message: "feature".to_string(),
                        attribute: None,
                        variable: "extra".to_string(),
                    },
                    ContractViolation::BoolSelectorMismatch {
                        message: "feature".to_string(),
                        attribute: None,
                        variable: "enabled".to_string(),
                        found: vec!["yes".to_string(), "no".to_string()],
                    },
                ]
            ),
            other => panic!("expected Validation error, got {other:?}"),
        }
    }

    #[test]
    fn a_reordered_element_sequence_is_a_violation() {
        let ftl = VALID.replace(
            "{ $icon } Du har { $count } olästa, se { -privacy-link }.",
            "Se { -privacy-link } { $icon } för { $count } olästa.",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::ElementMismatch {
                message: "notice".to_string(),
                expected: vec!["$icon".to_string(), "-privacy-link".to_string()],
                found: vec!["-privacy-link".to_string(), "$icon".to_string()],
            }]
        );
    }

    #[test]
    fn a_dropped_element_is_a_violation() {
        let ftl = VALID.replace("{ $icon } ", "");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::ElementMismatch {
                message: "notice".to_string(),
                expected: vec!["$icon".to_string(), "-privacy-link".to_string()],
                found: vec!["-privacy-link".to_string()],
            }]
        );
    }

    #[test]
    fn an_undefined_term_is_a_violation() {
        let ftl = VALID.replace("-privacy-link = vår integritetspolicy", "");
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownTerm {
                term: "privacy-link".to_string(),
                referenced_from: "notice".to_string(),
            }]
        );
    }

    #[test]
    fn an_undefined_term_behind_another_term_is_a_violation() {
        let ftl = VALID.replace(
            "-privacy-link = vår integritetspolicy",
            "-privacy-link = vår { -policy }",
        );
        assert_eq!(
            violations(&ftl),
            vec![ContractViolation::UnknownTerm {
                term: "policy".to_string(),
                referenced_from: "-privacy-link".to_string(),
            }]
        );
    }

    #[test]
    fn all_violations_are_collected() {
        let ftl = "notice = Bara { $typo } kvar\n";
        let found = violations(ftl);
        // Violations are collected both across and within messages: hello and
        // login are missing; notice dropped its markers and added `$typo`.
        assert_eq!(found.len(), 4, "got: {found:?}");
        assert!(found.iter().any(
            |v| matches!(v, ContractViolation::MissingMessage { message } if message == "hello")
        ));
        assert!(found.iter().any(
            |v| matches!(v, ContractViolation::ElementMismatch { message, .. } if message == "notice")
        ));
        assert!(found.iter().any(
            |v| matches!(v, ContractViolation::UnknownVariable { variable, .. } if variable == "typo")
        ));
    }

    #[test]
    fn a_parse_error_is_a_resource_parse_error() {
        let err = validate_ftl(b"hello = { $unclosed\n", CONTRACTS).unwrap_err();
        assert!(matches!(err, L10nError::ResourceParse(_)), "got: {err:?}");
    }

    #[test]
    fn invalid_utf8_is_an_utf8_error() {
        let err = validate_ftl(&[0xff, 0xfe], CONTRACTS).unwrap_err();
        assert!(matches!(err, L10nError::InvalidUtf8(_)), "got: {err:?}");
    }

    #[test]
    fn violations_render_readably() {
        let ftl = VALID.replace("{ $name }", "{ $nom }");
        let err = validate_ftl(ftl.as_bytes(), CONTRACTS).unwrap_err();
        let text = err.to_string();
        assert!(
            text.contains("message 'hello' references unknown variable '$nom'"),
            "got: {text}"
        );
    }
}
