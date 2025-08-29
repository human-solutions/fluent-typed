mod parse_ast;
mod type_in_comment;

use std::fmt::Display;

use crate::build::r#gen::StrExt;

#[derive(Debug, PartialEq)]
pub struct Message {
    pub id: Id,
    /// The name of the resource file.
    /// This is used for error messages.
    pub resource: String,
    pub comment: Vec<String>,
    pub variables: Vec<Variable>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Id {
    pub message: String,
    pub attribute: Option<String>,
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = &self.message;
        match &self.attribute {
            Some(a) => write!(f, "message '{msg}' with attribute '{a}'"),
            None => write!(f, "message '{msg}'"),
        }
    }
}

impl Id {
    #[cfg(test)]
    pub fn new_attr(message: &str, attribute: &str) -> Self {
        Self {
            message: message.to_owned(),
            attribute: Some(attribute.to_owned()),
        }
    }

    #[cfg(test)]
    pub fn new_msg(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            attribute: None,
        }
    }

    pub fn func_name(&self) -> String {
        let atr = self
            .attribute
            .as_ref()
            .map(|a| format!("_{a}"))
            .unwrap_or_default();
        format!("{}{atr}", self.message).rust_id()
    }
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub id: String,
    pub variables: Vec<Variable>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Variable {
    pub id: String,
    pub typ: VarType,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum VarType {
    Any,
    String,
    Number,
}
