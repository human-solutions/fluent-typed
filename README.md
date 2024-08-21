# Fluent-Typed

When using translation keys, there is often no easy way to know if they are being used
correctly and if they are being used at all. This project generates, using the `fluent` ast,
the function definitions for the translation keys in a fluent file.

In order to be light-weight and flexible, this simply generates an extension to the [FluentBundle](https://docs.rs/fluent-bundle/latest/fluent_bundle/bundle/struct.FluentBundle.html) with
a function for each message.

> [!IMPORTANT]
> Please make sure that you load the same resources into the bundles as the ones
> used to generate the function definitions.

In order to guarantee the safeness, funtions are only generated for messages that
are found in all the locales. For those only found for some locales
or if the signature of the messages are different a warning is printed.

## Usage

```toml
# fluent-typed is only used in build.rs
[build-dependencies]
fluent-typed = 0.1

# fluent-typed generates code that depends on fluent-bundle and fluent-syntax
[dependencies]
fluent-bundle = "0.15"
fluent-syntax = "0.11"
```

```rust
// in build.rs
fn main() -> std::process::ExitCode {
    fluent_typed::build_generate("fluent/file.ftl", "gen")
}
```

```rust
// in the rust package that uses the translations
include!("../gen/bundle_ext.rs"));
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
