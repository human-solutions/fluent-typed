#![allow(unused)]
mod multi_l10n;
mod single_gzip_l10n;
mod single_l10n;

/// `single_l10n` is generated with `BuildOptions::with_langneg`, so the
/// negotiation function exists and falls back to the default language.
pub fn negotiated_language(accept_language: &str) -> &'static str {
    let lang = single_l10n::L10n::langneg(accept_language);
    let _ = format!("{:?}", lang.load()); // L10nLanguage is Debug
    match lang {
        single_l10n::L10n::En => "en",
        single_l10n::L10n::Fr => "fr",
    }
}
