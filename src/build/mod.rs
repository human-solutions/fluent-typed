pub mod gen;
mod loader;
pub mod typed;
mod validations;

use crate::build;
use gen::generate;
pub use loader::from_locales_folder;
use std::{collections::HashSet, fs, process::ExitCode};
pub use validations::{analyze, Analyzed};

pub use typed::Message;

#[derive(Debug)]
pub struct LangBundle {
    pub language: String,
    pub messages: Vec<Message>,
    pub ftl: String,
}

pub struct BuildOptions {
    /// The path to the folder containing the locales.
    ///
    /// Defaults to "locales".
    pub locales_folder: String,
    /// The path to the file where the generated code will be written.
    ///
    /// Defaults to "src/l10n.rs".
    pub output_file_path: String,
    /// The prefix is a simple string that will be added to all generated function names.
    ///
    /// Defaults to "msg_".
    pub prefix: String,
    /// The indentation used in the generated file.
    ///
    /// Defaults to four spaces.
    pub indentation: String,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            locales_folder: "locales".to_string(),
            output_file_path: "src/l10n.rs".to_string(),
            prefix: "msg_".to_string(),
            indentation: "    ".to_string(),
        }
    }
}

impl BuildOptions {
    pub fn with_locales_folder(mut self, locales_folder: &str) -> Self {
        self.locales_folder = locales_folder.to_string();
        self
    }
    pub fn with_output_file_path(mut self, output_file_path: &str) -> Self {
        self.output_file_path = output_file_path.to_string();
        self
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }
    pub fn with_indentation(mut self, indentation: &str) -> Self {
        self.indentation = indentation.to_string();
        self
    }
}

/// Generate rust code from locales folder, which contains `<lang-id>/<resource-name>.ftl` files.
///
/// The generation should be done in a build script:
///
/// ```no_run
/// // in build.rs
/// fn main() -> std::process::ExitCode {
///    let options = fluent_typed::BuildOptions::default();
///    fluent_typed::build_from_locales_folder(options)
/// }
/// ```
///
/// This requires the dependencies:
///
/// ```toml
/// # in Cargo.toml
/// [dependencies]
/// fluent-typed = 0.1
///
/// [build-dependencies]
/// fluent-typed = { version = "0.1", features = ["build"] }
/// ```
/// During the generation, the build script will print warnings for all messages that are
/// not present in all locales, as well as for messages with different signatures.
///
/// It is recommended to generate the rust code to the output_file_path "src/l10n.rs" and include
/// it in the project, so that you get warnings for unused translation messages.
///
/// See [BuildOptions] for more configuration options.
///
pub fn build_from_locales_folder(options: BuildOptions) -> ExitCode {
    match try_build_from_locales_folder(options) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

/// Same as [build_from_locales_folder], but returns result instead of an ExitCode.
pub fn try_build_from_locales_folder(options: BuildOptions) -> Result<(), String> {
    let locales = &options.locales_folder;
    println!("cargo::rerun-if-changed={locales}");

    let locales = build::from_locales_folder(locales)
        .map_err(|e| format!("Could not read locales folder '{locales}': {e:?}"))?;

    let analyzed = build::analyze(&locales);

    let generated = generate_from_locales(&options.prefix, &locales, &analyzed)?
        .replace("    ", &options.indentation);

    for warn in analyzed.missing_messages {
        println!("cargo::warning={warn}");
    }
    for warn in analyzed.signature_mismatches {
        println!("cargo::warning={warn}");
    }

    let output_file_path = &options.output_file_path;
    if let Some(current_file) = fs::read_to_string(output_file_path).ok() {
        if current_file == generated {
            return Ok(());
        }
    }

    fs::write(output_file_path, generated)
        .map_err(|e| format!("Could not write rust file '{output_file_path}': {e:?}"))?;

    Ok(())
}

pub fn generate_from_locales(
    prefix: &str,
    locales: &[LangBundle],
    analyzed: &Analyzed,
) -> Result<String, String> {
    let mut added = HashSet::new();
    let messages = locales
        .iter()
        .flat_map(|r| &r.messages)
        .filter(|msg| analyzed.common.contains(&msg.id))
        .filter(|msg| added.insert(&msg.id));

    Ok(generate(prefix, messages))
}
