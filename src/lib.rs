#[cfg(feature = "build")]
mod build;
mod l10n_language;

#[cfg(all(test, feature = "build"))]
mod tests;

#[cfg(feature = "build")]
pub use build::{build_from_locales_folder, try_build_from_locales_folder, BuildOptions};

pub mod prelude {
    pub use crate::l10n_language::L10nLanguage;
    pub use fluent_bundle::{types::FluentNumber, FluentArgs, FluentValue};
}

mod l10n {
    #![allow(dead_code)]
    pub enum L10Lang {
        En,
        Fr,
    }

    impl IntoIterator for L10Lang {
        type IntoIter = std::vec::IntoIter<Self>;
        type Item = Self;

        fn into_iter(self) -> Self::IntoIter {
            vec![Self::En, Self::Fr].into_iter()
        }
    }

    pub trait L10nStore {
        fn language(&self, lang: &L10Lang) -> Result<&str, Box<dyn std::error::Error>>;
    }

    /// Loads all languages from the store.
    /// This is typically used server-side to load all languages at once inside a [LazyLock](std::sync::LazyLock)
    /// so that they are easily accessible across threads.
    pub struct L10nLanguageCollection {
        pub en: L10n,
    }

    pub struct L10n {}

    impl L10nLanguageCollection {
        pub fn load(_store: impl L10nStore) -> Result<Self, Box<dyn std::error::Error>> {
            todo!()
        }
        pub fn get(&self, _lang: &L10Lang) -> &L10n {
            todo!()
        }
    }
}
