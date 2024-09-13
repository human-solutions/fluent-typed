mod ast;
mod complex;
mod gen;

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

// #[test]
// fn test_locales_ld() {
//     let locales = build::from_locales_folder("../../../LeaveDates/frontend/app/locales").unwrap();
//     let analyzed = build::analyze(&locales);
//     let locales = build::generate_from_locales(&locales, &analyzed).unwrap();
//     write_generated("ld", true, &locales).unwrap();
// }
