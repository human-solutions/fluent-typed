# Changelog

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
