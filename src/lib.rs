mod ext;
mod gen;
mod loader;
#[cfg(test)]
mod tests;
mod typed;

use std::collections::HashMap;
use std::fs;
use std::process::ExitCode;
use typed::generate_for_messages;
use typed::Id;

use ext::StrExt;
pub use typed::generate_code;
pub use typed::BundleMessageExt;
pub use typed::Message;

pub struct LangResource {
    pub name: String,
    pub content: Vec<Message>,
}
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
    let generated = generate_from_locales(locales_folder)?;
    fs::create_dir_all(rust_code_folder)
        .map_err(|e| format!("Could not create rust folder '{rust_code_folder}': {e:?}"))?;
    let filename = format!("{}/bundle_ext.rs", rust_code_folder);

    fs::write(filename, generated)
        .map_err(|e| format!("Could not write rust file '{rust_code_folder}': {e:?}"))?;
    Ok(())
}

pub(crate) fn generate_from_locales(locales_folder: &str) -> Result<String, String> {
    let locales = loader::from_locales_folder(locales_folder)
        .map_err(|e| format!("Could not read locales folder '{locales_folder}': {e:?}"))?;

    let messages = locales
        .into_iter()
        .flat_map(|l| l.resources)
        .flat_map(|r| r.content)
        .map(|msg| (msg.id.clone(), msg))
        .collect::<HashMap<Id, Message>>();

    Ok(generate_for_messages(messages.values()))
}
