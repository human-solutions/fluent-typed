use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use unic_langid::LanguageIdentifier;

use super::{
    Analyzed, BuildError, BuildOptions, LangBundle, LintLevel, Message,
    r#gen::{INDENT, generate},
    lint,
    names::name_collisions,
    typed::Id,
    utils::{is_hidden, write_if_changed},
    validations::default_contract_errors,
};

pub struct Builder {
    options: BuildOptions,
    langbundles: Vec<LangBundle>,
}

impl Builder {
    pub fn load(options: BuildOptions) -> Result<Self, BuildError> {
        let mut langbundles = from_locales_folder(&options)?;

        langbundles.sort_by_cached_key(|lb| lb.language_id.clone());

        Ok(Self {
            langbundles,
            options,
        })
    }

    #[cfg(test)]
    pub fn load_one(
        options: BuildOptions,
        resource_name: &str,
        lang: &str,
        ftl: &str,
    ) -> Result<Self, BuildError> {
        let deny_duplicate_keys = options.deny_duplicate_keys;
        Ok(Self {
            options,
            langbundles: vec![LangBundle::from_ftl(
                ftl,
                resource_name,
                lang,
                deny_duplicate_keys,
            )?],
        })
    }

    pub fn generate(&self) -> Result<(), BuildError> {
        // The default locale is the single source of truth for every message's
        // typed signature, so it must exist.
        let default = self
            .langbundles
            .iter()
            .find(|b| b.language_id == self.options.default_language)
            .ok_or_else(|| BuildError::DefaultLanguageNotFound {
                language: self.options.default_language.clone(),
                folder: self.options.locales_folder.clone(),
            })?;

        let analyzed = Analyzed::from(&self.langbundles, default);
        for warn in &analyzed.warnings {
            println!("cargo::warning={warn}");
        }

        let contract_errors = default_contract_errors(default);
        if !contract_errors.is_empty() {
            return Err(BuildError::InvalidContract {
                messages: contract_errors,
            });
        }

        self.run_lints(default, &analyzed.common)?;

        let messages = &self.messages(default, &analyzed.common);
        let collisions = name_collisions(
            &self.options.prefix,
            &self.options.locales_folder,
            &self.langbundles,
            messages,
        );
        if !collisions.is_empty() {
            return Err(BuildError::NameCollision {
                messages: collisions,
            });
        }

        let mut generated =
            generate(&self.options, &self.langbundles, messages).map_err(BuildError::Generation)?;

        // Format before the skip-if-unchanged compare: the file on disk holds
        // rustfmt's output, so comparing pre-format text against it would
        // never match and every build would rewrite the file — bumping its
        // mtime and re-triggering anything watching it. This also means a
        // rustfmt failure leaves no unformatted file behind.
        if self.options.format {
            generated = rustfmt(&generated)?;
        } else if self.options.indentation != INDENT {
            generated = reindent(&generated, &self.options.indentation);
        }

        let output_file_path = &self.options.output_file_path;
        write_if_changed(Path::new(output_file_path), generated.as_bytes()).map_err(|e| {
            BuildError::WriteOutput {
                path: output_file_path.clone(),
                source: e,
            }
        })
    }

    /// Run the comment lints and report them according to the configured
    /// [`LintLevel`]. Returns an error only in strict mode, when there are
    /// hard lint failures.
    fn run_lints(&self, default: &LangBundle, common: &HashSet<Id>) -> Result<(), BuildError> {
        let lints = lint::check(&self.langbundles, default, common);

        match self.options.lint_level {
            LintLevel::Off => {}
            LintLevel::Warn => {
                for w in lints.mistakes.iter().chain(&lints.ineffective) {
                    println!("cargo::warning={w}");
                }
            }
            LintLevel::Deny | LintLevel::Strict => {
                // Diagnostics about non-default locales stay warnings — they
                // concern translator-owned files.
                for w in &lints.ineffective {
                    println!("cargo::warning={w}");
                }
                let mut errors = lints.mistakes;
                // `Strict` additionally requires every variable to be typed.
                if self.options.lint_level == LintLevel::Strict {
                    errors.extend(lints.untyped);
                }
                if !errors.is_empty() {
                    errors.sort();
                    return Err(BuildError::Lint { messages: errors });
                }
            }
        }
        Ok(())
    }

    /// The messages to generate: the default locale's, in declaration order,
    /// restricted to the ids that survived cross-locale analysis. An id is
    /// emitted only once, even if duplicate keys were allowed and the default
    /// locale defines it more than once.
    fn messages<'a>(&self, default: &'a LangBundle, common: &HashSet<Id>) -> Vec<&'a Message> {
        let mut seen = HashSet::new();
        default
            .messages
            .iter()
            .filter(|msg| common.contains(&msg.id))
            .filter(|msg| seen.insert(&msg.id))
            .collect()
    }
}

/// Replace each leading [`INDENT`] of every line with `indent`. Only leading
/// whitespace changes: spaces inside string literals and comments stay.
fn reindent(source: &str, indent: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let mut rest = line;
        while let Some(stripped) = rest.strip_prefix(INDENT) {
            out.push_str(indent);
            rest = stripped;
        }
        out.push_str(rest);
    }
    out
}

/// Format `source` by piping it through rustfmt's stdin/stdout, returning the
/// formatted text.
///
/// Config resolution differs slightly from formatting a file in place:
/// stdin mode resolves `rustfmt.toml` from the process cwd (the consumer crate
/// root when run from a build script) upward, while file mode resolves it from
/// the output file's directory upward. For the normal in-crate `gen/` layout
/// both walks reach the same crate/workspace config.
fn rustfmt(source: &str) -> Result<String, BuildError> {
    let map_io = |e: std::io::Error| BuildError::Rustfmt(e.to_string());
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BuildError::Rustfmt(
                    "rustfmt was not found. Install it with `rustup component add rustfmt`, \
                     or turn formatting off with `BuildOptions::without_format`."
                        .to_string(),
                )
            } else {
                map_io(e)
            }
        })?;
    // rustfmt parses all of stdin before emitting anything, so writing the
    // whole input first cannot deadlock against a filling stdout pipe.
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .map_err(map_io)?;
    let output = child.wait_with_output().map_err(map_io)?;
    if !output.status.success() {
        // The generator should always emit valid Rust, so this is a bug.
        return Err(BuildError::Rustfmt(format!(
            "{}\nThis is a fluent-typed bug; please report it. To see the generated code, \
             build with `BuildOptions::without_format`.",
            String::from_utf8_lossy(&output.stderr).trim_end()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| BuildError::Rustfmt(format!("rustfmt produced non-UTF-8 output: {e}")))
}

fn from_locales_folder(options: &BuildOptions) -> Result<Vec<LangBundle>, BuildError> {
    let folder = &options.locales_folder;
    let map_io = |e| BuildError::LocalesFolder {
        folder: folder.to_string(),
        source: e,
    };
    let locales_dir = fs::read_dir(folder).map_err(map_io)?;
    let mut locales = Vec::new();
    let mut errors: Vec<BuildError> = Vec::new();
    for entry in locales_dir {
        let entry = entry.map_err(map_io)?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if is_hidden(&path) {
            continue;
        }
        let name = entry.file_name();
        let lang = match language_id(&name, options.langneg) {
            Ok(lang) => lang,
            Err(reason) => {
                errors.push(BuildError::InvalidLanguageFolder { path, reason });
                continue;
            }
        };
        // Collect every locale's errors rather than stopping at the first,
        // so one rebuild surfaces them all.
        match LangBundle::from_folder(&path, lang, options.deny_duplicate_keys) {
            Ok(bundle) => locales.push(bundle),
            Err(errs) => errors.extend(errs),
        }
    }
    if !errors.is_empty() {
        return Err(BuildError::collapse(errors));
    }
    if locales.is_empty() {
        return Err(BuildError::NoLocaleFolders {
            folder: folder.to_string(),
        });
    }
    Ok(locales)
}

/// The language id named by a locale folder, or why it isn't one. The runtime
/// parses it when a language is loaded, so a folder that isn't a language id
/// would otherwise build and then fail on first use.
fn language_id(name: &OsStr, langneg: bool) -> Result<&str, String> {
    let lang = name
        .to_str()
        .ok_or_else(|| "the folder name is not valid UTF-8".to_string())?;
    let hint = "Name each locale folder after its language, such as 'en', 'en-GB' or 'zh-Hant'.";
    let id = lang
        .parse::<LanguageIdentifier>()
        .map_err(|e| format!("'{lang}' is not a language id ({e}). {hint}"))?;
    // BCP 47 allows 5-8 letter language subtags, but none is in use, so such
    // a folder (`shared`, `common`, `assets`) is a folder of something else.
    if id.language.as_str().len() > 3 {
        return Err(format!(
            "'{lang}' is not a language id: language codes have 2 or 3 letters. {hint}"
        ));
    }
    // `langid!` in the generated code (and `Accept-Language` matching) only
    // accept the standard `-` separator.
    if langneg && lang.contains('_') {
        return Err(format!(
            "'{lang}' uses '_', which language negotiation (`with_langneg`) doesn't \
             accept. Rename the folder to '{}'.",
            lang.replace('_', "-")
        ));
    }
    Ok(lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reindent_changes_only_leading_indentation() {
        let source = "fn f() {\n    if x {\n        s(\"a    b\");\n    }\n}\n";
        assert_eq!(
            reindent(source, "\t"),
            "fn f() {\n\tif x {\n\t\ts(\"a    b\");\n\t}\n}\n"
        );
    }

    #[test]
    fn rustfmt_formats_in_memory() {
        let formatted = rustfmt("fn  main( ){ }\n").unwrap();
        assert_eq!(formatted, "fn main() {}\n");
    }

    #[test]
    fn rustfmt_failure_is_a_build_error() {
        let err = rustfmt("fn {\n").unwrap_err();
        let BuildError::Rustfmt(message) = &err else {
            panic!("got: {err:?}");
        };
        // rustfmt's own diagnostic is part of the error, not just "failed".
        assert!(message.contains("error"), "{message}");
        assert!(message.contains("without_format"), "{message}");
    }
}
