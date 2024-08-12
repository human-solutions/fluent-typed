mod error;
mod ext;
#[cfg(test)]
mod tests;
mod typed;
use fluent_syntax::parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use typed::generate_for_messages;
use typed::to_messages;
use typed::Id;

pub use error::MessageError;
use ext::StrExt;
pub use typed::generate_code;
pub use typed::BundleMessageExt;
pub use typed::Message;

pub struct LangResource<C> {
    pub resource: String,
    pub content: C,
}
pub struct LangBundle<C> {
    pub language: String,
    pub resources: Vec<LangResource<C>>,
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
    let locale_files = try_load_locales(locales_folder)
        .map_err(|e| format!("Could not read locales folder '{locales_folder}': {e:?}"))?;

    let locale_messages = parse_locales(&locale_files)?;

    let messages = locale_messages
        .into_iter()
        .flat_map(|l| l.resources)
        .flat_map(|r| r.content)
        .map(|msg| (msg.id.clone(), msg))
        .collect::<HashMap<Id, Message>>();

    Ok(generate_for_messages(messages.values()))
}

fn parse_locales(locales: &[LangBundle<String>]) -> Result<Vec<LangBundle<Vec<Message>>>, String> {
    let mut msg_bundles = Vec::new();
    for bundle in locales {
        let mut msg_bundle = LangBundle {
            language: bundle.language.clone(),
            resources: Vec::new(),
        };

        let multiple_resources = bundle.resources.len() > 1;
        for resource in &bundle.resources {
            let ast = parser::parse(resource.content.as_str()).map_err(|e| {
                format!(
                    "Could not parse ftl file '{}' due to: {e:?}",
                    resource.resource,
                )
            })?;

            let name = multiple_resources.then_some(resource.resource.clone());
            let messages = to_messages(&name, ast);
            msg_bundle.resources.push(LangResource {
                resource: resource.resource.clone(),
                content: messages,
            });
        }

        msg_bundles.push(msg_bundle);
    }
    Ok(msg_bundles)
}

fn try_load_locales(folder: &str) -> Result<Vec<LangBundle<String>>, std::io::Error> {
    let locales_dir = fs::read_dir(folder)?;
    let mut locales = Vec::new();
    for entry in locales_dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let lang = path.file_name().unwrap().to_str().unwrap();
            locales.push(try_load_locale(&path, lang)?);
        }
    }
    Ok(locales)
}

fn try_load_locale(folder: &Path, lang: &str) -> Result<LangBundle<String>, std::io::Error> {
    let mut bundle = LangBundle {
        language: lang.to_string(),
        resources: Vec::new(),
    };
    let locales = fs::read_dir(folder)?;
    for entry in locales {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|s| s == "ftl") == Some(true) {
            let resource = LangResource {
                resource: path.file_stem().unwrap().to_str().unwrap().to_string(),
                content: fs::read_to_string(&path)?,
            };
            bundle.resources.push(resource);
        }
    }
    Ok(bundle)
}
