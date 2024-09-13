// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{borrow::Cow, ops::Range, slice::Iter, str::FromStr};

static LANG_DATA: &'static [u8] = include_bytes!("msg_number_gen.ftl");
static EN: LanguageIdentifier = langid!("en");

static ALL_LANGS: [L10n; 1] = [
    // languages as an array
    L10n::En,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
    En,
}

impl FromStr for L10n {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::En),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10n {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::En => "en",
        }
    }

    pub fn id(&self) -> &'static LanguageIdentifier {
        match self {
            Self::En => &EN,
        }
    }

    pub fn iter() -> Iter<'static, L10n> {
        ALL_LANGS.iter()
    }

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::En => 0..96,
        }
    }
    pub fn load(&self) -> Result<L10nLanguage, String> {
        let bytes = LANG_DATA[self.byte_range()].to_vec();
        L10nLanguage::new(self.as_str(), &bytes)
    }

    pub fn load_all() -> Result<Vec<L10nLanguage>, String> {
        Self::iter()
            .map(|lang| L10nLanguage::new(lang.as_str(), &LANG_DATA[lang.byte_range()]))
            .collect()
    }
}

/// A thin wrapper around the Fluent messages for one language.
///
/// It provides functions for each message that was found in
/// all the languages at build time.
pub struct L10nLanguage(LanguageBundle);

impl L10nLanguage {
    /// Load the L10n resources for the given language. The language
    /// has to be a valid unic_langid::LanguageIdentifier or otherwise
    /// an error is returned.
    ///
    /// The bytes are expected to be the contents of a .ftl file
    pub fn new(lang: &str, bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(LanguageBundle::new(lang, bytes)?))
    }

    pub fn language_identifier(&self) -> &LanguageIdentifier {
        self.0.lang()
    }

    fn msg_time_elapsed<F0: Into<FluentNumber>>(&self, duration: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("duration", duration.into());
        self.0.msg("time-elapsed", Some(args)).unwrap()
    }
}
