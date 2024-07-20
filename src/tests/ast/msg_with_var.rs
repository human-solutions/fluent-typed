use super::assert_gen;
use super::bundle;
use crate::tests::ast::AstResourceExt;
use crate::typed::*;
use fluent_syntax::ast;
use fluent_syntax::parser;

const FTL: &str = r#"

hello = Hi { $first-name }!

"#;

/// From: https://docs.rs/fluent-syntax/0.11.1/fluent_syntax/
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");

    assert_eq!(
        resource.body[0],
        ast::Entry::Message(ast::Message {
            id: ast::Identifier { name: "hello" },
            value: Some(ast::Pattern {
                elements: vec![
                    ast::PatternElement::TextElement { value: "Hi " },
                    ast::PatternElement::Placeable {
                        expression: ast::Expression::Inline(
                            ast::InlineExpression::VariableReference {
                                id: ast::Identifier { name: "first-name" }
                            }
                        )
                    },
                    ast::PatternElement::TextElement { value: "!" },
                ]
            }),
            attributes: vec![],
            comment: None,
        }),
    );
}

#[test]
fn ast_use() {
    let bundle = bundle(FTL);

    let msg = bundle.get_message("hello").expect("Message doesn't exist.");
    let pattern = msg.value().expect("Message has no value.");
    let mut errors = vec![];

    let mut args = fluent_bundle::FluentArgs::new();
    args.set("first-name", "Tom");
    let value = bundle.format_pattern(pattern, Some(&args), &mut errors);
    assert_eq!(&value, "Hi Tom!");
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = resource.first_message();

    println!("{:#?}", message);
    assert_eq!(
        message,
        Message {
            id: "hello",
            comment: vec![],
            variables: vec![Variable {
                id: "first-name",
                typ: VarType::Any
            }],
            attributes: vec![],
        }
    );
}

#[test]
fn typed_gen() {
    assert_gen(module_path!(), true, FTL);
}
