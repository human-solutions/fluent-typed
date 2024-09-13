#![doc = include_str!("../README.md")]
#[cfg(any(doc, feature = "build"))]
mod build;
mod l10n_bundle;
mod l10n_language_vec;

#[cfg(all(test, feature = "build"))]
mod tests;

#[cfg(any(doc, feature = "build"))]
pub use build::{
    build_from_locales_folder, try_build_from_locales_folder, BuildOptions, FtlOutputOptions,
};

pub mod prelude {
    pub use crate::l10n_bundle::L10nBundle;
    pub use crate::l10n_language_vec::L10nLanguageVec;
    pub use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
    #[cfg(feature = "langneg")]
    pub use icu_locid::{langid, LanguageIdentifier};

    #[cfg(feature = "langneg")]
    pub fn negotiate_languages<'a, A: 'a + AsRef<LanguageIdentifier> + PartialEq>(
        accept_language: &str,
        available: &'a [A],
        default: &'a A,
    ) -> &'a A {
        use fluent_langneg::{
            negotiate_languages, parse_accepted_languages, NegotiationStrategy::Filtering,
        };
        let requested = parse_accepted_languages(accept_language);

        negotiate_languages(&requested, available, Some(default), Filtering)
            .first()
            .map(|l| *l)
            .unwrap_or(default)
    }
}
