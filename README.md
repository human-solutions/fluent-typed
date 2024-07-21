# Fluent-Typed

When using translation keys, there is often no easy way to know if they are being used
correctly and if they are being used at all. This project generates, using the `fluent` ast,
the type definitions for the translation keys in a fluent file.

## Usage

```toml
[dependencies]
fluent-typed = 0.1
```

```rust
// in build.rs
fn main() -> std::process::ExitCode {
    fluent_typed::build_generate("path/to/your/fluent/file.ftl", "path/to/your/output.rs")
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
