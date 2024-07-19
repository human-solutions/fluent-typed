use super::bundle;
use crate::typed::*;
use fluent_syntax::ast;
use fluent_syntax::parser;

const FTL: &str = r#"

# $duration (Number) - The duration in seconds.
time-elapsed = Time elapsed: { $duration }s.

"#;

/// From: https://docs.rs/fluent-syntax/0.11.1/fluent_syntax/
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");

    assert_eq!(
        resource.body[0],
        ast::Entry::Message(ast::Message {
            comment: Some(ast::Comment {
                content: vec!["$duration (Number) - The duration in seconds."]
            }),
            id: ast::Identifier {
                name: "time-elapsed"
            },
            value: Some(ast::Pattern {
                elements: vec![
                    ast::PatternElement::TextElement {
                        value: "Time elapsed: "
                    },
                    ast::PatternElement::Placeable {
                        expression: ast::Expression::Inline(
                            ast::InlineExpression::VariableReference {
                                id: ast::Identifier { name: "duration" }
                            }
                        )
                    },
                    ast::PatternElement::TextElement { value: "s." },
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
        .get_message("time-elapsed")
        .expect("Message doesn't exist.");
    let pattern = msg.value().expect("Message has no value.");
    let mut errors = vec![];

    let mut args = fluent::FluentArgs::new();
    args.set("duration", 10.2);
    let value = bundle.format_pattern(pattern, Some(&args), &mut errors);
    assert_eq!(&value, "Time elapsed: 10.2s.");
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = match &resource.body[0] {
        ast::Entry::Message(message) => Message::parse(message),
        _ => panic!("Expected a message."),
    };

    println!("{:#?}", message);
    assert_eq!(
        message,
        Message {
            comment: vec!["$duration (Number) - The duration in seconds.",],
            id: "time-elapsed",
            variables: vec![Variable {
                id: "duration",
                typ: VarType::Number,
            },],
            attributes: vec![],
        }
    );
}
