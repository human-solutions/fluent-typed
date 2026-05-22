# Changelog

## Unreleased

### Added
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
- **Breaking:** `BuildOptions` has a new `use_isolating` field (default
  `true`). Direct struct construction must include this field;
  `BuildOptions::default()` users are unaffected.

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
