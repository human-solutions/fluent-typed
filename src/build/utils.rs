use std::fs;
use std::path::{Path, PathBuf};

/// The 1-based line number of `sub` within `src`.
///
/// `fluent_syntax` parses a `Resource<&str>` whose every `&str` is a subslice of
/// the source FTL string, so the line of any AST node is recoverable by pointer
/// arithmetic. Returns `0` if `sub` is not a subslice of `src` (a defensive
/// fallback that should never happen for AST-derived slices).
pub fn line_of(src: &str, sub: &str) -> usize {
    let offset = (sub.as_ptr() as usize).wrapping_sub(src.as_ptr() as usize);
    if offset > src.len() {
        return 0;
    }
    line_at_byte(src, offset)
}

/// The 1-based line number at byte `offset` within `src`.
///
/// `offset` need not fall on a UTF-8 char boundary — the count is taken over
/// raw bytes, and a newline is always a single ASCII byte. This makes it safe
/// for byte offsets that come straight from a parser (e.g. error positions),
/// which a `&str` slice would reject.
pub fn line_at_byte(src: &str, offset: usize) -> usize {
    let offset = offset.min(src.len());
    src.as_bytes()[..offset]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

pub trait Traversable {
    fn gather_all_files(
        &self,
        condition: impl Fn(&Path) -> bool,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>>;
}

impl Traversable for Path {
    fn gather_all_files(
        &self,
        condition: impl Fn(&Path) -> bool,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut paths = Vec::new();

        if self.is_file() && condition(self) {
            paths.push(self.to_path_buf());
        } else if self.is_dir() {
            gather_paths_recursive(self, &mut paths, &condition)?;
        }

        Ok(paths)
    }
}

fn gather_paths_recursive(
    dir: &Path,
    paths: &mut Vec<PathBuf>,
    condition: &impl Fn(&Path) -> bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && condition(&path) {
            paths.push(path);
        } else if path.is_dir() {
            gather_paths_recursive(&path, paths, condition)?;
        }
    }
    Ok(())
}

#[test]
fn line_at_byte_accepts_an_offset_inside_a_multibyte_char() {
    // `é` occupies bytes 3..5; `\n` is at byte 5.
    let src = "café\nx\n";
    // Offset 4 lands *inside* `é` — must not panic (a `&str` slice would).
    assert_eq!(line_at_byte(src, 4), 1);
    assert_eq!(line_at_byte(src, 6), 2);
    // An out-of-range offset is clamped, not a panic.
    assert_eq!(line_at_byte(src, 999), 3);
}
