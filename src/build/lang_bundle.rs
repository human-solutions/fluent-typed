use crate::build::utils::Traversable;

use super::Message;
use fluent_syntax::ast::Resource;
use fluent_syntax::parser;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct LangBundle {
    pub language_name: Option<String>,
    pub language_id: String,
    pub messages: Vec<Message>,
    pub ftl: String,
}

impl LangBundle {
    #[cfg(test)]
    pub fn from_ftl(ftl: &str, name: &str, lang: &str) -> Result<Self, String> {
        let ast = parser::parse(ftl).map_err(|e| format!("Could not parse ftl due to: {e:?}"))?;
        Ok(LangBundle {
            language_name: lang_name(&ast),
            language_id: lang.to_string(),
            messages: to_messages(name, &ast)?,
            ftl: ftl.to_string(),
        })
    }
    pub fn from_folder(folder: &Path, lang: &str) -> Result<Self, String> {
        let mut bundle = LangBundle {
            language_name: None,
            language_id: lang.to_string(),
            messages: Vec::new(),
            ftl: String::new(),
        };

        let mut paths = folder
            .gather_all_files(|file| file.extension().map(|s| s == "ftl") == Some(true))
            .map_err(|e| e.to_string())?;

        paths.sort();

        for path in paths {
            let ftl = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let ast = parser::parse(ftl.as_str())
                .map_err(|e| format!("Could not parse ftl due to: {e:?}"))?;

            if let Some(lang_name) = lang_name(&ast) {
                if bundle.language_name.is_none() {
                    bundle.language_name = Some(lang_name);
                }
            }
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();

            bundle.ftl.push_str(&format!(
                "\n## ########## Resource: {name} ###############\n\n"
            ));
            bundle.ftl.push_str(&ftl);
            bundle.ftl.push('\n');

            let messages = to_messages(&name, &ast)?;
            bundle.messages.extend(messages);
        }
        Ok(bundle)
    }
}

fn to_messages(name: &str, ast: &Resource<&str>) -> Result<Vec<Message>, String> {
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

fn lang_name(ast: &Resource<&str>) -> Option<String> {
    use fluent_syntax::ast::PatternElement::TextElement;
    ast.body
        .iter()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(m) => {
                if m.id.name != "language-name" || !m.attributes.is_empty() {
                    return None;
                }
                let Some(value) = &m.value else { return None };

                if let Some(TextElement { value }) = value.elements.iter().next() {
                    return Some(value.to_string());
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
}
