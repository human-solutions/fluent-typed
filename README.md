# Fluent-Typed

[![crates.io](https://img.shields.io/crates/v/fluent-typed.svg)](https://crates.io/crates/fluent-typed)
[![docs.rs](https://docs.rs/fluent-typed/badge.svg)](https://docs.rs/fluent-typed)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

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

```rust
let strs = L10n::En.load();

strs.msg_hello("Sam");   // ✓ compiles
strs.msg_hello(42);      // ✗ wrong argument type — caught at compile time
strs.msg_helo("Sam");    // ✗ misspelled key — caught at compile time
```

## Why fluent-typed

- **Typed accessors, no boilerplate.** One `build.rs` call turns every message into a
  typed function. Argument types are inferred from comments, `NUMBER()` calls and plural
  selectors.
- **Dead translations become warnings.** An unused message function raises a `cargo`
  warning, so stale keys don't quietly accumulate.
- **Cross-locale safety.** An accessor is generated only for a message present in *every*
  locale with a matching set of variables — the rest are skipped with a warning naming
  the `.ftl` file and line.
- **A linter for comment mistakes.** Typo'd type keywords, annotations of variables that
  don't exist, comments detached from their message — reported at `Warn`, `Deny` or
  `Strict` levels.
- **Embed or load on demand.** Bake every locale into the binary for server-side use, or
  load one language at a time on the client.
- **Automatic language negotiation.** With the `langneg` feature, `L10n::langneg("en-US")`
  resolves a user's preferred language to the closest available locale, falling back to
  your configured default.
- **Bidi-safe by default.** Interpolated values are wrapped in Unicode isolation marks so
  right-to-left text can't corrupt the surrounding message.
- **A free language menu.** Name a message `language-name` and the generated `L10n` enum
  hands you human-readable names for a language picker.

> To get the unused-message warnings, generate the file in the crate that *uses* the
> accessors, and keep the generated `L10n`/`L10nLanguage` types out of that crate's
> public API — otherwise every message looks "used".

## Usage

Translations live in a `locales/` folder, with one subfolder per language
(`locales/en/`, `locales/fr/`, …) holding `.ftl`
([Fluent](https://projectfluent.org)) files:

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

fluent-typed turns each message into a typed accessor on `L10nLanguage`,
inferring argument types from comments, `NUMBER()` calls and plural selectors:

```rust
strs.msg_greeting();            // plain message -> String
strs.msg_hello("Sam");          // string arg (typed via the comment)
strs.msg_unread_messages(3);    // number arg (typed via the plural selector)
strs.msg_login_placeholder();   // a message attribute
L10n::En.language_name();       // language name, for a language menu
```

```toml
# in Cargo.toml
[dependencies]
fluent-typed = "0.6"

[build-dependencies]
fluent-typed = { version = "0.6", features = ["build"] }
```

```rust
use fluent_typed::{build_from_locales_folder, BuildOptions};

// in build.rs
fn main() -> std::process::ExitCode {
    // Build with the default settings, which means to
    // generate the src/l10n.rs file from the fluent
    // translations found in the `locales/` folder,
    // prefix the generated functions with "msg_" and
    // indent the code with 4 spaces.It also generates a
    // single ftl file with all the languages, which is
    // embedded in the binary. See the BuildOptions and
    // FtlOutputOptions for all the configuration options.
    //
    // This function returns an ExitCode.
    build_from_locales_folder(BuildOptions::default())

    // Note: there are also `try_build_from_locales_folder`
    // which returns a Result
}
```

```rust
// in lib.rs or main.rs
mod l10n;
use l10n::L10n;

// Load English translations into an L10nLanguage struct.
// It provides safe functions for accessing all messages.
let strs: L10nLanguage = L10n::EnGb.load();

// With the feature "langneg" enabled you can do automatic language
// negotiation, which falls back on the default language as
// configured in the BuildOptions in build.rs when generating.
let found_lang: L10nLanguage = L10n::langneg("en");

// In Dioxus/Leptos/Silkenweb etc the L10nLanguage struct is typically
// used inside of a Signal or other reactive construct, so that all
// translations are automatically updated when the struct is changed.

// A message without arguments.
assert_eq!("Welcome!", strs.msg_greeting());
// A message with a string argument (AsRef<str>).
let hello: String = strs.msg_hello("world");
// A message with a number argument (Into<FluentNumber>).
let unread: String = strs.msg_unread_messages(2);
// Note: interpolated values are wrapped in Unicode bidi isolation
// marks by default, so `hello` is "Hello \u{2068}world\u{2069}".
// See "Bidi isolation" below.

// The list of translated, human-readable language names.
let language_names: Vec<&str>
  = L10n::iter().map(|lang| lang.language_name()).collect();

// Server-side, you typically load all the languages once.
let languages = L10n::load_all();
// `get` returns the lower-level `L10nBundle`; access messages by id:
let greeting = languages.get(L10n::En).msg("greeting", None).unwrap();
```

## Type deduction

Since the fluent syntax doesn't explicitly specify the type of the translation variables, this
project uses the following rules to infer the type of the translation variables.

Types are read from the **default locale** only (set with
`BuildOptions::with_default_language`, `en` by default). A type comment in any
other locale has no effect — the [linter](#linting) flags it. Translators never
need to maintain type metadata.

The rules:

- String:
  - If a variable's comment contains `(String)`, as in `# $name (String) - The name.`
- Number:
  - If a variable's comment contains `(Number)`, as in `# $count (Number) - How many.`
  - If a [NUMBER](https://projectfluent.org/fluent/guide/functions.html#number-1) function is used, as in `dpi-ratio = Your DPI ratio is { NUMBER($ratio) }`
  - If a [selector](https://projectfluent.org/fluent/guide/selectors.html) only contains numbers
    and CLDR plural categories (`zero`, `one`, `two`, `few`, `many`, `other`). For example:

```text
your-rank = { NUMBER($pos, type: "ordinal") ->
   [1] You finished first!
   [one] You finished {$pos}st
   [two] You finished {$pos}nd
   [few] You finished {$pos}rd
  *[other] You finished {$pos}th
}
```

## Linting

Because argument types come from message comments, a mistake in a comment would
otherwise silently leave a variable untyped. The linter catches the common ones
and reports the `.ftl` file and line of each:

- a typo'd keyword — `(Numbr)` instead of `(Number)`;
- an annotation of a variable the message doesn't have — `# $nme` vs `$name`;
- a type-annotation comment detached from its message by a blank line;
- a type annotation in a non-default locale, where it has no effect.

`BuildOptions::with_lint_level` controls how strict the check is:

- `LintLevel::Off` — no lint diagnostics.
- `LintLevel::Warn` (the default) — problems are reported as `cargo::warning=`
  lines; the build still succeeds.
- `LintLevel::Deny` — comment mistakes in the default locale become hard build
  errors. An untyped variable is still allowed.
- `LintLevel::Strict` — like `Deny`, and additionally every variable of every
  generated message must resolve to a concrete type (via a `(String)`/`(Number)`
  comment, a `NUMBER()` call or a plural selector). An untyped variable fails
  the build.

```rust,ignore
// in build.rs
let options = BuildOptions::default().with_lint_level(LintLevel::Strict);
```

Diagnostics about non-default locales stay warnings even under `Strict` — they
concern translator-owned files and must never block a build.

## Structured messages

Some translations need the app to inject a UI element mid-sentence — an icon, a
link, a button — without splitting the translated string on a placeholder token.
Annotate a variable or term with `(Element)` and fluent-typed generates a struct
of resolved text segments split at those points:

```ftl
# $count (Number) - How many unread.
# $icon (Element) - An icon injected by the app.
# -privacy-link (Element) - Link text the app wraps in an <a>.
notice = { $icon } You have { $count } unread, see { -privacy-link }.
-privacy-link = our privacy policy
```

An `(Element)` **variable** (`$icon`) is a pure positional gap — the app fills it
with its own element. An `(Element)` **term** (`-privacy-link`) carries
translatable text the app wraps. The generated accessor returns a struct whose
fields are the resolved text runs and the element slots, in render order:

```rust
pub struct Notice {
    pub s0: String,           // text before $icon
    pub icon: ElementGap,     // $icon slot — filled by the app
    pub s1: String,           // text between the elements
    pub privacy_link: String, // resolved -privacy-link text
    pub s2: String,           // text after -privacy-link
}

// $icon / -privacy-link are not parameters; only real arguments are:
let n: Notice = strs.notice(3);
// `Notice` also implements `Display`, joining the text for plain-text use.
```

Selectors and ordinary variables inside each segment are fully resolved, so the
app never re-implements plural logic. When rendering, wrap each field and each
injected element in an isolated bidi run (an HTML `<bdi>`, or
`unicode-bidi: isolate`) — see "Bidi isolation".

A term and a message cannot share a bare name — `-foo` and `foo` collide inside
fluent-bundle, which keys both under `foo`. fluent-typed catches this at build
time and reports the two source locations so it can be fixed before it becomes
a runtime crash.

## Bidi isolation

By default, the generated accessors wrap every interpolated variable in Unicode
bidi isolation marks (FSI `U+2068` … PDI `U+2069`). This is the safe default for
text rendered in a bidi-aware context such as a web UI: it keeps an interpolated
value's text direction from corrupting the surrounding message, which matters
whenever a right-to-left locale is used or user-provided text is interpolated.

```rust
// strs.msg_hello("world") == "Hello \u{2068}world\u{2069}"
```

If the generated strings are never rendered in a bidi-aware context — and you do
not use right-to-left locales or interpolate user-provided text — you can turn
the marks off:

```rust
// in build.rs
let options = BuildOptions::default().without_bidi_isolation();
```
