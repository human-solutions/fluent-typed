# Types and linting

Fluent doesn't declare the types of a message's variables, so fluent-typed decides
them from the message itself and from its comments. The type decides the accessor's
signature.

## How the type is decided

Types are read from the **default locale** only (`en`, unless you change it with
`BuildOptions::with_default_language`). Type comments in other locales have no effect,
and the linter flags them, so translators never have to maintain type information.

| The variable is… | Type | Accessor parameter |
| --- | --- | --- |
| commented `(String)` | String | `impl AsRef<str>` |
| commented `(Number)`, passed to `NUMBER()`, or a selector whose variants are numbers or plural categories | Number | `impl Into<FluentNumber>` |
| commented `(Bool)` | Bool | `bool` |
| anything else | untyped | `impl Into<FluentValue>` |

A type comment names the variable, then the type in parentheses, then an optional
description:

```ftl
# $name (String) - The user's name.
# $count (Number) - How many unread messages.
inbox = { $name }, you have { $count } unread
```

### Numbers from functions and selectors

`NUMBER($ratio)` makes `$ratio` a number:

```ftl
dpi-ratio = Your DPI ratio is { NUMBER($ratio) }
```

So does a selector whose variants are all numbers or
[CLDR plural categories](https://projectfluent.org/fluent/guide/selectors.html)
(`zero`, `one`, `two`, `few`, `many`, `other`):

```ftl
your-rank = { NUMBER($pos, type: "ordinal") ->
   [1] You finished first!
   [one] You finished {$pos}st
   [two] You finished {$pos}nd
   [few] You finished {$pos}rd
  *[other] You finished {$pos}th
}
```

### Booleans

A `(Bool)` variable gets a Rust `bool`. It reaches Fluent as the string `true` or
`false`, so a selector on it must have exactly a `[true]` and a `[false]` variant:

```ftl
# $enabled (Bool) - Whether the feature is enabled.
feature-status = { $enabled ->
    [true] Enabled
   *[false] Disabled
}
```

```rust
assert_eq!(strs.msg_feature_status(true), "Enabled");
```

Booleans are only ever explicit: a selector with `[true]` and `[false]` but no
`(Bool)` comment is an untyped selector.

### Untyped variables

A variable none of the rules apply to accepts anything Fluent can format, which is
strings and numbers:

```ftl
# no type comment for $who
greeting-for = Hi { $who }
```

```rust
strs.msg_greeting_for("Ana");
strs.msg_greeting_for(3);
```

That's convenient, but the compiler can no longer catch a wrong argument. Add a type
comment, or use `LintLevel::Strict` to require one for every variable.

## Linting

Because types come from comments, a mistake in a comment would silently leave a
variable untyped. The linter catches the common mistakes and reports the `.ftl` file
and line of each:

- a misspelled type, such as `(Numbr)`;
- a comment for a variable the message doesn't have, such as `# $nme` when the message
  uses `$name`;
- a type comment separated from its message by a blank line;
- a type comment in a non-default locale, where it has no effect.

`BuildOptions::with_lint_level` sets what happens when one is found:

| Level | Effect |
| --- | --- |
| `LintLevel::Off` | No lint output. |
| `LintLevel::Warn` (default) | Each problem is a `cargo` warning; the build succeeds. |
| `LintLevel::Deny` | Comment mistakes in the default locale fail the build. Untyped variables are still allowed. |
| `LintLevel::Strict` | Like `Deny`, and every variable must have a type (a comment, `NUMBER()` or a plural selector). |

```rust
// build.rs
let options = BuildOptions::default().with_lint_level(LintLevel::Strict);
```

Problems in non-default locales stay warnings at every level: those files belong to
translators and shouldn't break your build.
