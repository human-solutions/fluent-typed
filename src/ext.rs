pub trait StrExt {
    fn rust_id(&self) -> String;
    fn with_semicolon(&self) -> String;
}

impl StrExt for str {
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

    fn with_semicolon(&self) -> String {
        format!("{self};")
    }
}
