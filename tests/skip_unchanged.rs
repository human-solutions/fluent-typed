//! A no-op rebuild must not rewrite any output file: consumers watch the
//! generated files (`cargo::rerun-if-changed`, dev-loop file watchers), so a
//! bumped mtime retriggers them even when the bytes are identical.
//!
//! The regression this pins down: the skip-if-unchanged guard used to compare
//! the *pre*-rustfmt text against the *post*-rustfmt file on disk, so with
//! `format` on (the default) and a custom indentation the guard never fired
//! and every build rewrote the file. The `.ftl` outputs were rewritten
//! unconditionally.

use std::{fs, thread, time::Duration, time::SystemTime};

use fluent_typed::{BuildOptions, FtlOutputOptions, try_build_from_locales_folder};

fn mtime(path: &str) -> SystemTime {
    fs::metadata(path).unwrap().modified().unwrap()
}

#[test]
fn rebuild_with_unchanged_input_leaves_outputs_untouched() {
    let dir = "target/test-skip-unchanged";
    fs::create_dir_all(dir).unwrap();
    let rs = format!("{dir}/l10n.rs");
    let ftl = format!("{dir}/translations.ftl");

    // The exact repro: format on (the default) together with a custom
    // indentation, which rustfmt rewrites back to 4 spaces.
    let options = || {
        BuildOptions::default()
            .with_locales_folder("playground/example1/locales")
            .with_output_file_path(&rs)
            .with_ftl_output(FtlOutputOptions::single_file(&ftl))
            .with_indentation("  ")
    };

    // Clean slate so the first build genuinely writes both files.
    let _ = fs::remove_file(&rs);
    let _ = fs::remove_file(&ftl);

    try_build_from_locales_folder(options()).unwrap();

    let rs_bytes = fs::read(&rs).unwrap();
    let ftl_bytes = fs::read(&ftl).unwrap();
    let rs_mtime = mtime(&rs);
    let ftl_mtime = mtime(&ftl);

    // Filesystem mtime granularity can be a full second — make sure a rewrite
    // in the second build could not hide behind an identical timestamp.
    thread::sleep(Duration::from_millis(1100));

    try_build_from_locales_folder(options()).unwrap();

    assert_eq!(rs_bytes, fs::read(&rs).unwrap(), "generated .rs changed");
    assert_eq!(ftl_bytes, fs::read(&ftl).unwrap(), "generated .ftl changed");
    assert_eq!(
        rs_mtime,
        mtime(&rs),
        "the unchanged .rs output was rewritten"
    );
    assert_eq!(
        ftl_mtime,
        mtime(&ftl),
        "the unchanged .ftl output was rewritten"
    );
}
