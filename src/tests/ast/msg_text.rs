use crate::OutputMode;
use crate::build::typed::*;
use crate::tests::ast::AstResourceExt;
use crate::tests::ast::{assert_gen, assert_gen_with_output_mode};
use fluent_syntax::ast;
use fluent_syntax::parser;

use super::bundle;

const FTL: &str = r#"

hello-world = Hello World!

"#;

/// From: https://docs.rs/fluent-syntax/0.11.1/fluent_syntax/
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");

    assert_eq!(
        resource.body[0],
        ast::Entry::Message(ast::Message {
            id: ast::Identifier {
                name: "hello-world"
            },
            value: Some(ast::Pattern {
                elements: vec![ast::PatternElement::TextElement {
                    value: "Hello World!"
                },]
            }),
            attributes: vec![],
            comment: None,
        }),
    );
}

#[test]
fn ast_use() {
    let bundle = bundle(FTL);

    let msg = bundle
        .get_message("hello-world")
        .expect("Message doesn't exist.");
    let pattern = msg.value().expect("Message has no value.");
    let mut errors = vec![];
    let value = bundle.format_pattern(pattern, None, &mut errors);
    assert_eq!(&value, "Hello World!");
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
            id: Id::new_msg("hello-world"),
            comment: vec![],
            variables: vec![],
        }
    );
}

#[test]
fn typed_gen() {
    assert_gen(module_path!(), "test", FTL);
}

#[test]
fn typed_gen_pattern() {
    assert_gen_with_output_mode(
        module_path!(),
        "pattern",
        "test",
        FTL,
        OutputMode::default_pattern(),
    );
}

#[test]
fn typed_gen_both() {
    assert_gen_with_output_mode(
        module_path!(),
        "both",
        "test",
        FTL,
        OutputMode::default_both(),
    );
}
