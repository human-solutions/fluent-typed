mod ast;
mod complex;
mod r#gen;

use std::fs;

use crate::{BuildError, BuildOptions, FtlOutputOptions, OutputMode, build::Builder};

use fluent_bundle::{FluentBundle, FluentResource};
use unic_langid::langid;

fn bundle(ftl: &str) -> FluentBundle<FluentResource> {
    let res = FluentResource::try_new(ftl.to_string()).expect("Failed to parse an FTL string.");

    let langid_en = langid!("en-US");
    let mut bundle = FluentBundle::new(vec![langid_en]);
    bundle.set_use_isolating(false);

    bundle
        .add_resource(res)
        .expect("Failed to add FTL resources to the bundle.");
    bundle
}

#[track_caller]
fn assert_gen(module: &str, resource_name: &str, ftl: &str) {
    let mod_name = module.split("::").last().unwrap();
    let file = format!("src/tests/gen/{mod_name}_gen.rs");
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: format!("src/tests/gen/{mod_name}_gen.ftl"),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_output_file_path(&file)
        .with_ftl_output(ftl_opts);

    let builder = Builder::load_one(options, resource_name, "en", ftl).unwrap();
    builder.generate().unwrap();
}

#[track_caller]
fn assert_gen_with_output_mode(
    module: &str,
    suffix: &str,
    resource_name: &str,
    ftl: &str,
    output_mode: OutputMode,
) {
    let mod_name = module.split("::").last().unwrap();
    let file = format!("src/tests/gen/{mod_name}_{suffix}_gen.rs");
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: format!("src/tests/gen/{mod_name}_{suffix}_gen.ftl"),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_output_file_path(&file)
        .with_ftl_output(ftl_opts)
        .with_output_mode(output_mode);

    let builder = Builder::load_one(options, resource_name, "en", ftl).unwrap();
    builder.generate().unwrap();
}

#[test]
fn test_locales_folder() {
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: format!("src/tests/gen/test_locales.ftl"),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_locales_folder("src/tests/test_locales")
        .with_ftl_output(ftl_opts)
        .with_output_file_path("src/tests/gen/test_locales_gen.rs")
        .with_default_language("en-gb");
    Builder::load(options).unwrap().generate().unwrap();
}

#[test]
fn test_locales_multi_resources() {
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: format!("src/tests/gen/test_locales_multi_resources.ftl"),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_locales_folder("src/tests/test_locales_multi_resources")
        .with_ftl_output(ftl_opts)
        .with_output_file_path("src/tests/gen/test_locales_multi_resources_gen.rs")
        .with_default_language("en-gb");

    Builder::load(options).unwrap().generate().unwrap();
}

#[test]
fn test_locales_missing_msg() {
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: format!("src/tests/gen/test_locales_missing_msg.ftl"),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_locales_folder("src/tests/test_locales")
        .with_ftl_output(ftl_opts)
        .with_output_file_path("src/tests/gen/test_locales_missing_msg_gen.rs")
        .with_default_language("en-gb");
    Builder::load(options).unwrap().generate().unwrap();
}

#[test]
fn test_format_generated_rust_file() {
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: "src/tests/gen/test_unformated_generated_rust_file.ftl".to_string(),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_locales_folder("src/tests/test_format_rust_file")
        .with_ftl_output(ftl_opts)
        .with_output_file_path("src/tests/gen/test_unformated_generated_rust_file_gen.rs")
        .with_default_language("en-gb")
        .without_format()
        .with_output_mode(OutputMode::String {
            prefix: "".to_string(),
        });

    Builder::load(options).unwrap().generate().unwrap();

    let unformated_rust_file =
        fs::read_to_string("src/tests/gen/test_unformated_generated_rust_file_gen.rs").unwrap();

    insta::assert_snapshot!(unformated_rust_file);

    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: "src/tests/gen/test_format_generated_rust_file.ftl".to_string(),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_locales_folder("src/tests/test_format_rust_file")
        .with_ftl_output(ftl_opts)
        .with_output_file_path("src/tests/gen/test_format_generated_rust_file_gen.rs")
        .with_default_language("en-gb")
        .with_output_mode(OutputMode::String {
            prefix: "".to_string(),
        });

    Builder::load(options).unwrap().generate().unwrap();

    let formated_rust_file =
        fs::read_to_string("src/tests/gen/test_format_generated_rust_file_gen.rs").unwrap();

    assert_ne!(unformated_rust_file, formated_rust_file);

    insta::assert_snapshot!(formated_rust_file);
}

#[test]
fn test_locales_deep_folders() {
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: "src/tests/gen/test_locales_deep_folders.ftl".to_string(),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_locales_folder("src/tests/test_locales_deep_folders")
        .with_ftl_output(ftl_opts)
        .with_output_file_path("src/tests/gen/test_locales_deep_folders_gen.rs")
        .with_default_language("en");

    // This should successfully load all FTL files from deep folder structure
    Builder::load(options).unwrap().generate().unwrap();

    // Verify the generated file contains messages from all depths
    let generated = fs::read_to_string("src/tests/gen/test_locales_deep_folders_gen.rs").unwrap();

    // Check that all expected message functions were generated
    assert!(generated.contains("fn msg_root_message("));
    assert!(generated.contains("fn msg_level1_hello("));
    assert!(generated.contains("fn msg_level2_greeting("));
    assert!(generated.contains("fn msg_deep_message("));

    // Verify both languages are included
    assert!(generated.contains("L10n::De"));
    assert!(generated.contains("L10n::En"));
}

#[test]
fn test_duplicate_key_fails() {
    let ftl_opts = FtlOutputOptions::SingleFile {
        output_ftl_file: "src/tests/gen/test_duplicate_key.ftl".to_string(),
        compressor: None,
    };
    let options = BuildOptions::default()
        .with_locales_folder("src/tests/test_duplicate_key")
        .with_output_file_path("src/tests/gen/test_duplicate_key_gen.rs")
        .with_ftl_output(ftl_opts)
        .with_default_language("en")
        .with_deny_duplicate_keys();

    if let Err(BuildError::LocalesFolder { source, .. }) = &Builder::load(options)
        && let BuildError::DuplicateKey {
            key,
            original,
            duplicate,
        } = source.as_ref()
    {
        assert_eq!(key, "message 'hello-world'");
        assert!(
            original.ends_with("a.ftl"),
            "expected a.ftl, got {original:?}"
        );
        assert!(
            duplicate.ends_with("b.ftl"),
            "expected b.ftl, got {duplicate:?}"
        );
    } else {
        panic!("Expected a DuplicateKey error");
    }
}

// #[test]
// fn test_locales_ld() {
//     let locales = build::from_locales_folder("../../../LeaveDates/frontend/app/locales").unwrap();
//     let analyzed = build::analyze(&locales);
//     let locales = build::generate_from_locales(&locales, &analyzed).unwrap();
//     write_generated("ld", true, &locales).unwrap();
// }
