// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::str::FromStr;
use std::{borrow::Cow, collections::HashMap};
use unic_langid::{langid, LanguageIdentifier};

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
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Placeholder => "placeholder", // <<placeholder enum to_str>>
        }
    }

    pub fn id(&self) -> &'static LanguageIdentifier {
        match self {
            Self::Placeholder => &EN, // <<placeholder enum id>>
        }
    }
    pub fn as_arr() -> &'static [Self; 1] {
        &[
            // languages as an array
            Self::Placeholder, // <<placeholder enum as_arr>>
        ]
    }
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
    pub fn load(lang: &str, ftl: String) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, ftl)?))
    }

    // <<message implementations>>
}
