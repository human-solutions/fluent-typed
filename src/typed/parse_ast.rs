use super::*;
use fluent_syntax::ast;
use type_in_comment::TypeInComment;

impl<'ast> Message<'ast> {
    pub fn parse(message: &'ast ast::Message<&'ast str>) -> Self {
        let comment = message
            .comment
            .as_ref()
            .map(|v| v.content.clone())
            .unwrap_or_default();
        let mut variables = message
            .value
            .as_ref()
            .map(find_variable_references)
            .unwrap_or_default();
        let tic = TypeInComment::parse(&comment);
        tic.update_types(&mut variables);
        let attributes = find_attributes(&message.attributes);
        Self {
            id: message.id.name,
            comment,
            variables,
            attributes,
        }
    }
}

impl<'ast> Attribute<'ast> {
    pub fn parse(attribute: &'ast ast::Attribute<&'ast str>) -> Self {
        let variables = find_variable_references(&attribute.value);
        Self {
            id: attribute.id.name,
            variables,
        }
    }
}

impl<'ast> EnumEntry<'ast> {
    pub fn parse_vec(variants: &[ast::Variant<&'ast str>]) -> Vec<EnumEntry<'ast>> {
        variants.iter().map(EnumEntry::parse).collect()
    }

    pub fn parse(var: &ast::Variant<&'ast str>) -> EnumEntry<'ast> {
        EnumEntry {
            name: match var.key {
                ast::VariantKey::Identifier { name } => name,
                ast::VariantKey::NumberLiteral { value } => value,
            },
            default: var.default,
        }
    }
}

pub fn find_variable_references<'ast>(pattern: &ast::Pattern<&'ast str>) -> Vec<Variable<'ast>> {
    let mut variables = vec![];

    for element in &pattern.elements {
        match element {
            ast::PatternElement::Placeable { expression } => match expression {
                ast::Expression::Inline(ast::InlineExpression::VariableReference { id }) => {
                    variables.push(Variable {
                        id: id.name,
                        typ: VarType::Any,
                    });
                }
                ast::Expression::Select {
                    selector: ast::InlineExpression::VariableReference { id },
                    variants,
                } => {
                    variables.push(Variable {
                        id: id.name,
                        typ: VarType::Enumeration(EnumEntry::parse_vec(variants)),
                    });
                }
                _ => {}
            },
            ast::PatternElement::TextElement { value: _ } => {}
        }
    }
    variables
}

pub fn find_attributes<'ast>(
    attributes: &'ast [ast::Attribute<&'ast str>],
) -> Vec<Attribute<'ast>> {
    attributes.iter().map(Attribute::parse).collect()
}
