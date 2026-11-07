use std::{error::Error, fmt, io, path::PathBuf};

/// Why the build step failed. Its `Display` names the `.ftl` file and line
/// wherever there is one.
#[derive(Debug)]
#[non_exhaustive]
pub enum BuildError {
    /// An `.ftl` file has Fluent syntax errors.
    FtlParse {
        /// The file that failed to parse.
        path: PathBuf,
        /// One entry per parse error, each prefixed with its line number.
        errors: Vec<String>,
    },
    /// An `.ftl` file could not be read.
    FtlRead {
        /// The file that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// The same message is defined twice in one language. Allow it with
    /// [`BuildOptions::with_allow_duplicate_keys`](crate::BuildOptions::with_allow_duplicate_keys).
    DuplicateKey {
        /// The duplicated message id.
        key: String,
        /// The file with the first definition.
        original: PathBuf,
        /// The line of the first definition.
        original_line: usize,
        /// The file with the second definition.
        duplicate: PathBuf,
        /// The line of the second definition.
        duplicate_line: usize,
    },
    /// A term and a message share the same bare name (e.g. `-foo` and `foo`).
    /// fluent-bundle stores both under the same key, so loading the resource
    /// crashes at runtime with `FluentError::Overriding`. Detected at build
    /// time so the cliff never happens.
    TermMessageCollision {
        /// The shared name, without the term's `-`.
        name: String,
        /// The file defining the term.
        term_file: PathBuf,
        /// The line of the term.
        term_line: usize,
        /// The file defining the message.
        message_file: PathBuf,
        /// The line of the message.
        message_line: usize,
    },
    /// The locales folder could not be read.
    LocalesFolder {
        /// The configured locales folder.
        folder: String,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// The locales folder has no language subfolders.
    NoLocaleFolders {
        /// The configured locales folder.
        folder: String,
    },
    /// The default language has no subfolder in the locales folder. Set it
    /// with [`BuildOptions::with_default_language`](crate::BuildOptions::with_default_language).
    DefaultLanguageNotFound {
        /// The configured default language.
        language: String,
        /// The configured locales folder.
        folder: String,
    },
    /// One or more lint diagnostics, raised as a hard error under
    /// [`crate::LintLevel::Deny`] or [`crate::LintLevel::Strict`]. Each entry
    /// already carries `file:line`.
    Lint {
        /// One entry per diagnostic.
        messages: Vec<String>,
    },
    /// Invalid default-locale message contracts that would generate accessors
    /// with silently wrong runtime output.
    InvalidContract {
        /// One entry per invalid message.
        messages: Vec<String>,
    },
    /// The generated Rust file could not be written.
    WriteOutput {
        /// The configured output file.
        path: String,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// `rustfmt` could not be run on the generated file, or failed. Skip
    /// formatting with [`BuildOptions::without_format`](crate::BuildOptions::without_format).
    Rustfmt(String),
    /// The joined `.ftl` output could not be written or compressed, or its path
    /// relative to the generated Rust file could not be worked out.
    Generation(String),
    /// Several independent build errors, collected so they can all be fixed in
    /// one pass instead of one rebuild at a time.
    Multiple(Vec<BuildError>),
}

impl BuildError {
    /// Collapse a list of errors into one: the error itself when there is
    /// exactly one, otherwise a [`BuildError::Multiple`].
    pub(crate) fn collapse(mut errors: Vec<BuildError>) -> BuildError {
        if errors.len() == 1 {
            errors.pop().unwrap()
        } else {
            BuildError::Multiple(errors)
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FtlParse { path, errors } => {
                write!(f, "Could not parse '{}':", path.display())?;
                for e in errors {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
            Self::FtlRead { path, .. } => {
                write!(f, "Could not read '{}'", path.display())
            }
            Self::DuplicateKey {
                key,
                original,
                original_line,
                duplicate,
                duplicate_line,
            } => {
                if original == duplicate {
                    write!(
                        f,
                        "Duplicate message key '{key}' in '{}': lines {original_line} and \
                         {duplicate_line}",
                        duplicate.display(),
                    )
                } else {
                    write!(
                        f,
                        "Duplicate message key '{key}' in '{}:{duplicate_line}', first defined \
                         in '{}:{original_line}'",
                        duplicate.display(),
                        original.display(),
                    )
                }
            }
            Self::TermMessageCollision {
                name,
                term_file,
                term_line,
                message_file,
                message_line,
            } => {
                write!(
                    f,
                    "Term '-{name}' and message '{name}' share the same name — \
                     fluent-bundle treats them as the same key and will crash \
                     at runtime. Rename one. Term defined in '{}:{term_line}', \
                     message defined in '{}:{message_line}'.",
                    term_file.display(),
                    message_file.display(),
                )
            }
            Self::LocalesFolder { folder, .. } => {
                write!(f, "Could not read locales folder '{folder}'")
            }
            Self::NoLocaleFolders { folder } => {
                write!(
                    f,
                    "No locale subfolders found in '{folder}'. Expected \
                     '<lang-id>/<resource>.ftl' files, e.g. 'en/main.ftl'."
                )
            }
            Self::DefaultLanguageNotFound { language, folder } => {
                write!(
                    f,
                    "Default language '{language}' has no locale subfolder in '{folder}'. \
                     Set it with `BuildOptions::with_default_language`."
                )
            }
            Self::Lint { messages } => {
                write!(f, "fluent-typed found {} lint error(s):", messages.len())?;
                for m in messages {
                    write!(f, "\n  {m}")?;
                }
                Ok(())
            }
            Self::InvalidContract { messages } => {
                write!(
                    f,
                    "fluent-typed found {} invalid message contract(s):",
                    messages.len()
                )?;
                for m in messages {
                    write!(f, "\n  {m}")?;
                }
                Ok(())
            }
            Self::WriteOutput { path, .. } => {
                write!(f, "Could not write file '{path}'")
            }
            Self::Rustfmt(msg) => write!(f, "Rustfmt error: {msg}"),
            Self::Generation(msg) => write!(f, "{msg}"),
            Self::Multiple(errors) => {
                write!(f, "{} build errors:", errors.len())?;
                for e in errors {
                    for (i, line) in e.to_string().lines().enumerate() {
                        if i == 0 {
                            write!(f, "\n  - {line}")?;
                        } else {
                            write!(f, "\n    {line}")?;
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FtlRead { source, .. } => Some(source),
            Self::LocalesFolder { source, .. } => Some(source),
            Self::WriteOutput { source, .. } => Some(source),
            _ => None,
        }
    }
}
