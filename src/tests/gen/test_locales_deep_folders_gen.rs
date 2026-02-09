// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{
    fmt::Display,
    ops::{Deref, Range},
    slice::Iter,
    str::FromStr,
};

static LANG_DATA: &[u8] = include_bytes!("test_locales_deep_folders.ftl");

static ALL_LANGS: [L10n; 2] = [
    // languages as an array
    L10n::De,
    L10n::En,
];

static DE: LanguageIdentifier = langid!("de");
static EN: LanguageIdentifier = langid!("en");

/// The languages that have translations available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
    De,
    En,
}

impl Default for L10n {
    fn default() -> Self {
        Self::En
    }
}

impl FromStr for L10n {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "de" => Ok(Self::De),
            "en" => Ok(Self::En),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl Deref for L10n {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::De => "de",
            Self::En => "en",
        }
    }
}

impl AsRef<LanguageIdentifier> for L10n {
    fn as_ref(&self) -> &LanguageIdentifier {
        match self {
            Self::De => &DE,
            Self::En => &EN,
        }
    }
}

impl AsRef<str> for L10n {
    fn as_ref(&self) -> &str {
        self
    }
}

impl Display for L10n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl L10n {
    pub fn iter() -> Iter<'static, L10n> {
        ALL_LANGS.iter()
    }

    /// The language name as defined in the ftl message "language-name".
    pub fn language_name(&self) -> &'static str {
        match self {
            Self::De => "Deutsch",
            Self::En => "English",
        }
    }

    /// Negotiate the best language to use based on the `Accept-Language` header.
    ///
    /// Falls back to the default language if none of the languages in the header are available.
    pub fn langneg(accept_language: &str) -> L10n {
        negotiate_languages(accept_language, &ALL_LANGS)
    }

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::De => 0..546,
            Self::En => 546..1085,
        }
    }
    /// Load a L10nLanguage from the embedded data.
    pub fn load(&self) -> L10nLanguage {
        let bytes = LANG_DATA[self.byte_range()].to_vec();
        L10nLanguage::new(self, &bytes).unwrap()
    }

    /// Load all languages (L10nLanguage) from the embedded data.
    pub fn load_all() -> L10nLanguageVec {
        L10nLanguageVec::load(
            LANG_DATA,
            Self::iter().map(|lang| (lang, lang.byte_range())),
        )
        .unwrap()
    }
}

/// A thin wrapper around the Fluent messages for one language.
///
/// It provides functions for each message that was found in
/// all the languages at build time.
pub struct L10nLanguage(L10nBundle);

impl L10nLanguage {
    /// Load the L10n resources for the given language. The language
    /// has to be a valid LanguageIdentifier or otherwise
    /// an error is returned.
    ///
    /// The bytes are expected to be the contents of a .ftl file
    pub fn new(lang: impl AsRef<str>, bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(L10nBundle::new(lang, bytes)?))
    }

    pub fn msg_level1_hello(&self) -> String {
        self.0.msg("level1-hello", None).unwrap()
    }
    pub fn msg_level1_desc(&self) -> String {
        self.0.msg("level1-desc", None).unwrap()
    }
    pub fn msg_level2_greeting(&self) -> String {
        self.0.msg("level2-greeting", None).unwrap()
    }
    pub fn msg_level2_info(&self) -> String {
        self.0.msg("level2-info", None).unwrap()
    }
    pub fn msg_deep_message(&self) -> String {
        self.0.msg("deep-message", None).unwrap()
    }
    pub fn msg_deep_location(&self) -> String {
        self.0.msg("deep-location", None).unwrap()
    }
    #[allow(unused)]
    pub fn msg_language_name(&self) -> String {
        self.0.msg("language-name", None).unwrap()
    }
    pub fn msg_root_message(&self) -> String {
        self.0.msg("root-message", None).unwrap()
    }
}
