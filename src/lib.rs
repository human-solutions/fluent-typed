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
    pub use icu_locale_core::{langid, LanguageIdentifier};

    #[cfg(feature = "langneg")]
    pub fn negotiate_languages<'a, A>(accept_language: &str, available: &'a [A]) -> A
    where
        A: 'a + AsRef<LanguageIdentifier> + PartialEq + Default + Copy,
    {
        // Parse Accept-Language header into (LanguageIdentifier, quality) pairs, sorted by quality descending
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
                tag.parse::<LanguageIdentifier>().ok().map(|lid| (lid, quality))
            })
            .collect();
        requested.sort_by(|a, b| b.1.cmp(&a.1));

        // Find the first available language whose language subtag matches a requested one
        for (req, _) in &requested {
            for avail in available {
                if avail.as_ref().language == req.language {
                    return *avail;
                }
            }
        }
        A::default()
    }
}
