mod ext;
mod generated_ftl;
mod message;
#[allow(dead_code, unused_mut, unused_imports)]
mod template;

use super::{BuildOptions, LangBundle, Message};
pub use ext::StrExt;
pub use generated_ftl::GeneratedFtl;

pub fn generate<'a>(
    options: &BuildOptions,
    locales: &[LangBundle],
    messages: &[&'a Message],
) -> Result<String, String> {
    let generated_ftl = options.ftl_output.generate(&locales)?;

    let mut langs = locales
        .iter()
        .map(|r| r.language_id.as_str())
        .collect::<Vec<_>>();
    langs.sort();

    let indent = &options.indentation;
    let mut replacements: Vec<(&str, String)> = Vec::new();

    replacements.push((
        "<<placeholder lang_data>>",
        generated_ftl.include_replacement(&options.output_file_path)?,
    ));

    // ///////////////////////////
    let enum_entries = collect(langs.iter(), |lang| {
        format!("{indent}L10n::{},", lang.rust_var_name())
    });
    let all_langs = format!(
        r#"
static ALL_LANGS: [L10n; {}] = [
    // languages as an array
{enum_entries}
];"#,
        langs.iter().count()
    );
    replacements.push(("<<placeholder all_langs>>", all_langs));

    // ///////////////////////////
    let enum_lang_ids = if cfg!(feature = "langneg") {
        collect(langs.iter(), |lang| {
            format!(
                "static {}: LanguageIdentifier = langid!(\"{lang}\");",
                lang.rust_static_name()
            )
        })
    } else {
        "".to_string()
    };
    replacements.push(("<<placeholder static enum langid>>", enum_lang_ids));

    // ///////////////////////////
    let enum_variants = collect(langs.iter(), |lang| {
        format!("{indent}{},", lang.rust_var_name())
    });
    replacements.push(("<<placeholder enum variant>>", enum_variants));

    // ///////////////////////////

    if !langs.contains(&options.default_language.as_str()) {
        return Err(format!(
            "Default language '{}' not found in locales",
            options.default_language
        ));
    }

    let default_lang = format!(
        "        Self::{}",
        &options.default_language.rust_var_name()
    );

    replacements.push(("<<placeholder default lang>>", default_lang));

    // ///////////////////////////
    let enum_from_str = collect(langs.iter(), |lang| {
        format!(
            "{}\"{lang}\" => Ok(Self::{}),",
            indent.repeat(3),
            lang.rust_var_name(),
        )
    });
    replacements.push(("<<placeholder enum from_str>>", enum_from_str));

    // ///////////////////////////

    let as_ref_langid = if cfg!(feature = "langneg") {
        let id_entries = collect(langs.iter(), |lang| {
            format!(
                "{}Self::{} => &{},",
                indent.repeat(3),
                lang.rust_var_name(),
                lang.rust_static_name(),
            )
        });
        format!(
            r#"impl AsRef<LanguageIdentifier> for L10n {{
    fn as_ref(&self) -> &LanguageIdentifier {{
        match self {{
{id_entries}
        }}
    }}
}}
"#
        )
    } else {
        String::new()
    };
    replacements.push(("<<placeholder as_ref_langid>>", as_ref_langid));

    // ///////////////////////////
    let enum_to_str = collect(langs.iter(), |lang| {
        format!(
            "{}Self::{} => \"{}\",",
            indent.repeat(3),
            lang.rust_var_name(),
            lang,
        )
    });
    replacements.push(("<<placeholder enum to_str>>", enum_to_str));

    // ///////////////////////////

    let names = locales
        .iter()
        .filter_map(|l| {
            l.language_name
                .as_ref()
                .map(|name| (l.language_id.rust_var_name(), name))
        })
        .collect::<Vec<_>>();

    let lang_name_fn = if names.len() == locales.len() {
        format!(
            r#"
    /// The language name as defined in the ftl message "language-name".
    pub fn language_name(&self) -> &'static str {{
        match self {{
{}
        }}
    }}"#,
            names
                .iter()
                .map(|(var, name)| format!(r#"            Self::{var} => "{name}","#))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        String::new()
    };

    replacements.push(("<<placeholder lang_name function>>", lang_name_fn));

    // ///////////////////////////

    let enum_id = collect(langs.iter(), |lang| {
        format!(
            "{}Self::{} => &{},",
            indent.repeat(3),
            lang.rust_var_name(),
            lang.rust_static_name(),
        )
    });
    replacements.push(("<<placeholder enum id>>", enum_id));

    // ///////////////////////////

    let langneg_fn = if cfg!(feature = "langneg") {
        format!(
            r#"
    /// Negotiate the best language to use based on the `Accept-Language` header.
    ///
    /// Falls back to the default language if none of the languages in the header are available.
    pub fn langneg(accept_language: &str) -> L10n {{
        negotiate_languages(accept_language, &ALL_LANGS)
    }}"#,
        )
    } else {
        String::new()
    };

    replacements.push(("<<placeholder langneg function>>", langneg_fn));

    // ///////////////////////////
    replacements.push((
        "<<placeholder load functions>>",
        generated_ftl.accessor_replacement(),
    ));

    // ///////////////////////////
    let impls = collect(messages.iter(), |msg| {
        msg.implementations(
            &options.prefix,
            options.output_mode,
            &options.pattern_prefix,
        )
    });
    replacements.push(("<<message implementations>>", impls));

    let mut base = do_replace(include_str!("template.rs"), replacements);
    base.push('\n');

    #[cfg(not(test))]
    let base = base.replace("use crate::prelude::*;", "use fluent_typed::prelude::*;");
    Ok(base)
}

fn collect<T, F: Fn(T) -> String>(vals: impl Iterator<Item = T>, f: F) -> String {
    vals.map(f).collect::<Vec<_>>().join("\n")
}

fn do_replace(base: &str, replacements: Vec<(&str, String)>) -> String {
    base.lines()
        .filter_map(|line| {
            if !line.contains("<<") {
                return Some(line.to_string());
            }
            for (placeholder, replacement) in &replacements {
                if line.contains(placeholder) {
                    return if replacement.is_empty() {
                        eprintln!("Empty replacement for placeholder: {}", placeholder);
                        None
                    } else {
                        Some(replacement.to_string())
                    };
                }
            }
            panic!("Unknown placeholder in template: {}", line);
        })
        .collect::<Vec<_>>()
        .join("\n")
}
