mod error;
#[cfg(test)]
mod tests;
mod typed;

use fluent_syntax::parser;
use std::fs;
use std::process::ExitCode;

pub use error::MessageError;
pub use typed::generate_code;
pub use typed::BundleMessageExt;
pub use typed::Message;

pub fn build_generate(ftl: &str, rust: &str) -> ExitCode {
    match _generate(ftl, rust) {
        Ok(_) => {
            println!("cargo::rerun-if-changed={ftl}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn _generate(ftl: &str, rust: &str) -> Result<(), String> {
    let ftl_file = fs::read_to_string(ftl)
        .map_err(|e| format!("Could not read fluent ftl file '{ftl}': {e}"))?;
    let resource = parser::parse(ftl_file.as_str())
        .map_err(|e| format!("Could not parse ftl file '{ftl}' due to: {e:?}"))?;

    let code = generate_code(resource);
    fs::write(rust, code).map_err(|e| format!("Could not write rust file '{rust}': {e}"))
}
