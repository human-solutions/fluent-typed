use super::{
    Analyzed, BuildError, BuildOptions, LangBundle, LintLevel, Message, r#gen::generate, lint,
    typed::Id, utils::write_if_changed, validations::default_contract_errors,
};
use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

pub struct Builder {
    options: BuildOptions,
    langbundles: Vec<LangBundle>,
}

impl Builder {
    pub fn load(options: BuildOptions) -> Result<Self, BuildError> {
        let folder = &options.locales_folder;
        println!("cargo::rerun-if-changed={folder}");

        let mut langbundles = from_locales_folder(folder, options.deny_duplicate_keys)?;

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
        let mut generated = generate(&self.options, &self.langbundles, messages)
            .map_err(BuildError::Generation)?
            .replace("    ", &self.options.indentation);

        // Format before the skip-if-unchanged compare: the file on disk holds
        // rustfmt's output, so comparing pre-format text against it would
        // never match and every build would rewrite the file — bumping its
        // mtime and re-triggering anything watching it. This also means a
        // rustfmt failure leaves no unformatted file behind.
        if self.options.format {
            generated = rustfmt(&generated)?;
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
        .spawn()
        .map_err(map_io)?;
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
        return Err(BuildError::Rustfmt("rustfmt failed".to_string()));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| BuildError::Rustfmt(format!("rustfmt produced non-UTF-8 output: {e}")))
}

fn from_locales_folder(
    folder: &str,
    deny_duplicate_keys: bool,
) -> Result<Vec<LangBundle>, BuildError> {
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
        if path.is_dir() {
            let lang = path.file_name().unwrap().to_str().unwrap();
            // Collect every locale's errors rather than stopping at the first,
            // so one rebuild surfaces them all.
            match LangBundle::from_folder(&path, lang, deny_duplicate_keys) {
                Ok(bundle) => locales.push(bundle),
                Err(errs) => errors.extend(errs),
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rustfmt_formats_in_memory() {
        let formatted = rustfmt("fn  main( ){ }\n").unwrap();
        assert_eq!(formatted, "fn main() {}\n");
    }

    #[test]
    fn rustfmt_failure_is_a_build_error() {
        let err = rustfmt("fn {\n").unwrap_err();
        assert!(matches!(err, BuildError::Rustfmt(_)), "got: {err:?}");
    }
}
