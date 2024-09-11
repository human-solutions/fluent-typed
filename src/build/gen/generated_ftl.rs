use std::ops::Range;

use super::StrExt;

pub enum GeneratedFtl {
    SingleFile {
        output_ftl_file: String,
        positions: Vec<(String, Range<usize>)>,
        compressed: bool,
    },
    MultiFile,
}

impl GeneratedFtl {
    pub fn include_replacement(&self) -> String {
        match self {
            Self::SingleFile {
                output_ftl_file, ..
            } => {
                format!(
                    "static LANG_DATA: &'static [u8] = include_bytes!(\"{}\"); ",
                    output_ftl_file
                )
            }
            Self::MultiFile => "".to_string(),
        }
    }

    pub fn accessor_replacement(&self) -> String {
        match self {
            Self::SingleFile {
                positions,
                compressed,
                ..
            } => self.single_file_load_fn(positions, *compressed),
            Self::MultiFile => "".to_string(),
        }
    }

    fn single_file_load_fn(
        &self,
        positions: &[(String, Range<usize>)],
        compressed: bool,
    ) -> String {
        let mut out = String::new();

        let signature = if compressed {
            format!(
                r#"    pub fn load<D>(&self, decompressor: D) -> Result<L10n, String>
            where
                D: Fn(&[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>,
            {{
                "#
            )
        } else {
            format!("    pub fn load(&self) -> Result<L10n, String> {{\n")
        };
        out.push_str(&signature);
        let range_switch = range_switch(positions);
        out.push_str(&range_switch);

        let bytes = if compressed {
            "        let bytes = decompressor(LANG_DATA[lang_range])?;\n"
        } else {
            "        let bytes = LANG_DATA[lang_range].to_vec();\n"
        };
        out.push_str(bytes);

        out.push_str(
            r#"
        Ok(L10n::load(&String::from_utf8_lossy(&bytes))?)
    }

    pub fn load_all() -> Result<HashMap<L10Lang, L10n>, String> {
        let mut map = HashMap::new();
        for lang in Self::as_arr() {
            map.insert(lang.clone(), lang.load()?);
        }
        Ok(map)
    }
    "#,
        );
        out
    }
}

fn range_switch(positions: &[(String, Range<usize>)]) -> String {
    let range_statements = positions
        .iter()
        .map(|(name, range)| {
            format!(
                "            {} => {}..{},\n",
                name.rust_var_name(),
                range.start,
                range.end
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"        let lang_range = match self {{
{range_statements}
        }};
"#
    )
}

// /// Load the l10n resources for the given language. The ftl files
// /// are embedded in the binary.
// pub fn load(&self) -> Result<L10n, String> {
//     todo!("Load the L10n resources for the given language.");
// }

// /// Load the l10n resources for the given language. The ftl files
// /// are embedded in the binary.
// pub fn load2<Decompr>(&self, _decompressor: Decompr) -> Result<L10n, String>
// where
//     Decompr: Fn(Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>,
// {
//     let _lang_range = match self {
//         Self::Placeholder => 0..0,
//     };
//     todo!("Load the L10n resources for the given language.");
// }
// /// Load the l10n resources for the given language by providing
// /// the FTL string yourself. This is useful when you need to load
// /// the resource from a server.
// pub fn load_with_ftl(&self, _ftl: String) -> Result<L10n, String> {
//     todo!("Load the L10n resources for the given language.");
// }

// pub fn load_all() -> Result<HashMap<L10Lang, L10n>, String> {
//     let mut map = HashMap::new();
//     for lang in Self::as_arr() {
//         map.insert(lang.clone(), lang.load()?);
//     }
//     Ok(map)
// }
