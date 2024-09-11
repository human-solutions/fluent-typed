#[cfg(feature = "build")]
mod build;
mod l10n_language;

#[cfg(all(test, feature = "build"))]
mod tests;

#[cfg(feature = "build")]
pub use build::{
    build_from_locales_folder, try_build_from_locales_folder, BuildOptions, FtlOutputOptions,
};

pub mod prelude {
    pub use crate::l10n_language::L10nLanguage;
    pub use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
}
