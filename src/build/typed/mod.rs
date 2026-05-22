mod parse_ast;
mod type_in_comment;

use std::fmt::Display;

use crate::build::r#gen::StrExt;

pub use type_in_comment::{Annotation, annotation};

#[derive(Debug)]
pub struct Message {
    pub id: Id,
    pub comment: Vec<String>,
    pub variables: Vec<Variable>,
    /// The `(Element)`-annotated split points, in the order they appear in the
    /// message pattern. Empty for ordinary (non-structured) messages.
    pub elements: Vec<ElementMarker>,
    /// Every `$variable` and `-term` reference in the message pattern, in
    /// document order, derived purely from the AST (independent of comments).
    /// Used for comment-independent cross-locale compatibility checks.
    pub pattern_refs: Vec<Ref>,
    /// The `.ftl` file this message was parsed from (for diagnostics).
    pub file: String,
    /// The 1-based line of the message (or attribute) id.
    pub line: usize,
    /// The 1-based line of the first comment line, or `0` when there is no
    /// comment. Comment blocks are contiguous, so `comment[i]` is at
    /// `comment_line + i`.
    pub comment_line: usize,
}

/// Equality compares the *semantic* shape of a message — its id, comment,
/// variables, elements and pattern references — and deliberately ignores the
/// source-location fields (`file`, `line`, `comment_line`), which are diagnostic
/// metadata rather than identity.
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.comment == other.comment
            && self.variables == other.variables
            && self.elements == other.elements
            && self.pattern_refs == other.pattern_refs
    }
}

/// A `$variable` or `-term` reference in a message pattern.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Ref {
    pub name: String,
    pub kind: RefKind,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RefKind {
    Variable,
    Term,
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Variable {
    pub id: String,
    pub typ: VarType,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum VarType {
    Any,
    String,
    Number,
}

/// An `(Element)`-annotated split point in a message pattern.
///
/// A variable element (`$icon`) is a pure positional gap that the consuming
/// app fills with its own UI element. A term element (`-privacy-link`) carries
/// translatable text that the app wraps.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ElementMarker {
    pub name: String,
    pub kind: ElementKind,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ElementKind {
    Variable,
    Term,
}
