#[allow(unused_imports)]
use fluent_bundle::{types::FluentNumber, FluentValue};
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::Pattern;
use std::borrow::Cow;

pub trait TypedMessages {
    /// $duration (Number) - The duration in seconds.

    fn time_elapsed<F0: Into<FluentNumber>>(&self, duration: F0) -> Cow<'_, str>;
}

impl TypedMessages for FluentBundle<FluentResource> {
    fn time_elapsed<F0: Into<FluentNumber>>(&self, duration: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("duration", duration.into());
        self.msg("time-elapsed", Some(args)).unwrap()
    }
}

trait BundleExt {
    fn try_get_pattern(
        &self,
        msg_id: &str,
        attr_id: Option<&str>,
    ) -> Result<&Pattern<&str>, String>;

    fn format<'a>(
        &'a self,
        msg: &str,
        attr: Option<&str>,
        pattern: &'a Pattern<&str>,
        args: Option<&FluentArgs>,
    ) -> Result<Cow<'a, str>, String>;
}

impl BundleExt for FluentBundle<FluentResource> {
    fn try_get_pattern(
        &self,
        msg_id: &str,
        attr_id: Option<&str>,
    ) -> Result<&Pattern<&str>, String> {
        let message = self
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
    ) -> Result<Cow<'a, str>, String> {
        let mut errors = vec![];
        let value = self.format_pattern(pattern, args, &mut errors);
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
            Ok(value)
        }
    }
}

pub trait BundleMessageExt {
    fn msg(&self, id: &str, args: Option<FluentArgs>) -> Result<Cow<'_, str>, String>;

    fn attr(&self, msg: &str, id: &str, args: Option<FluentArgs>) -> Result<Cow<'_, str>, String>;
}

impl BundleMessageExt for FluentBundle<FluentResource> {
    fn msg(&self, id: &str, args: Option<FluentArgs>) -> Result<Cow<'_, str>, String> {
        let pattern = self.try_get_pattern(id, None)?;
        self.format(id, None, pattern, args.as_ref())
    }

    fn attr(
        &self,
        msg: &str,
        attr: &str,
        args: Option<FluentArgs>,
    ) -> Result<Cow<'_, str>, String> {
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
