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

    let gen = root.join("gen");
    if gen.exists() {
        fs::remove_dir_all(&gen).unwrap();
    }
    cargo(&root, ["build"]);

    assert!(single_l10n.exists());
    assert!(single_gzip_l10n.exists());
    assert!(multi_l10n.exists());

    let listing = ls_ascii(&gen, 0).unwrap();
    assert_snapshot!(&listing, @r###"
    gen/
      translations.ftl (401 bytes)
      translations.ftl.gzip (184 bytes)
      multi/
        en.ftl (185 bytes)
        fr.ftl (216 bytes)
    "###);
}
