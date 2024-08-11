mod ext;
mod gen;
mod parse_ast;
mod type_in_comment;

pub use ext::BundleMessageExt;
pub use gen::generate_code;

use crate::ext::StrExt;

#[derive(Debug, PartialEq)]
pub struct Message<'ast, 'res> {
    pub id: Id<'res, 'ast>,
    pub comment: Vec<&'ast str>,
    pub variables: Vec<Variable<'ast>>,
}

#[derive(Debug, PartialEq)]
pub struct Id<'res, 'ast> {
    pub resource: Option<&'res str>,
    pub message: &'ast str,
    pub attribute: Option<&'ast str>,
}

impl<'res, 'ast> Id<'res, 'ast> {
    pub fn new_attr(message: &'ast str, attribute: &'ast str) -> Self {
        Self {
            resource: None,
            message,
            attribute: Some(attribute),
        }
    }

    pub fn new_msg(message: &'ast str) -> Self {
        Self {
            resource: None,
            message,
            attribute: None,
        }
    }

    pub fn new_resource_msg(resource: &'res str, message: &'ast str) -> Self {
        Self {
            resource: Some(resource),
            message,
            attribute: None,
        }
    }
    pub fn func_name(&self) -> String {
        let res = self.resource.map(|r| format!("{r}_")).unwrap_or_default();
        let atr = self.attribute.map(|a| format!("_{a}")).unwrap_or_default();
        format!("{res}{}{atr}", self.message).rust_id()
    }
}

#[derive(Debug, PartialEq)]
pub struct Attribute<'ast> {
    pub id: &'ast str,
    pub variables: Vec<Variable<'ast>>,
}

#[derive(Debug, PartialEq)]
pub struct Variable<'ast> {
    pub id: &'ast str,
    pub typ: VarType,
}

#[derive(Debug, PartialEq)]
pub enum VarType {
    Any,
    String,
    Number,
}
