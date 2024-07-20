mod msg_number;
mod msg_select;
mod msg_string;
mod msg_text;
mod msg_with_attrib;
mod msg_with_var;
mod out;

use std::{fs, path::PathBuf};

use fluent_bundle::{FluentBundle, FluentResource};
use fluent_syntax::ast;
use unic_langid::langid;

use crate::{typed::generate_extension, Message};

#[test]
fn load_bundle() {
    let ftl_string = "hello-world = Hello, world!";
    let bundle = bundle(ftl_string);

    let msg = bundle
        .get_message("hello-world")
        .expect("Message doesn't exist.");
    let mut errors = vec![];
    let pattern = msg.value().expect("Message has no value.");
    let value = bundle.format_pattern(pattern, None, &mut errors);

    assert_eq!(&value, "Hello, world!");
}

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
fn assert_gen(module: &str, update: bool, ftl: &str) {
    let mod_name = module.split("::").last().unwrap();
    let file = format!("src/tests/ast/out/{mod_name}_gen.rs");
    let path = PathBuf::from(file);

    let generated = generate_extension(ftl);

    if update || !path.exists() {
        fs::write(&path, generated).unwrap();
    } else {
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, generated);
    }
}

trait AstResourceExt {
    fn first_message(&self) -> Message;
}

impl AstResourceExt for ast::Resource<&str> {
    fn first_message(&self) -> Message {
        match &self.body[0] {
            ast::Entry::Message(message) => Message::parse(message),
            _ => panic!("Expected a message."),
        }
    }
}
