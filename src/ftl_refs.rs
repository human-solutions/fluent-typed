//! Shared FTL pattern-reference utilities.
//!
//! Used by both the build-time cross-locale analysis (every locale is checked
//! against the default locale's contract) and the runtime validation of
//! external `.ftl` translations ([`validate_ftl`](crate::validate_ftl)).
//! Keeping a single reference walker and a single compatibility check
//! guarantees the two checks cannot drift apart: an external translation is
//! held to exactly the rules a build-time locale is held to.

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

/// Collect every `$variable` and `-term` reference in a pattern, in document
/// order, **independent of comments**. Walks selects (selector and every
/// variant body), call arguments and nested placeables. The result is
/// intentionally not deduplicated, so a repeated reference (e.g. the same
/// `(Element)` used twice) is preserved.
pub fn find_refs(pattern: &ast::Pattern<&str>) -> Vec<Ref> {
    let mut refs = Vec::new();
    collect_refs_pattern(pattern, &mut refs);
    refs
}

/// Collect select expressions whose selector is a direct `$variable`.
/// Nested selects and selects inside variant bodies are included.
pub fn find_selectors(pattern: &ast::Pattern<&str>) -> Vec<Selector> {
    let mut selectors = Vec::new();
    collect_selectors_pattern(pattern, &mut selectors);
    selectors
}

fn collect_selectors_pattern(pattern: &ast::Pattern<&str>, selectors: &mut Vec<Selector>) {
    for element in &pattern.elements {
        if let ast::PatternElement::Placeable { expression } = element {
            collect_selectors_expr(expression, selectors);
        }
    }
}

fn collect_selectors_expr(expression: &ast::Expression<&str>, selectors: &mut Vec<Selector>) {
    match expression {
        ast::Expression::Inline(inline) => collect_selectors_inline(inline, selectors),
        ast::Expression::Select { selector, variants } => {
            if let Some(variable) = selector_variable(selector) {
                selectors.push(Selector {
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
            collect_selectors_inline(selector, selectors);
            for variant in variants {
                collect_selectors_pattern(&variant.value, selectors);
            }
        }
    }
}

fn collect_selectors_inline(inline: &ast::InlineExpression<&str>, selectors: &mut Vec<Selector>) {
    match inline {
        ast::InlineExpression::TermReference { arguments, .. } => {
            if let Some(arguments) = arguments {
                collect_selectors_call_arguments(arguments, selectors);
            }
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            collect_selectors_call_arguments(arguments, selectors);
        }
        ast::InlineExpression::Placeable { expression } => {
            collect_selectors_expr(expression, selectors);
        }
        ast::InlineExpression::VariableReference { .. }
        | ast::InlineExpression::StringLiteral { .. }
        | ast::InlineExpression::NumberLiteral { .. }
        | ast::InlineExpression::MessageReference { .. } => {}
    }
}

fn collect_selectors_call_arguments(
    arguments: &ast::CallArguments<&str>,
    selectors: &mut Vec<Selector>,
) {
    for positional in &arguments.positional {
        collect_selectors_inline(positional, selectors);
    }
    for named in &arguments.named {
        collect_selectors_inline(&named.value, selectors);
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

fn collect_refs_pattern(pattern: &ast::Pattern<&str>, refs: &mut Vec<Ref>) {
    for element in &pattern.elements {
        if let ast::PatternElement::Placeable { expression } = element {
            collect_refs_expr(expression, refs);
        }
    }
}

fn collect_refs_expr(expression: &ast::Expression<&str>, refs: &mut Vec<Ref>) {
    match expression {
        ast::Expression::Inline(inline) => collect_refs_inline(inline, refs),
        ast::Expression::Select { selector, variants } => {
            collect_refs_inline(selector, refs);
            for variant in variants {
                collect_refs_pattern(&variant.value, refs);
            }
        }
    }
}

fn collect_refs_inline(inline: &ast::InlineExpression<&str>, refs: &mut Vec<Ref>) {
    match inline {
        ast::InlineExpression::VariableReference { id } => refs.push(Ref {
            name: id.name.to_owned(),
            kind: RefKind::Variable,
        }),
        ast::InlineExpression::TermReference { id, arguments, .. } => {
            refs.push(Ref {
                name: id.name.to_owned(),
                kind: RefKind::Term,
            });
            if let Some(arguments) = arguments {
                collect_refs_call_arguments(arguments, refs);
            }
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            collect_refs_call_arguments(arguments, refs)
        }
        ast::InlineExpression::Placeable { expression } => collect_refs_expr(expression, refs),
        ast::InlineExpression::StringLiteral { .. }
        | ast::InlineExpression::NumberLiteral { .. }
        | ast::InlineExpression::MessageReference { .. } => {}
    }
}

fn collect_refs_call_arguments(arguments: &ast::CallArguments<&str>, refs: &mut Vec<Ref>) {
    for arg in &arguments.positional {
        collect_refs_inline(arg, refs);
    }
    for named in &arguments.named {
        collect_refs_inline(&named.value, refs);
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
) -> Result<(), RefsIncompat> {
    for selector in selectors
        .iter()
        .filter(|selector| bool_vars.contains(&selector.variable.as_str()))
    {
        let mut keys = selector.keys.clone();
        keys.sort();
        if keys != ["false", "true"] {
            return Err(RefsIncompat::BoolSelectorMismatch {
                variable: selector.variable.clone(),
                found: selector.keys.clone(),
            });
        }
    }

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
            return Err(RefsIncompat::ElementMismatch { expected, found });
        }
        // Non-element variables fall through to the argument check below.
        for r in refs {
            if r.kind == RefKind::Variable
                && !element_names.contains(&r.name.as_str())
                && !vars.contains(&r.name.as_str())
            {
                return Err(RefsIncompat::UnknownVariable {
                    variable: r.name.clone(),
                });
            }
        }
        return Ok(());
    }

    for r in refs {
        if r.kind == RefKind::Variable && !vars.contains(&r.name.as_str()) {
            return Err(RefsIncompat::UnknownVariable {
                variable: r.name.clone(),
            });
        }
    }
    Ok(())
}
