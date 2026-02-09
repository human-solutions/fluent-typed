use crate::build::typed::*;
use crate::tests::ast::AstResourceExt;
use crate::tests::ast::bundle;
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
fn ast_use() {
    let bundle = bundle(FTL);

    let msg = bundle.get_message("key").expect("Message doesn't exist.");
    let pattern = msg.value().expect("Message has no value.");
    let mut errors = vec![];

    let mut args = fluent_bundle::FluentArgs::new();
    args.set("var", "key1");
    let value = bundle.format_pattern(pattern, Some(&args), &mut errors);
    assert_eq!(&value, "Value 1");
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = resource.first_message();

    println!("{:#?}", message);
    assert_eq!(
        message,
        Message {
            resource: "test".to_string(),
            id: Id::new_msg("key"),
            comment: vec![],
            variables: vec![Variable {
                id: "var".to_string(),
                typ: VarType::Any,
            }],
        }
    );
}
