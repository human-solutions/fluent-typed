use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::build::{LangBundle, r#gen::GeneratedFtl};

type CompressorFn = dyn Fn(Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>>;

/// The ftl output options for the build command. This allows you to
/// configure how the output ftl files are generated, and also what
/// type of access code is generated.
///
/// Defaults to a SingleFileOptions with the output_ftl_folder set to "gen"
/// and gzip set to true.
pub enum FtlOutputOptions {
    /// Generates FTL files as one file per language which means
    /// that individual resource files are appended into one file.
    ///
    /// This is especially useful client-side when you
    /// don't want to embed the files in the binary or
    /// download them all together.
    MultiFile {
        /// The path to the where the output ftl files will be written.
        /// For convenience fluent-typed joins all ftl resources for each language
        /// into a single file.
        ///
        /// Defaults to "gen/" in the root of the package.
        output_ftl_folder: String,
    },

    /// Generates FTL files as one file for all languages, which means
    /// that individual resource files are appended into one file.
    ///
    /// This is preferred for when you embed the files in your binary
    /// which typically is done server-side and also client-side when
    /// the files are small enough to either embed in the binary or
    /// download in a html request.
    SingleFile {
        /// The path to the where the output ftl file will be written.
        /// For convenience fluent-typed joins all ftl resources for each language
        /// into a single file.
        ///
        /// Defaults to "gen/translations.ftl" in the root of the package.
        output_ftl_file: String,
        /// The compresssor is an closure that takes the ftl file content
        /// as a byte array, compresses it and returns the compressed bytes.
        ///
        /// Any compression algorithm can be used, but it's up to the user
        /// to import the necessary crate and do the compression and when
        /// using it decompress in the same manner.
        compressor: Option<Box<CompressorFn>>,
    },
}

impl Default for FtlOutputOptions {
    fn default() -> Self {
        Self::SingleFile {
            output_ftl_file: "gen/translations.ftl".to_string(),
            compressor: None,
        }
    }
}

impl FtlOutputOptions {
    pub fn single_file(file: &str) -> Self {
        Self::SingleFile {
            output_ftl_file: file.to_string(),
            compressor: None,
        }
    }

    pub fn single_compressed_file<F>(file: &str, compressor: F) -> Self
    where
        F: Fn(Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> + 'static,
    {
        Self::SingleFile {
            output_ftl_file: file.to_string(),
            compressor: Some(Box::new(compressor)),
        }
    }

    pub fn multi_file(folder: &str) -> Self {
        Self::MultiFile {
            output_ftl_folder: folder.to_string(),
        }
    }

    pub fn generate(&self, locales: &[LangBundle]) -> Result<GeneratedFtl, String> {
        match self {
            Self::MultiFile { output_ftl_folder } => {
                let dir = PathBuf::from(output_ftl_folder);
                create_dir(&dir)?;
                for lang in locales {
                    let mut file = dir.join(&lang.language_id);
                    file.set_extension("ftl");
                    write(lang.ftl.as_bytes(), &file)?;
                }
                Ok(GeneratedFtl::MultiFile)
            }
            Self::SingleFile {
                output_ftl_file,
                compressor,
            } => {
                let file = PathBuf::from(&output_ftl_file);
                if let Some(folder) = file.parent() {
                    create_dir(folder)?;
                }
                let mut content = Vec::new();
                let mut pos = 0;
                let mut positions = Vec::new();

                for locale in locales {
                    let bytes = locale.ftl.bytes();
                    content.extend(bytes);
                    let lang = locale.language_id.clone();
                    positions.push((lang, pos..content.len()));
                    pos = content.len();
                }
                if let Some(compressor) = compressor {
                    let compressed = compressor(content)
                        .map_err(|e| format!("Could not compress ftl file: {e}"))?;
                    write(&compressed, &file)?;
                } else {
                    write(&content, &file)?;
                }
                Ok(GeneratedFtl::SingleFile {
                    output_ftl_file: output_ftl_file.clone(),
                    positions,
                    compressed: compressor.is_some(),
                })
            }
        }
    }
}

fn write(content: &[u8], file: &Path) -> Result<(), String> {
    fs::write(file, content).map_err(|e| format!("Could not write ftl file '{file:?}': {e:?}"))
}

fn create_dir(folder: &Path) -> Result<(), String> {
    if !folder.exists() {
        fs::create_dir_all(&folder)
            .map_err(|e| format!("Could not create ftl folder '{folder:?}': {e:?}"))?;
    }
    Ok(())
}
