use crate::build::r#gen::StrExt;
use crate::build::options::OutputMode;
use crate::build::typed::{Message, VarType, Variable};

impl Message {
    pub fn trait_signature(&self) -> String {
        let mut out = Vec::new();
        let func_name = self.id.func_name();
        out.push(self.comment_lines());
        out.push(self.signature(&self.variables, &func_name).with_semicolon());

        out.join("\n")
    }

    fn signature(&self, variables: &[Variable], func_name: &str) -> String {
        if variables.is_empty() {
            format!(r"    pub fn {func_name}(&self) -> String")
        } else {
            let ArgInfo { generic, arg } = args_declaration(variables);
            let lt = lifetime(variables);
            format!(r"    pub fn {func_name}<{lt}{generic}>(&self, {arg}) -> String")
        }
    }

    pub fn implementations(
        &self,
        prefix: &str,
        output_mode: OutputMode,
        pattern_prefix: &str,
    ) -> String {
        let func_name = self.id.func_name();
        let mut out = String::new();

        if output_mode != OutputMode::Pattern {
            let signature = self.signature(&self.variables, &format!("{prefix}{func_name}"));
            if func_name == "language_name" {
                out.push_str("    #[allow(unused)]\n");
            }
            out.push_str(&self.comment_lines());
            let implementation = if let Some(attr) = self.id.attribute.as_ref() {
                self.attr_impl(&self.variables, &self.id.message, attr, &signature)
            } else {
                self.func_impl(&self.variables, &self.id.message, &signature)
            };
            out.push_str(&implementation);
        }

        if output_mode != OutputMode::String {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&self.comment_lines());
            let ptn_signature =
                format!("    pub fn {pattern_prefix}{func_name}(&self) -> Pattern<String>");
            let ptn_impl = if let Some(attr) = self.id.attribute.as_ref() {
                format!(
                    r##"{ptn_signature} {{
        self.0.attr_pattern("{msg_id}", "{attr}")
    }}"##,
                    msg_id = self.id.message,
                )
            } else {
                format!(
                    r##"{ptn_signature} {{
        self.0.msg_pattern("{msg_id}")
    }}"##,
                    msg_id = self.id.message,
                )
            };
            out.push_str(&ptn_impl);
        }

        out
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
        self.0.attr("{msg_id}", "{attr_id}", None).unwrap()
    }}"##,
            )
        } else {
            let args = args_impl(variables);

            format!(
                r##"{signature} {{
        let mut args = FluentArgs::new();
{args}
        self.0.attr("{msg_id}", "{attr_id}", Some(args)).unwrap()
    }}"##,
            )
        }
    }
    fn func_impl(&self, variables: &[Variable], id: &str, signature: &str) -> String {
        if variables.is_empty() {
            format!(
                r##"{signature} {{
        self.0.msg("{id}", None).unwrap()
    }}"##,
            )
        } else {
            let args = args_impl(variables);

            format!(
                r##"{signature} {{
        let mut args = FluentArgs::new();
{args}
        self.0.msg("{id}", Some(args)).unwrap()
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

fn lifetime(vars: &[Variable]) -> &'static str {
    vars.iter()
        .any(|v| v.typ == VarType::Any)
        .then_some("'a, ")
        .unwrap_or_default()
}

fn args_declaration(vars: &[Variable]) -> ArgInfo {
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

fn args_impl(vars: &[Variable]) -> String {
    let mut impls = vec![];

    for var in vars {
        let name = var.id.as_str();
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
    fn new(num: usize, var: &Variable) -> Option<Self> {
        let generic = match var.typ {
            VarType::Any => format!("F{num}: Into<FluentValue<'a>>"),
            VarType::String => format!("F{num}: AsRef<str>"),
            VarType::Number => format!("F{num}: Into<FluentNumber>"),
        };
        let arg = format!("{}: F{num}", var.id.rust_id());
        Some(Self { generic, arg })
    }
}
