use super::*;
use fluent_syntax::ast;
use type_in_comment::TypeInComment;

impl<'ast, 'res> Message<'ast, 'res> {
    pub fn parse(resource: Option<&'res str>, message: &'ast ast::Message<&'ast str>) -> Vec<Self> {
        let mut found = Vec::new();
        let comment = message
            .comment
            .as_ref()
            .map(|v| v.content.clone())
            .unwrap_or_default();
        if let Some(value) = message.value.as_ref() {
            let mut variables = find_variable_references(value);
            let tic = TypeInComment::parse(&comment);
            tic.update_types(&mut variables);
            let id = Id {
                resource,
                message: message.id.name,
                attribute: None,
            };
            found.push(Self {
                id,
                comment,
                variables,
            });
        }
        for attribute in find_attributes(&message.attributes) {
            let variables = attribute.variables;
            let id = Id {
                resource,
                message: message.id.name,
                attribute: Some(attribute.id),
            };
            found.push(Self {
                id,
                comment: vec![],
                variables,
            });
        }
        found
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
                    let is_num = variants.iter().all(|v| v.is_number());
                    let typ = if is_num {
                        VarType::Number
                    } else {
                        VarType::Any
                    };
                    variables.push(Variable { id: id.name, typ });
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
