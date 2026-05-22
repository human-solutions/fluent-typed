//! Types for structured (`(Element)`-annotated) message output.
//!
//! A message that uses `(Element)` markers is generated as a struct whose
//! fields are the resolved text segments and the element slots, in render
//! order. The generated accessor builds that struct from the [`Segment`]s
//! returned by [`L10nBundle::msg_segments`](crate::prelude::L10nBundle::msg_segments).

/// One resolved piece of a structured message.
///
/// [`L10nBundle::msg_segments`](crate::prelude::L10nBundle::msg_segments)
/// returns these in render order, alternating `Text, Element, Text, …`.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// A resolved run of translated text.
    Text(String),
    /// A variable `(Element)` — a positional gap the app fills with its own
    /// UI element.
    Gap,
    /// A term `(Element)` — resolved translatable text the app wraps.
    Term(String),
}

impl Segment {
    /// The resolved text of this segment. A [`Segment::Gap`] carries no text
    /// and yields an empty string.
    pub fn into_text(self) -> String {
        match self {
            Segment::Text(s) | Segment::Term(s) => s,
            Segment::Gap => String::new(),
        }
    }
}

/// A variable `(Element)` slot in a generated structured-message struct.
///
/// It carries no data: the field exists only to mark — in render order — the
/// position where the consuming app injects its own UI element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ElementGap;
