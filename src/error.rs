use fluent_bundle::FluentError;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum MessageError {
    MissingMessage {
        id: String,
        bundle: String,
    },
    MissingValue {
        id: String,
        bundle: String,
    },
    Format {
        id: String,
        bundle: String,
        args: Option<String>,
        errors: Vec<FluentError>,
    },
}

impl Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::MissingMessage { id, bundle } => {
                write!(f, "Message '{id}' doesn't exist in bundle for {bundle}.")
            }
            MessageError::MissingValue { id, bundle } => {
                write!(
                    f,
                    "Message '{id}' in bundle for {bundle} doesn't have a value."
                )
            }
            MessageError::Format {
                id,
                bundle,
                args: Some(args),
                errors,
            } => {
                write!(
                    f,
                    "Errors when processing message '{id}' with args {args} in bundle for {bundle}: {errors:?}"
                )
            }
            MessageError::Format {
                id,
                bundle,
                args: None,
                errors,
            } => {
                write!(
                    f,
                    "Errors when processing message '{id}' in bundle for {bundle}: {errors:?}"
                )
            }
        }
    }
}
impl Error for MessageError {}
