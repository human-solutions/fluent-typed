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
        .filter_map(|msg| msg.variables.as_ref().map(|vars| msg.gen_signature(vars)))
        .collect::<Vec<_>>()
        .join("\n");

    let impls = messages
        .iter()
        .map(|msg| msg.gen_implementation())
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
