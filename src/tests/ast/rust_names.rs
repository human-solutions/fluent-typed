//! Fluent names that aren't valid Rust as they stand: keywords, a variable
//! named like a local of the generated accessor, and a language name with
//! characters that need escaping in a string literal. The generated module is
//! compiled (see `tests::gen`), so this fails if any of it stops compiling.

use super::assert_gen;

const FTL: &str = r#"
language-name = Say "hi" \ bye

# $type (String) - a keyword
kind = Kind: { $type }

# $self (Number) - a keyword that can't be a raw identifier
count = { $self } items

# $args (String) - the name of the accessor's argument list
# $match (String) - another keyword
args = { $args } and { $match }

menu = Menu
    .tooltip = Tooltip for { $type }

# $loop (Element) - a link
# $segments (String) - the name of the accessor's segment array
nav = Go { $loop } to { $segments }
"#;

#[test]
fn typed_gen() {
    assert_gen(module_path!(), "test", FTL);
}
