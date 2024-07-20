use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::Pattern;
use std::borrow::Cow;

use crate::MessageError;

pub trait BundleMessageExt {
    fn msg_with_args(&self, id: &str, args: FluentArgs) -> Result<Cow<'_, str>, MessageError>;
    fn msg(&self, id: &str) -> Result<Cow<'_, str>, MessageError>;
}

impl BundleMessageExt for FluentBundle<FluentResource> {
    fn msg(&self, id: &str) -> Result<Cow<'_, str>, MessageError> {
        let pattern = pattern(self, id);
        format(self, id, pattern, None)
    }

    fn msg_with_args(&self, id: &str, args: FluentArgs) -> Result<Cow<'_, str>, MessageError> {
        let pattern = pattern(self, id);
        format(self, id, pattern, Some(&args))
    }
}

fn arg_list(args: &FluentArgs) -> String {
    args.iter()
        .map(|(k, v)| format!("{}={:?}", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format<'a>(
    bundle: &'a FluentBundle<FluentResource>,
    id: &str,
    pattern: &'a Pattern<&str>,
    args: Option<&FluentArgs>,
) -> Result<Cow<'a, str>, MessageError> {
    let mut errors = vec![];
    let value = bundle.format_pattern(pattern, args, &mut errors);
    if !errors.is_empty() {
        let bundle = locales(bundle);
        let id = id.to_string();
        if let Some(args) = args {
            let args = Some(arg_list(args));
            Err(MessageError::Format {
                id,
                bundle,
                args,
                errors,
            })
        } else {
            Err(MessageError::Format {
                id,
                bundle,
                args: None,
                errors,
            })
        }
    } else {
        Ok(value)
    }
}

fn pattern<'a>(bundle: &'a FluentBundle<FluentResource>, id: &str) -> &'a Pattern<&'a str> {
    let Some(msg) = bundle.get_message(id) else {
        panic!(
            "Message '{id}' doesn't exist in bundle for {}.",
            locales(bundle)
        )
    };
    let Some(pattern) = msg.value() else {
        panic!(
            "Message '{id}' in bundle for {} doesn't have a value.",
            locales(bundle)
        )
    };
    pattern
}

fn locales(bundle: &FluentBundle<FluentResource>) -> String {
    bundle
        .locales
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
