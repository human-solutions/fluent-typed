//! Regression setup for issue #37: a workspace member crate whose locales and
//! generated ftl live at the *workspace* root, so the generated
//! `include_bytes!` path has to climb directories (`../../gen/...`). On
//! Windows this used to be emitted with `\` separators, which are invalid
//! escape sequences inside the string literal.
use fluent_typed::{BuildOptions, FtlOutputOptions, try_build_from_locales_folder};
use std::process::ExitCode;

fn main() -> ExitCode {
    let opts = BuildOptions::default()
        .with_locales_folder("../localization")
        .with_ftl_output(FtlOutputOptions::single_file("../gen/translations.ftl"))
        .with_output_file_path("src/l10n.rs");
    match try_build_from_locales_folder(opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Build failed: {e}");
            ExitCode::FAILURE
        }
    }
}
