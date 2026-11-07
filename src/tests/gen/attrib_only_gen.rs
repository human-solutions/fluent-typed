// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{
    fmt::Display,
    ops::{Deref, Range},
    slice::Iter,
    str::FromStr,
};

static LANG_DATA: &[u8] = include_bytes!("attrib_only_gen.ftl");
/// The message contract of every generated accessor: what an `.ftl` must
/// define for this API. Pass it to `fluent_typed::validate_ftl` to check a
/// translation without loading it (e.g. in CI).
#[allow(dead_code)]
pub static MESSAGE_CONTRACTS: &[MessageContract] = &[MessageContract {
    message: "hello",
    attribute: Some("tooltip"),
    vars: &["userName"],
    bool_vars: &[],
    elements: &[],
}];
#[allow(dead_code)]
static ALL_LANGS: [L10n; 1] = [
    // languages as an array
    L10n::En,
];

/// The languages that have translations available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum L10n {
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
            "en" => Ok(Self::En),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl Deref for L10n {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::En => "en",
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

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::En => 0..87,
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
    /// On success, every accessor on the returned [`L10nLanguage`] is safe to
    /// call: the external translation defines every generated message and
    /// references no unknown variables. A translation that does not — because
    /// a message is missing, misspelled, or references a variable this API
    /// never fills — is rejected here with `L10nError::Validation` (listing
    /// every violation) instead of panicking later in an accessor.
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

    /// This is a message comment
    pub fn msg_hello_tooltip<'a, F0: Into<FluentValue<'a>>>(&self, user_name: F0) -> String {
        let mut args = FluentArgs::new();
        args.set("userName", user_name);
        self.0.attr("hello", "tooltip", Some(args)).unwrap()
    }
}
