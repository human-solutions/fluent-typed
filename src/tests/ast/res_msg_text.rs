use crate::build::typed::*;
use crate::tests::ast::assert_gen;
use crate::tests::ast::AstResourceExt;
use fluent_syntax::parser;

const FTL: &str = r#"

hello-world = Hello World!

"#;

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
