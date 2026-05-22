use std::{error::Error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum BuildError {
    FtlParse(String),
    FtlRead {
        path: PathBuf,
        source: io::Error,
    },
    DuplicateKey {
        key: String,
        original: PathBuf,
        duplicate: PathBuf,
    },
    LocalesFolder {
        folder: String,
        source: io::Error,
    },
    NoLocaleFolders {
        folder: String,
    },
    WriteOutput {
        path: String,
        source: io::Error,
    },
    Rustfmt(String),
    Generation(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FtlParse(msg) => write!(f, "Could not parse ftl: {msg}"),
            Self::FtlRead { path, .. } => {
                write!(f, "Could not read '{}'", path.display())
            }
            Self::DuplicateKey {
                key,
                original,
                duplicate,
            } => {
                if original == duplicate {
                    write!(
                        f,
                        "Duplicate message key '{key}' in '{}'",
                        duplicate.display()
                    )
                } else {
                    write!(
                        f,
                        "Duplicate message key '{key}' in '{}', first defined in '{}'",
                        duplicate.display(),
                        original.display()
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
            Self::WriteOutput { path, .. } => {
                write!(f, "Could not write file '{path}'")
            }
            Self::Rustfmt(msg) => write!(f, "Rustfmt error: {msg}"),
            Self::Generation(msg) => write!(f, "{msg}"),
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
