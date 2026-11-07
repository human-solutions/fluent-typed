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
        /// One entry per parse error, each prefixed with `path:line:column:`.
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
    /// A subfolder of the locales folder isn't named after a language, such as
    /// `en` or `en-GB`. Hidden folders (starting with `.`) are skipped.
    InvalidLanguageFolder {
        /// The folder.
        path: PathBuf,
        /// Why its name isn't a usable language id.
        reason: String,
    },
    /// Messages, variables or locales whose generated Rust names would clash,
    /// such as `foo-bar` and `foo_bar` both becoming `msg_foo_bar()`.
    NameCollision {
        /// One entry per clash, each starting with `file:line:`.
        messages: Vec<String>,
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
                write!(f, "Fluent syntax error(s) in '{}':", path.display())?;
                for e in errors {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
            Self::FtlRead { path, source } => {
                write!(f, "Could not read '{}': {source}", path.display())
            }
            Self::DuplicateKey {
                key,
                original,
                original_line,
                duplicate,
                duplicate_line,
            } => {
                let first = if original == duplicate {
                    format!("line {original_line}")
                } else {
                    format!("{}:{original_line}", original.display())
                };
                write!(
                    f,
                    "{}:{duplicate_line}: message '{key}' is defined twice, first at {first}. \
                     Remove or rename one.",
                    duplicate.display(),
                )
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
                    "{}:{term_line}: term '-{name}' has the same name as message '{name}' \
                     ({}:{message_line}). fluent-bundle stores both under one key, so the \
                     translation would fail to load at runtime. Rename one.",
                    term_file.display(),
                    message_file.display(),
                )
            }
            Self::LocalesFolder { folder, source } => {
                write!(f, "Could not read the locales folder '{folder}': {source}")
            }
            Self::InvalidLanguageFolder { path, reason } => {
                write!(f, "Invalid locale folder '{}': {reason}", path.display())
            }
            Self::NameCollision { messages } => {
                write!(
                    f,
                    "fluent-typed found {} name collision(s) in the generated code:",
                    messages.len()
                )?;
                for m in messages {
                    write!(f, "\n  {m}")?;
                }
                Ok(())
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
            Self::WriteOutput { path, source } => {
                write!(f, "Could not write '{path}': {source}")
            }
            Self::Rustfmt(msg) => write!(f, "rustfmt failed on the generated code: {msg}"),
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
