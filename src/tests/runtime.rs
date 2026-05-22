//! Behavioral tests for the runtime types (`L10nBundle`, `L10nLanguageVec`).
//!
//! These exercise the actual message-formatting path end to end, including
//! the bidi-isolation behavior that the generated accessors depend on.

use crate::prelude::{FluentArgs, L10nBundle, L10nLanguageVec};

const FTL: &str = r#"
greeting = Hello world
hello = Hi { $name }!
tip =
    .label = Tip for { $name }
"#;

#[test]
fn msg_without_variables() {
    let bundle = L10nBundle::new("en", FTL.as_bytes()).unwrap();
    assert_eq!(bundle.msg("greeting", None).unwrap(), "Hello world");
}

#[test]
fn msg_with_variable_is_bidi_isolated_by_default() {
    let bundle = L10nBundle::new("en", FTL.as_bytes()).unwrap();
    let mut args = FluentArgs::new();
    args.set("name", "world");
    // By default, interpolated variables are wrapped in Unicode bidi
    // isolation marks: FSI (U+2068) ... PDI (U+2069).
    assert_eq!(
        bundle.msg("hello", Some(args)).unwrap(),
        "Hi \u{2068}world\u{2069}!"
    );
}

#[test]
fn msg_with_variable_without_isolation() {
    let bundle = L10nBundle::new_without_isolation("en", FTL.as_bytes()).unwrap();
    let mut args = FluentArgs::new();
    args.set("name", "world");
    assert_eq!(bundle.msg("hello", Some(args)).unwrap(), "Hi world!");
}

#[test]
fn attribute_message() {
    let bundle = L10nBundle::new_without_isolation("en", FTL.as_bytes()).unwrap();
    let mut args = FluentArgs::new();
    args.set("name", "Sam");
    assert_eq!(
        bundle.attr("tip", "label", Some(args)).unwrap(),
        "Tip for Sam"
    );
}

#[test]
fn unknown_message_is_an_error() {
    let bundle = L10nBundle::new("en", FTL.as_bytes()).unwrap();
    assert!(bundle.msg("does-not-exist", None).is_err());
}

#[test]
fn language_vec_loads_each_range() {
    let en = "greeting = Hello\n";
    let de = "greeting = Hallo\n";
    let mut data = Vec::new();
    data.extend_from_slice(en.as_bytes());
    data.extend_from_slice(de.as_bytes());

    let ranges = [("en", 0..en.len()), ("de", en.len()..data.len())];
    let vec = L10nLanguageVec::load(&data, ranges.into_iter()).unwrap();

    assert_eq!(vec.get("en").msg("greeting", None).unwrap(), "Hello");
    assert_eq!(vec.get("de").msg("greeting", None).unwrap(), "Hallo");
}
