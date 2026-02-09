mod attrib_only;
mod msg_number;
mod msg_select;
mod msg_select_num;
mod msg_string;
mod msg_text;
mod msg_with_attrib;
mod msg_with_var;
mod res_msg_text;

use super::{assert_gen, assert_gen_with_output_mode, bundle};
use crate::build::Message;
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
    fn first_message_in_resource<'a>(&'a self, resource: &'a str) -> Message;
    fn first_message(&self) -> Message;
    fn two_messages(&self) -> [Message; 2];
}

impl AstResourceExt for ast::Resource<&str> {
    fn first_message_in_resource<'a>(&'a self, resource: &'a str) -> Message {
        match &self.body[0] {
            ast::Entry::Message(message) => Message::parse(resource, message)
                .into_iter()
                .next()
                .unwrap(),
            _ => panic!("Expected a message."),
        }
    }
    fn first_message(&self) -> Message {
        match &self.body[0] {
            ast::Entry::Message(message) => {
                Message::parse("test", message).into_iter().next().unwrap()
            }
            _ => panic!("Expected a message."),
        }
    }

    fn two_messages(&self) -> [Message; 2] {
        match &self.body[0] {
            ast::Entry::Message(message) => {
                let msgs = Message::parse("test", message);
                assert_eq!(msgs.len(), 2);
                msgs.try_into().unwrap()
            }
            _ => panic!("Expected a message."),
        }
    }
}
