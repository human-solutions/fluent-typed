//! 1. Generates the l10n modules the documentation examples use.
//! 2. Extracts every ```rust block from README.md and docs/*.md into
//!    `$OUT_DIR/blocks.rs`, one `#[test]` function per block, so `cargo test`
//!    compiles them all and runs them (their `assert!`s check what the docs
//!    claim). A `build.rs` example becomes a nested `fn main` that is
//!    compiled but never called: running it would regenerate this crate's own
//!    l10n modules with other options.
//!
//! A block is controlled by an HTML comment on the line before its fence
//! (invisible on GitHub, crates.io and docs.rs), with space-separated words:
//!
//! - `skip`: not compiled (illustrations, or code that must live at module
//!   level, like `mod l10n;`).
//! - `no_run`: compiled but not run (it reads files the example assumes).
//! - `module=l10n_gz`: compiled against that generated module instead of the
//!   default `l10n`.
//!
//! For example `<!-- docs-check: module=l10n_gz no_run -->`.

use std::error::Error;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

use flate2::{Compression, write::GzEncoder};
use fluent_typed::{BuildOptions, FtlOutputOptions, try_build_from_locales_folder};

fn main() -> Result<(), Box<dyn Error>> {
    generate_l10n()?;

    let root = Path::new("../..");
    let mut docs = vec![root.join("README.md")];
    let mut guides: Vec<PathBuf> = fs::read_dir(root.join("docs"))?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    guides.retain(|p| p.extension().is_some_and(|e| e == "md"));
    guides.sort();
    docs.extend(guides);
    println!("cargo::rerun-if-changed={}", root.join("docs").display());

    let mut out = String::new();
    let mut count = 0;
    for doc in &docs {
        println!("cargo::rerun-if-changed={}", doc.display());
        let text = fs::read_to_string(doc)?;
        let name = doc.file_stem().unwrap().to_string_lossy().to_lowercase().replace('-', "_");
        for block in rust_blocks(&text) {
            if block.skip {
                continue;
            }
            count += 1;
            write_block(&mut out, &name, &block)?;
        }
    }
    assert!(count > 0, "no ```rust blocks found: is the path to the docs right?");
    fs::write(PathBuf::from(env::var("OUT_DIR")?).join("blocks.rs"), out)?;
    Ok(())
}

fn generate_l10n() -> Result<(), Box<dyn Error>> {
    // The default: one embedded file, with language negotiation.
    try_build_from_locales_folder(
        BuildOptions::default()
            .with_ftl_output(FtlOutputOptions::single_file("gen/translations.ftl"))
            .with_output_file_path("src/l10n.rs")
            .with_langneg(),
    )?;

    // Compressed single file.
    let gz = FtlOutputOptions::single_compressed_file("gen/translations.ftl.gz", gzip);
    try_build_from_locales_folder(
        BuildOptions::default()
            .with_ftl_output(gz)
            .with_output_file_path("src/l10n_gz.rs"),
    )?;

    // One file per language, loaded on demand.
    try_build_from_locales_folder(
        BuildOptions::default()
            .with_ftl_output(FtlOutputOptions::multi_file("gen/multi/"))
            .with_output_file_path("src/l10n_multi.rs"),
    )?;
    Ok(())
}

fn gzip(ftl: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&ftl)?;
    Ok(encoder.finish()?)
}

struct Block {
    line: usize,
    module: String,
    skip: bool,
    no_run: bool,
    code: String,
}

/// The ```rust blocks of a Markdown document. Fences like ```rust,ignore count
/// as Rust too: the marker comment, not the info string, decides what is
/// checked.
fn rust_blocks(text: &str) -> Vec<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let fence = lines[i].trim();
        let is_rust = fence.starts_with("```")
            && fence.trim_start_matches('`').split(',').next() == Some("rust");
        if !is_rust {
            i += 1;
            continue;
        }
        let marker = i
            .checked_sub(1)
            .map(|p| lines[p].trim())
            .and_then(|l| l.strip_prefix("<!-- docs-check:"))
            .and_then(|l| l.strip_suffix("-->"))
            .map(str::trim)
            .unwrap_or("");
        let start = i + 1;
        let mut end = start;
        while end < lines.len() && !is_closing_fence(lines[end]) {
            end += 1;
        }
        assert!(end < lines.len(), "unclosed ```rust block at line {}", i + 1);
        let words: Vec<&str> = marker.split_whitespace().collect();
        for word in &words {
            assert!(
                matches!(*word, "skip" | "no_run") || word.starts_with("module="),
                "unknown docs-check word `{word}` before line {}",
                i + 1
            );
        }
        blocks.push(Block {
            line: i + 1,
            module: words
                .iter()
                .find_map(|w| w.strip_prefix("module="))
                .unwrap_or("l10n")
                .to_string(),
            skip: words.contains(&"skip"),
            no_run: words.contains(&"no_run"),
            code: lines[start..end].join("\n"),
        });
        i = end + 1;
    }
    blocks
}

/// A line of three or more backticks and nothing else (whitespace aside).
fn is_closing_fence(line: &str) -> bool {
    let line = line.trim();
    line.len() >= 3 && line.chars().all(|c| c == '`')
}

fn write_block(out: &mut String, doc: &str, block: &Block) -> std::fmt::Result {
    // Names the examples use without defining them.
    let setup = match block.module.as_str() {
        "l10n" => "let strs: L10nLanguage = L10n::En.load();",
        _ => "",
    };
    writeln!(
        out,
        r#"
// {doc}.md line {line}
{test}
#[allow(unused, unreachable_code, clippy::all)]
fn {doc}_line_{line}() -> Result<(), Box<dyn std::error::Error>> {{
    // Glob imports, so a block's own `use` of the same names doesn't clash.
    use fluent_typed::*;
    use fluent_typed::prelude::*;
    use std::sync::*;
    use crate::{module}::*;
    let bytes: Vec<u8> = Vec::new();
    let path = std::path::PathBuf::new();
    let accept_language_header = "fr, en;q=0.5";
    {setup}
    {code}
    Ok(())
}}"#,
        line = block.line,
        test = if block.no_run { "" } else { "#[test]" },
        module = block.module,
        code = block.code,
    )
}
