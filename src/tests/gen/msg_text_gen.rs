// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{borrow::Cow, ops::Range, str::FromStr};

static LANG_DATA: &'static [u8] = include_bytes!("msg_text_gen.ftl");
static EN: LanguageIdentifier = langid!("en");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10Lang {
    En,
}

impl FromStr for L10Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::En),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10Lang {
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

    pub fn as_arr() -> &'static [Self; 1] {
        &[
            // languages as an array
            Self::En,
        ]
    }

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::En => 0..30,
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

    fn msg_hello_world(&self) -> Cow<'_, str> {
        self.0.msg("hello-world", None).unwrap()
    }
}
