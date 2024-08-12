mod message;

use fluent_syntax::ast::Resource;

use crate::Message;

pub fn to_messages(name: &Option<String>, resource: Resource<&str>) -> Vec<Message> {
    resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(m) => Some(Message::parse(name.clone(), m)),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn generate_code(name: &Option<String>, resource: Resource<&str>) -> String {
    let messages = to_messages(name, resource);
    generate_for_messages(messages.iter())
}

pub fn generate_for_messages<'a>(messages: impl Iterator<Item = &'a Message>) -> String {
    let mut signatures = vec![];
    let mut impls = vec![];
    for msg in messages {
        signatures.push(msg.trait_signature());
        impls.push(msg.implementations());
    }
    format_generated(&signatures, &impls)
}

pub fn format_generated(signatures: &[String], impls: &[String]) -> String {
    let signatures = signatures.join("\n");
    let impls = impls.join("\n");
    format!(
        r#"use crate::BundleMessageExt;
#[allow(unused_imports)]
use fluent_bundle::{{types::FluentNumber, FluentArgs, FluentBundle, FluentResource, FluentValue}};
use std::borrow::Cow;

pub trait MyExt {{
{signatures}
}}

impl MyExt for FluentBundle<FluentResource> {{
{impls}
}}
"#
    )
}
