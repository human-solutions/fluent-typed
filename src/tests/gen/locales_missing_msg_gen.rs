// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::borrow::Cow;

/// The resources used for generating the L10n functions at build-time.
///
/// The same resources should be loaded at runtime (or build-time)
/// to guarantee that the generated functions work correctly.
pub struct L10nResources {
    pub company: String,
    pub profile: String,
}

impl L10nResources {
    fn to_vec(self) -> Vec<String> {
        let mut vec = Vec::new();
        vec.push(self.company);
        vec.push(self.profile);
        vec
    }
}

/// A thin wrapper around the Fluent messages for one language.
///
/// It provides functions for each message that was found in
/// all the languages at build time.
pub struct L10n(L10nLanguage);

impl L10n {
    /// Load the L10n resources for the given language. The language
    /// has to be a valid unic_langid::LanguageIdentifier or otherwise
    /// an error is returned.
    pub fn load(lang: &str, resources: L10nResources) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, resources.to_vec())?))
    }

    fn msg_last_name(&self) -> Cow<'_, str> {
        self.0.msg("last-name", None).unwrap()
    }
    fn msg_company_name(&self) -> Cow<'_, str> {
        self.0.msg("company-name", None).unwrap()
    }
}
