use super::{VarType, Variable};

/// Type and `(Element)` annotations extracted from a message comment.
///
/// `(String)` / `(Number)` / `(Bool)` apply to `$variable`s; `(Element)` applies to either
/// a `$variable` (a positional gap) or a `-term` (translatable wrapped text).
#[derive(Debug, PartialEq, Default)]
pub struct TypeInComment {
    string: Vec<String>,
    number: Vec<String>,
    bool_: Vec<String>,
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
                Found::Bool(b) => tic.bool_.push(b.to_owned()),
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
            } else if self.bool_.contains(&variable.id) {
                variable.typ = VarType::Bool;
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

/// An annotation-shaped comment line: a `$variable` or `-term`, followed by a
/// parenthesized keyword — e.g. `$name (String) - the user's name`.
///
/// This describes the *shape* only; the `keyword` may be a typo. Use
/// [`Annotation::is_recognized`] to tell whether it actually means something.
#[derive(Debug, PartialEq)]
pub struct Annotation<'a> {
    /// The referenced name, without its `$` / `-` sigil.
    pub name: &'a str,
    /// `true` when the sigil was `-` (a term) rather than `$` (a variable).
    pub is_term: bool,
    /// The keyword found between the parentheses, e.g. `String` or `Numbr`.
    pub keyword: &'a str,
}

impl Annotation<'_> {
    /// `true` when this annotation is one fluent-typed actually acts on:
    /// `(Element)` on a variable or term, or `(String)`/`(Number)`/`(Bool)` on a
    /// variable. Anything else (a typo, `(String)` on a term, …) is inert.
    pub fn is_recognized(&self) -> bool {
        match self.keyword {
            "Element" => true,
            "String" | "Number" | "Bool" => !self.is_term,
            _ => false,
        }
    }

    /// `true` when this line was *plausibly meant* to be a type annotation —
    /// either a recognized one, or a near-miss typo of a real keyword.
    ///
    /// This distinguishes a botched annotation (`(Numbr)`, `(string)`) from
    /// ordinary prose that merely contains parentheses (`(optional)`,
    /// `(max 99)`). The linter must only flag the former.
    pub fn is_type_annotation(&self) -> bool {
        self.is_recognized() || looks_like_keyword(self.keyword)
    }

    /// The sigil that introduced the reference: `"$"` or `"-"`.
    pub fn sigil(&self) -> &'static str {
        if self.is_term { "-" } else { "$" }
    }
}

/// Whether `keyword` is close enough to a real type keyword (`String`,
/// `Number`, `Bool`, `Element`) to be a typo of it rather than ordinary prose.
/// Case-insensitive, Levenshtein distance up to 2.
fn looks_like_keyword(keyword: &str) -> bool {
    let kw = keyword.trim().to_ascii_lowercase();

    // Fuzzy matching is too broad for a four-letter keyword: ordinary prose
    // such as `(bold)`, `(cool)` and `(tool)` is within two edits of `Bool`.
    // Match it case-insensitively instead. This still catches `(bool)`/`(BOOL)`,
    // but spelling mistakes such as `(Bol)` remain unrecognized; `Strict`
    // linting still reports their variable as untyped.
    if kw == "bool" {
        return true;
    }

    ["string", "number", "element"]
        .iter()
        .any(|target| edit_distance_within(&kw, target, 2))
}

/// `true` when the Levenshtein edit distance between `a` and `b` is `<= max`.
/// Bails out early once every cell of a row exceeds `max`.
fn edit_distance_within(a: &str, b: &str, max: usize) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return false;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        let mut row_min = curr[0];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(curr[j] + 1);
            row_min = row_min.min(curr[j + 1]);
        }
        if row_min > max {
            return false;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()] <= max
}

/// Parse a single comment line as a type annotation, if it is shaped like one.
///
/// A line is annotation-shaped when its first whitespace-delimited token is a
/// `$variable` / `-term` reference and the remainder begins with a
/// `(keyword)`. The keyword is returned verbatim — recognized or not — so the
/// linter can flag typos.
pub fn annotation(line: &str) -> Option<Annotation<'_>> {
    let (id, rest) = line.trim().split_once(' ')?;
    let id = id.trim();

    let is_term = id.starts_with('-');
    let is_var = id.starts_with('$');
    if !is_term && !is_var {
        return None;
    }
    let name = &id[1..];
    if name.is_empty() {
        return None;
    }

    let keyword = rest.trim().strip_prefix('(')?.split_once(')')?.0.trim();
    Some(Annotation {
        name,
        is_term,
        keyword,
    })
}

enum Found<'a> {
    String(&'a str),
    Number(&'a str),
    Bool(&'a str),
    ElementVar(&'a str),
    ElementTerm(&'a str),
    Nothing,
}

fn parse_line(line: &str) -> Found<'_> {
    let Some(a) = annotation(line) else {
        return Found::Nothing;
    };
    match a.keyword {
        "Element" if a.is_term => Found::ElementTerm(a.name),
        "Element" => Found::ElementVar(a.name),
        "Number" if !a.is_term => Found::Number(a.name),
        "String" if !a.is_term => Found::String(a.name),
        "Bool" if !a.is_term => Found::Bool(a.name),
        _ => Found::Nothing,
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
            bool_: vec![],
            element_vars: vec![],
            element_terms: vec![],
        }
    );
}

#[test]
fn test_bool() {
    let tic = TypeInComment::parse(&["$enabled (Bool) - Whether enabled.".to_owned()]);

    assert_eq!(
        tic,
        TypeInComment {
            string: vec![],
            number: vec![],
            bool_: vec!["enabled".to_string()],
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

#[test]
fn test_annotation_shape_and_recognition() {
    let good = annotation("$name (String) - the user's name").unwrap();
    assert_eq!(good.name, "name");
    assert!(!good.is_term);
    assert_eq!(good.keyword, "String");
    assert!(good.is_recognized());

    // A spaced keyword is trimmed and still recognized.
    let spaced = annotation("$name ( String ) - desc").unwrap();
    assert_eq!(spaced.keyword, "String");
    assert!(spaced.is_recognized());

    // Annotation-shaped, but a typo'd keyword — still a *type annotation*.
    let typo = annotation("$count (Numbr)").unwrap();
    assert_eq!(typo.keyword, "Numbr");
    assert!(!typo.is_recognized());
    assert!(typo.is_type_annotation());

    // `(Number)` on a term is inert, but still a (botched) type annotation.
    let on_term = annotation("-brand (Number)").unwrap();
    assert!(on_term.is_term);
    assert!(!on_term.is_recognized());
    assert!(on_term.is_type_annotation());

    // A parenthesized word that is just prose is NOT a type annotation, so the
    // linter must not flag it as a typo.
    for prose in ["$name (optional) - desc", "$count (max 99)", "$x ()"] {
        let a = annotation(prose).unwrap();
        assert!(!a.is_recognized(), "{prose}");
        assert!(!a.is_type_annotation(), "{prose}");
    }

    // Short ordinary words must not become Bool typos. Fuzzy matching four
    // letters produces too many false positives.
    for prose in ["$label (bold)", "$state (cool)", "$tool (tool)"] {
        let a = annotation(prose).unwrap();
        assert!(!a.is_type_annotation(), "{prose}");
    }

    // Case-only mistakes are caught; spelling mistakes are intentionally not.
    assert!(annotation("$enabled (bool)").unwrap().is_type_annotation());
    assert!(!annotation("$enabled (Bol)").unwrap().is_type_annotation());

    // Plain prose is not annotation-shaped at all.
    assert!(annotation("$name - the user's name").is_none());
    assert!(annotation("just a comment").is_none());
}
