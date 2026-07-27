mod l10n;

pub use l10n::L10n;

#[cfg(test)]
mod tests {
    use super::L10n;

    #[test]
    fn embedded_translations_load() {
        let en = L10n::En.load();
        assert_eq!(en.msg_hello_world(), "Hello, world!");
    }
}
