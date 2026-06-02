# Changelog

## 0.6.2

### Changed
- The runtime types `L10nBundle`, `L10nLanguageVec` and the generated
  `L10nLanguage` are now `Send + Sync`, so loaded localizations can be shared
  across threads (e.g. behind an `Arc` in shared web-server state). Internally
  the bundle now uses fluent-bundle's concurrent (`Mutex`-based) memoizer.

## 0.6.1

### Added
- `BuildError::TermMessageCollision` — a term and a message that share a bare
  name (e.g. `-foo` and `foo`) collide in fluent-bundle's single-namespace
  entry map and would otherwise crash at resource-load time. This is now caught
  at build time and reported with the name and the file/line of both the term
  and the message definition.

### Changed
- **Breaking:** `BuildError` is now `#[non_exhaustive]`. Downstream code that
  matches it exhaustively must add a wildcard arm. This future-proofs the enum
  so subsequent variant additions stay on patch releases.

## 0.6.0

### Added
- Comment linting. `LintLevel` (`Off` / `Warn` / `Deny` / `Strict`) and
  `BuildOptions::with_lint_level()`. fluent-typed now checks `.ftl` comments for
  typo'd type keywords (`(Numbr)`), annotations of variables the message does
  not have, type-annotation comments detached from their message by a blank
  line, and (ineffective) type annotations in non-default locales — reporting
  the file and line of each. `Deny` turns these into hard build errors; `Strict`
  also requires every variable of every generated message to resolve to a
  concrete type.
- `BuildError::DefaultLanguageNotFound` and `BuildError::Lint` variants.
- `BuildError::Multiple` — parse errors, unreadable files and duplicate keys are
  now collected across every `.ftl` file and reported together, instead of the
  build failing on the first one encountered.
- Structured (`(Element)`) messages: annotate a variable or term with
  `(Element)` to have fluent-typed generate a struct of resolved text segments
  split at those points, instead of a single `String`. Variable elements are
  positional gaps the app fills; term elements carry translatable text. Adds
  `L10nBundle::msg_segments()` and the `Segment` / `ElementGap` prelude types.
- `BuildOptions::without_bidi_isolation()` and the `BuildOptions::use_isolating`
  field to control whether generated accessors wrap interpolated variables in
  Unicode bidi isolation marks (FSI/PDI). Defaults to enabled.
- `L10nBundle::new_without_isolation()` and
  `L10nLanguageVec::load_without_isolation()` constructors.

### Changed
- **Breaking:** message argument types are now read from the **default locale**
  only. Previously each locale was typed from its own comments and a message was
  dropped when locales disagreed; now the default locale defines the contract
  and other locales need only a compatible *set* of variables, not matching type
  comments. Type comments in non-default locales no longer affect output (the
  linter flags them). The generated doc comments now always come from the
  default locale.
- **Breaking:** `BuildError::FtlParse` is now a struct variant
  `{ path, errors }` — was a tuple `FtlParse(String)` — and reports the file and
  the line of each parse error. `BuildError::DuplicateKey` gains `original_line`
  and `duplicate_line` fields and its message now includes line numbers.
- **Breaking:** `BuildOptions` has a new `lint_level` field (default
  `LintLevel::Warn`) and a new `use_isolating` field (default `true`). Direct
  struct construction must include these fields; `BuildOptions::default()`
  users are unaffected.

### Removed
- **Breaking:** the `Pattern<String>` output mode and everything tied to it —
  the `OutputMode` enum, `BuildOptions::with_output_mode()`,
  `L10nBundle::msg_pattern()` / `attr_pattern()`, and the `Pattern` /
  `PatternElement` prelude re-exports. Generated accessors always return a
  resolved `String` (or a struct for `(Element)` messages); use the `(Element)`
  annotation for messages where the app injects its own UI elements. The
  `BuildOptions::output_mode` field is replaced by a `prefix: String` field,
  set with the (no longer deprecated) `BuildOptions::with_prefix()`.

### Fixed
- `NUMBER()` calls now trigger number type deduction as documented. A variable
  used inside `NUMBER(...)`, in a function-reference selector, or only inside a
  select variant's body was not detected at all, so the generated accessor
  silently omitted that parameter.
- FTL builtins (`NUMBER`, …) are now registered on the runtime `L10nBundle`.
  Previously a message using `{ NUMBER($x) }` failed to format, which made the
  generated accessor panic when called.
- An empty locales folder now returns the new `BuildError::NoLocaleFolders`
  variant instead of panicking.

## 0.5.0

### Added
- `BuildError` enum for structured, inspectable build errors with proper
  `Display` and `Error` trait implementations.
- Duplicate message key detection across FTL files within the same language,
  enabled by default. Disable with `BuildOptions::with_allow_duplicate_keys()`.

### Changed
- **Breaking:** All public build functions now return `Result<_, BuildError>`
  instead of `Result<_, String>`. Callers using `map_err` or matching on
  `String` errors must update to use `BuildError`.
- **Breaking:** `BuildOptions` has a new `deny_duplicate_keys` field
  (default `true`). Direct struct construction must include this field;
  `BuildOptions::default()` users are unaffected.

## 0.4.0

### Added
- `OutputMode` enum to control whether generated functions return `String`,
  `Pattern<String>`, or both. Configure via
  `BuildOptions::default().with_output_mode(OutputMode::default_pattern())`.
- `L10nBundle::msg_pattern()` and `attr_pattern()` methods for retrieving the
  raw fluent AST, enabling UI frameworks to render term references as
  interactive components.
- `Pattern` and `PatternElement` re-exported in the prelude.

### Changed
- **Breaking:** `BuildOptions::prefix` field replaced by `output_mode: OutputMode`.
  Direct struct construction must be updated. The `with_prefix()` builder method
  is preserved but deprecated — use `with_output_mode()` instead.

### Fixed
- Generated doc comment now references `LanguageIdentifier` instead of
  the old `unic_langid::LanguageIdentifier` path.

## 0.3.0

**Breaking:** Replaced `icu_locid` and `fluent-langneg` dependencies with `icu_locale_core` 2.1. The re-exported `LanguageIdentifier` type is now from `icu_locale_core` instead of `icu_locid`.

## 0.2.9 and earlier

See [git history](https://github.com/human-solutions/fluent-typed/commits/main).
