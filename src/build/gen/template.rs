// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{
    fmt::Display,
    ops::{Deref, Range},
    slice::Iter,
    str::FromStr,
};

static LANG_DATA: &[u8] = include_bytes!("./ftl.bin"); // <<placeholder lang_data>>
static ALL_LANGS: [L10n; 1] = [L10n::Placeholder]; // <<placeholder all_langs>>

// <<placeholder static enum langid>>

/// The languages that have translations available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
    Placeholder, // <<placeholder enum variant>>
}

impl Default for L10n {
    fn default() -> Self {
        Self::Placeholder // <<placeholder default lang>>
    }
}

impl FromStr for L10n {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "placeholder" => Ok(Self::Placeholder), // <<placeholder enum from_str>>
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl Deref for L10n {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Placeholder => "placeholder", // <<placeholder enum to_str>>
        }
    }
}

// <<placeholder as_ref_langid>>
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
    // <<placeholder lang_name function>>
    // <<placeholder langneg function>>
    // <<placeholder load functions>>
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

    // <<message implementations>>
}
