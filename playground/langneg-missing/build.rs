use fluent_typed::{BuildOptions, build_from_locales_folder};

fn main() -> std::process::ExitCode {
    build_from_locales_folder(BuildOptions::default().with_langneg())
}
