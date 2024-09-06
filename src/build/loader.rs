use std::{fs, path::Path};

use fluent_syntax::parser;

use super::typed::to_messages;

use super::LangBundle;

pub fn from_locales_folder(folder: &str) -> Result<Vec<LangBundle>, String> {
    let locales_dir = fs::read_dir(folder).map_err(|e| e.to_string())?;
    let mut locales = Vec::new();
    for entry in locales_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let lang = path.file_name().unwrap().to_str().unwrap();
            locales.push(try_load_locale(&path, lang)?);
        }
    }
    Ok(locales)
}

fn try_load_locale(folder: &Path, lang: &str) -> Result<LangBundle, String> {
    let mut bundle = LangBundle {
        language: lang.to_string(),
        messages: Vec::new(),
        ftl: String::new(),
    };
    let locales = fs::read_dir(folder).map_err(|e| e.to_string())?;
    let mut paths = vec![];
    for entry in locales {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|s| s == "ftl") == Some(true) {
            paths.push(path);
        }
    }

    for path in paths {
        let ftl = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        bundle.ftl.push_str(&format!(
            "\n## ########## Resource: {name} ###############\n\n"
        ));
        bundle.ftl.push_str(&ftl);
        bundle.ftl.push('\n');

        let ast = parser::parse(ftl.as_str()).map_err(|e| {
            format!(
                "Could not parse ftl file '{}' due to: {e:?}",
                path.file_name().unwrap().to_str().unwrap()
            )
        })?;
        let messages = to_messages(&name, ast);
        bundle.messages.extend(messages);
    }
    Ok(bundle)
}
