// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{borrow::Cow, ops::Range, str::FromStr};

static LANG_DATA: &'static [u8] = include_bytes!("../gen/translations.ftl");
static DE: LanguageIdentifier = langid!("de");
static EN_GB: LanguageIdentifier = langid!("en-gb");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10Lang {
    De,
    EnGb,
}

impl FromStr for L10Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "de" => Ok(Self::De),
            "en-gb" => Ok(Self::EnGb),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10Lang {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::De => "de",
            Self::EnGb => "en-gb",
        }
    }

    pub fn id(&self) -> &'static LanguageIdentifier {
        match self {
            Self::De => &DE,
            Self::EnGb => &EN_GB,
        }
    }

    pub fn as_arr() -> &'static [Self; 2] {
        &[
            // languages as an array
            Self::De,
            Self::EnGb,
        ]
    }

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::De => 0..148,
            Self::EnGb => 148..293,
        }
    }
    pub fn load(&self) -> Result<L10n, String> {
        let bytes = LANG_DATA[self.byte_range()].to_vec();
        L10n::new(self.as_str(), &bytes)
    }

    pub fn load_all() -> Result<Vec<L10n>, String> {
        Self::as_arr()
            .iter()
            .map(|lang| L10n::new(lang.as_str(), &LANG_DATA[lang.byte_range()]))
            .collect()
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
    ///
    /// The bytes are expected to be the contents of a .ftl file
    pub fn new(lang: &str, bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, bytes)?))
    }

    pub fn language_identifier(&self) -> &LanguageIdentifier {
        self.0.lang()
    }

    fn msg_greeting(&self) -> Cow<'_, str> {
        self.0.msg("greeting", None).unwrap()
    }
    fn msg_twenty_four_hour(&self) -> Cow<'_, str> {
        self.0.msg("twenty-four-hour", None).unwrap()
    }
}
