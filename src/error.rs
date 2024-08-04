use fluent_bundle::{FluentBundle, FluentError, FluentResource};
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum TranslationId {
    Message(MessageId),
    Attribute { message: MessageId, attr: String },
}

impl TranslationId {
    pub fn new(bundle: &FluentBundle<FluentResource>, msg: &str, attr: Option<&str>) -> Self {
        let message = MessageId::new(msg, bundle);
        match attr.map(|s|s.to_owned()) {
            Some(attr) => Self::Attribute { message , attr  },
            None => Self::Message(message)
        }
    }
}

impl Display for TranslationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "{msg}"),
            Self::Attribute { message, attr } => write!(f, "attribute '{attr}' in {message}"),
        }
    }
}

#[derive(Debug)]
pub struct MessageId {
    bundle: String,
    message: String,
}

impl MessageId {
    pub fn new(msg: &str, bundle: &FluentBundle<FluentResource>) -> Self {
        let bundle = bundle
            .locales
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        Self {
            bundle,
            message: msg.to_string(),
        }
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message '{}' in bundle '{}'", self.message, self.bundle)
    }
}

#[derive(Debug)]
pub enum MessageError {
    NotFound(TranslationId),
    InvalidFormat {
        id: TranslationId,
        args: Option<String>,
        errors: Vec<FluentError>,
    },
}

impl Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Could not find {id}"),
            Self::InvalidFormat {
                id,
                args: None,
                errors,
            } => {
                write!(f, "Invalid format for {id}: {errors:?}")
            }
            Self::InvalidFormat {
                id,
                args: Some(args),
                errors,
            } => {
                write!(f, "Invalid format for {id} with args {args}: {errors:?}")
            }
        }
    }
}
impl Error for MessageError {}
