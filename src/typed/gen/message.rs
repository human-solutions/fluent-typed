use super::super::{Message, VarType, Variable};

trait StrExt {
    fn rust_id(&self) -> String;
}

impl StrExt for str {
    fn rust_id(&self) -> String {
        self.replace('-', "_")
    }
}

impl<'ast> Message<'ast> {
    pub fn gen_signature(&self, variables: &[Variable]) -> String {
        let sig = if variables.is_empty() {
            self.signature_no_args()
        } else {
            self.signature(variables)
        };

        let comment = self.comment_lines();

        format!("{comment}{sig};")
    }

    fn signature(&self, variables: &[Variable]) -> String {
        let func = self.id.rust_id();
        let ArgInfo { generic, arg } = args_declaration(variables);
        let lt = lifetime(variables);

        format!(r"    fn {func}<{lt}{generic}>(&self, {arg}) -> Cow<'_, str>")
    }

    fn signature_no_args(&self) -> String {
        let comment = self.comment_lines();
        let func = self.id.rust_id();
        format!(r"{comment}    fn {func}(&self) -> Cow<'_, str>")
    }

    pub fn gen_implementation(&self) -> String {
        if let Some(variables) = &self.variables {
            if variables.is_empty() {
                self.gen_impl()
            } else {
                self.gen_impl_args(variables)
            }
        } else {
            String::new()
        }
    }

    fn gen_impl_args(&self, variables: &[Variable]) -> String {
        let signature = self.signature(variables);
        let args = args_impl(variables);
        let id = self.id;

        format!(
            r##"{signature} {{
        let mut args = FluentArgs::new();
{args}
        self.msg_with_args("{id}", args).unwrap()
    }}"##,
        )
    }

    fn gen_impl(&self) -> String {
        let fn_signature = self.signature_no_args();
        let id = self.id;

        format!(
            r##"{fn_signature} {{
        self.msg("{id}").unwrap()
    }}"##,
        )
    }

    fn comment_lines(&self) -> String {
        if self.comment.is_empty() {
            return String::new();
        }

        let comment = self
            .comment
            .iter()
            .map(|c| format!("    /// {c}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{comment}\n")
    }
}

fn lifetime(vars: &[Variable<'_>]) -> &'static str {
    let has_lifetime = vars.iter().any(|v| v.typ == VarType::Any);
    if has_lifetime {
        "'a, "
    } else {
        ""
    }
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
