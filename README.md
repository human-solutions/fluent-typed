# Fluent-Typed

When using translation keys, there is often no easy way to know if they are being used
correctly and if they are being used at all. This project generates, using the `fluent` ast,
the function definitions for the translation keys in a fluent file.

In order to guarantee the safeness, funtions are only generated for messages that
are found in all the locales. For those only found for some locales
or if the signature of the messages are different a warning is printed.

It is up to the user to load the translation resources, which gives him the liberty to
choose how they are retreived (embedded in the binary, loaded from a file, downloaded etc).

There is no need to handle any fallback language since it is guaranteed that all messages
are translated into all languages

## Usage

```toml
# in Cargo.toml
[dependencies]
fluent-typed = 0.1

[build-dependencies]
fluent-typed = { version = "0.1", features = ["build"] }
```

```rust
// in build.rs
fn main() -> std::process::ExitCode {
    fluent_typed::build_generate("locales", "src/l10n.rs", "    ")
}
```

```rust
// in lib.rs or main.rs
mod l10n;


fn main() {
    let strs = l10n::L10n::load("en", L10nResource {
      base: include_str!("locales/en/base.ftl"),
      settings: include_str!("locales/en/settings.ftl"),
    });

    // print a translated message
    println!("{}", l10n::base_hello("world"));
}
```

## Type deduction

Since the fluent syntax doesn't explicitly specify the type of the translation variables, this
project uses the following rules to infer the type of the translation variables:

- String:
  - If a variable's comment contains `(String)`, as in `# $name (String) - The name.`
- Number:
  - If a variable's comment contains `(Number)`, as in `# $count (Number) - How many.`
  - If a [NUMBER](https://projectfluent.org/fluent/guide/functions.html#number-1) function is used, asin `dpi-ratio = Your DPI ratio is { NUMBER($ratio) }`
  - If a [selector](https://projectfluent.org/fluent/guide/selectors.html) only contains numbers
    and CLDR plural catagories: `zero`, `one`, `two`, `few`, `many`, and `other`. Example:
    ```
    your-rank = { NUMBER($pos, type: "ordinal") ->
       [1] You finished first!
       [one] You finished {$pos}st
       [two] You finished {$pos}nd
       [few] You finished {$pos}rd
      *[other] You finished {$pos}th
    }
    ```
