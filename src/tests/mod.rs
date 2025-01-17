mod ast;
mod complex;
mod gen;

use std::fs;

use crate::{build::Builder, BuildOptions, FtlOutputOptions};

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
        .with_prefix("");

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
        .with_prefix("");

    Builder::load(options).unwrap().generate().unwrap();

    let formated_rust_file =
        fs::read_to_string("src/tests/gen/test_format_generated_rust_file_gen.rs").unwrap();

    assert_ne!(unformated_rust_file, formated_rust_file);

    insta::assert_snapshot!(formated_rust_file);
}

// #[test]
// fn test_locales_ld() {
//     let locales = build::from_locales_folder("../../../LeaveDates/frontend/app/locales").unwrap();
//     let analyzed = build::analyze(&locales);
//     let locales = build::generate_from_locales(&locales, &analyzed).unwrap();
//     write_generated("ld", true, &locales).unwrap();
// }
