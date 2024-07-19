mod parse_ast;
mod type_in_comment;

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
    pub typ: VarType<'ast>,
}

#[derive(Debug, PartialEq)]
pub enum VarType<'ast> {
    Any,
    String,
    Number,
    Enumeration(Vec<EnumEntry<'ast>>),
}

#[derive(Debug, PartialEq)]
pub struct EnumEntry<'ast> {
    pub name: &'ast str,
    pub default: bool,
}
