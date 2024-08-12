pub mod ext;
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
    let base_ext = include_str!("ext.rs");
    let base_use = base_ext
        .lines()
        .filter(|l| l.starts_with("use"))
        .collect::<Vec<_>>()
        .join("\n");

    let base_code = base_ext
        .lines()
        .filter(|l| !l.starts_with("use"))
        .collect::<Vec<_>>()
        .join("\n");

    let signatures = signatures.join("\n");
    let impls = impls.join("\n");
    format!(
        r#"#[allow(unused_imports)]
use fluent_bundle::{{types::FluentNumber, FluentValue}};
{base_use}

pub trait TypedMessages {{
{signatures}
}}

impl TypedMessages for FluentBundle<FluentResource> {{
{impls}
}}
{base_code}
"#
    )
}
