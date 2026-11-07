# Fluent-Typed

[![crates.io](https://img.shields.io/crates/v/fluent-typed.svg)](https://crates.io/crates/fluent-typed)
[![docs.rs](https://docs.rs/fluent-typed/badge.svg)](https://docs.rs/fluent-typed)
[![CI](https://github.com/akesson/fluent-typed/actions/workflows/rust.yml/badge.svg)](https://github.com/akesson/fluent-typed/actions/workflows/rust.yml)
[![msrv](https://img.shields.io/crates/msrv/fluent-typed.svg)](https://github.com/akesson/fluent-typed)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/akesson/fluent-typed/blob/main/LICENSE)

**Your [Fluent](https://projectfluent.org) translations as typed Rust functions.** A
misspelled key, a missing translation, or the wrong argument type becomes a compile
error — and a translation you no longer use becomes a build warning.

Write a message in an `.ftl` file:

```ftl
# locales/en/main.ftl
# $name (String) - the user's name
hello = Hello { $name }
```

…and `fluent-typed` generates a typed accessor for it. Calling it wrong no longer fails
silently at runtime — it fails the build:

<!-- docs-check: skip -->
```rust
let strs = L10n::En.load();

strs.msg_hello("Sam");   // ✓ compiles
strs.msg_hello(42);      // ✗ wrong argument type — caught at compile time
strs.msg_helo("Sam");    // ✗ misspelled key — caught at compile time
```

## Why fluent-typed

- **Typed accessors, no boilerplate.** One `build.rs` call turns every message into a
  typed function. Argument types come from comments, `NUMBER()` calls and plural
  selectors.
- **Dead translations become warnings.** An unused message accessor raises a compiler
  warning, so stale keys don't quietly accumulate.
- **Cross-locale safety.** An accessor is generated only for a message present in *every*
  locale with a matching set of variables — the rest are skipped with a warning naming
  the `.ftl` file and line.
- **A linter for comment mistakes.** Typo'd type keywords, annotations of variables that
  don't exist, comments detached from their message.
- **Embed or load on demand.** Bake every locale into the binary, or load one language
  at a time — on a server, or downloaded by a client.
- **Validated runtime loading.** Load an `.ftl` the build never saw — a language added
  after release, or a hot-reloaded translation — checked against the compiled message
  contract, so a missing message or a wrong variable is rejected when you load it
  instead of panicking in an accessor.
- **Language negotiation.** `L10n::langneg("fr-CA, en;q=0.5")` picks the closest
  available language, falling back to your default.
- **Bidi-safe by default.** Interpolated values are wrapped in Unicode isolation marks so
  right-to-left text can't corrupt the surrounding message.

## How it compares

The established way to get compile-time-checked Fluent in Rust is
[`i18n-embed`](https://crates.io/crates/i18n-embed) with its
[`i18n-embed-fl`](https://crates.io/crates/i18n-embed-fl) `fl!` macro, which checks a
message id and its arguments where you call it. fluent-typed takes a different route: a
build script generates one typed Rust function per message.

|                              | fluent-typed                                  | `fl!` macro (`i18n-embed-fl`)            |
| ---------------------------- | --------------------------------------------- | ---------------------------------------- |
| Call site                    | `strs.msg_hello("Sam")` — a real method       | `fl!(LOADER, "hello", name = "Sam")`     |
| Editor support               | autocomplete, go-to-definition, rename        | string ids, no completion                |
| Unused translations          | compiler **warning** for messages you dropped | not detected                             |
| Argument types               | inferred & enforced (`String`/`Number`/…)     | checked as present                       |
| Output                       | plain Rust you can read & commit              | macro-expanded at the call site          |

Trade-offs, honestly: fluent-typed runs a build script and commits a generated file,
where `i18n-embed` has no codegen step; `i18n-embed` has a much larger ecosystem and
user base; and fluent-typed needs a recent toolchain (see
[Versioning and MSRV](#versioning-and-msrv)). Pick fluent-typed when you want
translations to feel like ordinary typed functions and want dead keys to surface
themselves.

## Getting started

**1. Add the dependency twice** — once for the generated code at runtime, once for the
build script:

```toml
# Cargo.toml
[dependencies]
fluent-typed = "0.10"

[build-dependencies]
fluent-typed = { version = "0.10", features = ["build"] }
```

**2. Generate the code in `build.rs`:**

```rust
// build.rs
use fluent_typed::{build_from_locales_folder, BuildOptions};

fn main() -> std::process::ExitCode {
    build_from_locales_folder(BuildOptions::default())
}
```

With the defaults, this reads `locales/`, writes the Rust code to `src/l10n.rs`, and
joins all translations into `gen/translations.ftl`, which the generated code embeds in
your binary. Use `try_build_from_locales_folder` instead if you want a `Result`. See
[`BuildOptions`](https://docs.rs/fluent-typed/latest/fluent_typed/struct.BuildOptions.html)
for every option.

**3. Write your translations** in `locales/<language>/*.ftl`, one folder per language:

```ftl
# locales/en/main.ftl
language-name = English
greeting = Welcome!

# $name (String) - the user's name
hello = Hello { $name }

unread-messages = { $count ->
    [one] You have one message
   *[other] You have { $count } messages
}

login = Log in
    .placeholder = Enter your email
```

Add the same messages to `locales/fr/main.ftl`, `locales/de/main.ftl`, and so on.
Argument types are read from the default locale only, `en` unless you change it (see
[Types and linting](https://github.com/akesson/fluent-typed/blob/main/docs/types-and-linting.md)).

**4. Use the generated code:**

<!-- docs-check: skip -->
```rust
// src/main.rs (or lib.rs)
mod l10n;
use l10n::{L10n, L10nLanguage};
```

```rust
let strs: L10nLanguage = L10n::En.load();

assert_eq!(strs.msg_greeting(), "Welcome!");
let hello: String = strs.msg_hello("Sam");        // a String argument
let unread: String = strs.msg_unread_messages(3); // a number (the plural selector)
let placeholder = strs.msg_login_placeholder();   // a message attribute
```

Every message becomes a method named after it, with a `msg_` prefix (configurable with
`BuildOptions::with_prefix`). `L10n` is an enum of your languages: `L10n::iter()` lists
them, and when every locale has a `language-name` message,
`L10n::Fr.language_name()` gives you the label for a language menu.

> **Interpolated values are wrapped in bidi isolation marks**, so `hello` above is
> `"Hello \u{2068}Sam\u{2069}"`. That's the safe default for UI text; see
> [Bidi isolation](https://github.com/akesson/fluent-typed/blob/main/docs/structured-messages.md#bidi-isolation)
> to turn it off.

> **To get the unused-message warnings**, generate the file in the crate that *uses* the
> accessors, and keep the generated types out of that crate's public API — otherwise
> every message looks "used".

## Guides

- [**Output modes**](https://github.com/akesson/fluent-typed/blob/main/docs/output-modes.md)
  — embed all languages, compress them, or load one language at a time on demand
  (for example in a WASM client).
- [**Languages and negotiation**](https://github.com/akesson/fluent-typed/blob/main/docs/languages.md)
  — pick a language from an `Accept-Language` header, and serve all languages from
  one server.
- [**Loading translations at runtime**](https://github.com/akesson/fluent-typed/blob/main/docs/runtime-loading.md)
  — add a language after release, hot-reload a translation, validate a translator's
  file in CI.
- [**Types and linting**](https://github.com/akesson/fluent-typed/blob/main/docs/types-and-linting.md)
  — how argument types are decided, and the checks for mistakes in type comments.
- [**Structured messages and bidi**](https://github.com/akesson/fluent-typed/blob/main/docs/structured-messages.md)
  — put a link or an icon in the middle of a sentence, and control bidi isolation.

The guides are also on [docs.rs](https://docs.rs/fluent-typed/latest/fluent_typed/guide/index.html).

## Versioning and MSRV

The minimum supported Rust version is **1.88** (the crate uses the 2024 edition and
let-chains). Raising the MSRV is treated as a minor-version change.

fluent-typed is pre-1.0 and its API is still settling — minor releases may contain
breaking changes, each called out in the
[CHANGELOG](https://github.com/akesson/fluent-typed/blob/main/CHANGELOG.md). The runtime
re-exports of `fluent-bundle` types (`FluentArgs`, `FluentValue`, `FluentNumber`)
track that crate's own (pre-1.0) releases. Depend on a minor version
(`fluent-typed = "0.10"`) and read the changelog before upgrading.
