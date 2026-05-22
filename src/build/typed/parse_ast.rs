use super::*;
use crate::build::LineIndex;
use fluent_syntax::ast;
use type_in_comment::TypeInComment;

impl Message {
    /// Parse an AST message into one `Message` for its value (if any) plus one
    /// per attribute. `lines` is the newline index of the FTL source the
    /// message was parsed from (used to recover line numbers) and `file` is its
    /// path (for diagnostics).
    pub fn parse(message: &ast::Message<&str>, lines: &LineIndex, file: &str) -> Vec<Self> {
        let mut found = Vec::new();
        let comment = message
            .comment
            .as_ref()
            .map(|v| v.content.iter().map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        let comment_line = message
            .comment
            .as_ref()
            .and_then(|c| c.content.first())
            .map(|first| lines.line_of(first))
            .unwrap_or(0);
        let tic = TypeInComment::parse(&comment);

        if let Some(value) = message.value.as_ref() {
            let mut variables = find_variable_references(value);
            tic.update_types(&mut variables);
            let elements = find_elements(value, tic.element_vars(), tic.element_terms());
            // Element variables are positional gaps, not function arguments.
            // Retain on the elements `find_elements` actually matched (a
            // top-level placeable), so an `(Element)` annotation on a name
            // that is not a real element placeable leaves the variable as an
            // ordinary argument rather than dropping it from both lists.
            let element_var_names: Vec<&str> = elements
                .iter()
                .filter(|e| e.kind == ElementKind::Variable)
                .map(|e| e.name.as_str())
                .collect();
            variables.retain(|v| !element_var_names.contains(&v.id.as_str()));
            found.push(Self {
                id: Id {
                    message: message.id.name.to_owned(),
                    attribute: None,
                },
                comment: comment.clone(),
                variables,
                elements,
                pattern_refs: find_refs(value),
                file: file.to_owned(),
                line: lines.line_of(message.id.name),
                comment_line,
            });
        }
        for (idx, attribute) in message.attributes.iter().enumerate() {
            // The message comment also types the attributes' variables — a
            // fluent attribute cannot carry a comment of its own.
            let mut variables = find_variable_references(&attribute.value);
            tic.update_types(&mut variables);
            // A message with no value of its own produced no value-Message to
            // carry its comment; attach it to the first attribute so the
            // linter can still inspect that comment.
            let carries_comment = message.value.is_none() && idx == 0;
            found.push(Self {
                id: Id {
                    message: message.id.name.to_owned(),
                    attribute: Some(attribute.id.name.to_owned()),
                },
                comment: if carries_comment {
                    comment.clone()
                } else {
                    vec![]
                },
                variables,
                elements: vec![],
                pattern_refs: find_refs(&attribute.value),
                file: file.to_owned(),
                line: lines.line_of(attribute.id.name),
                comment_line: if carries_comment { comment_line } else { 0 },
            });
        }
        found
    }
}

/// Collect every `$variable` and `-term` reference in a pattern, in document
/// order, **independent of comments**. Walks the same depth as
/// [`find_variable_references`] (selects, variants, call arguments, nested
/// placeables). The result is used for comment-independent cross-locale
/// compatibility checks; it is intentionally not deduplicated, so a repeated
/// reference (e.g. the same `(Element)` used twice) is preserved.
pub fn find_refs(pattern: &ast::Pattern<&str>) -> Vec<Ref> {
    let mut refs = Vec::new();
    collect_refs_pattern(pattern, &mut refs);
    refs
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

/// Collect every `$variable` referenced anywhere in a message pattern, in the
/// order they first appear, inferring `Number` where the syntax demands it.
///
/// The walk descends into selects (the selector *and* every variant body),
/// function- and term-call arguments, and nested placeables — anywhere a
/// variable can hide. A variable is typed `Number` when it is the value
/// formatted by a `NUMBER()` call or the selector of an all-numeric select;
/// otherwise it is `Any` (a `(String)`/`(Number)` comment annotation may still
/// refine it later — see `TypeInComment::update_types`).
pub fn find_variable_references(pattern: &ast::Pattern<&str>) -> Vec<Variable> {
    let mut collector = VarCollector::default();
    collector.visit_pattern(pattern);
    collector.variables
}

#[derive(Default)]
struct VarCollector {
    variables: Vec<Variable>,
}

impl VarCollector {
    /// Record a reference to `$id`. The same variable may be referenced many
    /// times; it is collected once and keeps its most specific type — once
    /// known to be a `Number` it is never downgraded back to `Any`.
    fn add(&mut self, id: &str, typ: VarType) {
        match self.variables.iter_mut().find(|v| v.id == id) {
            Some(existing) => {
                if existing.typ == VarType::Any {
                    existing.typ = typ;
                }
            }
            None => self.variables.push(Variable {
                id: id.to_owned(),
                typ,
            }),
        }
    }

    fn visit_pattern(&mut self, pattern: &ast::Pattern<&str>) {
        for element in &pattern.elements {
            if let ast::PatternElement::Placeable { expression } = element {
                self.visit_expression(expression);
            }
        }
    }

    fn visit_expression(&mut self, expression: &ast::Expression<&str>) {
        match expression {
            ast::Expression::Inline(inline) => self.visit_inline(inline, VarType::Any),
            ast::Expression::Select { selector, variants } => {
                // A selector whose variants are all numbers / CLDR plural
                // categories forces the selected variable to `Number`.
                let typ = if variants.iter().all(|v| v.is_number()) {
                    VarType::Number
                } else {
                    VarType::Any
                };
                self.visit_inline(selector, typ);
                for variant in variants {
                    self.visit_pattern(&variant.value);
                }
            }
        }
    }

    /// Walk an inline expression. `ctx` is the type to assign to a bare
    /// `$variable` found directly here (e.g. `Number` for a numeric selector).
    fn visit_inline(&mut self, inline: &ast::InlineExpression<&str>, ctx: VarType) {
        match inline {
            ast::InlineExpression::VariableReference { id } => self.add(id.name, ctx),
            ast::InlineExpression::FunctionReference { id, arguments } => {
                // `NUMBER()` formats its positional argument as a number.
                let positional = if id.name == "NUMBER" {
                    VarType::Number
                } else {
                    ctx
                };
                self.visit_call_arguments(arguments, positional);
            }
            ast::InlineExpression::TermReference { arguments, .. } => {
                if let Some(arguments) = arguments {
                    self.visit_call_arguments(arguments, VarType::Any);
                }
            }
            ast::InlineExpression::Placeable { expression } => self.visit_expression(expression),
            // Literals and message references contain no variables.
            ast::InlineExpression::StringLiteral { .. }
            | ast::InlineExpression::NumberLiteral { .. }
            | ast::InlineExpression::MessageReference { .. } => {}
        }
    }

    /// Positional arguments inherit `positional`; named option values are only
    /// ever `Any` — they configure the call, they are not the formatted value.
    fn visit_call_arguments(&mut self, arguments: &ast::CallArguments<&str>, positional: VarType) {
        for arg in &arguments.positional {
            self.visit_inline(arg, positional);
        }
        for named in &arguments.named {
            self.visit_inline(&named.value, VarType::Any);
        }
    }
}

/// Walk the pattern and collect the `(Element)`-annotated placeables, in order.
///
/// A placeable is a marker iff its variable/term name was annotated `(Element)`
/// in the message comment. Everything else (text, ordinary variables, selects,
/// un-annotated term references) is ordinary content.
pub fn find_elements(
    pattern: &ast::Pattern<&str>,
    element_vars: &[String],
    element_terms: &[String],
) -> Vec<ElementMarker> {
    let mut elements = vec![];

    for element in &pattern.elements {
        let ast::PatternElement::Placeable { expression } = element else {
            continue;
        };
        match expression {
            ast::Expression::Inline(ast::InlineExpression::VariableReference { id })
                if element_vars.iter().any(|v| v == id.name) =>
            {
                elements.push(ElementMarker {
                    name: id.name.to_owned(),
                    kind: ElementKind::Variable,
                });
            }
            ast::Expression::Inline(ast::InlineExpression::TermReference { id, .. })
                if element_terms.iter().any(|t| t == id.name) =>
            {
                elements.push(ElementMarker {
                    name: id.name.to_owned(),
                    kind: ElementKind::Term,
                });
            }
            _ => {}
        }
    }
    elements
}

trait AstVariantExt {
    fn is_number(&self) -> bool;
}
impl AstVariantExt for ast::Variant<&str> {
    fn is_number(&self) -> bool {
        match self.key {
            ast::VariantKey::NumberLiteral { .. } => true,
            ast::VariantKey::Identifier { name } => {
                ["zero", "one", "two", "few", "many", "other"].contains(&name)
            }
        }
    }
}
