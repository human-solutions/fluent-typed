//! Compiles and runs the documentation examples: `cargo test`. See build.rs.
#![allow(dead_code)]
#![deny(warnings)] // the generated code must not warn

mod l10n;
mod l10n_gz;
mod l10n_multi;

mod blocks {
    include!(concat!(env!("OUT_DIR"), "/blocks.rs"));
}

/// Regression: the compressed `load` used to load every language into one
/// bundle (all but single-language projects failed with `Overriding` errors).
#[test]
fn compressed_load_picks_its_language() {
    use std::io::Read;
    let gunzip = |data: &[u8]| -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(data)
            .read_to_end(&mut out)
            .map_err(|e| e.to_string())?;
        Ok(out)
    };
    assert_eq!(l10n_gz::L10n::Fr.load(gunzip).unwrap().msg_greeting(), "Bienvenue !");
    assert_eq!(l10n_gz::L10n::En.load(gunzip).unwrap().msg_greeting(), "Welcome!");
    let all = l10n_gz::L10n::load_all(gunzip).unwrap();
    assert_eq!(all.get(l10n_gz::L10n::Fr).msg("greeting", None).unwrap(), "Bienvenue !");
}
