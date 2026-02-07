use crate::build::typed::*;
use crate::tests::ast::AstResourceExt;
use crate::tests::ast::bundle;
use fluent_syntax::ast;
use fluent_syntax::parser;

const FTL: &str = r#"

liked-count = { $num ->
        [0]     No likes yet.
       *[other] { $num } people liked your message
    }

"#;

/// From: https://docs.rs/fluent-syntax/0.11.1/fluent_syntax/ast/enum.Expression.html
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    println!("{:#?}", resource);
    assert_eq!(
        resource,
        ast::Resource {
            body: vec![ast::Entry::Message(ast::Message {
                id: ast::Identifier {
                    name: "liked-count"
                },
                value: Some(ast::Pattern {
                    elements: vec![ast::PatternElement::Placeable {
                        expression: ast::Expression::Select {
                            selector: ast::InlineExpression::VariableReference {
                                id: ast::Identifier { name: "num" },
                            },
                            variants: vec![
                                ast::Variant {
                                    key: ast::VariantKey::NumberLiteral { value: "0" },
                                    value: ast::Pattern {
                                        elements: vec![ast::PatternElement::TextElement {
                                            value: "No likes yet.",
                                        }]
                                    },
                                    default: false,
                                },
                                ast::Variant {
                                    key: ast::VariantKey::Identifier { name: "other" },
                                    value: ast::Pattern {
                                        elements: vec![
                                            ast::PatternElement::Placeable {
                                                expression: ast::Expression::Inline(
                                                    ast::InlineExpression::VariableReference {
                                                        id: ast::Identifier { name: "num" }
                                                    }
                                                )
                                            },
                                            ast::PatternElement::TextElement {
                                                value: " people liked your message",
                                            }
                                        ]
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

    let msg = bundle
        .get_message("liked-count")
        .expect("Message doesn't exist.");
    let pattern = msg.value().expect("Message has no value.");
    let mut errors = vec![];

    let mut args = fluent_bundle::FluentArgs::new();
    args.set("num", 3);
    let value = bundle.format_pattern(pattern, Some(&args), &mut errors);
    assert_eq!(&value, "3 people liked your message");
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
            id: Id::new_msg("liked-count"),
            comment: vec![],
            variables: vec![Variable {
                id: "num".to_string(),
                typ: VarType::Number,
            }],
        }
    );
}
