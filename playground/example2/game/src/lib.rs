mod l10n;

pub use l10n::L10n;

#[cfg(test)]
mod tests {
    use super::L10n;
    use super::l10n::L10nLanguage;

    #[test]
    fn embedded_translations_load() {
        let en = L10n::En.load();
        assert_eq!(en.msg_hello_world(), "Hello, world!");
    }

    /// Issue #38: an external `.ftl` — here a language that was never part of
    /// the build — loads at runtime, validated against the compiled contract.
    /// The validation is strict: every generated message must be defined
    /// (`language-name` included), so every accessor is safe to call.
    #[test]
    fn external_translation_loads_at_runtime() {
        let ftl = "language-name = Polski\nhello-world = Witaj świecie!\n";
        let pl = L10nLanguage::new_external("pl", ftl.as_bytes()).unwrap();
        assert_eq!(pl.msg_hello_world(), "Witaj świecie!");
    }

    /// A broken external translation (misspelled message id) is rejected at
    /// load time instead of panicking later in the accessor.
    #[test]
    fn broken_external_translation_is_rejected() {
        let err = match L10nLanguage::new_external("pl", "helo-world = oops".as_bytes()) {
            Err(err) => err,
            Ok(_) => panic!("a translation missing 'hello-world' must be rejected"),
        };
        assert!(err.to_string().contains("'hello-world' is missing"), "{err}");
    }
}
