#![doc = include_str!("../README.md")]
#[cfg(any(doc, feature = "build"))]
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

#[cfg(any(doc, feature = "build"))]
pub use build::{
    BuildError, BuildOptions, FtlOutputOptions, LintLevel, build_from_locales_folder,
    try_build_from_locales_folder,
};

/// Internal build-pipeline pieces exposed for the benchmark suite only. Gated
/// behind the non-default `bench-internals` feature; not a stable API.
#[cfg(feature = "bench-internals")]
pub use build::bench_internals;

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

    /// Pick the first of `available` whose language subtag matches an entry of
    /// the `Accept-Language` header (highest quality first), or
    /// `A::default()` when none matches. The generated `L10n::langneg` calls
    /// this with every built-in language.
    #[cfg(feature = "langneg")]
    pub fn negotiate_languages<'a, A>(accept_language: &str, available: &'a [A]) -> A
    where
        A: 'a + AsRef<LanguageIdentifier> + PartialEq + Default + Copy,
    {
        // Find the first available language whose language subtag matches a requested one
        for req in requested_languages(accept_language) {
            for avail in available {
                if avail.as_ref().language == req.language {
                    return *avail;
                }
            }
        }
        A::default()
    }
}
