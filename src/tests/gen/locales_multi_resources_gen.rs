// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::str::FromStr;
use std::{borrow::Cow, collections::HashMap};
use unic_langid::{langid, LanguageIdentifier};

static LANG_DATA: &'static [u8] = include_bytes!("gen/translations.ftl"); 
static DE: LanguageIdentifier = langid!("de");
static EN_GB: LanguageIdentifier = langid!("en-gb");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10Lang {
    De,
    EnGb,
}

impl FromStr for L10Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "de" => Ok(Self::De),
            "en-gb" => Ok(Self::EnGb),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl L10Lang {
    pub fn to_str(&self) -> &'static str {
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
    pub fn as_arr() -> &'static [Self; 1] {
        &[
            // languages as an array
            Self::De,
            Self::EnGb,
        ]
    }
    pub fn load(&self) -> Result<L10n, String> {
        let lang_range = match self {
            EnGb => 0..145,
            De => 145..293,

        };
        let bytes = LANG_DATA[lang_range].to_vec();

        Ok(L10n::load(&String::from_utf8_lossy(&bytes))?)
    }

    pub fn load_all() -> Result<HashMap<L10Lang, L10n>, String> {
        let mut map = HashMap::new();
        for lang in Self::as_arr() {
            map.insert(lang.clone(), lang.load()?);
        }
        Ok(map)
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
    pub fn load(lang: &str, ftl: String) -> Result<Self, String> {
        Ok(Self(L10nLanguage::new(lang, ftl)?))
    }

    fn msg_twenty_four_hour(&self) -> Cow<'_, str> {
        self.0.msg("twenty-four-hour", None).unwrap()
    }
    fn msg_greeting(&self) -> Cow<'_, str> {
        self.0.msg("greeting", None).unwrap()
    }
}
