pub trait StrExt {
    fn rust_static_name(&self) -> String;
    fn rust_var_name(&self) -> String;
    fn rust_id(&self) -> String;
    fn with_semicolon(&self) -> String;
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

    fn with_semicolon(&self) -> String {
        format!("{self};")
    }
}
