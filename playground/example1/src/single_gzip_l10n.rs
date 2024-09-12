// This file is generated. Do not edit it manually.
use fluent_typed::prelude::*;
use std::{borrow::Cow, ops::Range, str::FromStr};

static LANG_DATA: &'static [u8] = include_bytes!("../gen/translations.ftl.gzip");
static EN: LanguageIdentifier = langid!("en");
static FR: LanguageIdentifier = langid!("fr");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
    En,
    Fr,
}

impl FromStr for L10n {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::En),
            "fr" => Ok(Self::Fr),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10n {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
        }
    }

    pub fn id(&self) -> &'static LanguageIdentifier {
        match self {
            Self::En => &EN,
            Self::Fr => &FR,
        }
    }

    pub fn as_arr() -> &'static [Self; 2] {
        &[
            // languages as an array
            Self::En,
            Self::Fr,
        ]
    }

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::En => 0..185,
            Self::Fr => 185..401,
        }
    }
    pub fn load<D>(&self, decompressor: D) -> Result<L10nLanguage, String>
    where
        D: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        let bytes = decompressor(LANG_DATA)?;
        L10nLanguage::new(self.as_str(), &bytes)
    }

    pub fn load_all<D>(decompressor: D) -> Result<Vec<L10nLanguage>, String>
    where
        D: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        let bytes = decompressor(LANG_DATA)?;
        Self::as_arr()
            .iter()
            .map(|lang| L10nLanguage::new(lang.as_str(), &bytes[lang.byte_range()]))
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

    fn msg_greeting<'a, F0: Into<FluentValue<'a>>>(&self, gender: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("gender", gender);
        self.0.msg("greeting", Some(args)).unwrap()
    }
    fn msg_enter_details(&self) -> Cow<'_, str> {
        self.0.msg("enter-details", None).unwrap()
    }
}
