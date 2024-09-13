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
        .map(|r| r.language.as_str())
        .collect::<Vec<_>>();
    langs.sort();

    let indent = &options.indentation;
    let mut replacements: Vec<(&str, String)> = Vec::new();

    replacements.push((
        "<<placeholder lang_data>>",
        generated_ftl.include_replacement(&options.output_file_path)?,
    ));

    let enum_lang_ids = collect(langs.iter(), |lang| {
        format!(
            "static {}: LanguageIdentifier = langid!(\"{lang}\");",
            lang.rust_static_name()
        )
    });
    replacements.push(("<<placeholder static enum langid>>", enum_lang_ids));

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

    let enum_variants = collect(langs.iter(), |lang| {
        format!("{indent}{},", lang.rust_var_name())
    });
    replacements.push(("<<placeholder enum variant>>", enum_variants));

    let enum_from_str = collect(langs.iter(), |lang| {
        format!(
            "{}\"{lang}\" => Ok(Self::{}),",
            indent.repeat(3),
            lang.rust_var_name(),
        )
    });
    replacements.push(("<<placeholder enum from_str>>", enum_from_str));

    let enum_to_str = collect(langs.iter(), |lang| {
        format!(
            "{}Self::{} => \"{}\",",
            indent.repeat(3),
            lang.rust_var_name(),
            lang,
        )
    });
    replacements.push(("<<placeholder enum to_str>>", enum_to_str));

    let enum_id = collect(langs.iter(), |lang| {
        format!(
            "{}Self::{} => &{},",
            indent.repeat(3),
            lang.rust_var_name(),
            lang.rust_static_name(),
        )
    });
    replacements.push(("<<placeholder enum id>>", enum_id));

    replacements.push((
        "<<placeholder load functions>>",
        generated_ftl.accessor_replacement(),
    ));

    let impls = collect(messages.iter(), |msg| msg.implementations(&options.prefix));
    replacements.push(("<<message implementations>>", impls));

    let base = include_str!("template.rs");

    let mut base = base
        .lines()
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
        .join("\n");
    base.push('\n');

    #[cfg(not(test))]
    let base = base.replace("use crate::prelude::*;", "use fluent_typed::prelude::*;");
    Ok(base)
}

fn collect<T, F: Fn(T) -> String>(vals: impl Iterator<Item = T>, f: F) -> String {
    vals.map(f).collect::<Vec<_>>().join("\n")
}
