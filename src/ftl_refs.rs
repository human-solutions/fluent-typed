//! Shared FTL pattern-reference utilities.
//!
//! Used by both the build-time cross-locale analysis (every locale is checked
//! against the default locale's contract) and the runtime validation of
//! external `.ftl` translations ([`validate_ftl`](crate::validate_ftl)).
//! Keeping a single reference walker and a single compatibility check
//! guarantees the two checks cannot drift apart: an external translation is
//! held to exactly the rules a build-time locale is held to.

use std::collections::HashSet;

use fluent_syntax::ast;

/// A `$variable` or `-term` reference in a message pattern.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Ref {
    pub name: String,
    pub kind: RefKind,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RefKind {
    Variable,
    Term,
}

/// A select expression driven directly by a `$variable`.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Selector {
    pub variable: String,
    pub keys: Vec<String>,
}

impl Selector {
    /// Whether this selector exposes exactly the two keys accepted by a
    /// Boolean argument's `"true"` / `"false"` string encoding.
    pub fn has_bool_keys(&self) -> bool {
        self.keys.len() == 2
            && self.keys.iter().any(|key| key == "true")
            && self.keys.iter().any(|key| key == "false")
    }
}

/// Collect every `$variable` and `-term` reference in a pattern, in document
/// order, **independent of comments**. Walks selects (selector and every
/// variant body), call arguments and nested placeables. The result is
/// intentionally not deduplicated, so a repeated reference (e.g. the same
/// `(Element)` used twice) is preserved.
pub fn find_refs(pattern: &ast::Pattern<&str>) -> Vec<Ref> {
    find_refs_and_selectors(pattern).0
}

/// Collect references and selectors in one AST traversal. Callers needing
/// both should use this instead of walking the same pattern twice.
pub fn find_refs_and_selectors(pattern: &ast::Pattern<&str>) -> (Vec<Ref>, Vec<Selector>) {
    let mut collector = Collector::default();
    collector.pattern(pattern);
    (collector.refs, collector.selectors)
}

#[derive(Default)]
struct Collector {
    refs: Vec<Ref>,
    selectors: Vec<Selector>,
}

impl Collector {
    fn pattern(&mut self, pattern: &ast::Pattern<&str>) {
        for element in &pattern.elements {
            if let ast::PatternElement::Placeable { expression } = element {
                self.expression(expression);
            }
        }
    }

    fn expression(&mut self, expression: &ast::Expression<&str>) {
        match expression {
            ast::Expression::Inline(inline) => self.inline(inline),
            ast::Expression::Select { selector, variants } => {
                if let Some(variable) = selector_variable(selector) {
                    self.selectors.push(Selector {
                        variable: variable.to_owned(),
                        keys: variants
                            .iter()
                            .map(|variant| match variant.key {
                                ast::VariantKey::Identifier { name } => name.to_owned(),
                                ast::VariantKey::NumberLiteral { value } => value.to_owned(),
                            })
                            .collect(),
                    });
                }
                self.inline(selector);
                for variant in variants {
                    self.pattern(&variant.value);
                }
            }
        }
    }

    fn inline(&mut self, inline: &ast::InlineExpression<&str>) {
        match inline {
            ast::InlineExpression::VariableReference { id } => self.refs.push(Ref {
                name: id.name.to_owned(),
                kind: RefKind::Variable,
            }),
            ast::InlineExpression::TermReference { id, arguments, .. } => {
                self.refs.push(Ref {
                    name: id.name.to_owned(),
                    kind: RefKind::Term,
                });
                if let Some(arguments) = arguments {
                    self.call_arguments(arguments);
                }
            }
            ast::InlineExpression::FunctionReference { arguments, .. } => {
                self.call_arguments(arguments)
            }
            ast::InlineExpression::Placeable { expression } => self.expression(expression),
            ast::InlineExpression::StringLiteral { .. }
            | ast::InlineExpression::NumberLiteral { .. }
            | ast::InlineExpression::MessageReference { .. } => {}
        }
    }

    fn call_arguments(&mut self, arguments: &ast::CallArguments<&str>) {
        for positional in &arguments.positional {
            self.inline(positional);
        }
        for named in &arguments.named {
            self.inline(&named.value);
        }
    }
}

fn selector_variable<'a>(inline: &'a ast::InlineExpression<&'a str>) -> Option<&'a str> {
    match inline {
        ast::InlineExpression::VariableReference { id } => Some(id.name),
        ast::InlineExpression::Placeable { expression } => match expression.as_ref() {
            ast::Expression::Inline(inline) => selector_variable(inline),
            ast::Expression::Select { .. } => None,
        },
        _ => None,
    }
}

/// Why a candidate pattern is incompatible with a message contract.
#[derive(Debug, PartialEq, Eq)]
pub enum RefsIncompat {
    /// The pattern references a variable that is not a contract argument.
    /// Such a variable would be unfilled at runtime and fail to format.
    UnknownVariable { variable: String },
    /// The `(Element)` marker sequence of the pattern does not match the
    /// contract's, so the segment split would not line up.
    ElementMismatch {
        expected: Vec<(String, RefKind)>,
        found: Vec<(String, RefKind)>,
    },
    /// A Boolean variable drives a selector without exactly the `true` and
    /// `false` keys expected by its string encoding.
    BoolSelectorMismatch {
        variable: String,
        found: Vec<String>,
    },
    /// A Boolean variable is referenced outside a select expression. Its
    /// string encoding would otherwise render as an untranslated literal.
    BoolReferenceOutsideSelector { variable: String },
}

/// Check a candidate pattern's references against a message contract.
///
/// `vars` are the contract's argument names and `elements` its exact
/// `(Element)` marker sequence (empty for a plain message); `refs` are the
/// candidate pattern's references as returned by [`find_refs`].
///
/// A plain message is compatible when every variable it references is a known
/// contract argument (referencing *fewer* is fine — a translation may not need
/// every argument). An element message additionally requires the element
/// markers to appear exactly as in the contract, in name, kind and order, so
/// that the runtime segment split produces the same slots.
pub fn check_refs(
    vars: &[&str],
    bool_vars: &[&str],
    elements: &[(&str, RefKind)],
    refs: &[Ref],
    selectors: &[Selector],
) -> Result<(), Vec<RefsIncompat>> {
    let mut incompatibilities = Vec::new();

    if !elements.is_empty() {
        let element_names: Vec<&str> = elements.iter().map(|(n, _)| *n).collect();
        let found: Vec<(String, RefKind)> = refs
            .iter()
            .filter(|r| element_names.contains(&r.name.as_str()))
            .map(|r| (r.name.clone(), r.kind))
            .collect();
        let expected: Vec<(String, RefKind)> =
            elements.iter().map(|(n, k)| (n.to_string(), *k)).collect();
        if expected != found {
            incompatibilities.push(RefsIncompat::ElementMismatch { expected, found });
        }
        // Non-element variables fall through to the argument check below.
        let mut unknown = HashSet::new();
        for r in refs {
            if r.kind == RefKind::Variable
                && !element_names.contains(&r.name.as_str())
                && !vars.contains(&r.name.as_str())
                && unknown.insert(r.name.as_str())
            {
                incompatibilities.push(RefsIncompat::UnknownVariable {
                    variable: r.name.clone(),
                });
            }
        }
    } else {
        let mut unknown = HashSet::new();
        for r in refs {
            if r.kind == RefKind::Variable
                && !vars.contains(&r.name.as_str())
                && unknown.insert(r.name.as_str())
            {
                incompatibilities.push(RefsIncompat::UnknownVariable {
                    variable: r.name.clone(),
                });
            }
        }
    }

    for variable in bool_vars {
        let reference_count = refs
            .iter()
            .filter(|r| r.kind == RefKind::Variable && r.name == *variable)
            .count();
        let selector_count = selectors
            .iter()
            .filter(|selector| selector.variable == *variable)
            .count();
        if reference_count != selector_count {
            incompatibilities.push(RefsIncompat::BoolReferenceOutsideSelector {
                variable: (*variable).to_owned(),
            });
        }
    }

    for selector in selectors
        .iter()
        .filter(|selector| bool_vars.contains(&selector.variable.as_str()))
    {
        if !selector.has_bool_keys() {
            incompatibilities.push(RefsIncompat::BoolSelectorMismatch {
                variable: selector.variable.clone(),
                found: selector.keys.clone(),
            });
        }
    }

    if incompatibilities.is_empty() {
        Ok(())
    } else {
        Err(incompatibilities)
    }
}
