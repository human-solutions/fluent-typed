// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::str::FromStr;
use std::{borrow::Cow, collections::HashMap};
use unic_langid::{langid, LanguageIdentifier};

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
    pub fn to_str(&self) -> &'static str {
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

    fn msg_hello(&self) -> Cow<'_, str> {
        self.0.msg("hello", None).unwrap()
    }
    fn msg_hello_tooltip<'a, F0: Into<FluentValue<'a>>>(&self, user_name: F0) -> Cow<'_, str> {
        let mut args = FluentArgs::new();
        args.set("userName", user_name);
        self.0.attr("hello", "tooltip", Some(args)).unwrap()
    }
}
