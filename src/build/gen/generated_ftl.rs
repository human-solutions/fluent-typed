use std::{
    collections::VecDeque,
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
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

                format!("static LANG_DATA: &'static [u8] = include_bytes!(\"{path}\");")
            }
            Self::MultiFile => "".to_string(),
        })
    }

    pub fn accessor_replacement(&self) -> String {
        match self {
            Self::SingleFile {
                positions,
                compressed,
                ..
            } => self.single_file_load_fn(positions, *compressed),
            Self::MultiFile => "".to_string(),
        }
    }

    fn single_file_load_fn(
        &self,
        positions: &[(String, Range<usize>)],
        compressed: bool,
    ) -> String {
        let mut out = String::new();

        out.push_str(&byte_range_fn(positions));

        let load_fn = if compressed {
            r#"
    pub fn load<D>(&self, decompressor: D) -> Result<L10n, String>
    where
        D: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        let bytes = decompressor(LANG_DATA)?;
        L10n::new(self.as_str(), &bytes)
    }
    "#
        } else {
            r#"
    pub fn load(&self) -> Result<L10n, String> {
        let bytes = LANG_DATA[self.byte_range()].to_vec();
        L10n::new(self.as_str(), &bytes)
    }
    "#
        };

        out.push_str(&load_fn);

        let load_all_fn = if compressed {
            r#"
    pub fn load_all<D>(decompressor: D) -> Result<Vec<L10n>, String>
    where
        D: Fn(&[u8]) -> Result<Vec<u8>, String>,
    {
        let bytes = decompressor(LANG_DATA)?;
        Self::as_arr()
            .iter()
            .map(|lang| L10n::new(lang.as_str(), &bytes[lang.byte_range()]))
            .collect()
    }
    "#
        } else {
            r#"
    pub fn load_all() -> Result<Vec<L10n>, String>
    {
        Self::as_arr()
            .iter()
            .map(|lang| L10n::new(lang.as_str(), &LANG_DATA[lang.byte_range()]))
            .collect()
    }
    "#
        };

        out.push_str(&load_all_fn);
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

fn relative_path(from: &str, to: &str) -> io::Result<String> {
    let mut from_path = fs::canonicalize(from)?;
    if from_path.is_file() {
        from_path.pop();
    }
    let to_path = fs::canonicalize(to)?;

    relative(&from_path, &to_path)?
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Could not convert relative path to string",
            )
        })
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

    // Remove common components
    while let (Some(fr_comp), Some(to_comp)) = (from.get(0), to.get(0)) {
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

#[test]
fn test_relative_path() {
    let rel = relative(Path::new("/a/b/c.rs"), Path::new("/a/d/e.flt")).unwrap();
    assert_eq!(rel, PathBuf::from("../../d/e.flt"));
}
