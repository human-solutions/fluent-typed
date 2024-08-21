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
pub struct LangResource {
    pub name: String,
    pub content: Vec<Message>,
}

#[derive(Debug)]
pub struct LangBundle {
    pub language: String,
    pub resources: Vec<LangResource>,
}

/// Generate rust code from locales folder, which contains `<lang-id>/<resource-name>.ftl` files.
///
/// The generation should be done in a build script:
///
/// ```no_run
/// // in build.rs
/// fn main() -> std::process::ExitCode {
///    fluent_typed::build_from_locales_folder("locales", "src/l10n.rs", "    ")
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
/// The last argument is the indentation used in the generated file. It is typically four spaces.
///
pub fn build_from_locales_folder(
    locales: &str,
    output_file_path: &str,
    indentation: &'static str,
) -> ExitCode {
    match try_build_from_locales_folder(locales, output_file_path, indentation) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

/// Same as [build_from_locales_folder], but returns result instead of an ExitCode.
pub fn try_build_from_locales_folder(
    locales: &str,
    output_file_path: &str,
    indentation: &'static str,
) -> Result<(), String> {
    println!("cargo::rerun-if-changed={locales}");

    let locales = build::from_locales_folder(locales)
        .map_err(|e| format!("Could not read locales folder '{locales}': {e:?}"))?;

    let analyzed = build::analyze(&locales);

    let generated = generate_from_locales(&locales, &analyzed)?.replace("    ", indentation);

    for warn in analyzed.missing_messages {
        println!("cargo::warning={warn}");
    }
    for warn in analyzed.signature_mismatches {
        println!("cargo::warning={warn}");
    }

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
    locales: &[LangBundle],
    analyzed: &Analyzed,
) -> Result<String, String> {
    let mut added = HashSet::new();
    let messages = locales
        .iter()
        .flat_map(|l| &l.resources)
        .flat_map(|r| &r.content)
        .filter(|msg| analyzed.common.contains(&msg.id))
        .filter(|msg| added.insert(&msg.id));

    let resources: HashSet<&str> = locales
        .iter()
        .map(|loc| {
            loc.resources
                .iter()
                .map(|res| res.name.as_str())
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect();
    let mut resources = resources.iter().map(|r| r.as_ref()).collect::<Vec<_>>();
    resources.sort();
    Ok(generate(&resources, messages))
}
