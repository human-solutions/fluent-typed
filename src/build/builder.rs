use super::{Analyzed, BuildOptions, LangBundle, Message, r#gen::generate, typed::Id};
use std::{collections::HashSet, fs};

pub struct Builder {
    options: BuildOptions,
    langbundles: Vec<LangBundle>,
}

impl Builder {
    pub fn load(options: BuildOptions) -> Result<Self, String> {
        let folder = &options.locales_folder;
        println!("cargo::rerun-if-changed={folder}");

        let mut langbundles = from_locales_folder(folder, options.deny_duplicate_keys)
            .map_err(|e| format!("Could not read locales folder '{folder}': {e:?}"))?;

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
    ) -> Result<Self, String> {
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

    pub fn generate(&self) -> Result<(), String> {
        let analyzed = Analyzed::from(&self.langbundles);

        for warn in analyzed.missing_messages {
            println!("cargo::warning={warn}");
        }
        for warn in analyzed.signature_mismatches {
            println!("cargo::warning={warn}");
        }

        let messages = &self.messages(&analyzed.common);
        let generated = generate(&self.options, &self.langbundles, messages)?
            .replace("    ", &self.options.indentation);

        let output_file_path = &self.options.output_file_path;
        if let Some(current_file) = fs::read_to_string(output_file_path).ok() {
            if current_file == generated {
                return Ok(());
            }
        }

        fs::write(output_file_path, generated)
            .map_err(|e| format!("Could not write rust file '{output_file_path}': {e:?}"))?;

        if self.options.format {
            let status = std::process::Command::new("rustfmt")
                .arg(output_file_path)
                .status()
                .map_err(|e| format!("Could not run rustfmt: {e:?}"))?;
            if !status.success() {
                return Err("rustfmt failed".to_string());
            }
        }

        Ok(())
    }

    fn messages(&self, common: &HashSet<Id>) -> Vec<&Message> {
        let mut added = HashSet::new();
        self.langbundles
            .iter()
            .flat_map(|r| &r.messages)
            .filter(|msg| common.contains(&msg.id))
            .filter(|msg| added.insert(&msg.id))
            .collect::<Vec<_>>()
    }
}

fn from_locales_folder(folder: &str, deny_duplicate_keys: bool) -> Result<Vec<LangBundle>, String> {
    let locales_dir = fs::read_dir(folder).map_err(|e| e.to_string())?;
    let mut locales = Vec::new();
    for entry in locales_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            let lang = path.file_name().unwrap().to_str().unwrap();
            locales.push(LangBundle::from_folder(&path, lang, deny_duplicate_keys)?);
        }
    }
    Ok(locales)
}
