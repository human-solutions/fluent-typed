mod builder;
mod error;
pub mod r#gen;
mod lang_bundle;
pub mod lint;
pub mod options;
pub mod typed;
mod utils;
mod validations;

pub use error::BuildError;
pub use options::{BuildOptions, FtlOutputOptions, LintLevel};
use std::process::ExitCode;

// Crate-internal: the `build` module itself is private, so these are not part
// of the public API. They are re-exported here only for use within the crate.
pub(crate) use builder::Builder;
pub(crate) use lang_bundle::LangBundle;
pub(crate) use typed::Message;
pub(crate) use validations::Analyzed;

/// Internal build-pipeline pieces re-exported for the benchmark suite
/// (`benches/build_pipeline.rs`).
///
/// Gated behind the non-default `bench-internals` feature. **Not** part of the
/// stable public API: these items may change or be removed without a semver
/// bump. Application code must not depend on this module.
#[cfg(feature = "bench-internals")]
pub mod bench_internals {
    pub use super::lang_bundle::LangBundle;
    pub use super::lint::{Lints, check};
    pub use super::r#gen::generate;
    pub use super::typed::{Id, Message};
    pub use super::validations::Analyzed;
}

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
pub fn try_build_from_locales_folder(options: BuildOptions) -> Result<(), BuildError> {
    let locales = &options.locales_folder;
    println!("cargo::rerun-if-changed={locales}");

    Builder::load(options)?.generate()
}
