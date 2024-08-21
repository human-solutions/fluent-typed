use super::assert_gen;
use super::bundle;
use crate::build::typed::*;
use crate::tests::ast::AstResourceExt;
use fluent_syntax::ast;
use fluent_syntax::parser;

const FTL: &str = r#"

# $name (String) - The name.
greeting = Hi { $name }

"#;

/// From: https://docs.rs/fluent-syntax/0.11.1/fluent_syntax/
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");

    assert_eq!(
        resource.body[0],
        ast::Entry::Message(ast::Message {
            comment: Some(ast::Comment {
                content: vec!["$name (String) - The name."]
            }),
            id: ast::Identifier { name: "greeting" },
            value: Some(ast::Pattern {
                elements: vec![
                    ast::PatternElement::TextElement { value: "Hi " },
                    ast::PatternElement::Placeable {
                        expression: ast::Expression::Inline(
                            ast::InlineExpression::VariableReference {
                                id: ast::Identifier { name: "name" }
                            }
                        )
                    },
                ]
            }),
            attributes: vec![],
        }),
    );
}

#[test]
fn ast_use() {
    let bundle = bundle(FTL);

    let msg = bundle
        .get_message("greeting")
        .expect("Message doesn't exist.");
    let pattern = msg.value().expect("Message has no value.");
    let mut errors = vec![];

    let mut args = fluent_bundle::FluentArgs::new();
    args.set("name", "Tom");
    let value = bundle.format_pattern(pattern, Some(&args), &mut errors);
    assert_eq!(&value, "Hi Tom");
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = resource.first_message();

    println!("{:#?}", message);
    assert_eq!(
        message,
        Message {
            comment: vec!["$name (String) - The name.".to_string()],
            id: Id::new_msg("greeting"),
            variables: vec![Variable {
                id: "name".to_string(),
                typ: VarType::String,
            }],
        }
    );
}

#[test]
fn typed_gen() {
    assert_gen(module_path!(), None, true, FTL);
}
