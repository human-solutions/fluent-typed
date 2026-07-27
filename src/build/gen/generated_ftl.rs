use std::{
    collections::VecDeque,
    fs, io,
    ops::Range,
    path::{Component, Path, PathBuf},
};

use super::StrExt;

pub enum GeneratedFtl {
    SingleFile {
        output_ftl_file: String,
        positions: Vec<(String, Range<usize>)>,
        compressed: bool,
    },
    MultiFile,
}

impl GeneratedFtl {
    pub fn include_replacement(&self, rs_path: &str) -> Result<String, String> {
        Ok(match self {
            Self::SingleFile {
                output_ftl_file, ..
            } => {
                let path = relative_path(rs_path, output_ftl_file).map_err(|e| {
                    format!("Could not create relative path between ftl and rs: {e}")
                })?;

                format!("static LANG_DATA: &[u8] = include_bytes!(\"{path}\");")
            }
            Self::MultiFile => "".to_string(),
        })
    }

    pub fn accessor_replacement(&self, use_isolating: bool) -> String {
        match self {
            Self::SingleFile {
                positions,
                compressed,
                ..
            } => self.single_file_load_fn(positions, *compressed, use_isolating),
            Self::MultiFile => "".to_string(),
        }
    }

    fn single_file_load_fn(
        &self,
        positions: &[(String, Range<usize>)],
        compressed: bool,
        use_isolating: bool,
    ) -> String {
        let mut out = String::new();

        out.push_str(&byte_range_fn(positions));

        let load_fn = if compressed {
            r#"
    /// Load a L10nLanguage from the embedded data.
    /// 
    /// The provided decompressor function is used to decompress the data
    /// and has to be the same as when the data was generated in the build.rs script.
    pub fn load<D>(&self, decompressor: D) -> Result<L10nLanguage, L10nError>
    where
        D: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        let bytes = decompressor(LANG_DATA).map_err(L10nError::Decompression)?;
        L10nLanguage::new(self, &bytes)
    }
"#
        } else {
            r#"
    /// Load a L10nLanguage from the embedded data.
    pub fn load(&self) -> L10nLanguage {
        let bytes = LANG_DATA[self.byte_range()].to_vec();
        L10nLanguage::new(self, &bytes)
            .expect("fluent-typed: the embedded .ftl could not be loaded. This is a build-time bug; please report it.")
    }
"#
        };

        out.push_str(load_fn);

        let load_all_fn = if compressed {
            r#"
    /// Load all languages (L10nLanguage) from the embedded data.
    /// 
    /// The provided decompressor function is used to decompress the data
    /// and has to be the same as when the data was generated in the build.rs script.
    pub fn load_all<D>(decompressor: D) -> Result<L10nLanguageVec, L10nError>
    where
        D: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        let bytes = decompressor(LANG_DATA).map_err(L10nError::Decompression)?;
        L10nLanguageVec::load(
            &bytes,
            Self::iter().map(|lang| (lang, lang.byte_range())),
        )
    }"#
        } else {
            r#"
    /// Load all languages (L10nLanguage) from the embedded data.
    pub fn load_all() -> L10nLanguageVec {
        L10nLanguageVec::load(
            LANG_DATA,
            Self::iter().map(|lang| (lang, lang.byte_range())),
        )
        .expect("fluent-typed: the embedded .ftl could not be loaded. This is a build-time bug; please report it.")
    }"#
        };

        out.push_str(load_all_fn);

        if !use_isolating {
            out = out.replace(
                "L10nLanguageVec::load(",
                "L10nLanguageVec::load_without_isolation(",
            );
        }
        out
    }
}

fn byte_range_fn(positions: &[(String, Range<usize>)]) -> String {
    let range_statements = positions
        .iter()
        .map(|(name, range)| {
            format!(
                "            Self::{} => {}..{},",
                name.rust_var_name(),
                range.start,
                range.end
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"
    fn byte_range(&self) -> Range<usize> {{
        match self {{
{range_statements}
        }}
    }}"#,
    )
}

fn file_to_absolute_dir(file: &str) -> io::Result<PathBuf> {
    let mut dir = PathBuf::from(file);
    dir.pop();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    fs::canonicalize(dir)
}

// from = rs, to = ftl
fn relative_path(from_file: &str, to_file: &str) -> io::Result<String> {
    let from_dir = file_to_absolute_dir(from_file)?;
    let to_file_name = Path::new(to_file).file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not get file name from path",
        )
    })?;
    let to_dir = file_to_absolute_dir(to_file)?;

    let mut rel_file = relative(&from_dir, &to_dir)?;
    rel_file.push(to_file_name);

    to_forward_slashes(&rel_file)
}

/// Join a relative path's components with `/` regardless of platform.
///
/// The result is spliced into `include_bytes!("...")`, where a native Windows
/// separator would form invalid escape sequences like `\g` (issue #37).
/// `include_bytes!` accepts forward slashes on every platform.
fn to_forward_slashes(path: &Path) -> io::Result<String> {
    let mut parts = Vec::new();
    for comp in path.components() {
        parts.push(comp.as_os_str().to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Could not convert relative path to string",
            )
        })?);
    }
    Ok(parts.join("/"))
}

fn relative(from_path: &Path, to_path: &Path) -> io::Result<PathBuf> {
    if from_path.is_relative() || to_path.is_relative() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Both paths must be absolute",
        ));
    }

    let mut from = from_path.components().collect::<VecDeque<_>>();
    let mut to = to_path.components().collect::<VecDeque<_>>();

    // On Windows two absolute paths can be rooted on different drives, in
    // which case no relative path between them exists. Without this check the
    // loop below would emit `..` segments followed by the other drive's
    // absolute path — garbage that only fails later inside `include_bytes!`.
    if let (Some(Component::Prefix(from_prefix)), Some(Component::Prefix(to_prefix))) =
        (from.front(), to.front())
        && from_prefix != to_prefix
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "No relative path exists between '{}' and '{}' (different drives). \
                 Place the generated .rs file and the .ftl output on the same drive.",
                from_path.display(),
                to_path.display()
            ),
        ));
    }

    // Remove common components
    while let (Some(fr_comp), Some(to_comp)) = (from.front(), to.front()) {
        if fr_comp != to_comp {
            break;
        }
        from.pop_front();
        to.pop_front();
    }

    let mut relative = PathBuf::new();
    for _ in 0..(from.len()) {
        relative.push("..");
    }
    while let Some(comp) = to.pop_front() {
        relative.push(comp);
    }
    Ok(relative)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Absolute-path fixtures per platform: `/a/b` is not absolute on Windows.
    #[cfg(not(windows))]
    const FIXTURE: (&str, &str) = ("/a/b/c.rs", "/a/d/e.flt");
    #[cfg(windows)]
    const FIXTURE: (&str, &str) = (r"C:\a\b\c.rs", r"C:\a\d\e.flt");

    #[test]
    fn test_relative_path() {
        let rel = relative(Path::new(FIXTURE.0), Path::new(FIXTURE.1)).unwrap();
        // Regression for issue #37: the string spliced into `include_bytes!`
        // must use `/` on every platform, never the native `\`.
        assert_eq!(to_forward_slashes(&rel).unwrap(), "../../d/e.flt");
    }

    #[cfg(windows)]
    #[test]
    fn test_relative_path_across_drives_is_an_error() {
        let err = relative(Path::new(r"C:\a\b"), Path::new(r"D:\c\d")).unwrap_err();
        assert!(err.to_string().contains("different drives"), "{err}");
    }
}
