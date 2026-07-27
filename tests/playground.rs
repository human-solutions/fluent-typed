use std::{fs, path::PathBuf};

use common::{cargo, ls_ascii};
use insta::assert_snapshot;

mod common;

#[test]
fn build_example1() {
    let root = PathBuf::from("playground/example1");
    let single_l10n = root.join("src/single_l10n.rs");
    if single_l10n.exists() {
        fs::remove_file(&single_l10n).unwrap();
    }
    let single_gzip_l10n = root.join("src/single_gzip_l10n.rs");
    if single_gzip_l10n.exists() {
        fs::remove_file(&single_gzip_l10n).unwrap();
    }
    let multi_l10n = root.join("src/multi_l10n.rs");
    if multi_l10n.exists() {
        fs::remove_file(&multi_l10n).unwrap();
    }

    let target = root.join("target");
    if target.exists() {
        fs::remove_dir_all(&target).unwrap();
    }

    let r#gen = root.join("gen");
    if r#gen.exists() {
        fs::remove_dir_all(&r#gen).unwrap();
    }
    cargo(&root, ["build"]);

    assert!(single_l10n.exists());
    assert!(single_gzip_l10n.exists());
    assert!(multi_l10n.exists());

    let listing = ls_ascii(&r#gen, 0).unwrap();
    assert_snapshot!(&listing, @r###"
    gen/
      translations.ftl (452 bytes)
      translations.ftl.gzip (209 bytes)
      multi/
        en.ftl (209 bytes)
        fr.ftl (243 bytes)
    "###);
}

/// Issue #37: in a workspace, the generated `.rs` and the output `.ftl` live in
/// different directories, so the `include_bytes!` path has to climb out of
/// `src/`. On Windows that path was emitted with `\` separators, which are
/// invalid escape sequences in the string literal and failed the build.
#[test]
fn build_example2_workspace() {
    let root = PathBuf::from("playground/example2");
    let l10n = root.join("game/src/l10n.rs");
    if l10n.exists() {
        fs::remove_file(&l10n).unwrap();
    }
    for dir in [root.join("target"), root.join("gen")] {
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
    }

    cargo(&root, ["test"]);

    let generated = fs::read_to_string(&l10n).unwrap();
    // The include path must use forward slashes on every platform — `\` would
    // be an escape sequence inside the literal.
    assert!(
        generated.contains(r#"include_bytes!("../../gen/translations.ftl")"#),
        "expected a forward-slash include path, got: {}",
        generated
            .lines()
            .find(|l| l.contains("include_bytes!"))
            .unwrap_or("<no include_bytes! line>")
    );
}
