# Output modes

The build joins each language's `.ftl` files into one translation per language. How
that output is written, and therefore how your app gets at it, is set with
`BuildOptions::with_ftl_output`:

| Mode | `FtlOutputOptions` | Translations live | You load with |
| --- | --- | --- | --- |
| Single file (default) | `single_file("gen/translations.ftl")` | embedded in the binary | `L10n::En.load()` |
| Compressed single file | `single_compressed_file(path, compressor)` | embedded, compressed | `L10n::En.load(decompressor)?` |
| One file per language | `multi_file("gen/multi/")` | next to the binary, or on a server | `L10nLanguage::new(lang, bytes)?` |

The mode changes the generated API, so switching modes means updating the code that
loads translations. The message accessors (`strs.msg_hello(..)`) are the same in
every mode.

## Single file (the default)

All languages are joined into `gen/translations.ftl`, which the generated code embeds
with `include_bytes!`. Nothing to ship or fetch, and loading can't fail:

```rust
let strs: L10nLanguage = L10n::Fr.load();
let all = L10n::load_all(); // every language, for a server; see the languages guide
```

Use this unless the translations are a noticeable part of your binary size.

## Compressed single file

The same embedded file, compressed by a function you supply at build time. You bring
the compression crate, and decompress the same way at runtime. With
[`flate2`](https://crates.io/crates/flate2) in both `[dependencies]` and
`[build-dependencies]`:

```rust
// build.rs
use std::io::Write;

use flate2::{Compression, write::GzEncoder};
use fluent_typed::{BuildOptions, FtlOutputOptions, try_build_from_locales_folder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ftl = FtlOutputOptions::single_compressed_file("gen/translations.ftl.gz", |ftl| {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&ftl)?;
        Ok(encoder.finish()?)
    });
    try_build_from_locales_folder(BuildOptions::default().with_ftl_output(ftl))?;
    Ok(())
}
```

`load` and `load_all` now take the decompressor and return a `Result`:

<!-- docs-check: module=l10n_gz -->
```rust
use std::io::Read;

fn gunzip(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(data)
        .read_to_end(&mut out)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

let strs: L10nLanguage = L10n::Fr.load(gunzip)?;
let all = L10n::load_all(gunzip)?;
```

The whole file is decompressed on each call, so load once and keep the result.

## One file per language

Each language is written to its own file, `gen/multi/en.ftl`, `gen/multi/fr.ftl`, and
so on. Nothing is embedded: you ship the files and load the one you need. This is the
mode for a client, such as a WASM app, that should download only the user's language.

```rust
// build.rs
use fluent_typed::{BuildOptions, FtlOutputOptions, build_from_locales_folder};

fn main() -> std::process::ExitCode {
    let ftl = FtlOutputOptions::multi_file("gen/multi/");
    build_from_locales_folder(BuildOptions::default().with_ftl_output(ftl))
}
```

The generated `L10n` has no `load` in this mode. Get the bytes however your app gets
files, and pass them to `L10nLanguage::new`:

<!-- docs-check: module=l10n_multi -->
```rust
let lang = L10n::Fr;
// Read from disk here; a client would fetch `/l10n/fr.ftl` over HTTP instead.
let bytes = std::fs::read(format!("gen/multi/{lang}.ftl"))?;
let strs: L10nLanguage = L10nLanguage::new(lang, bytes)?;
```

`L10n` displays as its language id (`fr`), which makes building the file name easy.

If the files can get out of step with the binary, for example because a CDN caches an
old version, load them with `L10nLanguage::new_external` instead. It checks the file
against the messages your binary expects and returns an error instead of loading a
translation that would make an accessor panic; see
[Loading translations at runtime](https://github.com/akesson/fluent-typed/blob/main/docs/runtime-loading.md).

## What to commit

`gen/` is rewritten on every build, so you can add it to `.gitignore`. The generated
Rust file (`src/l10n.rs` by default) is also rewritten on every build, but it's worth
committing: it's readable Rust, and a diff of it in review shows exactly which
accessors a translation change added, removed or changed.
