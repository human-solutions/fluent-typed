// This file is generated. Do not edit it manually.
use crate::prelude::*;
use std::{
    fmt::Display,
    ops::{Deref, Range},
    slice::Iter,
    str::FromStr,
};

static LANG_DATA: &[u8] = include_bytes!("msg_element_gen.ftl");

static ALL_LANGS: [L10n; 1] = [
    // languages as an array
    L10n::En,
];

static EN: LanguageIdentifier = langid!("en");

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

impl AsRef<LanguageIdentifier> for L10n {
    fn as_ref(&self) -> &LanguageIdentifier {
        match self {
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

    /// Negotiate the best language to use based on the `Accept-Language` header.
    ///
    /// Falls back to the default language if none of the languages in the header are available.
    pub fn langneg(accept_language: &str) -> L10n {
        negotiate_languages(accept_language, &ALL_LANGS)
    }

    fn byte_range(&self) -> Range<usize> {
        match self {
            Self::En => 0..434,
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
    pub fn new(lang: impl AsRef<str>, bytes: &[u8]) -> Result<Self, L10nError> {
        Ok(Self(L10nBundle::new(lang, bytes)?))
    }

    /// $num (Number) - How many.
    /// $provider (String) - The calendar provider.
    /// $icon (Element) - A UI element injected by the app.
    /// -privacy-link (Element) - Translatable text the app wraps.
    pub fn calendar_sync_description<F0: Into<FluentNumber>, F1: AsRef<str>>(
        &self,
        num: F0,
        provider: F1,
    ) -> CalendarSyncDescription {
        let mut args = FluentArgs::new();
        args.set("num", num.into());
        args.set("provider", provider.as_ref());
        let segments: [Segment; 5] = self
            .0
            .msg_segments(
                "calendar-sync-description",
                &["icon"],
                &["privacy-link"],
                Some(args),
            )
            .unwrap()
            .try_into()
            .unwrap();
        let [seg0, _, seg2, seg3, seg4] = segments;
        CalendarSyncDescription {
            s0: seg0.into_text(),
            icon: ElementGap,
            s1: seg2.into_text(),
            privacy_link: seg3.into_text(),
            s2: seg4.into_text(),
        }
    }
}

/// Structured output for message 'calendar-sync-description'.
///
/// Render the fields in declaration order. To keep bidirectional text correct,
/// wrap each field — and each element the app injects — in an isolated bidi run
/// (an HTML `<bdi>`, or `unicode-bidi: isolate`), with the container set to the
/// locale's base direction.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarSyncDescription {
    /// A resolved run of translated text.
    pub s0: String,
    /// Element slot: the app injects its own UI element here.
    pub icon: ElementGap,
    /// A resolved run of translated text.
    pub s1: String,
    /// Resolved text of an `(Element)` term; wrap it in the app.
    pub privacy_link: String,
    /// A resolved run of translated text.
    pub s2: String,
}

impl ::core::fmt::Display for CalendarSyncDescription {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{}{}{}{}", self.s0, self.s1, self.privacy_link, self.s2)
    }
}
