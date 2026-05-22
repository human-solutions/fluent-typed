use super::assert_gen;
use crate::build::typed::*;
use crate::tests::ast::AstResourceExt;
use fluent_syntax::parser;

const FTL: &str = r#"
# $num (Number) - How many.
# $provider (String) - The calendar provider.
# $icon (Element) - A UI element injected by the app.
# -privacy-link (Element) - Translatable text the app wraps.
calendar-sync-description =
    Sync calendar { $num } using { $provider ->
        [google] Google Calendar
       *[other] external calendar
    } { $icon } feed, see { -privacy-link } for more information.
-privacy-link = our privacy policy
"#;

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = resource.first_message(FTL);

    println!("{message:#?}");
    assert_eq!(
        message,
        Message {
            id: Id::new_msg("calendar-sync-description"),
            comment: vec![
                "$num (Number) - How many.".to_string(),
                "$provider (String) - The calendar provider.".to_string(),
                "$icon (Element) - A UI element injected by the app.".to_string(),
                "-privacy-link (Element) - Translatable text the app wraps.".to_string(),
            ],
            // Element variables ($icon) are positional gaps, not arguments.
            variables: vec![
                Variable {
                    id: "num".to_string(),
                    typ: VarType::Number,
                },
                Variable {
                    id: "provider".to_string(),
                    typ: VarType::String,
                },
            ],
            elements: vec![
                ElementMarker {
                    name: "icon".to_string(),
                    kind: ElementKind::Variable,
                },
                ElementMarker {
                    name: "privacy-link".to_string(),
                    kind: ElementKind::Term,
                },
            ],
            // Every placeable, comment-independent and in document order.
            pattern_refs: vec![
                Ref {
                    name: "num".to_string(),
                    kind: RefKind::Variable,
                },
                Ref {
                    name: "provider".to_string(),
                    kind: RefKind::Variable,
                },
                Ref {
                    name: "icon".to_string(),
                    kind: RefKind::Variable,
                },
                Ref {
                    name: "privacy-link".to_string(),
                    kind: RefKind::Term,
                },
            ],
            file: String::new(),
            line: 0,
            comment_line: 0,
        }
    );
}

#[test]
fn typed_gen() {
    assert_gen(module_path!(), "test", FTL);
}
