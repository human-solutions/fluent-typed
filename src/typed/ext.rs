use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::Pattern;
use std::borrow::Cow;

use crate::{error::TranslationId, MessageError};

trait BundleExt {
    fn try_get_pattern(
        &self,
        msg_id: &str,
        attr_id: Option<&str>,
    ) -> Result<&Pattern<&str>, MessageError>;
    fn format<'a>(
        &'a self,
        msg: &str,
        attr: Option<&str>,
        pattern: &'a Pattern<&str>,
        args: Option<&FluentArgs>,
    ) -> Result<Cow<'a, str>, MessageError>;
}

impl BundleExt for FluentBundle<FluentResource> {
    fn try_get_pattern(
        &self,
        msg_id: &str,
        attr_id: Option<&str>,
    ) -> Result<&Pattern<&str>, MessageError> {
        let message = self
            .get_message(msg_id)
            .ok_or_else(|| MessageError::NotFound(TranslationId::new(self, msg_id, attr_id)))?;
        if let Some(attr_id) = attr_id {
            message.get_attribute(attr_id).map(|attr| attr.value())
        } else {
            message.value()
        }
        .ok_or_else(|| MessageError::NotFound(TranslationId::new(self, msg_id, None)))
    }

    fn format<'a>(
        &'a self,
        msg: &str,
        attr: Option<&str>,
        pattern: &'a Pattern<&str>,
        args: Option<&FluentArgs>,
    ) -> Result<Cow<'a, str>, MessageError> {
        let mut errors = vec![];
        let value = self.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            let id = TranslationId::new(self, msg, attr);
            let args = args.map(arg_list);
            Err(MessageError::InvalidFormat { id, args, errors })
        } else {
            Ok(value)
        }
    }
}

pub trait BundleMessageExt {
    fn msg(&self, id: &str, args: Option<FluentArgs>) -> Result<Cow<'_, str>, MessageError>;
    fn attr(
        &self,
        msg: &str,
        id: &str,
        args: Option<FluentArgs>,
    ) -> Result<Cow<'_, str>, MessageError>;
}

impl BundleMessageExt for FluentBundle<FluentResource> {
    fn msg(&self, id: &str, args: Option<FluentArgs>) -> Result<Cow<'_, str>, MessageError> {
        let pattern = self.try_get_pattern(id, None)?;
        self.format(id, None, pattern, args.as_ref())
    }

    fn attr(
        &self,
        msg: &str,
        attr: &str,
        args: Option<FluentArgs>,
    ) -> Result<Cow<'_, str>, MessageError> {
        let pattern = self.try_get_pattern(msg, Some(attr))?;
        self.format(msg, Some(attr), pattern, args.as_ref())
    }
}

fn arg_list(args: &FluentArgs) -> String {
    args.iter()
        .map(|(k, v)| format!("{}={:?}", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}
