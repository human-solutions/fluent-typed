use super::{VarType, Variable};

#[derive(Debug, PartialEq, Default)]
pub struct TypeInComment {
    string: Vec<String>,
    number: Vec<String>,
}

impl TypeInComment {
    pub fn parse(comment: &[String]) -> Self {
        let mut string = vec![];
        let mut number = vec![];

        for line in comment {
            match parse_line(line) {
                Found::String(s) => string.push(s.to_owned()),
                Found::Number(n) => number.push(n.to_owned()),
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

fn parse_line(line: &str) -> Found<'_> {
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
    let s = "$duration (Number) - The duration in seconds.".to_owned();

    let tic = TypeInComment::parse(&[s]);
    assert_eq!(
        tic,
        TypeInComment {
            string: vec![],
            number: vec!["duration".to_string()],
        }
    );
}
