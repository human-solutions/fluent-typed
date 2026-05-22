//! Performance benchmarks for the fluent-typed build pipeline.
//!
//! A consumer's `build.rs` runs four stages in sequence — parse, analyze, lint,
//! generate. This suite times each stage separately, on deterministically
//! generated FTL data at three project sizes, so regressions in any one stage
//! are visible.
//!
//! Run with:
//!
//! ```text
//! cargo bench --features bench-internals
//! ```
//!
//! The `bench-internals` feature re-exports the otherwise-private pipeline
//! functions; it is not part of the stable public API.
//!
//! To run a single scale, filter by benchmark id — useful because the `large`
//! scale is slow (see the note below):
//!
//! ```text
//! cargo bench --features bench-internals -- /small
//! ```
//!
//! ## Why the `large` scale is slow
//!
//! `Analyzed::from` does a linear `.find()` over a locale's messages for every
//! (message, locale) pair — O(messages² · locales). `lint::comment_mistakes`
//! rescans every message for every commented message — O(messages²). Neither is
//! a problem at hundreds of messages, but both blow up at ten thousand. The
//! three scales exist precisely to make that growth measurable; the cost is
//! that the `large` `analyze` benchmark takes minutes per run.

use std::collections::HashSet;
use std::path::Path;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fluent_typed::bench_internals::{Analyzed, Id, LangBundle, Message, check, generate};
use fluent_typed::{BuildOptions, FtlOutputOptions};

// ---------------------------------------------------------------------------
// Synthetic FTL generator
//
// The data is generated deterministically (no RNG, no `rand` dependency) so
// every run produces byte-identical FTL and timings are comparable. The feature
// mix per message is a fixed, realistic distribution modelled on real Fluent
// projects — that is what makes the benchmark representative rather than a
// best-case of trivial messages.
// ---------------------------------------------------------------------------

/// Locale codes; index 0 (`en`) is always the default locale.
const LOCALE_CODES: [&str; 30] = [
    "en", "de", "fr", "es", "it", "pt", "nl", "sv", "da", "nb", "fi", "pl", "cs", "sk", "hu",
    "ro", "el", "tr", "ru", "uk", "bg", "hr", "sl", "et", "lv", "lt", "ja", "ko", "zh", "ar",
];

/// Message-id prefixes, cycled so the generated ids resemble a real project's.
const ID_PREFIXES: [&str; 6] = ["settings", "account", "dialog", "error", "navigation", "form"];

/// Percentage of messages that carry a `#` comment. Comments drive lint cost,
/// so this is kept at a realistic fraction rather than 0% or 100%.
const COMMENT_DENSITY: usize = 55;

/// One locale's complete FTL source. Every locale gets structurally identical
/// messages (only `language-name` differs) so that cross-locale analysis keeps
/// all of them in the generated `common` set — otherwise the lint and generate
/// stages would be handed an empty set and measure nothing.
fn generate_locale_ftl(locale: &str, num_messages: usize) -> String {
    let mut s = String::with_capacity(num_messages * 110);
    s.push_str("language-name = ");
    s.push_str(locale);
    s.push('\n');
    s.push_str("\n-app-name = Acme\n");
    s.push_str("-company = Acme Corporation\n");
    s.push_str("-product = Acme Suite\n");

    let terms = ["-app-name", "-company", "-product"];

    for i in 0..num_messages {
        let id = format!("{}-field-{i:05}", ID_PREFIXES[i % ID_PREFIXES.len()]);
        let has_comment = (i % 100) < COMMENT_DENSITY;

        // A standalone (detached) comment block every 40 messages, so the
        // linter's `standalone_comments` path is exercised too.
        if i % 40 == 0 && i != 0 {
            s.push_str("\n\n# Section ");
            s.push_str(&(i / 40).to_string());
            s.push_str(": a group of related messages.\n");
        }

        s.push('\n');
        match i % 20 {
            // 40% plain text (one in eight references a term).
            0..=7 => {
                if has_comment {
                    s.push_str("# A short description of this message.\n");
                }
                if i % 8 == 7 {
                    let term = terms[(i / 8) % terms.len()];
                    s.push_str(&format!("{id} = Please see {{ {term} }} for more details.\n"));
                } else {
                    s.push_str(&format!(
                        "{id} = Translated message number {i} with some content text.\n"
                    ));
                }
            }
            // 25% one String variable.
            8..=12 => {
                if has_comment {
                    s.push_str("# $name (String) - The user's display name.\n");
                }
                s.push_str(&format!("{id} = Hello {{ $name }}, your account is ready.\n"));
            }
            // 15% one Number variable.
            13..=15 => {
                if has_comment {
                    s.push_str("# $count (Number) - The number of unread messages.\n");
                }
                s.push_str(&format!("{id} = You have {{ NUMBER($count) }} unread messages.\n"));
            }
            // 10% a select expression.
            16..=17 => {
                if has_comment {
                    s.push_str("# A pluralized status message.\n");
                }
                s.push_str(&format!("{id} =\n"));
                s.push_str("    Status: { $items ->\n");
                s.push_str("        [one] one item\n");
                s.push_str("       *[other] { $items } items\n");
                s.push_str("    }\n");
            }
            // 10% a message with attributes.
            _ => {
                if has_comment {
                    s.push_str("# $name (String) - The user's display name.\n");
                }
                s.push_str(&format!("{id} = Open the dialog.\n"));
                s.push_str("    .label = { $name } preferences\n");
                s.push_str("    .tooltip = Click to see more options.\n");
            }
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Scales & pipeline helpers
// ---------------------------------------------------------------------------

struct Scale {
    name: &'static str,
    messages: usize,
    locales: usize,
}

/// Three project sizes. `large` targets ~10k messages / 30 locales; it is slow
/// to run because of the super-linear `analyze`/`lint` cost noted in the module
/// docs. Shrink these while iterating, or filter a run with `-- /small`.
const SCALES: [Scale; 3] = [
    Scale { name: "small", messages: 100, locales: 2 },
    Scale { name: "medium", messages: 1_000, locales: 10 },
    Scale { name: "large", messages: 10_000, locales: 30 },
];

/// The `(language, ftl_source)` pairs for one scale — one resource per locale.
fn sources_for(scale: &Scale) -> Vec<(String, String)> {
    assert!(
        scale.locales <= LOCALE_CODES.len(),
        "scale '{}' wants {} locales but only {} codes are defined",
        scale.name,
        scale.locales,
        LOCALE_CODES.len(),
    );
    LOCALE_CODES
        .iter()
        .take(scale.locales)
        .map(|&lang| (lang.to_string(), generate_locale_ftl(lang, scale.messages)))
        .collect()
}

/// Parse every locale into a `LangBundle`. The `.expect()` doubles as a
/// correctness guard: if the synthetic generator ever emits invalid FTL the
/// benchmark fails loudly instead of timing nothing.
fn parse_all(sources: &[(String, String)]) -> Vec<LangBundle> {
    sources
        .iter()
        .map(|(lang, ftl)| {
            LangBundle::from_ftl(ftl, "bench.ftl", lang, true)
                .expect("synthetic FTL must parse cleanly")
        })
        .collect()
}

fn default_bundle(bundles: &[LangBundle]) -> &LangBundle {
    bundles
        .iter()
        .find(|b| b.language_id == "en")
        .expect("the `en` default locale must be present")
}

/// The default locale's generated messages — mirrors `Builder::messages`: the
/// ids that survived cross-locale analysis, each emitted once.
fn common_messages<'a>(default: &'a LangBundle, common: &HashSet<Id>) -> Vec<&'a Message> {
    let mut seen = HashSet::new();
    default
        .messages
        .iter()
        .filter(|m| common.contains(&m.id))
        .filter(|m| seen.insert(&m.id))
        .collect()
}

/// Build options for the generate benchmark, with output redirected to a temp
/// directory so the benchmark never writes into the repository.
fn bench_options(out_dir: &Path) -> BuildOptions {
    BuildOptions::default()
        .with_output_file_path(out_dir.join("l10n.rs").to_str().unwrap())
        .with_ftl_output(FtlOutputOptions::single_file(
            out_dir.join("translations.ftl").to_str().unwrap(),
        ))
}

// ---------------------------------------------------------------------------
// Benchmarks — one criterion group per pipeline stage
// ---------------------------------------------------------------------------

/// Stage 1: parse FTL strings into `LangBundle`s.
fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    group.sample_size(10);
    for scale in &SCALES {
        let sources = sources_for(scale);
        group.bench_function(BenchmarkId::from_parameter(scale.name), |b| {
            b.iter(|| parse_all(&sources));
        });
    }
    group.finish();
}

/// Stage 2: cross-locale analysis (the default-locale contract check).
fn bench_analyze(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze");
    group.sample_size(10);
    for scale in &SCALES {
        let bundles = parse_all(&sources_for(scale));
        let default = default_bundle(&bundles);
        group.bench_function(BenchmarkId::from_parameter(scale.name), |b| {
            b.iter(|| Analyzed::from(&bundles, default));
        });
    }
    group.finish();
}

/// Stage 3: lint the FTL comments.
fn bench_lint(c: &mut Criterion) {
    let mut group = c.benchmark_group("lint");
    group.sample_size(10);
    for scale in &SCALES {
        let bundles = parse_all(&sources_for(scale));
        let default = default_bundle(&bundles);
        let analyzed = Analyzed::from(&bundles, default);
        assert!(
            !analyzed.common.is_empty(),
            "synthetic data for scale '{}' produced no generatable messages",
            scale.name,
        );
        group.bench_function(BenchmarkId::from_parameter(scale.name), |b| {
            b.iter(|| check(&bundles, default, &analyzed.common));
        });
    }
    group.finish();
}

/// Stage 4: generate the Rust source.
///
/// `generate` also serialises the embedded FTL to disk via `FtlOutputOptions`;
/// output is redirected to a temp directory, but this stage's timing inherently
/// includes that write — it cannot be separated without changing `generate`.
fn bench_generate(c: &mut Criterion) {
    let out_dir = std::env::temp_dir().join("fluent-typed-bench");
    std::fs::create_dir_all(&out_dir).expect("create temp output dir for the generate benchmark");

    let mut group = c.benchmark_group("generate");
    group.sample_size(10);
    for scale in &SCALES {
        let bundles = parse_all(&sources_for(scale));
        let default = default_bundle(&bundles);
        let analyzed = Analyzed::from(&bundles, default);
        let messages = common_messages(default, &analyzed.common);
        let options = bench_options(&out_dir);
        group.bench_function(BenchmarkId::from_parameter(scale.name), |b| {
            b.iter(|| generate(&options, &bundles, &messages).expect("code generation"));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse, bench_analyze, bench_lint, bench_generate);
criterion_main!(benches);
