use crate::build::r#gen::StrExt;
use crate::build::options::OutputMode;
use crate::build::typed::{ElementKind, ElementMarker, Message, VarType, Variable};

impl Message {
    pub fn trait_signature(&self) -> String {
        let mut out = Vec::new();
        let func_name = self.id.func_name();
        out.push(self.comment_lines());
        let mut sig = self.signature(&self.variables, &func_name);
        if !self.elements.is_empty() {
            sig.push_str(&format!(" /* elements: {} */", self.elements_summary()));
        }
        out.push(sig.with_semicolon());

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

    pub fn implementations(&self, output_mode: &OutputMode) -> String {
        if !self.elements.is_empty() {
            return self.element_impl();
        }
        let func_name = self.id.func_name();
        let mut out = String::new();

        if let Some(prefix) = output_mode.string_prefix() {
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

        if let Some(prefix) = output_mode.pattern_prefix() {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&self.comment_lines());
            let ptn_signature = format!("    pub fn {prefix}{func_name}(&self) -> Pattern<String>");
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

    /// A short, human-readable summary of the `(Element)` layout. Used so that
    /// cross-locale signature-mismatch warnings reflect element order.
    fn elements_summary(&self) -> String {
        let parts = self
            .elements
            .iter()
            .map(|m| match m.kind {
                ElementKind::Variable => format!("${}", m.name),
                ElementKind::Term => format!("-{}", m.name),
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{parts}]")
    }

    /// The ordered struct fields for a structured (element) message: a text
    /// slot, then each element slot, ending with a trailing text slot — always
    /// `2 * elements + 1` fields.
    fn element_fields(&self) -> Vec<Field> {
        let mut fields: Vec<Field> = Vec::new();
        for (i, marker) in self.elements.iter().enumerate() {
            fields.push(Field {
                name: format!("s{i}"),
                kind: FieldKind::Text,
            });
            let kind = match marker.kind {
                ElementKind::Variable => FieldKind::Gap,
                ElementKind::Term => FieldKind::Term,
            };
            fields.push(Field {
                name: marker.name.rust_id(),
                kind,
            });
        }
        fields.push(Field {
            name: format!("s{}", self.elements.len()),
            kind: FieldKind::Text,
        });
        dedup_field_names(&mut fields);
        fields
    }

    /// The module-level `struct` + `Display` impl for a structured message.
    /// Returns `None` for ordinary (non-element) messages.
    pub fn element_struct(&self) -> Option<String> {
        if self.elements.is_empty() {
            return None;
        }
        let struct_name = self.id.message.rust_var_name();
        let fields = self.element_fields();

        let field_defs = fields
            .iter()
            .map(|f| {
                let (ty, doc) = match f.kind {
                    FieldKind::Text => ("String", "    /// A resolved run of translated text."),
                    FieldKind::Gap => (
                        "ElementGap",
                        "    /// Element slot: the app injects its own UI element here.",
                    ),
                    FieldKind::Term => (
                        "String",
                        "    /// Resolved text of an `(Element)` term; wrap it in the app.",
                    ),
                };
                format!("{doc}\n    pub {}: {ty},", f.name)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let display_args = fields
            .iter()
            .filter(|f| f.kind != FieldKind::Gap)
            .map(|f| format!("self.{}", f.name))
            .collect::<Vec<_>>();
        let display_fmt = "{}".repeat(display_args.len());
        let display_args = display_args.join(", ");

        Some(format!(
            r##"/// Structured output for {id}.
///
/// Render the fields in declaration order. To keep bidirectional text correct,
/// wrap each field — and each element the app injects — in an isolated bidi run
/// (an HTML `<bdi>`, or `unicode-bidi: isolate`), with the container set to the
/// locale's base direction.
#[derive(Debug, Clone, PartialEq)]
pub struct {struct_name} {{
{field_defs}
}}

impl ::core::fmt::Display for {struct_name} {{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {{
        write!(f, "{display_fmt}", {display_args})
    }}
}}"##,
            id = self.id,
        ))
    }

    /// The `impl L10nLanguage` accessor for a structured message: resolves the
    /// segments and builds the generated struct.
    fn element_impl(&self) -> String {
        let func_name = self.id.func_name();
        let struct_name = self.id.message.rust_var_name();
        let fields = self.element_fields();
        let seg_count = fields.len();

        let signature = if self.variables.is_empty() {
            format!("    pub fn {func_name}(&self) -> {struct_name}")
        } else {
            let ArgInfo { generic, arg } = args_declaration(&self.variables);
            let lt = lifetime(&self.variables);
            format!("    pub fn {func_name}<{lt}{generic}>(&self, {arg}) -> {struct_name}")
        };

        let args_setup = if self.variables.is_empty() {
            String::new()
        } else {
            format!(
                "        let mut args = FluentArgs::new();\n{}\n",
                args_impl(&self.variables)
            )
        };
        let args_expr = if self.variables.is_empty() {
            "None"
        } else {
            "Some(args)"
        };

        let vars_arr = quoted_element_names(&self.elements, ElementKind::Variable);
        let terms_arr = quoted_element_names(&self.elements, ElementKind::Term);

        let destructure = fields
            .iter()
            .enumerate()
            .map(|(i, f)| match f.kind {
                FieldKind::Gap => "_".to_string(),
                _ => format!("seg{i}"),
            })
            .collect::<Vec<_>>()
            .join(", ");

        let inits = fields
            .iter()
            .enumerate()
            .map(|(i, f)| match f.kind {
                FieldKind::Gap => format!("            {}: ElementGap,", f.name),
                _ => format!("            {}: seg{i}.into_text(),", f.name),
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r##"{comment}{signature} {{
{args_setup}        let segments: [Segment; {seg_count}] = self
            .0
            .msg_segments("{msg_id}", &[{vars_arr}], &[{terms_arr}], {args_expr})
            .unwrap()
            .try_into()
            .unwrap();
        let [{destructure}] = segments;
        {struct_name} {{
{inits}
        }}
    }}"##,
            comment = self.comment_lines(),
            msg_id = self.id.message,
        )
    }
}

fn lifetime(vars: &[Variable]) -> &'static str {
    if vars.iter().any(|v| v.typ == VarType::Any) {
        "'a, "
    } else {
        ""
    }
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

/// A field of a generated structured-message struct.
struct Field {
    name: String,
    kind: FieldKind,
}

#[derive(PartialEq)]
enum FieldKind {
    /// A resolved text slot (`s0`, `s1`, …).
    Text,
    /// A variable `(Element)` — a positional gap.
    Gap,
    /// A term `(Element)` — resolved translatable text.
    Term,
}

/// Ensure every generated struct field name is unique by suffixing repeats
/// (e.g. the same element used twice, or an element named `s0`).
fn dedup_field_names(fields: &mut [Field]) {
    let mut seen: Vec<String> = Vec::new();
    for field in fields.iter_mut() {
        if seen.contains(&field.name) {
            let base = field.name.clone();
            let mut n = 2;
            while seen.contains(&field.name) {
                field.name = format!("{base}_{n}");
                n += 1;
            }
        }
        seen.push(field.name.clone());
    }
}

/// Comma-separated, quoted names of the element markers of the given kind.
fn quoted_element_names(elements: &[ElementMarker], kind: ElementKind) -> String {
    elements
        .iter()
        .filter(|m| m.kind == kind)
        .map(|m| format!("\"{}\"", m.name))
        .collect::<Vec<_>>()
        .join(", ")
}
