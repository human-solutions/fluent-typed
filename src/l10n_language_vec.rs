use std::ops::Range;

use crate::error::L10nError;
use crate::prelude::L10nBundle;

pub struct L10nLanguageVec {
    langs: Vec<L10nBundle>,
}

impl L10nLanguageVec {
    /// Load all languages, with Unicode bidi isolation marks around
    /// interpolated variables. See [`L10nBundle::new`].
    pub fn load<S, I>(bytes: &[u8], iter: I) -> Result<Self, L10nError>
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, Range<usize>)>,
    {
        Self::build(bytes, iter, true)
    }

    /// Load all languages, without Unicode bidi isolation marks. See
    /// [`L10nBundle::new_without_isolation`].
    pub fn load_without_isolation<S, I>(bytes: &[u8], iter: I) -> Result<Self, L10nError>
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, Range<usize>)>,
    {
        Self::build(bytes, iter, false)
    }

    fn build<S, I>(bytes: &[u8], iter: I, use_isolating: bool) -> Result<Self, L10nError>
    where
        S: AsRef<str>,
        I: Iterator<Item = (S, Range<usize>)>,
    {
        Ok(Self {
            langs: iter
                .map(|(lang, range)| {
                    let data = &bytes[range];
                    if use_isolating {
                        L10nBundle::new(lang, data)
                    } else {
                        L10nBundle::new_without_isolation(lang, data)
                    }
                })
                .collect::<Result<Vec<_>, L10nError>>()?,
        })
    }

    /// Add `bundle` to the loaded set, replacing any bundle already loaded for
    /// the same language.
    ///
    /// This is how a translation that was not part of the build becomes
    /// available at runtime: load an external `.ftl` (with the generated
    /// `L10nLanguage::new_external`, or [`L10nBundle::new`] directly) and
    /// insert it. Inserting an already-loaded language replaces it, which is
    /// the hot-reload path for a translation under revision.
    pub fn insert(&mut self, bundle: L10nBundle) {
        match self.langs.iter_mut().find(|b| b.lang() == bundle.lang()) {
            Some(slot) => *slot = bundle,
            None => self.langs.push(bundle),
        }
    }

    /// Negotiate the best of the *loaded* languages for an `Accept-Language`
    /// header — including any added at runtime with [`Self::insert`], which
    /// the generated `L10n::langneg` (compiled from the build-time language
    /// set) can never return.
    ///
    /// Returns `None` when nothing matches; there is no compiled-in default
    /// here, so the caller picks its own fallback, e.g.
    /// `vec.langneg(header).unwrap_or_else(|| vec.get(L10n::default()))`.
    #[cfg(feature = "langneg")]
    pub fn langneg(&self, accept_language: &str) -> Option<&L10nBundle> {
        use icu_locale_core::LanguageIdentifier;

        let available: Vec<Option<LanguageIdentifier>> =
            self.langs.iter().map(|b| b.lang().parse().ok()).collect();
        for req in crate::prelude::requested_languages(accept_language) {
            let found = available
                .iter()
                .position(|a| a.as_ref().is_some_and(|a| a.language == req.language));
            if let Some(idx) = found {
                return Some(&self.langs[idx]);
            }
        }
        None
    }

    /// Get the bundle for `lang`, or `None` if no such language was loaded.
    ///
    /// `lang` is matched against the language ids the vec was loaded from (the
    /// `L10n` enum variants). Reach for this when `lang` comes from outside that
    /// set — a raw header, a query parameter, user input — and a miss is a case
    /// you want to handle rather than a bug. When you already hold a known `L10n`
    /// variant, [`Self::get`] is more convenient.
    pub fn try_get(&self, lang: impl AsRef<str>) -> Option<&L10nBundle> {
        let lang = lang.as_ref();
        self.langs.iter().find(|b| b.lang() == lang)
    }

    /// Get the bundle for `lang`.
    ///
    /// Pass an `L10n` enum variant (one of the languages the vec was loaded
    /// from); the lookup is then guaranteed to succeed. The variant returned by
    /// the generated `L10n::langneg` always qualifies.
    ///
    /// # Panics
    ///
    /// Panics if `lang` was not loaded. Use [`Self::try_get`] when `lang` may be
    /// outside the loaded set and you want to handle the miss.
    pub fn get(&self, lang: impl AsRef<str>) -> &L10nBundle {
        let lang = lang.as_ref();
        self.try_get(lang).unwrap_or_else(|| {
            panic!(
                "L10nLanguageVec::get: language '{lang}' was not loaded. Pass an \
                 `L10n` enum variant (the languages the vec was loaded from), or \
                 use `try_get` to handle a missing language."
            )
        })
    }
}
