use super::assert_gen;
use crate::build::typed::*;
use crate::tests::ast::AstResourceExt;
use fluent_syntax::ast;
use fluent_syntax::parser;

// A `NUMBER()` call marks its argument as a number, so `$ratio` must be
// detected as a variable *and* typed `Number` — see README "Type deduction".
const FTL: &str = r#"

dpi-ratio = Your DPI ratio is { NUMBER($ratio) }

"#;

/// From: https://docs.rs/fluent-syntax/0.12.0/fluent_syntax/ast/enum.InlineExpression.html
#[test]
fn ast() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");

    assert_eq!(
        resource.body[0],
        ast::Entry::Message(ast::Message {
            id: ast::Identifier { name: "dpi-ratio" },
            value: Some(ast::Pattern {
                elements: vec![
                    ast::PatternElement::TextElement {
                        value: "Your DPI ratio is "
                    },
                    ast::PatternElement::Placeable {
                        expression: ast::Expression::Inline(
                            ast::InlineExpression::FunctionReference {
                                id: ast::Identifier { name: "NUMBER" },
                                arguments: ast::CallArguments {
                                    positional: vec![ast::InlineExpression::VariableReference {
                                        id: ast::Identifier { name: "ratio" }
                                    }],
                                    named: vec![],
                                }
                            }
                        )
                    },
                ]
            }),
            attributes: vec![],
            comment: None,
        }),
    );
}

#[test]
fn typed() {
    let resource = parser::parse(FTL).expect("Failed to parse an FTL resource.");
    let message = resource.first_message();

    println!("{message:#?}");
    assert_eq!(
        message,
        Message {
            id: Id::new_msg("dpi-ratio"),
            comment: vec![],
            variables: vec![Variable {
                id: "ratio".to_string(),
                typ: VarType::Number,
            }],
            elements: vec![],
        }
    );
}

#[test]
fn typed_gen() {
    assert_gen(module_path!(), "test", FTL);
}

/// The README's `your-rank` example: a `NUMBER()` function reference used as a
/// select selector. `$pos` appears in the selector and in four variant bodies;
/// it must be collected exactly once and typed `Number`.
#[test]
fn typed_number_function_selector() {
    let ftl = r#"
your-rank = { NUMBER($pos, type: "ordinal") ->
   [1] You finished first!
   [one] You finished {$pos}st
   [two] You finished {$pos}nd
   [few] You finished {$pos}rd
  *[other] You finished {$pos}th
}
"#;
    let resource = parser::parse(ftl).expect("Failed to parse an FTL resource.");
    let message = resource.first_message();

    println!("{message:#?}");
    assert_eq!(
        message,
        Message {
            id: Id::new_msg("your-rank"),
            comment: vec![],
            variables: vec![Variable {
                id: "pos".to_string(),
                typ: VarType::Number,
            }],
            elements: vec![],
        }
    );
}

/// A variable referenced only inside a select variant body — never in the
/// selector — must still be collected as an argument.
#[test]
fn typed_variable_only_in_variant() {
    let ftl = r#"
status = { $state ->
    [active]   { $detail } is running
   *[inactive] stopped
}
"#;
    let resource = parser::parse(ftl).expect("Failed to parse an FTL resource.");
    let message = resource.first_message();

    println!("{message:#?}");
    assert_eq!(
        message,
        Message {
            id: Id::new_msg("status"),
            comment: vec![],
            variables: vec![
                Variable {
                    id: "state".to_string(),
                    typ: VarType::Any,
                },
                Variable {
                    id: "detail".to_string(),
                    typ: VarType::Any,
                },
            ],
            elements: vec![],
        }
    );
}
