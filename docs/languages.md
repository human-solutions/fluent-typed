# Languages and negotiation

## The `L10n` enum

The generated `L10n` enum has one variant per locale folder: `locales/en/` becomes
`L10n::En`, `locales/en-gb/` becomes `L10n::EnGb`. It's `Copy`, `Hash` and `Eq`, and
converts to and from the language id:

```rust
let lang: L10n = "fr".parse()?;
assert_eq!(lang.to_string(), "fr");
assert_eq!(L10n::default(), L10n::En); // the default language from build.rs

for lang in L10n::iter() {
    println!("{lang}: {}", lang.language_name());
}
```

`language_name()` exists when every locale defines a `language-name` message, such as
`language-name = Français`. It returns that text, for a language menu.

## Picking a language from `Accept-Language`

Language negotiation is opt-in. Turn it on in `build.rs`:

```rust
// build.rs
use fluent_typed::{BuildOptions, build_from_locales_folder};

fn main() -> std::process::ExitCode {
    build_from_locales_folder(BuildOptions::default().with_langneg())
}
```

The generated code then needs the `langneg` feature on the regular dependency. The
build dependency doesn't:

```toml
[dependencies]
fluent-typed = { version = "0.10", features = ["langneg"] }

[build-dependencies]
fluent-typed = { version = "0.10", features = ["build"] }
```

If you forget the feature, the build fails with an error that says to add it.

`L10n::langneg` takes an `Accept-Language` header and returns the best match, or the
default language when nothing matches:

```rust
let lang: L10n = L10n::langneg("fr-CA, en;q=0.5");
assert_eq!(lang, L10n::Fr);
let strs = lang.load();
```

The header's entries are tried in order of their `q` weight (entries with `q=0` mean
"not acceptable" and are skipped), and the first one that matches any locale wins. For
an entry such as `en-GB`, the closest locale is, in order:

1. the exact locale, `en-gb`;
2. the same region with another script, such as `en-latn-gb`;
3. the generic locale of the language, `en`;
4. another region, such as `en-us`;
5. last, a locale with a different script than the one asked for (`sr-cyrl` for a
   request for `sr-Latn`).

Scripts aren't inferred from regions: a request for `zh-TW` doesn't know it means
Traditional Chinese. If you have both `zh-hans` and `zh-hant`, also add regional
folders that name the script, such as `zh-hant-tw`.

With `en`, `en-gb` and `fr`:

```rust
assert_eq!(L10n::langneg("en-GB"), L10n::EnGb); // exact
assert_eq!(L10n::langneg("en-US"), L10n::En);   // no en-us: the generic en
assert_eq!(L10n::langneg("fr-CA"), L10n::Fr);   // no fr-ca: fr
assert_eq!(L10n::langneg("de"), L10n::En);      // no match: the default language
```

## Serving every language from one server

A server answers requests in many languages. Load each language once at startup and
pick one per request. The loaded languages are `Send + Sync`, so they can be shared
across threads, for example in your web framework's app state:

```rust
use std::collections::HashMap;

// At startup:
let languages: HashMap<L10n, L10nLanguage> =
    L10n::iter().map(|&lang| (lang, lang.load())).collect();

// Per request:
let strs = &languages[&L10n::langneg(accept_language_header)];
let greeting = strs.msg_greeting();
```

### `L10nLanguageVec`

`L10n::load_all()` returns an `L10nLanguageVec`, which holds untyped `L10nBundle`s and
looks messages up by id:

```rust
let all = L10n::load_all();
let greeting: String = all.get(L10n::Fr).msg("greeting", None)?;
```

Use it when you need what the typed API can't do: messages chosen by id at runtime, or
languages that weren't part of the build. Its `insert` adds a language at runtime, and
its `langneg` negotiates over the languages actually loaded, including inserted ones
(the generated `L10n::langneg` only knows the built-in languages). See
[Loading translations at runtime](https://github.com/akesson/fluent-typed/blob/main/docs/runtime-loading.md).
