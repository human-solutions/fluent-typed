use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Write `content` to `path` only when it differs from the file's current
/// content.
///
/// Build outputs are watched by consumers — `cargo::rerun-if-changed` on the
/// generated files, dev-loop file watchers — and several build units may share
/// one `gen/` dir (host + wasm32 target dirs each keep their own fingerprints
/// watching the same files). An unconditional rewrite bumps the mtime and
/// re-triggers all of them on every no-op rebuild.
pub fn write_if_changed(path: &Path, content: &[u8]) -> io::Result<()> {
    if let Ok(current) = fs::read(path)
        && current == content
    {
        return Ok(());
    }
    fs::write(path, content)
}

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

    /// The 1-based `(line, column)` at byte `offset`, the column counted in
    /// characters. Like [`Self::line_at_byte`], `offset` may fall inside a
    /// multi-byte character.
    pub fn line_col_at_byte(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.src.len());
        let line = self.line_at_byte(offset);
        let start = self.line_start(line);
        // Count the characters before `offset`: every byte that doesn't
        // continue a multi-byte character starts one.
        let col = self.src.as_bytes()[start..offset]
            .iter()
            .filter(|&&b| b & 0xC0 != 0x80)
            .count();
        (line, col + 1)
    }

    /// The text of 1-based `line`, without its line ending.
    pub fn line_text(&self, line: usize) -> &'a str {
        let start = self.line_start(line);
        let end = self
            .newlines
            .get(line - 1)
            .copied()
            .unwrap_or(self.src.len());
        self.src[start..end].trim_end_matches('\r')
    }

    /// The byte offset where 1-based `line` starts.
    fn line_start(&self, line: usize) -> usize {
        match line {
            0 | 1 => 0,
            n => self.newlines[n - 2] + 1,
        }
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

/// Whether `path` names a hidden file or folder (`.git`, `.history`): editor
/// and tool state, never translations.
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes().starts_with(b"."))
}

/// Every `.ftl` file under `dir`, recursively, skipping hidden folders.
pub fn ftl_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_ftl_files(dir, &mut paths)?;
    Ok(paths)
}

fn collect_ftl_files(dir: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            if !is_hidden(&path) {
                collect_ftl_files(&path, paths)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "ftl") {
            paths.push(path);
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

#[test]
fn line_col_counts_characters() {
    let src = "café\nxé = y\r\n";
    let lines = LineIndex::new(src);
    assert_eq!(lines.line_col_at_byte(0), (1, 1));
    // `é` is two bytes, but one column.
    assert_eq!(lines.line_col_at_byte(5), (1, 5));
    assert_eq!(lines.line_col_at_byte(6), (2, 1));
    assert_eq!(lines.line_col_at_byte(9), (2, 3));
    assert_eq!(lines.line_text(1), "café");
    assert_eq!(lines.line_text(2), "xé = y");
}
