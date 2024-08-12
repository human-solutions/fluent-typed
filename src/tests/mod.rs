mod ast;
mod complex;
mod gen;

use crate::{generate_from_locales, typed::generate_code};
use fluent_bundle::{FluentBundle, FluentResource};
use fluent_syntax::parser;
use std::{fs, path::PathBuf};
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
fn assert_gen(module: &str, resource_name: Option<&str>, update: bool, ftl: &str) {
    let resource = parser::parse(ftl).expect("Failed to parse an FTL string.");
    let generated = generate_code(&resource_name.map(|s| s.to_owned()), resource);

    if let Some(current) = write_generated(module, update, &generated).unwrap() {
        assert_eq!(current, generated);
    }
}

fn write_generated(
    module: &str,
    update: bool,
    content: &str,
) -> Result<Option<String>, std::io::Error> {
    let mod_name = module.split("::").last().unwrap();
    let file = format!("src/tests/gen/{mod_name}_gen.rs");
    let path = PathBuf::from(file);

    if update || !path.exists() {
        fs::write(&path, content)?;
        Ok(None)
    } else {
        Ok(Some(fs::read_to_string(&path)?))
    }
}

#[test]
fn test_locales_folder() {
    let locales = generate_from_locales("src/tests/test_locales").unwrap();
    write_generated("locales_folder", true, &locales).unwrap();
}

#[test]
fn test_locales_multi_resources() {
    let locales = generate_from_locales("src/tests/test_locales_multi_resources").unwrap();
    write_generated("locales_multi_resources", true, &locales).unwrap();
}
