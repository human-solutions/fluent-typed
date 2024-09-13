use std::borrow::Cow;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::ast::Pattern;
use unic_langid::LanguageIdentifier;

pub struct L10nBundle {
    lang: String,
    bundle: FluentBundle<FluentResource>,
}

impl L10nBundle {
    pub fn new(lang: impl AsRef<str>, bytes: &[u8]) -> Result<Self, String> {
        let ftl = String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("Could not read ftl string due to: {e}"))?;
        let lang_id: LanguageIdentifier = lang.as_ref().parse().map_err(|e| format!("{e:?}"))?;
        let mut bundle = FluentBundle::new(vec![lang_id]);
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

    pub fn msg(&self, id: &str, args: Option<FluentArgs>) -> Result<Cow<'_, str>, String> {
        let pattern = self.try_get_pattern(id, None)?;
        self.format(id, None, pattern, args.as_ref())
    }

    pub fn attr(
        &self,
        msg: &str,
        attr: &str,
        args: Option<FluentArgs>,
    ) -> Result<Cow<'_, str>, String> {
        let pattern = self.try_get_pattern(msg, Some(attr))?;
        self.format(msg, Some(attr), pattern, args.as_ref())
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
    ) -> Result<Cow<'a, str>, String> {
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
            Ok(value)
        }
    }
}

fn arg_list(args: &FluentArgs) -> String {
    args.iter()
        .map(|(k, v)| format!("{}={:?}", k, v))
        .collect::<Vec<_>>()
        .join(", ")
}
