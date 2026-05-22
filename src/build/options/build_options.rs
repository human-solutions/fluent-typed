use super::ftl_output_options::FtlOutputOptions;

const DEFAULT_PREFIX: &str = "msg_";

pub struct BuildOptions {
    /// The path to the folder containing the locales.
    ///
    /// Defaults to "locales".
    pub locales_folder: String,

    /// The path to the file where the generated code will be written. It is recommended
    /// to use a path inside of `src/` and to include the file in the project so that
    /// you get warnings for unused translation messages.
    ///
    /// Defaults to "src/l10n.rs".
    pub output_file_path: String,

    /// The the ftl output options, which let you configure how the output ftl
    /// files are generated and accessed.
    pub ftl_output: FtlOutputOptions,

    /// The indentation used in the generated file.
    ///
    /// Defaults to four spaces.
    pub indentation: String,

    /// The default language to use for the L10n enum. An error is thrown
    /// during build if the default language is not found in the locales.
    ///
    /// It defaults to "en"
    pub default_language: String,

    /// Whether to format the generated file or not (uses rustfmt).
    ///
    /// Defaults to true.
    pub format: bool,

    /// The prefix prepended to every generated message accessor's name,
    /// e.g. `"msg_"` produces `msg_hello_world()`.
    ///
    /// Defaults to `"msg_"`.
    pub prefix: String,

    /// Whether to return an error if duplicate message keys are found
    /// within the same language.
    ///
    /// Defaults to true.
    pub deny_duplicate_keys: bool,

    /// Whether the generated code wraps interpolated variables in Unicode
    /// bidi isolation marks (FSI `U+2068` / PDI `U+2069`).
    ///
    /// This is the safe default for text rendered in a bidi-aware context
    /// such as a web UI, and is required for correct rendering when a
    /// right-to-left locale is used or user-provided text is interpolated.
    ///
    /// Defaults to true.
    pub use_isolating: bool,
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
            use_isolating: true,
        }
    }
}

impl BuildOptions {
    pub fn with_locales_folder(mut self, locales_folder: &str) -> Self {
        self.locales_folder = locales_folder.to_string();
        self
    }

    pub fn with_output_file_path(mut self, output_file_path: &str) -> Self {
        self.output_file_path = output_file_path.to_string();
        self
    }

    pub fn with_indentation(mut self, indentation: &str) -> Self {
        self.indentation = indentation.to_string();
        self
    }

    pub fn with_ftl_output(mut self, opts: FtlOutputOptions) -> Self {
        self.ftl_output = opts;
        self
    }

    pub fn with_default_language(mut self, lang: &str) -> Self {
        self.default_language = lang.to_string();
        self
    }

    pub fn without_format(mut self) -> Self {
        self.format = false;
        self
    }

    /// Set the prefix prepended to every generated message accessor's name.
    /// See [`BuildOptions::prefix`].
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn with_allow_duplicate_keys(mut self) -> Self {
        self.deny_duplicate_keys = false;
        self
    }

    /// Disable Unicode bidi isolation marks around interpolated variables
    /// in the generated code. See [`BuildOptions::use_isolating`].
    ///
    /// Only do this if the generated strings are never rendered in a
    /// bidi-aware context, or you never use right-to-left locales or
    /// interpolate user-provided text.
    pub fn without_bidi_isolation(mut self) -> Self {
        self.use_isolating = false;
        self
    }
}
