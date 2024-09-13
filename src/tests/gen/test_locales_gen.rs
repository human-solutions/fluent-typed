// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{borrow::Cow, ops::Range, slice::Iter, str::FromStr};

static LANG_DATA: &'static [u8] = include_bytes!("test_locales.ftl");
static DE: LanguageIdentifier = langid!("de");
static EN_GB: LanguageIdentifier = langid!("en-gb");

static ALL_LANGS: [L10n; 2] = [
    // languages as an array
    L10n::De,
    L10n::EnGb,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
    De,
    EnGb,
}

impl FromStr for L10n {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "de" => Ok(Self::De),
            "en-gb" => Ok(Self::EnGb),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10n {
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

    pub fn iter() -> Iter<'static, L10n> {
        ALL_LANGS.iter()
    }

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::De => 0..107,
            Self::EnGb => 107..208,
        }
    }
    pub fn load(&self) -> Result<L10nLanguage, String> {
        let bytes = LANG_DATA[self.byte_range()].to_vec();
        L10nLanguage::new(self.as_str(), &bytes)
    }

    pub fn load_all() -> Result<LanguageCollection, String> {
        LanguageCollection::load(
            &LANG_DATA,
            Self::iter().map(|lang| (lang.as_str(), lang.byte_range())),
        )
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

    fn msg_twenty_four_hour(&self) -> Cow<'_, str> {
        self.0.msg("twenty-four-hour", None).unwrap()
    }
    fn msg_twelve_hour(&self) -> Cow<'_, str> {
        self.0.msg("twelve-hour", None).unwrap()
    }
}
