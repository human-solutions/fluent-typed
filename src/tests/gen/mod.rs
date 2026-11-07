// These are generated fixtures; the manual `impl Default` is intentional
// (the default language is chosen at generation time).
#![allow(unused, clippy::derivable_impls)]
mod attrib_only_gen;
mod complex_gen;
mod msg_element_gen;
mod msg_number_gen;
mod msg_string_gen;
mod msg_text_gen;
mod msg_with_attrib_gen;
mod msg_with_var_gen;
mod res_msg_text_gen;
mod rust_names_gen;
mod test_locales_gen;
mod test_locales_missing_msg_gen;
mod test_locales_multi_resources_gen;

/// Keyword-named variables, a variable named like the accessor's own locals,
/// and a language name that needs escaping all compile and render.
#[test]
fn generated_names_are_escaped() {
    use rust_names_gen::{L10n, Nav};
    let strs = L10n::En.load();
    let plain = |s: String| s.replace(['\u{2068}', '\u{2069}'], "");
    assert_eq!(L10n::En.language_name(), r#"Say "hi" \ bye"#);
    assert_eq!(plain(strs.msg_kind("noun")), "Kind: noun");
    assert_eq!(plain(strs.msg_count(3)), "3 items");
    assert_eq!(plain(strs.msg_args("a", "b")), "a and b");
    assert_eq!(plain(strs.msg_menu_tooltip("x")), "Tooltip for x");
    let nav = strs.msg_nav("home");
    assert_eq!(nav.s0, "Go ");
    assert_eq!(plain(nav.s1), " to home");
    let _: Nav = strs.msg_nav("home");
}

#[test]
fn generated_bool_accessor_selects_both_branches() {
    let strings = msg_string_gen::L10n::En.load();
    assert_eq!(strings.msg_feature_status(true), "Enabled");
    assert_eq!(strings.msg_feature_status(false), "Disabled");
}
