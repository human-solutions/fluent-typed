use std::{error::Error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum BuildError {
    FtlParse {
        path: PathBuf,
        /// One entry per parse error, each prefixed with its line number.
        errors: Vec<String>,
    },
    FtlRead {
        path: PathBuf,
        source: io::Error,
    },
    DuplicateKey {
        key: String,
        original: PathBuf,
        original_line: usize,
        duplicate: PathBuf,
        duplicate_line: usize,
    },
    LocalesFolder {
        folder: String,
        source: io::Error,
    },
    NoLocaleFolders {
        folder: String,
    },
    DefaultLanguageNotFound {
        language: String,
        folder: String,
    },
    /// One or more lint diagnostics, raised as a hard error under
    /// [`crate::LintLevel::Deny`] or [`crate::LintLevel::Strict`]. Each entry
    /// already carries `file:line`.
    Lint {
        messages: Vec<String>,
    },
    WriteOutput {
        path: String,
        source: io::Error,
    },
    Rustfmt(String),
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
