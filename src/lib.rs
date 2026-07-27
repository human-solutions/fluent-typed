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

#[cfg(any(doc, feature = "build"))]
pub use build::{
    BuildError, BuildOptions, FtlOutputOptions, LintLevel, build_from_locales_folder,
    try_build_from_locales_folder,
};

/// Internal build-pipeline pieces exposed for the benchmark suite only. Gated
/// behind the non-default `bench-internals` feature; not a stable API.
#[cfg(feature = "bench-internals")]
pub use build::bench_internals;

pub mod prelude {
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
