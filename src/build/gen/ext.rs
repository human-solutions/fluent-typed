/// Every Rust keyword, strict and reserved, in any edition, sorted for binary
/// search. A generated name equal to one of these doesn't parse as an
/// identifier.
const KEYWORDS: &[&str] = &[
    "Self", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
    "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if",
    "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// `name` as an identifier that compiles anywhere: a keyword becomes a raw
/// identifier (`type` → `r#type`), except the four that can't be raw, which
/// get a trailing `_` (`self` → `self_`).
pub fn escape_ident(name: String) -> String {
    if KEYWORDS.binary_search(&name.as_str()).is_err() {
        name
    } else if matches!(name.as_str(), "self" | "Self" | "super" | "crate") {
        name + "_"
    } else {
        format!("r#{name}")
    }
}

pub trait StrExt {
    /// Uppercased snake case name.
    fn rust_static_name(&self) -> String;
    fn rust_var_name(&self) -> String;
    fn rust_id(&self) -> String;
}

impl StrExt for str {
    fn rust_static_name(&self) -> String {
        self.chars()
            .map(|c| {
                if c == '-' {
                    '_'
                } else {
                    c.to_ascii_uppercase()
                }
            })
            .collect()
    }

    fn rust_var_name(&self) -> String {
        let mut s = String::with_capacity(self.len());
        let mut next_uppercased = true;
        for c in self.chars() {
            if next_uppercased {
                s.push(c.to_ascii_uppercase());
                next_uppercased = false;
            } else if c == '_' || c == '-' {
                next_uppercased = true;
            } else {
                s.push(c);
            }
        }
        s
    }

    fn rust_id(&self) -> String {
        let mut s = String::with_capacity(self.len());
        for (i, c) in self.chars().enumerate() {
            if c == '-' {
                s.push('_');
            } else if c.is_ascii_uppercase() {
                if i != 0 {
                    s.push('_');
                }
                s.push(c.to_ascii_lowercase());
            } else {
                s.push(c)
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::escape_ident;

    #[test]
    fn keywords_are_sorted_for_binary_search() {
        assert!(super::KEYWORDS.is_sorted());
    }

    #[test]
    fn keywords_are_escaped() {
        assert_eq!(escape_ident("name".into()), "name");
        assert_eq!(escape_ident("type".into()), "r#type");
        assert_eq!(escape_ident("gen".into()), "r#gen");
        assert_eq!(escape_ident("self".into()), "self_");
        assert_eq!(escape_ident("Self".into()), "Self_");
    }
}
