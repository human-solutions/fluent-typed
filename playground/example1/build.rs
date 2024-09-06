fn main() -> std::process::ExitCode {
    fluent_typed::build_from_locales_folder(fluent_typed::BuildOptions::default())
}
