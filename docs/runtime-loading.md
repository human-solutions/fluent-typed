# Loading translations at runtime

The build bakes the translations it knows about into the binary, but the typed
accessors look messages up at runtime. So a translation the build never saw can be
loaded too, as long as it defines what the accessors expect.

## Adding a language after release

`L10nLanguage::new_external` takes the bytes of an `.ftl` file and checks them against
the *message contract* compiled into the generated file before loading them:

- every generated message and attribute is defined;
- no message uses a variable its accessor doesn't pass;
- structured messages keep their `(Element)` markers;
- every message and term it references exists, directly or through other
  references, without reference cycles;
- every function it calls is `NUMBER()`, the only one the runtime registers.

These are the checks each locale gets at build time, using the same code. At build
time a message that fails them in any locale isn't generated, with a warning naming
the file and line; at runtime the file is rejected.

<!-- docs-check: no_run -->
```rust
// A Polish translation shipped next to the binary after release.
let bytes = std::fs::read("locales/pl.ftl")?;
let pl: L10nLanguage = L10nLanguage::new_external("pl", bytes)?;
pl.msg_hello("Ala");
```

A broken translation is rejected when it's loaded, with an `L10nError::Validation`
that lists every problem, instead of panicking later inside an accessor:

```rust
let err = L10nLanguage::new_external("pl", "helo = oops").unwrap_err();
assert!(matches!(err, L10nError::Validation { .. }));
println!("{err}"); // names the missing messages, e.g. `hello`
```

The check can't verify argument *types* at runtime, and doesn't need to: the accessor
signatures already fixed what your code passes, at compile time. Matching variable
*names* is the same guarantee the build gives every locale.

`new_external` (like `new`) takes anything that converts into a `Vec<u8>`. Pass an
owned buffer, such as the `Vec<u8>` from `std::fs::read`, and it's used without a
copy; a `&[u8]` or `&str` is copied once.

## Adding a language to `L10nLanguageVec`

If you serve languages through `L10nLanguageVec`, `insert` adds one. Inserting a
language that's already there replaces it:

```rust
let mut languages = L10n::load_all();
languages.insert(L10nBundle::new("pl", bytes)?);

// `langneg` here considers every loaded language, including "pl". It returns an
// `Option`, so you pick the fallback:
let bundle = languages
    .langneg(accept_language_header)
    .unwrap_or_else(|| languages.get(L10n::default()));
```

`L10nBundle::new` doesn't validate. Use `L10nLanguage::new_external` first if the file
might not match your messages.

## Hot reloading

To see a translator's changes while the app runs, reload the file when it changes and
swap the loaded translation. fluent-typed doesn't watch files itself; use a crate such
as [`notify`](https://crates.io/crates/notify), and keep the current translation
behind a lock (or an [`arc-swap`](https://crates.io/crates/arc-swap)):

<!-- docs-check: no_run -->
```rust
let current = RwLock::new(Arc::new(L10n::En.load()));

// When the file changes:
match L10nLanguage::new_external("en", std::fs::read(&path)?) {
    Ok(fresh) => *current.write().unwrap() = Arc::new(fresh),
    Err(e) => eprintln!("translation rejected: {e}"), // keep the last good one
}
```

Because `new_external` validates, a half-finished edit is rejected and the app keeps
the last good version.

## Validating a file without loading it

To check a translator's file in CI, call the validation directly. The generated module
exports the contract as `MESSAGE_CONTRACTS`:

<!-- docs-check: no_run -->
```rust
let bytes = std::fs::read("locales/pl.ftl")?;
fluent_typed::validate_ftl(&bytes, crate::l10n::MESSAGE_CONTRACTS)?;
```

This is the same check `new_external` runs.

## What isn't checked

Argument *types* aren't checked at runtime (see above), and neither are variables
inside terms: a term that uses a variable its caller doesn't pass, such as
`-brand = Acme { $x }` used as `{ -brand }`, renders as `Acme {$x}`. That's wrong
output, but not a panic.
