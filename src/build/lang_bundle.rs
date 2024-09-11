use super::Message;
use fluent_syntax::parser;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct LangBundle {
    pub language: String,
    pub messages: Vec<Message>,
    pub ftl: String,
}

impl LangBundle {
    #[cfg(test)]
    pub fn from_ftl(ftl: &str, name: &str, lang: &str) -> Result<Self, String> {
        Ok(LangBundle {
            language: lang.to_string(),
            messages: to_messages(name, ftl)?,
            ftl: ftl.to_string(),
        })
    }
    pub fn from_folder(folder: &Path, lang: &str) -> Result<Self, String> {
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

        paths.sort();

        for path in paths {
            let ftl = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            bundle.ftl.push_str(&format!(
                "\n## ########## Resource: {name} ###############\n\n"
            ));
            bundle.ftl.push_str(&ftl);
            bundle.ftl.push('\n');

            let messages = to_messages(&name, &ftl)?;
            bundle.messages.extend(messages);
        }
        Ok(bundle)
    }
}

fn to_messages(name: &str, ftl: &str) -> Result<Vec<Message>, String> {
    let ast = parser::parse(ftl).map_err(|e| format!("Could not parse ftl due to: {e:?}",))?;
    Ok(ast
        .body
        .iter()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(m) => Some(Message::parse(name, m)),
            _ => None,
        })
        .flatten()
        .collect())
}
