// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{borrow::Cow, ops::Range, slice::Iter, str::FromStr};

static LANG_DATA: &'static [u8] = include_bytes!("msg_with_attrib_gen.ftl");
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
            Self::En => 0..99,
        }
    }
    pub fn load(&self) -> Result<L10nLanguage, String> {
        let bytes = LANG_DATA[self.byte_range()].to_vec();
        L10nLanguage::new(self.as_str(), &bytes)
    }

    pub fn load_all() -> Result<L10nLanguageVec, String> {
        L10nLanguageVec::load(
            &LANG_DATA,
            Self::iter().map(|lang| (lang.as_str(), lang.byte_range())),
        )
    }
}

/// A thin wrapper around the Fluent messages for one language.
///
/// It provides functions for each message that was found in
/// all the languages at build time.
pub struct L10nLanguage(L10nBundle);

impl L10nLanguage {
    /// Load the L10n resources for the given language. The language
    /// has to be a valid unic_langid::LanguageIdentifier or otherwise
    /// an error is returned.
    ///
    /// The bytes are expected to be the contents of a .ftl file
    pub fn new(lang: &str, bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(L10nBundle::new(lang, bytes)?))
    }

    pub fn language_identifier(&self) -> &LanguageIdentifier {
        self.0.lang()
    }

    fn msg_hello(&self) -> Cow<'_, str> {
        self.0.msg("hello", None).unwrap()
    }
    fn msg_hello_tooltip<'a, F0: Into<FluentValue<'a>>>(&self, user_name: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("userName", user_name);
        self.0.attr("hello", "tooltip", Some(args)).unwrap()
    }
}
