// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{borrow::Cow, ops::Range, str::FromStr};

static LANG_DATA: &'static [u8] = include_bytes!("./ftl.bin"); // <<placeholder lang_data>>
static EN: LanguageIdentifier = langid!("en"); // <<placeholder static enum langid>>

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10Lang {
    Placeholder, // <<placeholder enum variant>>
}

impl FromStr for L10Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "placeholder" => Ok(Self::Placeholder), // <<placeholder enum from_str>>
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10Lang {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Placeholder => "placeholder", // <<placeholder enum to_str>>
        }
    }

    pub fn id(&self) -> &'static LanguageIdentifier {
        match self {
            Self::Placeholder => &EN, // <<placeholder enum id>>
        }
    }
    // <<placeholder as_arr>>
    // <<placeholder load functions>>
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

    // <<message implementations>>
}
