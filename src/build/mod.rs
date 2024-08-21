mod loader;
mod validations;

use crate::{build, gen::generate};
pub use loader::from_locales_folder;
use std::{collections::HashSet, fs, process::ExitCode};
pub use validations::{analyze, Analyzed};

use crate::Message;

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

pub fn build_from_locales_folder(locales_folder: &str, rust_code_folder: &str) -> ExitCode {
    println!("cargo::rerun-if-changed={locales_folder}");
    match try_build_from_folder(locales_folder, rust_code_folder) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

pub(crate) fn try_build_from_folder(
    locales_folder: &str,
    rust_code_folder: &str,
) -> Result<(), String> {
    let locales = build::from_locales_folder(locales_folder)
        .map_err(|e| format!("Could not read locales folder '{locales_folder}': {e:?}"))?;

    let analyzed = build::analyze(&locales);

    let generated = generate_from_locales(&locales, &analyzed)?;

    fs::create_dir_all(rust_code_folder)
        .map_err(|e| format!("Could not create rust folder '{rust_code_folder}': {e:?}"))?;

    let filename = format!("{}/l10n.rs", rust_code_folder);

    fs::write(filename, generated)
        .map_err(|e| format!("Could not write rust file '{rust_code_folder}': {e:?}"))?;

    for warn in analyzed.missing_messages {
        println!("cargo::warning={warn}");
    }
    for warn in analyzed.signature_mismatches {
        println!("cargo::warning={warn}");
    }
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

    let resources: Vec<&str> = locales
        .iter()
        .map(|loc| {
            loc.resources
                .iter()
                .map(|res| res.name.as_str())
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect();
    Ok(generate(&resources, messages))
}
