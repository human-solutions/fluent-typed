// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::borrow::Cow;

/// A thin wrapper around the Fluent messages for one language.
///
/// It provides functions for each message that was found in
/// all the languages at build time.
pub struct L10n(L10nLanguage);

impl L10n {
    /// Load the L10n resources for the given language. The language
    /// has to be a valid unic_langid::LanguageIdentifier or otherwise
    /// an error is returned.
    pub fn load(lang: &str, ftl: String) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, ftl)?))
    }

    fn msg_last_name(&self) -> Cow<'_, str> {
        self.0.msg("last-name", None).unwrap()
    }
    fn msg_company_name(&self) -> Cow<'_, str> {
        self.0.msg("company-name", None).unwrap()
    }
}
