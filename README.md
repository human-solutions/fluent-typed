# Fluent-Typed

When using translation keys, there is often no easy way to know if they are being used
correctly and if they are being used at all. This project generates, using the `fluent` ast,
the function definitions for the translation keys in a fluent file.

In order to guarantee the safeness, funtions are only generated for messages that
are found in all the locales. For those only found for some locales
or if the signature of the messages are different a warning is printed.

Each locale's ftl resources are appended into a single ftl file, and you can configure it
to either embed all of them into the binary with accessors suitable both for server-side
where all of them loaded at startup and accessed via a LazyLock, or client-side where
a single one is loaded and then can be used in a signal. This single ftl file can be
compressed to your liking using a hook.

You also have the freedom
to handle the loading of them yourself, which is especially useful if you want to
download a single language at a time without the need for storing them in the binary.

A little extra feature is that if you name one of the messages as `language-name` and it
doesn't use any variables plus it's present in all languages, then the generated L10n
enum will also contain the names of all the languages, which is really useful when you
want to present the user with a drop-down menu listing all the available languages.

Note that in order to get the warnings for unused message functions, you have to generate
the file in the same crate as where you use them, and you cannot make L10nLanguage file
part of the crate's public interface.

## Usage

```toml
# in Cargo.toml
[dependencies]
fluent-typed = "0.5"

[build-dependencies]
fluent-typed = { version = "0.5", features = ["build"] }
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

## Output modes

By default, generated functions return a resolved `String`. You can configure `OutputMode` to
return the fluent AST `Pattern<String>` instead, which preserves the message structure (selectors,
variable references, etc). Use `OutputMode::default_both()` to generate both.

```rust
// in build.rs
let options = BuildOptions::default()
    .with_output_mode(OutputMode::default_pattern());
```

The generated functions will then return `Pattern<String>` instead of `String`:

```rust
// default (String mode) generates:
pub fn msg_hello_world(&self) -> String { .. }

// Pattern mode generates:
pub fn ptn_hello_world(&self) -> Pattern<String> { .. }

// Both mode generates both:
pub fn msg_hello_world(&self) -> String { .. }
pub fn ptn_hello_world(&self) -> Pattern<String> { .. }
```

## Type deduction

Since the fluent syntax doesn't explicitly specify the type of the translation variables, this
project uses the following rules to infer the type of the translation variables:

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
