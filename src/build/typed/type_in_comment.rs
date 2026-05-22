use super::{VarType, Variable};

/// Type and `(Element)` annotations extracted from a message comment.
///
/// `(String)` / `(Number)` apply to `$variable`s; `(Element)` applies to either
/// a `$variable` (a positional gap) or a `-term` (translatable wrapped text).
#[derive(Debug, PartialEq, Default)]
pub struct TypeInComment {
    string: Vec<String>,
    number: Vec<String>,
    element_vars: Vec<String>,
    element_terms: Vec<String>,
}

impl TypeInComment {
    pub fn parse(comment: &[String]) -> Self {
        let mut tic = Self::default();

        for line in comment {
            match parse_line(line) {
                Found::String(s) => tic.string.push(s.to_owned()),
                Found::Number(n) => tic.number.push(n.to_owned()),
                Found::ElementVar(v) => tic.element_vars.push(v.to_owned()),
                Found::ElementTerm(t) => tic.element_terms.push(t.to_owned()),
                Found::Nothing => {}
            }
        }
        tic
    }

    pub fn update_types(&self, variables: &mut Vec<Variable>) {
        for variable in variables {
            if self.string.contains(&variable.id) {
                variable.typ = VarType::String;
            } else if self.number.contains(&variable.id) {
                variable.typ = VarType::Number;
            }
        }
    }

    /// Names of `$variable`s annotated `(Element)`.
    pub fn element_vars(&self) -> &[String] {
        &self.element_vars
    }

    /// Names of `-term`s annotated `(Element)`.
    pub fn element_terms(&self) -> &[String] {
        &self.element_terms
    }
}

enum Found<'a> {
    String(&'a str),
    Number(&'a str),
    ElementVar(&'a str),
    ElementTerm(&'a str),
    Nothing,
}

fn parse_line(line: &str) -> Found<'_> {
    let Some((id, rest)) = line.trim().split_once(' ') else {
        return Found::Nothing;
    };

    let id = id.trim();
    let rest = rest.trim();

    let is_term = id.starts_with('-');
    let is_var = id.starts_with('$');
    if !is_term && !is_var {
        return Found::Nothing;
    }
    let name = &id[1..];

    if rest.starts_with("(Element)") {
        if is_term {
            Found::ElementTerm(name)
        } else {
            Found::ElementVar(name)
        }
    } else if is_var && rest.starts_with("(Number)") {
        Found::Number(name)
    } else if is_var && rest.starts_with("(String)") {
        Found::String(name)
    } else {
        Found::Nothing
    }
}

#[test]
fn test_number() {
    let s = "$duration (Number) - The duration in seconds.".to_owned();

    let tic = TypeInComment::parse(&[s]);
    assert_eq!(
        tic,
        TypeInComment {
            string: vec![],
            number: vec!["duration".to_string()],
            element_vars: vec![],
            element_terms: vec![],
        }
    );
}

#[test]
fn test_element_var_and_term() {
    let lines = [
        "$icon (Element) - A UI element injected by the app.".to_owned(),
        "-privacy-link (Element) - Translatable text wrapped by the app.".to_owned(),
    ];

    let tic = TypeInComment::parse(&lines);
    assert_eq!(tic.element_vars(), ["icon".to_string()]);
    assert_eq!(tic.element_terms(), ["privacy-link".to_string()]);
}
