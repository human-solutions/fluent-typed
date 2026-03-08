# Changelog

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
