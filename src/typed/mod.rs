mod ext;
mod gen;
mod parse_ast;
mod type_in_comment;

pub use ext::BundleMessageExt;
pub use gen::generate_extension;

#[derive(Debug, PartialEq)]
pub struct Message<'ast> {
    pub comment: Vec<&'ast str>,
    pub id: &'ast str,
    pub variables: Vec<Variable<'ast>>,
    pub attributes: Vec<Attribute<'ast>>,
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
