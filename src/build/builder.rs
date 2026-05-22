use super::{
    Analyzed, BuildError, BuildOptions, LangBundle, LintLevel, Message, r#gen::generate, lint,
    typed::Id,
};
use std::{collections::HashSet, fs};

pub struct Builder {
    options: BuildOptions,
    langbundles: Vec<LangBundle>,
}

impl Builder {
    pub fn load(options: BuildOptions) -> Result<Self, BuildError> {
        let folder = &options.locales_folder;
        println!("cargo::rerun-if-changed={folder}");

        let mut langbundles = from_locales_folder(folder, options.deny_duplicate_keys)?;

        langbundles.sort_by_cached_key(|lb| lb.language_id.clone());

        Ok(Self {
            langbundles,
            options,
        })
    }

    #[cfg(test)]
    pub fn load_one(
        options: BuildOptions,
        resource_name: &str,
        lang: &str,
        ftl: &str,
    ) -> Result<Self, BuildError> {
        let deny_duplicate_keys = options.deny_duplicate_keys;
        Ok(Self {
            options,
            langbundles: vec![LangBundle::from_ftl(
                ftl,
                resource_name,
                lang,
                deny_duplicate_keys,
            )?],
        })
    }

    pub fn generate(&self) -> Result<(), BuildError> {
        // The default locale is the single source of truth for every message's
        // typed signature, so it must exist.
        let default = self
            .langbundles
            .iter()
            .find(|b| b.language_id == self.options.default_language)
            .ok_or_else(|| BuildError::DefaultLanguageNotFound {
                language: self.options.default_language.clone(),
                folder: self.options.locales_folder.clone(),
            })?;

        let analyzed = Analyzed::from(&self.langbundles, default);
        for warn in &analyzed.warnings {
            println!("cargo::warning={warn}");
        }

        self.run_lints(default, &analyzed.common)?;

        let messages = &self.messages(default, &analyzed.common);
        let generated = generate(&self.options, &self.langbundles, messages)
            .map_err(BuildError::Generation)?
            .replace("    ", &self.options.indentation);

        let output_file_path = &self.options.output_file_path;
        if let Ok(current_file) = fs::read_to_string(output_file_path)
            && current_file == generated
        {
            return Ok(());
        }

        fs::write(output_file_path, &generated).map_err(|e| BuildError::WriteOutput {
            path: output_file_path.clone(),
            source: e,
        })?;

        if self.options.format {
            let status = std::process::Command::new("rustfmt")
                .arg(output_file_path)
                .status()
                .map_err(|e| BuildError::Rustfmt(e.to_string()))?;
            if !status.success() {
                return Err(BuildError::Rustfmt("rustfmt failed".to_string()));
            }
        }

        Ok(())
    }

    /// Run the comment lints and report them according to the configured
    /// [`LintLevel`]. Returns an error only in strict mode, when there are
    /// hard lint failures.
    fn run_lints(&self, default: &LangBundle, common: &HashSet<Id>) -> Result<(), BuildError> {
        let lints = lint::check(&self.langbundles, default, common);

        match self.options.lint_level {
            LintLevel::Off => {}
            LintLevel::Warn => {
                for w in lints.mistakes.iter().chain(&lints.ineffective) {
                    println!("cargo::warning={w}");
                }
            }
            LintLevel::Deny | LintLevel::Strict => {
                // Diagnostics about non-default locales stay warnings — they
                // concern translator-owned files.
                for w in &lints.ineffective {
                    println!("cargo::warning={w}");
                }
                let mut errors = lints.mistakes;
                // `Strict` additionally requires every variable to be typed.
                if self.options.lint_level == LintLevel::Strict {
                    errors.extend(lints.untyped);
                }
                if !errors.is_empty() {
                    errors.sort();
                    return Err(BuildError::Lint { messages: errors });
                }
            }
        }
        Ok(())
    }

    /// The messages to generate: the default locale's, in declaration order,
    /// restricted to the ids that survived cross-locale analysis. An id is
    /// emitted only once, even if duplicate keys were allowed and the default
    /// locale defines it more than once.
    fn messages<'a>(&self, default: &'a LangBundle, common: &HashSet<Id>) -> Vec<&'a Message> {
        let mut seen = HashSet::new();
        default
            .messages
            .iter()
            .filter(|msg| common.contains(&msg.id))
            .filter(|msg| seen.insert(&msg.id))
            .collect()
    }
}

fn from_locales_folder(
    folder: &str,
    deny_duplicate_keys: bool,
) -> Result<Vec<LangBundle>, BuildError> {
    let map_io = |e| BuildError::LocalesFolder {
        folder: folder.to_string(),
        source: e,
    };
    let locales_dir = fs::read_dir(folder).map_err(map_io)?;
    let mut locales = Vec::new();
    for entry in locales_dir {
        let entry = entry.map_err(map_io)?;
        let path = entry.path();
        if path.is_dir() {
            let lang = path.file_name().unwrap().to_str().unwrap();
            locales.push(LangBundle::from_folder(&path, lang, deny_duplicate_keys)?);
        }
    }
    if locales.is_empty() {
        return Err(BuildError::NoLocaleFolders {
            folder: folder.to_string(),
        });
    }
    Ok(locales)
}
