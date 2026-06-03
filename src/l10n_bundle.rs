use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_syntax::ast::{Expression, InlineExpression, Pattern, PatternElement};
use unic_langid::LanguageIdentifier;

use crate::error::L10nError;
use crate::structured::Segment;

pub struct L10nBundle {
    lang: String,
    bundle: FluentBundle<FluentResource>,
}

impl L10nBundle {
    /// Load the messages for `lang` from the bytes of a `.ftl` file.
    ///
    /// Interpolated variables are wrapped in Unicode bidi isolation marks
    /// (FSI `U+2068` / PDI `U+2069`), which is the safe default for text
    /// rendered in a bidi-aware context such as a web UI. Use
    /// [`Self::new_without_isolation`] to disable this.
    pub fn new(lang: impl AsRef<str>, bytes: &[u8]) -> Result<Self, L10nError> {
        Self::build(lang, bytes, true)
    }

    /// Like [`Self::new`], but without Unicode bidi isolation marks around
    /// interpolated variables. Only use this if the strings are never
    /// rendered in a bidi-aware context, or you never use right-to-left
    /// locales or interpolate user-provided text.
    pub fn new_without_isolation(lang: impl AsRef<str>, bytes: &[u8]) -> Result<Self, L10nError> {
        Self::build(lang, bytes, false)
    }

    fn build(lang: impl AsRef<str>, bytes: &[u8], use_isolating: bool) -> Result<Self, L10nError> {
        let ftl = String::from_utf8(bytes.to_vec()).map_err(L10nError::InvalidUtf8)?;
        let lang_id: LanguageIdentifier =
            lang.as_ref()
                .parse()
                .map_err(|e| L10nError::InvalidLanguage {
                    lang: lang.as_ref().to_string(),
                    reason: format!("{e:?}"),
                })?;
        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
        bundle.set_use_isolating(use_isolating);
        // Register the FTL builtins (`NUMBER`, …). Without this, any message
        // using `{ NUMBER($x) }` fails to format and `msg`/`attr` return an
        // `Err`, which the generated accessors `.unwrap()` into a panic.
        bundle
            .add_builtins()
            .map_err(|e| L10nError::Builtins(format!("{e:?}")))?;
        let resource = FluentResource::try_new(ftl).map_err(|(_, errors)| {
            L10nError::ResourceParse(errors.iter().map(|e| format!("{e:?}")).collect())
        })?;
        bundle.add_resource(resource).map_err(|errors| {
            L10nError::AddResource(errors.iter().map(|e| format!("{e:?}")).collect())
        })?;

        Ok(Self {
            bundle,
            lang: lang.as_ref().to_string(),
        })
    }

    pub fn lang(&self) -> &str {
        &self.lang
    }

    pub fn msg(&self, id: &str, args: Option<FluentArgs>) -> Result<String, L10nError> {
        let pattern = self.try_get_pattern(id, None)?;
        self.format(id, None, pattern, args.as_ref())
    }

    pub fn attr(
        &self,
        msg: &str,
        attr: &str,
        args: Option<FluentArgs>,
    ) -> Result<String, L10nError> {
        let pattern = self.try_get_pattern(msg, Some(attr))?;
        self.format(msg, Some(attr), pattern, args.as_ref())
    }

    /// Resolve `id` into ordered [`Segment`]s, split at the `(Element)` markers
    /// named in `element_vars` / `element_terms`.
    ///
    /// The result alternates `Text, Element, Text, …` and always holds
    /// `2 * markers + 1` entries (text segments may be empty). Generated
    /// structured-message accessors map this directly into a typed struct.
    pub fn msg_segments(
        &self,
        id: &str,
        element_vars: &[&str],
        element_terms: &[&str],
        args: Option<FluentArgs>,
    ) -> Result<Vec<Segment>, L10nError> {
        let pattern = self.try_get_pattern(id, None)?;
        let args = args.as_ref();

        let mut segments = Vec::new();
        let mut current: Vec<PatternElement<&str>> = Vec::new();

        for element in &pattern.elements {
            match element_marker(element, element_vars, element_terms) {
                Some(Marker::Variable) => {
                    segments.push(Segment::Text(self.resolve_segment(id, &current, args)?));
                    current.clear();
                    segments.push(Segment::Gap);
                }
                Some(Marker::Term) => {
                    segments.push(Segment::Text(self.resolve_segment(id, &current, args)?));
                    current.clear();
                    let text = self.resolve_segment(id, std::slice::from_ref(element), args)?;
                    segments.push(Segment::Term(text));
                }
                None => current.push(element.clone()),
            }
        }
        segments.push(Segment::Text(self.resolve_segment(id, &current, args)?));
        Ok(segments)
    }

    /// Resolve a slice of pattern elements as their own sub-pattern.
    ///
    /// A leading empty `TextElement` is prepended so the sub-pattern always has
    /// more than one element. Fluent only wraps a placeable in bidi isolation
    /// marks when its pattern has `len > 1`, so this padding makes a split
    /// segment resolve byte-identically to the same span of the whole,
    /// un-split message.
    fn resolve_segment(
        &self,
        id: &str,
        elements: &[PatternElement<&str>],
        args: Option<&FluentArgs>,
    ) -> Result<String, L10nError> {
        let mut padded = Vec::with_capacity(elements.len() + 1);
        padded.push(PatternElement::TextElement { value: "" });
        padded.extend(elements.iter().cloned());
        let sub = Pattern { elements: padded };

        let mut errors = vec![];
        let value = self.bundle.format_pattern(&sub, args, &mut errors);
        if errors.is_empty() {
            Ok(value.to_string())
        } else {
            Err(L10nError::Format {
                id: id.to_string(),
                attribute: None,
                errors: errors.iter().map(|e| format!("{e:?}")).collect(),
            })
        }
    }

    fn try_get_pattern(
        &self,
        msg_id: &str,
        attr_id: Option<&str>,
    ) -> Result<&Pattern<&str>, L10nError> {
        let message =
            self.bundle
                .get_message(msg_id)
                .ok_or_else(|| L10nError::MessageNotFound {
                    id: msg_id.to_string(),
                })?;
        if let Some(attr_id) = attr_id {
            message
                .get_attribute(attr_id)
                .map(|attr| attr.value())
                .ok_or_else(|| L10nError::AttributeNotFound {
                    id: msg_id.to_string(),
                    attribute: attr_id.to_string(),
                })
        } else {
            message.value().ok_or_else(|| L10nError::MessageHasNoValue {
                id: msg_id.to_string(),
            })
        }
    }

    fn format<'a>(
        &'a self,
        msg: &str,
        attr: Option<&str>,
        pattern: &'a Pattern<&str>,
        args: Option<&FluentArgs>,
    ) -> Result<String, L10nError> {
        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            Err(L10nError::Format {
                id: msg.to_string(),
                attribute: attr.map(|a| a.to_string()),
                errors: errors.iter().map(|e| format!("{e:?}")).collect(),
            })
        } else {
            Ok(value.to_string())
        }
    }
}

/// Which kind of `(Element)` marker a pattern element is, if any.
enum Marker {
    Variable,
    Term,
}

fn element_marker(
    element: &PatternElement<&str>,
    element_vars: &[&str],
    element_terms: &[&str],
) -> Option<Marker> {
    let PatternElement::Placeable { expression } = element else {
        return None;
    };
    match expression {
        Expression::Inline(InlineExpression::VariableReference { id })
            if element_vars.contains(&id.name) =>
        {
            Some(Marker::Variable)
        }
        Expression::Inline(InlineExpression::TermReference { id, .. })
            if element_terms.contains(&id.name) =>
        {
            Some(Marker::Term)
        }
        _ => None,
    }
}
