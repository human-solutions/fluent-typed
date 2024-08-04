mod message;

use fluent_syntax::ast::Resource;

use crate::Message;

pub fn generate_code(resource: Resource<&str>) -> String {
    let messages = resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(m) => Some(Message::parse(m)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let signatures = messages
        .iter()
        .map(|msg| msg.trait_signature())
        .collect::<Vec<_>>()
        .join("\n");

    let impls = messages
        .iter()
        .map(|msg| msg.implementations())
        .collect::<Vec<_>>()
        .join("\n");

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
