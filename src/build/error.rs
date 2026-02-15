use std::{error::Error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum BuildError {
    FtlParse(String),
    Io(io::Error),
    DuplicateKey {
        key: String,
        original: PathBuf,
        duplicate: PathBuf,
    },
    LocalesFolder {
        folder: String,
        source: Box<BuildError>,
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
            Self::Io(err) => write!(f, "{err}"),
            Self::DuplicateKey {
                key,
                original,
                duplicate,
            } => write!(
                f,
                "Duplicate message key '{key}' in '{}', first defined in '{}'",
                duplicate.display(),
                original.display()
            ),
            Self::LocalesFolder { folder, source } => {
                write!(f, "Could not read locales folder '{folder}': {source}")
            }
            Self::WriteOutput { path, source } => {
                write!(f, "Could not write file '{path}': {source}")
            }
            Self::Rustfmt(msg) => write!(f, "Rustfmt error: {msg}"),
            Self::Generation(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::LocalesFolder { source, .. } => Some(source.as_ref()),
            Self::WriteOutput { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for BuildError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
