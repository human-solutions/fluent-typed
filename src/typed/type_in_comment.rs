use super::{VarType, Variable};

#[derive(Debug, PartialEq, Default)]
pub struct TypeInComment<'a> {
    string: Vec<&'a str>,
    number: Vec<&'a str>,
}

impl<'a> TypeInComment<'a> {
    pub fn parse(comment: &[&'a str]) -> Self {
        let mut string = vec![];
        let mut number = vec![];

        for line in comment {
            match parse_line(line) {
                Found::String(s) => string.push(s),
                Found::Number(n) => number.push(n),
                Found::Nothing => {}
            }
        }

        Self { string, number }
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
}

enum Found<'a> {
    String(&'a str),
    Number(&'a str),
    Nothing,
}

fn parse_line(line: &str) -> Found {
    let Some((id, rest)) = line.trim().split_once(' ') else {
        return Found::Nothing;
    };

    let id = id.trim();

    if !id.starts_with('$') {
        return Found::Nothing;
    }
    let id = &id[1..];

    let rest = rest.trim();
    if rest.trim().starts_with("(Number)") {
        Found::Number(id)
    } else if rest.starts_with("(String)") {
        Found::String(id)
    } else {
        Found::Nothing
    }
}

#[test]
fn test_number() {
    let s = "$duration (Number) - The duration in seconds.";

    let tic = TypeInComment::parse(&[s]);
    assert_eq!(
        tic,
        TypeInComment {
            string: vec![],
            number: vec!["duration"],
        }
    );
}
