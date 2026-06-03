//! The runtime error type, [`L10nError`].

use std::fmt;

/// An error from loading a localization bundle or formatting one of its
/// messages at runtime.
///
/// Returned by [`L10nBundle`](crate::prelude::L10nBundle),
/// [`L10nLanguageVec`](crate::prelude::L10nLanguageVec) and the generated
/// `L10nLanguage::new`. The generated message accessors `unwrap` this: a failure
/// there means the generated code and the embedded `.ftl` have drifted apart,
/// which is a build-time bug rather than something to handle at runtime.
///
/// The variants carry owned strings rather than `fluent-bundle`'s own error
/// types on purpose, so this enum does not change shape when that (pre-1.0)
/// dependency is updated. It is `#[non_exhaustive]` so new variants can be added
/// in a minor release.
#[derive(Debug)]
#[non_exhaustive]
pub enum L10nError {
    /// The user-supplied decompressor (for a compressed single-file build)
    /// returned an error. Carries that error's message.
    Decompression(String),

    /// The `.ftl` bytes were not valid UTF-8.
    InvalidUtf8(std::string::FromUtf8Error),

    /// The language identifier (e.g. `"en-US"`) could not be parsed.
    InvalidLanguage {
        /// The identifier that failed to parse.
        lang: String,
        /// A human-readable description of why it failed.
        reason: String,
    },

    /// The Fluent builtins (`NUMBER`, …) could not be registered on the bundle.
    Builtins(String),

    /// The `.ftl` resource failed to parse. One entry per parser error.
    ResourceParse(Vec<String>),

    /// The resource could not be added to the bundle (e.g. a message id that
    /// collides with one already present).
    AddResource(Vec<String>),

    /// No message with this id exists in the bundle.
    MessageNotFound {
        /// The message id that was looked up.
        id: String,
    },

    /// The message exists but has no value of its own (it has only attributes).
    MessageHasNoValue {
        /// The message id that was looked up.
        id: String,
    },

    /// The message exists but has no attribute with this name.
    AttributeNotFound {
        /// The message id.
        id: String,
        /// The attribute that was looked up.
        attribute: String,
    },

    /// Formatting the message or attribute failed — typically a missing or
    /// wrongly-typed argument. One entry per formatting error.
    Format {
        /// The message id being formatted.
        id: String,
        /// The attribute being formatted, if any.
        attribute: Option<String>,
        /// One entry per formatting error reported by fluent-bundle.
        errors: Vec<String>,
    },
}

impl fmt::Display for L10nError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decompression(e) => write!(f, "Could not decompress the .ftl data: {e}"),
            Self::InvalidUtf8(e) => write!(f, "The .ftl content is not valid UTF-8: {e}"),
            Self::InvalidLanguage { lang, reason } => {
                write!(f, "Invalid language identifier '{lang}': {reason}")
            }
            Self::Builtins(e) => write!(f, "Could not register the Fluent builtins: {e}"),
            Self::ResourceParse(errors) => {
                write!(f, "Could not parse the .ftl resource:")?;
                for e in errors {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
            Self::AddResource(errors) => {
                write!(f, "Could not add the .ftl resource to the bundle:")?;
                for e in errors {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
            Self::MessageNotFound { id } => write!(f, "No message '{id}' found"),
            Self::MessageHasNoValue { id } => {
                write!(f, "Message '{id}' has no value of its own")
            }
            Self::AttributeNotFound { id, attribute } => {
                write!(f, "Message '{id}' has no attribute '{attribute}'")
            }
            Self::Format {
                id,
                attribute,
                errors,
            } => {
                match attribute {
                    Some(a) => write!(f, "Could not format attribute '{a}' of message '{id}':")?,
                    None => write!(f, "Could not format message '{id}':")?,
                }
                for e in errors {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for L10nError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(e) => Some(e),
            _ => None,
        }
    }
}
