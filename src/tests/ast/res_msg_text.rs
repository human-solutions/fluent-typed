use crate::build::typed::*;
use crate::tests::ast::AstResourceExt;
use crate::tests::ast::assert_gen;
use fluent_syntax::ast;
use fluent_syntax::parser;

const FTL: &str = r#"

hello-world = Hello World!

"#;

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
            comment: None
        }),
    );
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = resource.first_message_in_resource("cookie-disclaimer");

    println!("{:#?}", message);
    assert_eq!(
        message,
        Message {
            resource: "cookie-disclaimer".to_string(),
            id: Id::new_msg("hello-world"),
            comment: vec![],
            variables: vec![],
        }
    );
}

#[test]
fn typed_gen() {
    assert_gen(module_path!(), "cookie-disclaimer", FTL);
}
