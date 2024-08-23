mod ext;
mod message;
#[allow(dead_code, unused_mut, unused_imports)]
mod template;

use super::Message;
pub use ext::StrExt;
use fluent_syntax::ast::Resource;

pub fn to_messages(name: &str, resource: Resource<&str>) -> Vec<Message> {
    resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(m) => Some(Message::parse(name, m)),
            _ => None,
        })
        .flatten()
        .collect()
}

#[cfg(test)]
pub fn generate_code(name: &str, resource: Resource<&str>) -> String {
    let messages = to_messages(name, resource);
    generate(&["base"], messages.iter())
}

pub fn generate<'a>(resources: &[&str], messages: impl Iterator<Item = &'a Message>) -> String {
    let res_def = resources
        .iter()
        .map(|res| format!("    pub {}: String,", res.rust_id()))
        .collect::<Vec<_>>()
        .join("\n");
    let res_decomp = resources
        .iter()
        .map(|res| format!("        vec.push(self.{});", res.rust_id()))
        .collect::<Vec<_>>()
        .join("\n");
    let impls = messages
        .map(|msg| msg.implementations())
        .collect::<Vec<_>>()
        .join("\n");

    let base = include_str!("template.rs");
    let base = base
        .replace("    // <<message implementations>>", &impls)
        .replace("    // <<resource definitions>>", &res_def)
        .replace("        // <<resource decomposition>>", &res_decomp);

    #[cfg(not(test))]
    let base = base.replace("use crate::prelude::*;", "use fluent_typed::prelude::*;");
    base
}
