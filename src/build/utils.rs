use std::fs;
use std::path::{Path, PathBuf};

/// A precomputed newline index for one source string.
///
/// A source file has its line numbers looked up once per message (and once per
/// standalone comment line). Counting newlines from the start of the string on
/// every lookup is `O(messages * file_length)` — quadratic in the file size.
/// `LineIndex` does a single `O(n)` scan up front, after which any lookup is a
/// binary search.
pub struct LineIndex<'a> {
    src: &'a str,
    /// Ascending byte offsets of every `\n` in `src`.
    newlines: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub fn new(src: &'a str) -> Self {
        let newlines = src
            .bytes()
            .enumerate()
            .filter_map(|(i, b)| (b == b'\n').then_some(i))
            .collect();
        Self { src, newlines }
    }

    /// The 1-based line number at byte `offset`.
    ///
    /// `offset` need not fall on a UTF-8 char boundary — a newline is always a
    /// single ASCII byte — so this is safe for byte offsets that come straight
    /// from a parser (e.g. error positions). Out-of-range offsets are clamped.
    pub fn line_at_byte(&self, offset: usize) -> usize {
        let offset = offset.min(self.src.len());
        // Newlines strictly before `offset`, plus one.
        self.newlines.partition_point(|&nl| nl < offset) + 1
    }

    /// The 1-based line number of `sub`, a subslice of this index's source.
    ///
    /// `fluent_syntax` parses a `Resource<&str>` whose every `&str` is a
    /// subslice of the source FTL string, so the line of any AST node is
    /// recoverable by pointer arithmetic. Returns `0` if `sub` is not a
    /// subslice of the source (a defensive fallback that should never happen
    /// for AST-derived slices).
    pub fn line_of(&self, sub: &str) -> usize {
        let offset = (sub.as_ptr() as usize).wrapping_sub(self.src.as_ptr() as usize);
        if offset > self.src.len() {
            return 0;
        }
        self.line_at_byte(offset)
    }
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
    let lines = LineIndex::new(src);
    // Offset 4 lands *inside* `é` — must not panic (a `&str` slice would).
    assert_eq!(lines.line_at_byte(4), 1);
    assert_eq!(lines.line_at_byte(6), 2);
    // An out-of-range offset is clamped, not a panic.
    assert_eq!(lines.line_at_byte(999), 3);
}
