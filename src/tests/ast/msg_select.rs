use crate::tests::ast::AstResourceExt;
use crate::typed::*;
use fluent_syntax::ast;
use fluent_syntax::parser;

const FTL: &str = r#"

key = { $var ->
    [key1] Value 1
   *[other] Value 2
}

"#;

/// From: https://docs.rs/fluent-syntax/0.11.1/fluent_syntax/ast/enum.Expression.html
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");

    assert_eq!(
        resource,
        ast::Resource {
            body: vec![ast::Entry::Message(ast::Message {
                id: ast::Identifier { name: "key" },
                value: Some(ast::Pattern {
                    elements: vec![ast::PatternElement::Placeable {
                        expression: ast::Expression::Select {
                            selector: ast::InlineExpression::VariableReference {
                                id: ast::Identifier { name: "var" },
                            },
                            variants: vec![
                                ast::Variant {
                                    key: ast::VariantKey::Identifier { name: "key1" },
                                    value: ast::Pattern {
                                        elements: vec![ast::PatternElement::TextElement {
                                            value: "Value 1",
                                        }]
                                    },
                                    default: false,
                                },
                                ast::Variant {
                                    key: ast::VariantKey::Identifier { name: "other" },
                                    value: ast::Pattern {
                                        elements: vec![ast::PatternElement::TextElement {
                                            value: "Value 2",
                                        }]
                                    },
                                    default: true,
                                },
                            ]
                        }
                    }]
                }),
                attributes: vec![],
                comment: None,
            }),]
        }
    );
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = resource.first_message();

    println!("{:#?}", message);
    assert_eq!(
        message,
        Message {
            comment: vec![],
            id: "key",
            variables: vec![Variable {
                id: "var",
                typ: VarType::Enumeration(vec![
                    EnumEntry {
                        name: "key1",
                        default: false,
                    },
                    EnumEntry {
                        name: "other",
                        default: true,
                    },
                ],),
            },],
            attributes: vec![],
        }
    );
}
