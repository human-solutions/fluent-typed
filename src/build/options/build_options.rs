use super::ftl_output_options::FtlOutputOptions;
use super::lint_level::LintLevel;

const DEFAULT_PREFIX: &str = "msg_";

/// Configuration for the build-time code generation performed by
/// [`build_from_locales_folder`](crate::build_from_locales_folder) and
/// [`try_build_from_locales_folder`](crate::try_build_from_locales_folder).
///
/// Start from [`BuildOptions::default`] and adjust it with the `with_*` /
/// `without_*` builder methods:
///
/// ```no_run
/// # use fluent_typed::{BuildOptions, LintLevel};
/// let options = BuildOptions::default()
///     .with_locales_folder("locales")
///     .with_lint_level(LintLevel::Deny);
/// ```
///
/// The fields are private on purpose: this lets new options be added in a minor
/// release without breaking direct struct construction in your build script.
/// Every option has a corresponding builder method.
pub struct BuildOptions {
    /// The path to the folder containing the locales.
    ///
    /// Defaults to "locales".
    pub(crate) locales_folder: String,

    /// The path to the file where the generated code will be written. It is recommended
    /// to use a path inside of `src/` and to include the file in the project so that
    /// you get warnings for unused translation messages.
    ///
    /// Defaults to "src/l10n.rs".
    pub(crate) output_file_path: String,

    /// The the ftl output options, which let you configure how the output ftl
    /// files are generated and accessed.
    pub(crate) ftl_output: FtlOutputOptions,

    /// The indentation used in the generated file.
    ///
    /// Only effective together with `format: false` — rustfmt re-indents the
    /// generated file back to its own style, silently undoing this option.
    ///
    /// Defaults to four spaces.
    pub(crate) indentation: String,

    /// The default language to use for the L10n enum. An error is thrown
    /// during build if the default language is not found in the locales.
    ///
    /// It defaults to "en"
    pub(crate) default_language: String,

    /// Whether to format the generated file or not (uses rustfmt).
    ///
    /// Defaults to true.
    pub(crate) format: bool,

    /// The prefix prepended to every generated message accessor's name,
    /// e.g. `"msg_"` produces `msg_hello_world()`.
    ///
    /// Defaults to `"msg_"`.
    pub(crate) prefix: String,

    /// Whether to return an error if duplicate message keys are found
    /// within the same language.
    ///
    /// Defaults to true.
    pub(crate) deny_duplicate_keys: bool,

    /// How strictly the comments in your `.ftl` files are checked.
    ///
    /// Defaults to [`LintLevel::Warn`].
    pub(crate) lint_level: LintLevel,

    /// Whether the generated code wraps interpolated variables in Unicode
    /// bidi isolation marks (FSI `U+2068` / PDI `U+2069`).
    ///
    /// This is the safe default for text rendered in a bidi-aware context
    /// such as a web UI, and is required for correct rendering when a
    /// right-to-left locale is used or user-provided text is interpolated.
    ///
    /// Defaults to true.
    pub(crate) use_isolating: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            locales_folder: "locales".to_string(),
            output_file_path: "src/l10n.rs".to_string(),
            ftl_output: Default::default(),
            indentation: "    ".to_string(),
            default_language: "en".to_string(),
            format: true,
            prefix: DEFAULT_PREFIX.to_string(),
            deny_duplicate_keys: true,
            lint_level: LintLevel::default(),
            use_isolating: true,
        }
    }
}

impl BuildOptions {
    /// Set the folder scanned for `<lang-id>/<resource>.ftl` files.
    /// Defaults to `"locales"`.
    pub fn with_locales_folder(mut self, locales_folder: &str) -> Self {
        self.locales_folder = locales_folder.to_string();
        self
    }

    /// Set the path the generated Rust file is written to. Keeping it inside
    /// `src/` (the default `"src/l10n.rs"`) and committing it is what produces
    /// the unused-message warnings.
    pub fn with_output_file_path(mut self, output_file_path: &str) -> Self {
        self.output_file_path = output_file_path.to_string();
        self
    }

    /// Set the indentation used in the generated file. Defaults to four spaces.
    ///
    /// Only takes effect together with
    /// [`without_format`](Self::without_format): formatting is on by default,
    /// and rustfmt re-indents the generated file back to its own style
    /// (4 spaces), silently undoing this option.
    pub fn with_indentation(mut self, indentation: &str) -> Self {
        self.indentation = indentation.to_string();
        self
    }

    /// Set how the joined FTL is emitted (single embedded file vs. one file per
    /// language). See [`FtlOutputOptions`].
    pub fn with_ftl_output(mut self, opts: FtlOutputOptions) -> Self {
        self.ftl_output = opts;
        self
    }

    /// Set the default locale — the single source of truth for every message's
    /// typed signature. A build error is raised if it has no subfolder in the
    /// locales folder. Defaults to `"en"`.
    pub fn with_default_language(mut self, lang: &str) -> Self {
        self.default_language = lang.to_string();
        self
    }

    /// Do not run `rustfmt` on the generated file. Formatting is on by default.
    ///
    /// Required for [`with_indentation`](Self::with_indentation) to have any
    /// effect — rustfmt would otherwise re-indent the file back to its own
    /// style.
    pub fn without_format(mut self) -> Self {
        self.format = false;
        self
    }

    /// Set the prefix prepended to every generated message accessor's name,
    /// e.g. `"msg_"` produces `msg_hello_world()`. Defaults to `"msg_"`.
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// Allow duplicate message keys within a language instead of failing the
    /// build. Duplicates are denied by default.
    pub fn with_allow_duplicate_keys(mut self) -> Self {
        self.deny_duplicate_keys = false;
        self
    }

    /// Set how strictly the comments in your `.ftl` files are checked.
    /// See [`LintLevel`].
    pub fn with_lint_level(mut self, level: LintLevel) -> Self {
        self.lint_level = level;
        self
    }

    /// Disable the Unicode bidi isolation marks (FSI `U+2068` / PDI `U+2069`)
    /// that the generated code otherwise wraps around interpolated variables.
    ///
    /// Only do this if the generated strings are never rendered in a
    /// bidi-aware context, or you never use right-to-left locales or
    /// interpolate user-provided text.
    pub fn without_bidi_isolation(mut self) -> Self {
        self.use_isolating = false;
        self
    }
}
