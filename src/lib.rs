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
    /// quality (highest first). Unparseable entries are skipped.
    #[cfg(feature = "langneg")]
    pub(crate) fn requested_languages(accept_language: &str) -> Vec<LanguageIdentifier> {
        let mut requested: Vec<(LanguageIdentifier, u16)> = accept_language
            .split(',')
            .filter_map(|entry| {
                let entry = entry.trim();
                if entry.is_empty() {
                    return None;
                }
                let (tag, quality) = if let Some((tag, params)) = entry.split_once(';') {
                    let q = params
                        .trim()
                        .strip_prefix("q=")
                        .and_then(|v| v.parse::<f32>().ok())
                        .unwrap_or(1.0);
                    (tag.trim(), (q * 1000.0) as u16)
                } else {
                    (entry, 1000)
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
    /// match is, in order: `en-GB` itself; the same language and region with
    /// another script; the generic `en`; then any other `en-*`, in the order of
    /// `available`.
    #[cfg(feature = "langneg")]
    pub fn negotiate_languages<'a, A>(accept_language: &str, available: &'a [A]) -> A
    where
        A: 'a + AsRef<LanguageIdentifier> + PartialEq + Default + Copy,
    {
        let ids: Vec<Option<&LanguageIdentifier>> =
            available.iter().map(|a| Some(a.as_ref())).collect();
        match best_match(accept_language, &ids) {
            Some(idx) => available[idx],
            None => A::default(),
        }
    }

    /// The index of the best match in `available` for an `Accept-Language`
    /// header, as described on [`negotiate_languages`]; `None` entries never
    /// match. Shared by the generated `L10n::langneg` and
    /// `L10nLanguageVec::langneg`.
    #[cfg(feature = "langneg")]
    pub(crate) fn best_match(
        accept_language: &str,
        available: &[Option<&LanguageIdentifier>],
    ) -> Option<usize> {
        for req in requested_languages(accept_language) {
            let same_language = |a: &LanguageIdentifier| a.language == req.language;
            let tiers: [&dyn Fn(&LanguageIdentifier) -> bool; 4] = [
                &|a| *a == req,
                &|a| same_language(a) && a.region == req.region,
                &|a| same_language(a) && a.region.is_none(),
                &same_language,
            ];
            for tier in tiers {
                if let Some(idx) = available.iter().position(|a| a.is_some_and(tier)) {
                    return Some(idx);
                }
            }
        }
        None
    }
}
