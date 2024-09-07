use std::{fs, path::PathBuf};

use common::{cargo, ls_ascii};
use insta::assert_snapshot;

mod common;

#[test]
fn build_example1() {
    let root = PathBuf::from("playground/example1");
    let l10n = root.join("src/l10n.rs");
    if l10n.exists() {
        fs::remove_file(&l10n).unwrap();
    }

    let target = root.join("target");
    if target.exists() {
        fs::remove_dir_all(&target).unwrap();
    }

    let gen = root.join("gen");
    if gen.exists() {
        fs::remove_dir_all(&gen).unwrap();
    }
    cargo(&root, ["build"]);

    assert!(l10n.exists());

    let listing = ls_ascii(&gen, 0).unwrap();
    assert_snapshot!(&listing, @r#"
        gen/
          translations.ftl (401 bytes)
          translations.ftl.gzip (181 bytes)
          multi/
            en.ftl (185 bytes)
            fr.ftl (216 bytes)
    "#);
}
