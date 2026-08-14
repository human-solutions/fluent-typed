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
mod test_locales_gen;
mod test_locales_missing_msg_gen;
mod test_locales_multi_resources_gen;

#[test]
fn generated_bool_accessor_selects_both_branches() {
    let strings = msg_string_gen::L10n::En.load();
    assert_eq!(strings.msg_feature_status(true), "Enabled");
    assert_eq!(strings.msg_feature_status(false), "Disabled");
}
