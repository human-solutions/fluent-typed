use super::super::{Message, VarType, Variable};

trait StrExt {
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

impl<'ast> Message<'ast> {
    pub fn trait_signature(&self) -> String {
        let mut out = Vec::new();
        let func_name = self.id.rust_id();
        if let Some(variables) = &self.variables {
            out.push(self.comment_lines());
            out.push(self.signature(variables, &func_name).with_semicolon());
        }

        for attr in &self.attributes {
            let attr_name = format!("{func_name}_{}", attr.id.rust_id());
            out.push(self.signature(&attr.variables, &attr_name).with_semicolon());
        }

        out.join("\n")
    }

    fn signature(&self, variables: &[Variable], func_name: &str) -> String {
        if variables.is_empty() {
            format!(r"    fn {func_name}(&self) -> Cow<'_, str>")
        } else {
            let ArgInfo { generic, arg } = args_declaration(variables);
            let lt = lifetime(variables);
            format!(r"    fn {func_name}<{lt}{generic}>(&self, {arg}) -> Cow<'_, str>")
        }
    }

    pub fn implementations(&self) -> String {
        let mut impls = vec![];

        if let Some(variables) = &self.variables {
            let signature = self.signature(variables, &self.id.rust_id());
            impls.push(self.func_impl(variables, &self.id, &signature))
        }

        for attr in &self.attributes {
            let func_name = format!("{}_{}", self.id.rust_id(), attr.id.rust_id());
            let signature = self.signature(&attr.variables, &func_name);
            impls.push(self.attr_impl(&attr.variables, &self.id, &attr.id, &signature));
        }

        impls.join("\n")
    }

    fn attr_impl(
        &self,
        variables: &[Variable],
        msg_id: &str,
        attr_id: &str,
        signature: &str,
    ) -> String {
        if variables.is_empty() {
            format!(
                r##"{signature} {{
        self.attr("{msg_id}", "{attr_id}", None).unwrap()
    }}"##,
            )
        } else {
            let args = args_impl(variables);

            format!(
                r##"{signature} {{
        let mut args = FluentArgs::new();
{args}
        self.attr("{msg_id}", "{attr_id}", Some(args)).unwrap()
    }}"##,
            )
        }
    }
    fn func_impl(&self, variables: &[Variable], id: &str, signature: &str) -> String {
        if variables.is_empty() {
            format!(
                r##"{signature} {{
        self.msg("{id}", None).unwrap()
    }}"##,
            )
        } else {
            let args = args_impl(variables);

            format!(
                r##"{signature} {{
        let mut args = FluentArgs::new();
{args}
        self.msg("{id}", Some(args)).unwrap()
    }}"##,
            )
        }
    }

    fn comment_lines(&self) -> String {
        self.comment
            .iter()
            .map(|c| format!("    /// {c}\n"))
            .collect::<Vec<_>>()
            .join("")
    }
}

fn lifetime(vars: &[Variable<'_>]) -> &'static str {
    vars.iter()
        .any(|v| v.typ == VarType::Any)
        .then_some("'a, ")
        .unwrap_or_default()
}

fn args_declaration(vars: &[Variable<'_>]) -> ArgInfo {
    let mut generics = vec![];
    let mut args = vec![];

    for (num, var) in vars.iter().enumerate() {
        let Some(ArgInfo { generic, arg }) = ArgInfo::new(num, var) else {
            continue;
        };
        generics.push(generic);
        args.push(arg);
    }
    if args.is_empty() {
        return ArgInfo::default();
    }

    ArgInfo {
        generic: generics.join(", "),
        arg: args.join(", "),
    }
}

fn args_impl(vars: &[Variable<'_>]) -> String {
    let mut impls = vec![];

    for var in vars {
        let name = var.id;
        let id = var.id.rust_id();

        let impl_ = match var.typ {
            VarType::Any => format!(r#"        args.set("{name}", {id});"#),
            VarType::String => format!(r#"        args.set("{name}", {id}.as_ref());"#),
            VarType::Number => format!(r#"        args.set("{name}", {id}.into());"#),
        };
        impls.push(impl_);
    }
    impls.join("\n")
}

#[derive(Default)]
struct ArgInfo {
    generic: String,
    arg: String,
}

impl ArgInfo {
    fn new(num: usize, var: &Variable<'_>) -> Option<Self> {
        let generic = match var.typ {
            VarType::Any => format!("F{num}: Into<FluentValue<'a>>"),
            VarType::String => format!("F{num}: AsRef<str>"),
            VarType::Number => format!("F{num}: Into<FluentNumber>"),
        };
        let arg = format!("{}: F{num}", var.id.rust_id());
        Some(Self { generic, arg })
    }
}
