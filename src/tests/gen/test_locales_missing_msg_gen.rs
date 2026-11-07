// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{fmt::Display, ops::Deref, slice::Iter, str::FromStr};

static LANG_DATA: &[u8] = include_bytes!("test_locales_missing_msg.ftl");
/// The message contract of every generated accessor: what an `.ftl` must
/// define for this API. Pass it to `fluent_typed::validate_ftl` to check a
/// translation without loading it (e.g. in CI).
#[allow(dead_code)]
pub static MESSAGE_CONTRACTS: &[MessageContract] = &[
    MessageContract {
        message: "twenty-four-hour",
        attribute: None,
        vars: &[],
        bool_vars: &[],
        elements: &[],
    },
    MessageContract {
        message: "twelve-hour",
        attribute: None,
        vars: &[],
        bool_vars: &[],
        elements: &[],
    },
];
#[allow(dead_code)]
static ALL_LANGS: [L10n; 2] = [
    // languages as an array
    L10n::De,
    L10n::EnGb,
];

/// The languages that have translations available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
    De,
    EnGb,
}

impl Default for L10n {
    fn default() -> Self {
        Self::EnGb
    }
}

impl FromStr for L10n {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "de" => Ok(Self::De),
            "en-gb" => Ok(Self::EnGb),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl Deref for L10n {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::De => "de",
            Self::EnGb => "en-gb",
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

// Language helpers: not every app uses all of them, and an unused one is not a
// stale translation, so they don't produce dead-code warnings.
#[allow(dead_code)]
impl L10n {
    /// All languages with built-in translations.
    pub fn iter() -> Iter<'static, L10n> {
        ALL_LANGS.iter()
    }

    fn byte_range(&self) -> std::ops::Range<usize> {
        match self {
            Self::De => 0..107,
            Self::EnGb => 107..208,
        }
    }
    /// Load a L10nLanguage from the embedded data.
    pub fn load(&self) -> L10nLanguage {
        L10nLanguage::new(self, &LANG_DATA[self.byte_range()])
            .expect("fluent-typed: the embedded .ftl could not be loaded. This is a build-time bug; please report it.")
    }

    /// Load all languages (L10nLanguage) from the embedded data.
    pub fn load_all() -> L10nLanguageVec {
        L10nLanguageVec::load(
            LANG_DATA,
            Self::iter().map(|lang| (lang, lang.byte_range())),
        )
        .expect("fluent-typed: the embedded .ftl could not be loaded. This is a build-time bug; please report it.")
    }
}

/// A thin wrapper around the Fluent messages for one language.
///
/// It provides functions for each message that was found in
/// all the languages at build time.
#[derive(Debug)]
pub struct L10nLanguage(L10nBundle);

impl L10nLanguage {
    /// Load the L10n resources for the given language. The language
    /// has to be a valid LanguageIdentifier or otherwise
    /// an error is returned.
    ///
    /// The bytes are expected to be the contents of a .ftl file. An owned
    /// buffer (`Vec<u8>`, `String`) is moved in without copying; a borrowed
    /// one (`&[u8]`, `&str`) is copied once.
    #[allow(dead_code)]
    pub fn new(lang: impl AsRef<str>, bytes: impl Into<Vec<u8>>) -> Result<Self, L10nError> {
        Ok(Self(L10nBundle::new(lang, bytes)?))
    }

    /// Load external `.ftl` bytes — read from disk, downloaded, etc. — after
    /// validating them against the message contract compiled into this API.
    ///
    /// A translation where a message is missing or misspelled, uses a variable
    /// this API never passes, drops an `(Element)` marker, references a
    /// message or term that doesn't exist, or calls a function other than
    /// `NUMBER()` is rejected here with `L10nError::Validation` (listing every
    /// violation) instead of panicking later in an accessor.
    ///
    /// Use this to load a language that was not compiled in, or to hot-reload
    /// a translation under revision. Pass the buffer by value (e.g. straight
    /// from `std::fs::read`) to avoid a copy.
    #[allow(dead_code)]
    pub fn new_external(
        lang: impl AsRef<str>,
        bytes: impl Into<Vec<u8>>,
    ) -> Result<Self, L10nError> {
        let bytes = bytes.into();
        validate_ftl(&bytes, MESSAGE_CONTRACTS)?;
        Self::new(lang, bytes)
    }

    pub fn msg_twenty_four_hour(&self) -> String {
        self.0.msg("twenty-four-hour", None).unwrap()
    }
    pub fn msg_twelve_hour(&self) -> String {
        self.0.msg("twelve-hour", None).unwrap()
    }
}
