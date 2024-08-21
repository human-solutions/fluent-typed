mod build;
mod ext;
mod gen;
mod l10n_language;
#[cfg(test)]
mod tests;
mod typed;

use ext::StrExt;
use typed::Message;

pub use build::build_from_locales_folder;

pub use l10n_language::L10nLanguage;
