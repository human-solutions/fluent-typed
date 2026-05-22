use crate::build::utils::{Traversable, line_at_byte, line_of};

use super::{BuildError, Message};
use fluent_syntax::ast::{Entry, Resource};
use fluent_syntax::parser::{self, ParserError};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A single line of a standalone `#` comment — one that is *not* attached to a
/// message. Used by the linter to flag type annotations detached from their
/// message by a blank line.
#[derive(Debug, Clone)]
pub struct CommentLine {
    pub file: String,
    pub line: usize,
    pub text: String,
}

#[derive(Debug)]
pub struct LangBundle {
    pub language_name: Option<String>,
    pub language_id: String,
    pub messages: Vec<Message>,
    pub standalone_comments: Vec<CommentLine>,
    pub ftl: String,
}

impl LangBundle {
    #[cfg(test)]
    pub fn from_ftl(
        ftl: &str,
        name: &str,
        lang: &str,
        deny_duplicate_keys: bool,
    ) -> Result<Self, BuildError> {
        let path = PathBuf::from(name);
        let ast = parse_ftl(ftl, &path)?;
        let mut seen = HashMap::new();
        Ok(LangBundle {
            language_name: lang_name(&ast),
            language_id: lang.to_string(),
            messages: to_messages(&ast, deny_duplicate_keys, &mut seen, &path, ftl)?,
            standalone_comments: standalone_comments(&ast, ftl, &path.display().to_string()),
            ftl: ftl.to_string(),
        })
    }

    pub fn from_folder(
        folder: &Path,
        lang: &str,
        deny_duplicate_keys: bool,
    ) -> Result<Self, BuildError> {
        let mut bundle = LangBundle {
            language_name: None,
            language_id: lang.to_string(),
            messages: Vec::new(),
            standalone_comments: Vec::new(),
            ftl: String::new(),
        };

        let mut paths = folder
            .gather_all_files(|file| file.extension().map(|s| s == "ftl") == Some(true))
            .map_err(|e| BuildError::FtlRead {
                path: folder.to_path_buf(),
                source: std::io::Error::other(e.to_string()),
            })?;

        paths.sort();

        let mut seen: HashMap<String, (PathBuf, usize)> = HashMap::new();

        for path in paths {
            let ftl = fs::read_to_string(&path).map_err(|e| BuildError::FtlRead {
                path: path.clone(),
                source: e,
            })?;
            let ast = parse_ftl(&ftl, &path)?;

            if let Some(lang_name) = lang_name(&ast)
                && bundle.language_name.is_none()
            {
                bundle.language_name = Some(lang_name);
            }
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();

            bundle.ftl.push_str(&format!(
                "\n## ########## Resource: {name} ###############\n\n"
            ));
            bundle.ftl.push_str(&ftl);
            bundle.ftl.push('\n');

            let file = path.display().to_string();
            bundle
                .standalone_comments
                .extend(standalone_comments(&ast, &ftl, &file));

            let messages = to_messages(&ast, deny_duplicate_keys, &mut seen, &path, &ftl)?;
            bundle.messages.extend(messages);
        }
        Ok(bundle)
    }
}

/// Parse an FTL string, turning any parse errors into a [`BuildError::FtlParse`]
/// that names the file and the line of each error.
fn parse_ftl<'a>(ftl: &'a str, path: &Path) -> Result<Resource<&'a str>, BuildError> {
    parser::parse(ftl).map_err(|(_, errors)| BuildError::FtlParse {
        path: path.to_path_buf(),
        errors: errors.iter().map(|e| format_parse_error(ftl, e)).collect(),
    })
}

fn format_parse_error(src: &str, error: &ParserError) -> String {
    // `error.pos.start` is a raw byte offset that may land inside a multi-byte
    // character, so it must not be used to slice `src` directly.
    format!(
        "line {}: {:?}",
        line_at_byte(src, error.pos.start),
        error.kind
    )
}

fn to_messages(
    ast: &Resource<&str>,
    deny_duplicate_keys: bool,
    seen: &mut HashMap<String, (PathBuf, usize)>,
    path: &Path,
    src: &str,
) -> Result<Vec<Message>, BuildError> {
    let file = path.display().to_string();
    ast.body
        .iter()
        .filter_map(|entry| match entry {
            Entry::Message(m) => Some(Message::parse(m, src, &file)),
            _ => None,
        })
        .flatten()
        .map(|msg| {
            if deny_duplicate_keys {
                let seen_key = msg.id.to_string();
                if let Some((original, original_line)) = seen.get(&seen_key) {
                    return Err(BuildError::DuplicateKey {
                        key: msg.id.message.clone(),
                        original: original.clone(),
                        original_line: *original_line,
                        duplicate: path.to_path_buf(),
                        duplicate_line: msg.line,
                    });
                }
                seen.insert(seen_key, (path.to_path_buf(), msg.line));
            }
            Ok(msg)
        })
        .collect()
}

/// Collect the lines of every standalone `#` comment (an `Entry::Comment` — a
/// comment not attached to a message).
fn standalone_comments(ast: &Resource<&str>, src: &str, file: &str) -> Vec<CommentLine> {
    let mut out = Vec::new();
    for entry in &ast.body {
        if let Entry::Comment(comment) = entry {
            for line in &comment.content {
                out.push(CommentLine {
                    file: file.to_owned(),
                    line: line_of(src, line),
                    text: (*line).to_owned(),
                });
            }
        }
    }
    out
}

fn lang_name(ast: &Resource<&str>) -> Option<String> {
    use fluent_syntax::ast::PatternElement::TextElement;
    ast.body
        .iter()
        .filter_map(|entry| match entry {
            Entry::Message(m) => {
                if m.id.name != "language-name" || !m.attributes.is_empty() {
                    return None;
                }
                let Some(value) = &m.value else { return None };

                if let Some(TextElement { value }) = value.elements.first() {
                    Some(value.to_string())
                } else {
                    None
                }
            }
            _ => None,
        })
        .next()
}
