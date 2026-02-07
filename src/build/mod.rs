mod builder;
pub mod r#gen;
mod lang_bundle;
pub mod options;
pub mod typed;
mod utils;
mod validations;

pub use builder::Builder;
pub use lang_bundle::LangBundle;
pub use options::{BuildOptions, FtlOutputOptions, OutputMode};
use std::process::ExitCode;
pub use typed::Message;
pub use validations::Analyzed;

/// Generate rust code and ftl files from locales folder, which contains `<lang-id>/<resource-name>.ftl` files.
///
/// The generation should be done in a build script:
///
/// ```no_run
/// // in build.rs
/// fn main() -> std::process::ExitCode {
///    let options = fluent_typed::BuildOptions::default();
///    fluent_typed::build_from_locales_folder(options)
/// }
/// ```
///
/// This requires the dependencies:
///
/// ```toml
/// # in Cargo.toml
/// [dependencies]
/// fluent-typed = 0.1
///
/// [build-dependencies]
/// fluent-typed = { version = "0.1", features = ["build"] }
/// ```
/// During the generation, the build script will print warnings for all messages that are
/// not present in all locales, as well as for messages with different signatures.
///
/// It is recommended to generate the rust code to the output_file_path "src/l10n.rs" and include
/// it in the project, so that you get warnings for unused translation messages.
///
/// See [BuildOptions] for more configuration options.
///
pub fn build_from_locales_folder(options: BuildOptions) -> ExitCode {
    match try_build_from_locales_folder(options) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

/// Same as [build_from_locales_folder], but returns result instead of an ExitCode.
pub fn try_build_from_locales_folder(options: BuildOptions) -> Result<(), String> {
    let locales = &options.locales_folder;
    println!("cargo::rerun-if-changed={locales}");

    Builder::load(options)?.generate()
}
