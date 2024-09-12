// This file is generated. Do not edit it manually.
use fluent_typed::prelude::*;
use std::{borrow::Cow, ops::Range, str::FromStr};

static EN: LanguageIdentifier = langid!("en");
static FR: LanguageIdentifier = langid!("fr");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10Lang {
    En,
    Fr,
}

impl FromStr for L10Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Self::En),
            "fr" => Ok(Self::Fr),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10Lang {
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

    fn msg_greeting<'a, F0: Into<FluentValue<'a>>>(&self, gender: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("gender", gender);
        self.0.msg("greeting", Some(args)).unwrap()
    }
    fn msg_enter_details(&self) -> Cow<'_, str> {
        self.0.msg("enter-details", None).unwrap()
    }
}
