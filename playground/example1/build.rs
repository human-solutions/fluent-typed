use flate2::{write::GzEncoder, Compression};
use fluent_typed::{BuildError, try_build_from_locales_folder, BuildOptions, FtlOutputOptions};
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Build failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<(), BuildError> {
    let multi_opts = BuildOptions::default()
        .with_ftl_output(FtlOutputOptions::MultiFile {
            output_ftl_folder: "gen/multi/".to_string(),
        })
        .with_output_file_path("src/multi_l10n.rs");
    try_build_from_locales_folder(multi_opts)?;

    let single_opts = BuildOptions::default()
        .with_ftl_output(FtlOutputOptions::single_file("gen/translations.ftl"))
        .with_output_file_path("src/single_l10n.rs");
    try_build_from_locales_folder(single_opts)?;

    let ftl_opts = FtlOutputOptions::single_compressed_file("gen/translations.ftl.gzip", |buf| {
        let mut output = Vec::new();
        let mut encoder = GzEncoder::new(&mut output, Compression::best());
        encoder
            .write_all(&buf)
            .map_err(|e| format!("Gz encoding failed: {e}"))?;
        drop(encoder);
        Ok(output)
    });
    let single_gzip_opts = BuildOptions::default()
        .with_ftl_output(ftl_opts)
        .with_output_file_path("src/single_gzip_l10n.rs");
    try_build_from_locales_folder(single_gzip_opts)
}
