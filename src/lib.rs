#![doc = include_str!("../README.md")]
#[cfg(any(doc, feature = "build"))]
mod build;
mod language_bundle;

#[cfg(all(test, feature = "build"))]
mod tests;

#[cfg(any(doc, feature = "build"))]
pub use build::{
    build_from_locales_folder, try_build_from_locales_folder, BuildOptions, FtlOutputOptions,
};

pub mod prelude {
    pub use crate::language_bundle::LanguageBundle;
    pub use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
    pub use unic_langid::{langid, LanguageIdentifier};
}
