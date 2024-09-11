pub mod gen;
mod loader;
pub mod options;
mod store;
pub mod typed;
mod validations;

use gen::generate;
pub use loader::from_locales_folder;
pub use options::{BuildOptions, FtlOutputOptions};
use std::{collections::HashSet, fs, process::ExitCode};
pub use typed::Message;
pub use validations::{analyze, Analyzed};

#[derive(Debug)]
pub struct LangBundle {
    pub language: String,
    pub messages: Vec<Message>,
    pub ftl: String,
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

    let locales = from_locales_folder(locales)
        .map_err(|e| format!("Could not read locales folder '{locales}': {e:?}"))?;

    let analyzed = analyze(&locales);

    let generated =
        generate_from_locales(&options, &locales, &analyzed)?.replace("    ", &options.indentation);

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
    options: &BuildOptions,
    locales: &[LangBundle],
    analyzed: &Analyzed,
) -> Result<String, String> {
    let generated_ftl = options.ftl_output.generate(&locales)?;

    let mut added = HashSet::new();
    let messages = locales
        .iter()
        .flat_map(|r| &r.messages)
        .filter(|msg| analyzed.common.contains(&msg.id))
        .filter(|msg| added.insert(&msg.id));

    let mut langs = locales
        .iter()
        .map(|r| r.language.as_str())
        .collect::<Vec<_>>();
    langs.sort();
    Ok(generate(options, &langs, generated_ftl, messages))
}
