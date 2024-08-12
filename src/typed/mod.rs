mod parse_ast;
mod type_in_comment;

pub use crate::gen::ext::BundleMessageExt;
pub use crate::gen::{generate_code, generate_for_messages, to_messages};

use crate::ext::StrExt;

#[derive(Debug, PartialEq)]
pub struct Message {
    pub id: Id,
    pub comment: Vec<String>,
    pub variables: Vec<Variable>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Id {
    pub resource: Option<String>,
    pub message: String,
    pub attribute: Option<String>,
}

impl Id {
    pub fn new_attr(message: &str, attribute: &str) -> Self {
        Self {
            resource: None,
            message: message.to_owned(),
            attribute: Some(attribute.to_owned()),
        }
    }

    pub fn new_msg(message: &str) -> Self {
        Self {
            resource: None,
            message: message.to_owned(),
            attribute: None,
        }
    }

    pub fn new_resource_msg(resource: &str, message: &str) -> Self {
        Self {
            resource: Some(resource.to_owned()),
            message: message.to_owned(),
            attribute: None,
        }
    }
    pub fn func_name(&self) -> String {
        let res = self
            .resource
            .as_ref()
            .map(|r| format!("{r}_"))
            .unwrap_or_default();
        let atr = self
            .attribute
            .as_ref()
            .map(|a| format!("_{a}"))
            .unwrap_or_default();
        format!("{res}{}{atr}", self.message).rust_id()
    }
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub id: String,
    pub variables: Vec<Variable>,
}

#[derive(Debug, PartialEq)]
pub struct Variable {
    pub id: String,
    pub typ: VarType,
}

#[derive(Debug, PartialEq)]
pub enum VarType {
    Any,
    String,
    Number,
}
