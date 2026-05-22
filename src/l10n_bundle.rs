use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::{
    CallArguments, Expression, Identifier, InlineExpression, NamedArgument, Pattern,
    PatternElement, Variant, VariantKey,
};
use unic_langid::LanguageIdentifier;

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
    pub fn new(lang: impl AsRef<str>, bytes: &[u8]) -> Result<Self, String> {
        Self::build(lang, bytes, true)
    }

    /// Like [`Self::new`], but without Unicode bidi isolation marks around
    /// interpolated variables. Only use this if the strings are never
    /// rendered in a bidi-aware context, or you never use right-to-left
    /// locales or interpolate user-provided text.
    pub fn new_without_isolation(lang: impl AsRef<str>, bytes: &[u8]) -> Result<Self, String> {
        Self::build(lang, bytes, false)
    }

    fn build(lang: impl AsRef<str>, bytes: &[u8], use_isolating: bool) -> Result<Self, String> {
        let ftl = String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("Could not read ftl string due to: {e}"))?;
        let lang_id: LanguageIdentifier = lang.as_ref().parse().map_err(|e| format!("{e:?}"))?;
        let mut bundle = FluentBundle::new(vec![lang_id]);
        bundle.set_use_isolating(use_isolating);
        let resource = FluentResource::try_new(ftl).map_err(|e| format!("{e:?}"))?;
        bundle
            .add_resource(resource)
            .map_err(|e| format!("{e:?}"))?;

        Ok(Self {
            bundle,
            lang: lang.as_ref().to_string(),
        })
    }

    pub fn lang(&self) -> &str {
        &self.lang
    }

    pub fn msg(&self, id: &str, args: Option<FluentArgs>) -> Result<String, String> {
        let pattern = self.try_get_pattern(id, None)?;
        self.format(id, None, pattern, args.as_ref())
    }

    pub fn attr(&self, msg: &str, attr: &str, args: Option<FluentArgs>) -> Result<String, String> {
        let pattern = self.try_get_pattern(msg, Some(attr))?;
        self.format(msg, Some(attr), pattern, args.as_ref())
    }

    pub fn msg_pattern(&self, id: &str) -> Pattern<String> {
        let pattern = self.try_get_pattern(id, None).unwrap();
        to_owned_pattern(pattern)
    }

    pub fn attr_pattern(&self, msg: &str, attr: &str) -> Pattern<String> {
        let pattern = self.try_get_pattern(msg, Some(attr)).unwrap();
        to_owned_pattern(pattern)
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
    ) -> Result<Vec<Segment>, String> {
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
    ) -> Result<String, String> {
        let mut padded = Vec::with_capacity(elements.len() + 1);
        padded.push(PatternElement::TextElement { value: "" });
        padded.extend(elements.iter().cloned());
        let sub = Pattern { elements: padded };

        let mut errors = vec![];
        let value = self.bundle.format_pattern(&sub, args, &mut errors);
        if errors.is_empty() {
            Ok(value.to_string())
        } else {
            Err(format!("Invalid format for message '{id}': {errors:?}"))
        }
    }

    fn try_get_pattern(
        &self,
        msg_id: &str,
        attr_id: Option<&str>,
    ) -> Result<&Pattern<&str>, String> {
        let message = self
            .bundle
            .get_message(msg_id)
            .ok_or_else(|| format!("Could not find {msg_id}"))?;
        if let Some(attr_id) = attr_id {
            message
                .get_attribute(attr_id)
                .map(|attr| attr.value())
                .ok_or_else(|| {
                    format!("Could not find attribute '{attr_id}' for message '{msg_id}'")
                })
        } else {
            message
                .value()
                .ok_or_else(|| format!("Could not find value for '{msg_id}'"))
        }
    }

    fn format<'a>(
        &'a self,
        msg: &str,
        attr: Option<&str>,
        pattern: &'a Pattern<&str>,
        args: Option<&FluentArgs>,
    ) -> Result<String, String> {
        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            let attr_str = attr
                .map(|a| format!("attribute '{a}' in "))
                .unwrap_or_default();
            let arg_str = args
                .map(|a| format!(" with args {}", arg_list(a)))
                .unwrap_or_default();
            Err(format!(
                "Invalid format for {attr_str}message '{msg}'{arg_str}: {errors:?}"
            ))
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

fn arg_list(args: &FluentArgs) -> String {
    args.iter()
        .map(|(k, v)| format!("{}={:?}", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}

fn to_owned_pattern(pattern: &Pattern<&str>) -> Pattern<String> {
    Pattern {
        elements: pattern.elements.iter().map(to_owned_element).collect(),
    }
}

fn to_owned_element(element: &PatternElement<&str>) -> PatternElement<String> {
    match element {
        PatternElement::TextElement { value } => PatternElement::TextElement {
            value: value.to_string(),
        },
        PatternElement::Placeable { expression } => PatternElement::Placeable {
            expression: to_owned_expression(expression),
        },
    }
}

fn to_owned_expression(expr: &Expression<&str>) -> Expression<String> {
    match expr {
        Expression::Select { selector, variants } => Expression::Select {
            selector: to_owned_inline(selector),
            variants: variants.iter().map(to_owned_variant).collect(),
        },
        Expression::Inline(inline) => Expression::Inline(to_owned_inline(inline)),
    }
}

fn to_owned_inline(expr: &InlineExpression<&str>) -> InlineExpression<String> {
    match expr {
        InlineExpression::StringLiteral { value } => InlineExpression::StringLiteral {
            value: value.to_string(),
        },
        InlineExpression::NumberLiteral { value } => InlineExpression::NumberLiteral {
            value: value.to_string(),
        },
        InlineExpression::FunctionReference { id, arguments } => {
            InlineExpression::FunctionReference {
                id: to_owned_id(id),
                arguments: to_owned_call_args(arguments),
            }
        }
        InlineExpression::MessageReference { id, attribute } => {
            InlineExpression::MessageReference {
                id: to_owned_id(id),
                attribute: attribute.as_ref().map(to_owned_id),
            }
        }
        InlineExpression::TermReference {
            id,
            attribute,
            arguments,
        } => InlineExpression::TermReference {
            id: to_owned_id(id),
            attribute: attribute.as_ref().map(to_owned_id),
            arguments: arguments.as_ref().map(to_owned_call_args),
        },
        InlineExpression::VariableReference { id } => InlineExpression::VariableReference {
            id: to_owned_id(id),
        },
        InlineExpression::Placeable { expression } => InlineExpression::Placeable {
            expression: Box::new(to_owned_expression(expression)),
        },
    }
}

fn to_owned_id(id: &Identifier<&str>) -> Identifier<String> {
    Identifier {
        name: id.name.to_string(),
    }
}

fn to_owned_call_args(args: &CallArguments<&str>) -> CallArguments<String> {
    CallArguments {
        positional: args.positional.iter().map(to_owned_inline).collect(),
        named: args
            .named
            .iter()
            .map(|n| NamedArgument {
                name: to_owned_id(&n.name),
                value: to_owned_inline(&n.value),
            })
            .collect(),
    }
}

fn to_owned_variant(variant: &Variant<&str>) -> Variant<String> {
    Variant {
        key: match &variant.key {
            VariantKey::Identifier { name } => VariantKey::Identifier {
                name: name.to_string(),
            },
            VariantKey::NumberLiteral { value } => VariantKey::NumberLiteral {
                value: value.to_string(),
            },
        },
        value: to_owned_pattern(&variant.value),
        default: variant.default,
    }
}
