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

## Usage

```toml
# in Cargo.toml
[dependencies]
fluent-typed = 0.1

[build-dependencies]
fluent-typed = { version = "0.1", features = ["build"] }
```

```rust
use fluent_typed::{build_from_locales_folder, BuildOptions, FtlOutputOptions};

// in build.rs
fn main() -> std::process::ExitCode {
    // Build with the default settings, which means to generate the src/l10n.rs file from
    // the fluent translations found in the `locales/` folder, prefix the generated functions
    // with "msg_" and indent the code with 4 spaces.It also generates a single ftl file with
    // all the languages, which is embedded in the binary. See the BuildOptions and
    // FtlOutputOptions for all the configuration options.
    //
    // This function returns an ExitCode.
    build_from_locales_folder(BuildOptions::default())

    // Note: there are also fluent_typed::try_build_from_locales_folder which returns a Result
}
```

```rust
// in lib.rs or main.rs
mod l10n;
use l10n::L10nLang;


fn main() {

    // load english translations.
    let strs = L10nLang::En.load().unwrap();

    // A message without arguments.
    assert_eq!("Welcome!", strs.msg_greeting());
    // A message with a string argument (AsRef<str>).
    assert_eq!("Hello world", strs.msg_hello("world"));
    // A message with a number argument (Into<FluentNumber>).
    assert_eq!("You have 2 unread messages", strs.msg_unread_messages(2));
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
