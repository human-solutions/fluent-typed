// This file is generated. Do not edit it manually.
use fluent_typed::prelude::*;
use std::{
    fmt::Display,
    ops::{Deref, Range},
    slice::Iter,
    str::FromStr,
};

static ALL_LANGS: [L10n; 2] = [
    // languages as an array
    L10n::En,
    L10n::Fr,
];

static EN: LanguageIdentifier = langid!("en");
static FR: LanguageIdentifier = langid!("fr");

/// The languages that have translations available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
    En,
    Fr,
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
            "en" => Ok(Self::En),
            "fr" => Ok(Self::Fr),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl Deref for L10n {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
        }
    }
}

impl AsRef<LanguageIdentifier> for L10n {
    fn as_ref(&self) -> &LanguageIdentifier {
        match self {
            Self::En => &EN,
            Self::Fr => &FR,
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
            Self::En => "English",
            Self::Fr => "Français",
        }
    }

    /// Negotiate the best language to use based on the `Accept-Language` header.
    ///
    /// Falls back to the default language if none of the languages in the header are available.
    pub fn langneg(accept_language: &str) -> L10n {
        negotiate_languages(accept_language, &ALL_LANGS)
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

    #[allow(unused)]
    pub fn msg_language_name(&self) -> String {
        self.0.msg("language-name", None).unwrap()
    }
    pub fn msg_greeting<'a, F0: Into<FluentValue<'a>>>(&self, gender: F0) -> String {
        let mut args = FluentArgs::new();
        args.set("gender", gender);
        self.0.msg("greeting", Some(args)).unwrap()
    }
    pub fn msg_enter_details(&self) -> String {
        self.0.msg("enter-details", None).unwrap()
    }
}
