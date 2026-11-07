// The README and the guides call generated code, which doesn't exist when
// rustdoc compiles doctests, so they're excluded from doctests. Their examples
// are compiled and run by playground/docs-check instead.
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
// Documented even without the feature (docs.rs enables it anyway), but not
// collected as doctests then: its examples need the feature to compile.
#[cfg(any(all(doc, not(doctest)), feature = "build"))]
mod build;
mod contract;
mod error;
pub(crate) mod ftl_refs;
mod l10n_bundle;
mod l10n_language_vec;
mod structured;

pub use contract::{ContractViolation, ElementContract, MessageContract, validate_ftl};
pub use error::L10nError;

#[cfg(all(test, feature = "build"))]
mod tests;

// The unit tests need the build pipeline. Without this, a plain `cargo test`
// (no features, since the defaults are empty) would compile zero tests and
// report success.
#[cfg(all(test, not(feature = "build")))]
compile_error!("run the tests with `cargo test --features build,langneg`");

/// Emitted by generated code built with `BuildOptions::with_langneg`, so a
/// missing runtime `langneg` feature fails with an actionable message instead
/// of unresolved-name errors inside the generated file.
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "langneg")]
macro_rules! __require_langneg {
    () => {};
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "langneg"))]
macro_rules! __require_langneg {
    () => {
        compile_error!(
            "the generated l10n code uses language negotiation (`BuildOptions::with_langneg`), \
             so enable the `langneg` feature of fluent-typed in [dependencies]: \
             fluent-typed = { version = \"…\", features = [\"langneg\"] }"
        );
    };
}

// Documented even without the feature (docs.rs enables it anyway), but not
// collected as doctests then: its examples need the feature to compile.
#[cfg(any(all(doc, not(doctest)), feature = "build"))]
pub use build::{
    BuildError, BuildOptions, FtlOutputOptions, LintLevel, build_from_locales_folder,
    try_build_from_locales_folder,
};

/// Internal build-pipeline pieces exposed for the benchmark suite only. Gated
/// behind the non-default `bench-internals` feature; not a stable API.
#[cfg(feature = "bench-internals")]
pub use build::bench_internals;

/// Guides, also readable on GitHub in the `docs/` folder.
pub mod guide {
    /// Embed all languages, compress them, or load one language at a time.
    #[cfg_attr(not(doctest), doc = include_str!("../docs/output-modes.md"))]
    pub mod output_modes {}

    /// The `L10n` enum, `Accept-Language` negotiation, and serving every
    /// language from one server.
    #[cfg_attr(not(doctest), doc = include_str!("../docs/languages.md"))]
    pub mod languages {}

    /// Add a language after release, hot-reload a translation, validate a file.
    #[cfg_attr(not(doctest), doc = include_str!("../docs/runtime-loading.md"))]
    pub mod runtime_loading {}

    /// How argument types are decided, and the linter for type comments.
    #[cfg_attr(not(doctest), doc = include_str!("../docs/types-and-linting.md"))]
    pub mod types_and_linting {}

    /// Elements in the middle of a sentence, and bidi isolation.
    #[cfg_attr(not(doctest), doc = include_str!("../docs/structured-messages.md"))]
    pub mod structured_messages {}
}

/// Everything the generated code needs, imported with
/// `use fluent_typed::prelude::*;` at the top of the generated file.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::__require_langneg as require_langneg;
    pub use crate::contract::{ContractViolation, ElementContract, MessageContract, validate_ftl};
    pub use crate::error::L10nError;
    pub use crate::l10n_bundle::L10nBundle;
    pub use crate::l10n_language_vec::L10nLanguageVec;
    pub use crate::structured::{ElementGap, Segment};
    pub use fluent_bundle::{FluentArgs, FluentValue, types::FluentNumber};
    #[cfg(feature = "langneg")]
    pub use icu_locale_core::{LanguageIdentifier, langid};

    /// Parse an `Accept-Language` header into language identifiers, sorted by
    /// quality (highest first; equal qualities keep their order). Entries that
    /// don't parse, have an invalid `q`, or have `q=0` ("not acceptable") are
    /// skipped.
    #[cfg(feature = "langneg")]
    pub(crate) fn requested_languages(accept_language: &str) -> Vec<LanguageIdentifier> {
        let mut requested: Vec<(LanguageIdentifier, u16)> = accept_language
            .split(',')
            .filter_map(|entry| {
                let mut parts = entry.split(';');
                let tag = parts.next()?.trim();
                let quality = match parts.find_map(|p| p.trim().strip_prefix("q=")) {
                    None => 1000,
                    Some(q) => {
                        let q = q.trim().parse::<f32>().ok()?;
                        // q=0 means "not acceptable", and NaN isn't a weight.
                        if q.is_nan() || q <= 0.0 {
                            return None;
                        }
                        (q.min(1.0) * 1000.0) as u16
                    }
                };
                tag.parse::<LanguageIdentifier>()
                    .ok()
                    .map(|lid| (lid, quality))
            })
            .collect();
        requested.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        requested.into_iter().map(|(lid, _)| lid).collect()
    }

    /// Pick the best of `available` for an `Accept-Language` header, or
    /// `A::default()` when none matches. The generated `L10n::langneg` calls
    /// this with every built-in language.
    ///
    /// The header's entries are tried highest quality first, and the first
    /// entry with any match wins. For one entry, such as `en-GB`, the best
    /// match is, in order:
    ///
    /// 1. `en-GB` itself;
    /// 2. the same region, with no conflicting script (`en-Latn-GB`);
    /// 3. the generic locale of the language (`en`);
    /// 4. another region with no conflicting script (`en-US`);
    /// 5. a locale with a different script than the one requested (for
    ///    `sr-Latn`: `sr-Cyrl`).
    ///
    /// Ties go to the locale without a script the request didn't ask for, then
    /// to the order of `available`. Scripts are not inferred from regions:
    /// `zh-TW` doesn't know it means Traditional Chinese, so with `zh-Hans`
    /// and `zh-Hant` available, name the script in the locale folders that
    /// have regions (`zh-Hant-TW`) or add the regional locale.
    #[cfg(feature = "langneg")]
    pub fn negotiate_languages<A>(accept_language: &str, available: &[A]) -> A
    where
        A: AsRef<LanguageIdentifier> + Default + Copy,
    {
        match best_match(accept_language, available.len(), |i| {
            Some(available[i].as_ref())
        }) {
            Some(idx) => available[idx],
            None => A::default(),
        }
    }

    /// The index of the best match among `len` candidates for an
    /// `Accept-Language` header, as described on [`negotiate_languages`].
    /// `candidate(i)` returns the `i`th candidate's id; `None` never matches.
    /// Shared by the generated `L10n::langneg` and `L10nLanguageVec::langneg`.
    #[cfg(feature = "langneg")]
    pub(crate) fn best_match<'a>(
        accept_language: &str,
        len: usize,
        candidate: impl Fn(usize) -> Option<&'a LanguageIdentifier>,
    ) -> Option<usize> {
        requested_languages(accept_language).iter().find_map(|req| {
            (0..len)
                .filter_map(|i| Some((rank(req, candidate(i)?)?, i)))
                .min()
                .map(|(_, i)| i)
        })
    }

    /// How well `a` matches the request `req`: lower is better, `None` is no
    /// match. The first element is the tier listed on [`negotiate_languages`];
    /// the second prefers a locale without a script `req` didn't ask for.
    #[cfg(feature = "langneg")]
    fn rank(req: &LanguageIdentifier, a: &LanguageIdentifier) -> Option<(u8, bool)> {
        if a.language != req.language {
            return None;
        }
        if a == req {
            return Some((0, false));
        }
        let script_conflict = matches!((req.script, a.script), (Some(r), Some(s)) if r != s);
        let extra_script = req.script.is_none() && a.script.is_some();
        let tier = if script_conflict {
            5
        } else if req.region.is_some() && a.region == req.region {
            2
        } else if a.region.is_none() {
            3
        } else {
            4
        };
        Some((tier, extra_script))
    }
}
