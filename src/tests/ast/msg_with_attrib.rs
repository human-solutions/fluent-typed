use super::bundle;
use crate::build::typed::*;
use crate::tests::assert_gen;
use crate::tests::ast::AstResourceExt;
use fluent_syntax::ast;
use fluent_syntax::parser;

const FTL: &str = r#"

# This is a message comment
hello = Hello World!
    .tooltip = Tooltip for you, { $userName }.

"#;

/// From https://docs.rs/fluent-syntax/0.11.1/fluent_syntax/ast/index.html
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");

    assert_eq!(
        resource.body[0],
        ast::Entry::Message(ast::Message {
            id: ast::Identifier { name: "hello" },
            value: Some(ast::Pattern {
                elements: vec![ast::PatternElement::TextElement {
                    value: "Hello World!"
                },]
            }),
            attributes: vec![ast::Attribute {
                id: ast::Identifier { name: "tooltip" },
                value: ast::Pattern {
                    elements: vec![
                        ast::PatternElement::TextElement {
                            value: "Tooltip for you, "
                        },
                        ast::PatternElement::Placeable {
                            expression: ast::Expression::Inline(
                                ast::InlineExpression::VariableReference {
                                    id: ast::Identifier { name: "userName" }
                                }
                            )
                        },
                        ast::PatternElement::TextElement { value: "." },
                    ]
                }
            }],
            comment: Some(ast::Comment {
                content: vec!["This is a message comment"]
            })
        }),
    );
}

#[test]
fn ast_use() {
    let bundle = bundle(FTL);

    let msg = bundle.get_message("hello").expect("Message doesn't exist.");
    let pattern = msg.value().expect("Message has no value.");
    let mut errors = vec![];

    let value = bundle.format_pattern(pattern, None, &mut errors);
    assert_eq!(&value, "Hello World!");

    let mut args = fluent_bundle::FluentArgs::new();
    args.set("userName", "Tom");
    let attr = msg
        .get_attribute("tooltip")
        .expect("Attribute doesn't exist.");

    let value = bundle.format_pattern(attr.value(), Some(&args), &mut errors);
    assert_eq!(&value, "Tooltip for you, Tom.");
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let [message, attr] = resource.two_messages();

    println!("{:#?}", message);
    assert_eq!(
        message,
        Message {
            resource: "test".to_string(),
            comment: vec!["This is a message comment".to_string()],
            id: Id::new_msg("hello"),
            variables: vec![],
        }
    );
    println!("{:#?}", attr);
    assert_eq!(
        attr,
        Message {
            resource: "test".to_string(),
            comment: vec![],
            id: Id::new_attr("hello", "tooltip"),
            variables: vec![Variable {
                id: "userName".to_string(),
                typ: VarType::Any
            }],
        }
    );
}

#[test]
fn typed_gen() {
    assert_gen(module_path!(), "test", FTL);
}
