//! Behavioral tests for the runtime types (`L10nBundle`, `L10nLanguageVec`).
//!
//! These exercise the actual message-formatting path end to end, including
//! the bidi-isolation behavior that the generated accessors depend on.

use crate::prelude::{FluentArgs, L10nBundle, L10nLanguageVec, Segment};

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

const ELEMENT_FTL: &str = r#"
calendar-sync-description =
    Sync calendar { $num } using { $provider ->
        [google] Google Calendar
       *[other] external calendar
    } { $icon } feed, see { -privacy-link } for more information.
-privacy-link = our privacy policy
edge = { $before }{ $count }{ $after }
"#;

#[test]
fn msg_segments_splits_at_element_markers() {
    let bundle = L10nBundle::new("en", ELEMENT_FTL.as_bytes()).unwrap();
    let mut args = FluentArgs::new();
    args.set("num", 2);
    args.set("provider", "google");

    let segments = bundle
        .msg_segments(
            "calendar-sync-description",
            &["icon"],
            &["privacy-link"],
            Some(args),
        )
        .unwrap();

    // Text segments resolve with selects evaluated and variables bidi-isolated;
    // the variable element is a gap, the term element carries resolved text.
    assert_eq!(
        segments,
        vec![
            Segment::Text(
                "Sync calendar \u{2068}2\u{2069} using \u{2068}Google Calendar\u{2069} "
                    .to_string()
            ),
            Segment::Gap,
            Segment::Text(" feed, see ".to_string()),
            Segment::Term("our privacy policy".to_string()),
            Segment::Text(" for more information.".to_string()),
        ]
    );
}

#[test]
fn msg_segments_pads_lone_placeable_for_bidi_isolation() {
    // `$count` sits alone between two adjacent element markers. Fluent skips
    // bidi isolation for a single-element pattern; the empty-TextElement
    // padding in `resolve_segment` makes the lone placeable isolate exactly
    // as it would in the whole, un-split message.
    let bundle = L10nBundle::new("en", ELEMENT_FTL.as_bytes()).unwrap();
    let mut args = FluentArgs::new();
    args.set("count", 5);

    let segments = bundle
        .msg_segments("edge", &["before", "after"], &[], Some(args))
        .unwrap();

    assert_eq!(
        segments,
        vec![
            Segment::Text(String::new()),
            Segment::Gap,
            Segment::Text("\u{2068}5\u{2069}".to_string()),
            Segment::Gap,
            Segment::Text(String::new()),
        ]
    );
}

#[test]
fn number_builtin_is_registered() {
    // `NUMBER()` is an FTL builtin; it must be registered on the bundle or the
    // message fails to format and `msg` returns an `Err`. See README
    // "Type deduction".
    let ftl = "dpi-ratio = Your DPI ratio is { NUMBER($ratio) }\n";
    let bundle = L10nBundle::new("en", ftl.as_bytes()).unwrap();
    let mut args = FluentArgs::new();
    args.set("ratio", 2);
    assert_eq!(
        bundle.msg("dpi-ratio", Some(args)).unwrap(),
        "Your DPI ratio is \u{2068}2\u{2069}"
    );
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
