mod msg_number;
mod msg_select;
mod msg_select_num;
mod msg_string;
mod msg_text;
mod msg_with_attrib;
mod msg_with_var;

use super::{assert_gen, bundle};
use crate::Message;
use fluent_syntax::ast;

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
