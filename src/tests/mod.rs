mod ast;
mod complex;
mod gen;

use crate::typed::generate_code;
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
    let mod_name = module.split("::").last().unwrap();
    let file = format!("src/tests/gen/{mod_name}_gen.rs");
    let path = PathBuf::from(file);
    let resource = parser::parse(ftl).expect("Failed to parse an FTL string.");

    let generated = generate_code(resource_name, resource);

    if update || !path.exists() {
        fs::write(&path, generated).unwrap();
    } else {
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, generated);
    }
}
