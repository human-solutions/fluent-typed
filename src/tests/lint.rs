//! Tests for comment linting, strict mode, the contract-based cross-locale
//! analysis, and the file/line detail of build errors.

use crate::build::typed::Id;
use crate::build::{Analyzed, Builder, LangBundle, lint};
use crate::{BuildError, BuildOptions, FtlOutputOptions, LintLevel};

fn bundle(ftl: &str, file: &str, lang: &str) -> LangBundle {
    LangBundle::from_ftl(ftl, file, lang, true).expect("ftl should parse")
}

/// Run the linter over a single default-locale bundle.
fn lints_of(ftl: &str) -> lint::Lints {
    let langs = vec![bundle(ftl, "main.ftl", "en")];
    let analyzed = Analyzed::from(&langs, &langs[0]);
    lint::check(&langs, &langs[0], &analyzed.common)
}

/// Run the full build of a single `en` locale at the given lint level.
fn build_at(dir: &str, level: LintLevel, ftl: &str) -> Result<(), BuildError> {
    let dir = format!("target/test-lint/{dir}");
    std::fs::create_dir_all(&dir).unwrap();
    let opts = BuildOptions::default()
        .with_lint_level(level)
        .without_format()
        .with_output_file_path(&format!("{dir}/l10n.rs"))
        .with_ftl_output(FtlOutputOptions::SingleFile {
            output_ftl_file: format!("{dir}/t.ftl"),
            compressor: None,
        });
    Builder::load_one(opts, "test", "en", ftl)?.generate()
}

// ----------------------------------------------------------------------------
// L1–L3: comment mistakes in the default locale.
// ----------------------------------------------------------------------------

#[test]
fn l1_unrecognized_keyword() {
    let lints = lints_of("# $count (Numbr) - how many\nfoo = You have { $count }\n");
    assert_eq!(
        lints.mistakes,
        [
            "main.ftl:1: unrecognized type annotation '(Numbr)' for $count — \
          expected (String), (Number), (Bool) or (Element)"
        ],
    );
}

#[test]
fn l2_annotation_of_missing_variable() {
    let lints = lints_of("# $nme (String) - the name\nhello = Hello { $name }\n");
    assert_eq!(
        lints.mistakes,
        [
            "main.ftl:1: comment annotates $nme but message 'hello' references no \
          such variable"
        ],
    );
}

#[test]
fn l3_detached_annotation_comment() {
    // The blank line detaches the comment from `hello`.
    let lints = lints_of("# $name (String) - the name\n\nhello = Hello { $name }\n");
    assert_eq!(
        lints.mistakes,
        [
            "main.ftl:1: type annotation '$name (String)' is detached from its \
          message by a blank line and has no effect"
        ],
    );
}

#[test]
fn a_correct_comment_produces_no_mistakes() {
    let lints = lints_of("# $name (String) - the name\nhello = Hello { $name }\n");
    assert!(lints.mistakes.is_empty(), "{:?}", lints.mistakes);
}

// ----------------------------------------------------------------------------
// L5: annotations in a non-default locale have no effect.
// ----------------------------------------------------------------------------

#[test]
fn l5_annotation_in_non_default_locale() {
    let langs = vec![
        bundle("hello = Hello { $name }\n", "en.ftl", "en"),
        bundle(
            "# $name (String) - le nom\nhello = Bonjour { $name }\n",
            "fr.ftl",
            "fr",
        ),
    ];
    let analyzed = Analyzed::from(&langs, &langs[0]);
    let lints = lint::check(&langs, &langs[0], &analyzed.common);

    assert!(lints.mistakes.is_empty(), "{:?}", lints.mistakes);
    assert_eq!(
        lints.ineffective,
        [
            "fr.ftl:1: type annotation '$name (String)' in non-default locale 'fr' \
          has no effect — type comments only apply to the default locale"
        ],
    );
}

// ----------------------------------------------------------------------------
// Strict mode.
// ----------------------------------------------------------------------------

#[test]
fn strict_rejects_an_untyped_variable() {
    let err = build_at("untyped", LintLevel::Strict, "hello = Hello { $name }\n")
        .expect_err("an untyped variable must fail strict mode");
    match err {
        BuildError::Lint { messages } => assert_eq!(
            messages,
            [
                "test:1: variable $name in message 'hello' has no type — add a \
              '# $name (String)', '# $name (Number)' or '# $name (Bool)' comment"
            ],
        ),
        other => panic!("expected BuildError::Lint, got {other:?}"),
    }
}

#[test]
fn strict_accepts_a_fully_typed_message() {
    build_at(
        "typed",
        LintLevel::Strict,
        "# $name (String) - the name\nhello = Hello { $name }\n",
    )
    .expect("a fully typed message must pass strict mode");
}

#[test]
fn strict_accepts_a_bool_variable() {
    build_at(
        "bool",
        LintLevel::Strict,
        "# $enabled (Bool) - feature state\nfeature = { $enabled ->\n    [true] On\n   *[false] Off\n}\n",
    )
    .expect("a Boolean variable must count as typed");
}

#[test]
fn bool_selector_requires_true_and_false_keys() {
    let lints = lints_of(
        "# $enabled (Bool) - feature state\nfeature = { $enabled ->\n    [yes] On\n   *[no] Off\n}\n",
    );
    assert_eq!(
        lints.mistakes,
        [
            "main.ftl:2: Boolean selector $enabled in message 'feature' has keys \
             [yes, no] — expected [true] and [false]"
        ],
    );
}

#[test]
fn strict_promotes_comment_mistakes_to_errors() {
    let err = build_at("typo", LintLevel::Strict, "# $x (Strng)\nhi = Hi { $x }\n")
        .expect_err("a typo'd keyword must fail strict mode");
    assert!(matches!(err, BuildError::Lint { .. }), "{err:?}");
}

#[test]
fn lint_level_off_silences_everything() {
    // A typo'd keyword and an untyped variable — both ignored when Off.
    build_at("off", LintLevel::Off, "# $x (Strng)\nhi = Hi { $x }\n")
        .expect("LintLevel::Off must not fail the build");
}

#[test]
fn warn_level_does_not_fail_the_build() {
    // The default level: a typo is only a warning, the build still succeeds.
    build_at("warn", LintLevel::Warn, "# $x (Strng)\nhi = Hi { $x }\n")
        .expect("LintLevel::Warn must not fail the build");
}

#[test]
fn deny_fails_on_a_comment_mistake() {
    let err = build_at(
        "deny-typo",
        LintLevel::Deny,
        "# $x (Strng)\nhi = Hi { $x }\n",
    )
    .expect_err("Deny must fail on a comment mistake");
    assert!(matches!(err, BuildError::Lint { .. }), "{err:?}");
}

#[test]
fn deny_allows_an_untyped_variable() {
    // `$name` is untyped — allowed under Deny, unlike Strict.
    build_at("deny-untyped", LintLevel::Deny, "hello = Hello { $name }\n")
        .expect("Deny must allow an untyped variable");
}

// ----------------------------------------------------------------------------
// Source locations: the file and line in a diagnostic must be exact, including
// for an annotation partway down a multi-line comment block.
// ----------------------------------------------------------------------------

#[test]
fn a_diagnostic_points_at_the_right_line_of_a_multi_line_comment() {
    // line 1: (blank)
    // line 2: # greeting shown on login   <- prose, not an annotation
    // line 3: # $nam (String) - the name  <- the mistake (typo'd variable)
    // line 4: # $count (Number) - count
    // line 5: welcome = ...
    let ftl = "\n# greeting shown on login\n# $nam (String) - the name\n\
               # $count (Number) - count\nwelcome = Hi { $name }, { $count } new\n";
    let lints = lints_of(ftl);
    assert_eq!(
        lints.mistakes,
        [
            "main.ftl:3: comment annotates $nam but message 'welcome' references no \
          such variable"
        ],
        "the annotation is the 2nd comment line, so it must be reported at line 3",
    );
}

#[test]
fn the_untyped_error_points_at_the_message_line() {
    // The leading comment is detached by a blank line, so `hello` (line 4) has
    // no comment and `$name` is untyped.
    let err = build_at(
        "untyped-line",
        LintLevel::Strict,
        "# a comment\n# another\n\nhello = Hello { $name }\n",
    )
    .expect_err("an untyped variable must fail strict mode");
    match err {
        BuildError::Lint { messages } => assert_eq!(
            messages,
            [
                "test:4: variable $name in message 'hello' has no type — add a \
              '# $name (String)', '# $name (Number)' or '# $name (Bool)' comment"
            ],
        ),
        other => panic!("expected BuildError::Lint, got {other:?}"),
    }
}

#[test]
fn diagnostics_are_per_resource_file_in_a_multi_resource_locale() {
    // A locale folder with two `.ftl` files. The mistake is on line 2 of the
    // *second* file — its diagnostic must name that file and that line, not an
    // offset into the concatenated bundle.
    let dir = "target/test-lint/multi-resource";
    let en = format!("{dir}/locales/en");
    std::fs::create_dir_all(&en).unwrap();
    std::fs::write(format!("{en}/a.ftl"), "first = First message\n").unwrap();
    std::fs::write(
        format!("{en}/b.ftl"),
        "greeting = Hi\nwelcome = Hello { $name }\n",
    )
    .unwrap();

    let opts = BuildOptions::default()
        .with_lint_level(LintLevel::Strict)
        .without_format()
        .with_locales_folder(&format!("{dir}/locales"))
        .with_output_file_path(&format!("{dir}/l10n.rs"))
        .with_ftl_output(FtlOutputOptions::SingleFile {
            output_ftl_file: format!("{dir}/t.ftl"),
            compressor: None,
        });

    let err = Builder::load(opts)
        .unwrap()
        .generate()
        .expect_err("the untyped $name in b.ftl must fail strict mode");
    match err {
        BuildError::Lint { messages } => {
            assert_eq!(messages.len(), 1, "{messages:?}");
            let m = &messages[0];
            assert!(m.contains("b.ftl:2:"), "must name b.ftl line 2: {m}");
            assert!(m.contains("variable $name in message 'welcome'"), "{m}");
        }
        other => panic!("expected BuildError::Lint, got {other:?}"),
    }
}

// ----------------------------------------------------------------------------
// Contract-based cross-locale analysis.
// ----------------------------------------------------------------------------

#[test]
fn a_type_difference_across_locales_is_not_a_mismatch() {
    // Only the default locale types `$name`; the message is still generated.
    let langs = vec![
        bundle(
            "# $name (String) - name\nhello = Hello { $name }\n",
            "en.ftl",
            "en",
        ),
        bundle("hello = Bonjour { $name }\n", "fr.ftl", "fr"),
    ];
    let analyzed = Analyzed::from(&langs, &langs[0]);
    assert!(analyzed.warnings.is_empty(), "{:?}", analyzed.warnings);
    assert!(analyzed.common.contains(&Id::new_msg("hello")));
}

#[test]
fn an_extra_variable_in_another_locale_drops_the_message() {
    let langs = vec![
        bundle("hello = Hello { $name }\n", "en.ftl", "en"),
        bundle("hello = Bonjour { $name } et { $extra }\n", "fr.ftl", "fr"),
    ];
    let analyzed = Analyzed::from(&langs, &langs[0]);
    assert!(!analyzed.common.contains(&Id::new_msg("hello")));
    assert_eq!(
        analyzed.warnings,
        [
            "en.ftl:1: message 'hello' is not generated — incompatible variables, \
          Boolean selectors or elements in locale(s): fr (fr.ftl:1)"
        ],
    );
}

#[test]
fn a_message_missing_from_the_default_locale_is_reported() {
    let langs = vec![
        bundle("hello = Hello\n", "en.ftl", "en"),
        bundle("hello = Bonjour\nbye = Au revoir\n", "fr.ftl", "fr"),
    ];
    let analyzed = Analyzed::from(&langs, &langs[0]);
    assert!(analyzed.common.contains(&Id::new_msg("hello")));
    assert_eq!(
        analyzed.warnings,
        [
            "message 'bye' is not generated — present in locale(s) fr but missing \
          from the default locale 'en'"
        ],
    );
}

#[test]
fn invalid_bool_selector_in_another_locale_drops_the_message() {
    let langs = vec![
        bundle(
            "# $enabled (Bool) - state\nfeature = { $enabled ->\n    [true] On\n   *[false] Off\n}\n",
            "en.ftl",
            "en",
        ),
        bundle(
            "feature = { $enabled ->\n    [yes] Oui\n   *[no] Non\n}\n",
            "fr.ftl",
            "fr",
        ),
    ];
    let analyzed = Analyzed::from(&langs, &langs[0]);
    assert!(!analyzed.common.contains(&Id::new_msg("feature")));
    assert_eq!(analyzed.warnings.len(), 1, "{:?}", analyzed.warnings);
    assert!(
        analyzed.warnings[0].contains("Boolean selectors"),
        "{:?}",
        analyzed.warnings
    );
}

#[test]
fn a_message_missing_from_another_locale_is_reported() {
    let langs = vec![
        bundle("hello = Hello\nbye = Bye\n", "en.ftl", "en"),
        bundle("hello = Bonjour\n", "fr.ftl", "fr"),
    ];
    let analyzed = Analyzed::from(&langs, &langs[0]);
    assert!(analyzed.common.contains(&Id::new_msg("hello")));
    assert!(!analyzed.common.contains(&Id::new_msg("bye")));
    assert_eq!(
        analyzed.warnings,
        ["en.ftl:2: message 'bye' is not generated — missing from locale(s): fr"],
    );
}

// ----------------------------------------------------------------------------
// Build errors name the file (and line) of the problem.
// ----------------------------------------------------------------------------

#[test]
fn missing_default_language_is_an_error() {
    let err = Builder::load_one(BuildOptions::default(), "test", "fr", "hello = Bonjour\n")
        .unwrap()
        .generate()
        .expect_err("a missing default language must fail the build");
    match &err {
        BuildError::DefaultLanguageNotFound { language, .. } => assert_eq!(language, "en"),
        other => panic!("expected DefaultLanguageNotFound, got {other:?}"),
    }
    assert!(err.to_string().contains("'en'"), "{err}");
}

#[test]
fn a_parse_error_names_the_file_and_line() {
    let err = LangBundle::from_ftl("ok = fine\nbroken { $x }\n", "bad.ftl", "en", true)
        .expect_err("malformed ftl must fail to parse");
    match &err {
        BuildError::FtlParse { path, errors } => {
            assert!(path.ends_with("bad.ftl"), "{path:?}");
            assert!(!errors.is_empty());
            assert!(errors[0].starts_with("line "), "{:?}", errors[0]);
        }
        other => panic!("expected FtlParse, got {other:?}"),
    }
    assert!(err.to_string().contains("bad.ftl"), "{err}");
}

#[test]
fn a_parse_error_carries_a_human_readable_message() {
    // fluent-syntax's own `Display` message, not the terse `Debug` enum form.
    let err = LangBundle::from_ftl("broken { $x }\n", "bad.ftl", "en", true)
        .expect_err("a message without `=` must fail to parse");
    match &err {
        BuildError::FtlParse { errors, .. } => {
            assert_eq!(errors.len(), 1, "{errors:?}");
            assert!(
                errors[0].contains("Expected a token"),
                "expected a readable message, got {:?}",
                errors[0],
            );
        }
        other => panic!("expected FtlParse, got {other:?}"),
    }
}

#[test]
fn a_parse_error_in_a_non_ascii_file_does_not_panic() {
    // Malformed line preceded by multi-byte UTF-8 — the error byte offset must
    // not be used to slice the source at a non-char-boundary.
    let err = LangBundle::from_ftl(
        "grüße = Hallo schöne Welt café\nbroken { $x }\n",
        "bad.ftl",
        "en",
        true,
    )
    .expect_err("malformed ftl must fail to parse");
    assert!(matches!(err, BuildError::FtlParse { .. }), "{err:?}");
}

// ----------------------------------------------------------------------------
// Regression tests for review findings.
// ----------------------------------------------------------------------------

#[test]
fn a_prose_parenthetical_is_not_flagged_as_a_typo() {
    // `(required)` is ordinary prose, not a botched type annotation — the
    // linter must leave it alone (it would otherwise fail Deny/Strict builds).
    let lints = lints_of("# $name (required) - the user's name\nhello = Hi { $name }\n");
    assert!(lints.mistakes.is_empty(), "{:?}", lints.mistakes);
}

#[test]
fn a_near_miss_keyword_is_still_flagged() {
    // `(Strng)` is one edit away from `(String)` — a real typo, still caught.
    let lints = lints_of("# $name (Strng)\nhello = Hi { $name }\n");
    assert_eq!(lints.mistakes.len(), 1, "{:?}", lints.mistakes);
    assert!(
        lints.mistakes[0].contains("unrecognized type annotation '(Strng)'"),
        "{:?}",
        lints.mistakes,
    );
}

#[test]
fn an_attribute_only_message_comment_is_linted() {
    // `hello` has no value, only a `.tooltip` attribute. Its comment must
    // still be linted — here, a typo'd keyword.
    let lints = lints_of("# $name (Strng)\nhello =\n    .tooltip = Hi { $name }\n");
    assert_eq!(lints.mistakes.len(), 1, "{:?}", lints.mistakes);
    assert!(
        lints.mistakes[0].contains("unrecognized type annotation '(Strng)'"),
        "{:?}",
        lints.mistakes,
    );
}

#[test]
fn strict_allows_an_element_variable_reused_in_an_attribute() {
    // `$icon` is annotated `(Element)`; it is also referenced in an attribute,
    // where it stays a runtime argument. Strict must not demand a type for it.
    build_at(
        "element-attr",
        LintLevel::Strict,
        "# $icon (Element) - an injected icon\n\
         notice = Click { $icon } now\n    .aria = icon { $icon }\n",
    )
    .expect("an (Element) variable reused in an attribute must pass strict mode");
}

#[test]
fn a_duplicate_key_generates_the_accessor_only_once() {
    let dir = "target/test-lint/dup-accessor";
    std::fs::create_dir_all(dir).unwrap();
    let out = format!("{dir}/l10n.rs");
    let opts = BuildOptions::default()
        .with_allow_duplicate_keys()
        .without_format()
        .with_output_file_path(&out)
        .with_ftl_output(FtlOutputOptions::SingleFile {
            output_ftl_file: format!("{dir}/t.ftl"),
            compressor: None,
        });
    Builder::load_one(opts, "test", "en", "hello = One\nhello = Two\n")
        .unwrap()
        .generate()
        .unwrap();
    let generated = std::fs::read_to_string(&out).unwrap();
    assert_eq!(
        generated.matches("fn msg_hello").count(),
        1,
        "a duplicated key must still generate exactly one accessor",
    );
}

#[test]
fn a_deny_failure_message_does_not_mention_strict_mode() {
    let err = build_at(
        "deny-msg",
        LintLevel::Deny,
        "# $x (Strng)\nhi = Hi { $x }\n",
    )
    .expect_err("Deny must fail on a comment mistake");
    let display = err.to_string();
    assert!(
        !display.contains("strict"),
        "Deny message must not say strict: {display}"
    );
    assert!(display.contains("lint error"), "{display}");
}

// ----------------------------------------------------------------------------
// Errors are collected across files, not reported one rebuild at a time.
// ----------------------------------------------------------------------------

#[test]
fn parse_errors_in_several_files_are_reported_together() {
    let dir = "target/test-lint/multi-parse-err";
    let en = format!("{dir}/locales/en");
    std::fs::create_dir_all(&en).unwrap();
    std::fs::write(format!("{en}/a.ftl"), "broken { $x }\n").unwrap();
    std::fs::write(format!("{en}/b.ftl"), "ok = fine\nalso-broken { $y }\n").unwrap();

    let opts = BuildOptions::default()
        .with_locales_folder(&format!("{dir}/locales"))
        .with_output_file_path(&format!("{dir}/l10n.rs"))
        .with_ftl_output(FtlOutputOptions::SingleFile {
            output_ftl_file: format!("{dir}/t.ftl"),
            compressor: None,
        });

    let err = Builder::load(opts)
        .err()
        .expect("two malformed files must fail the build");
    let display = err.to_string();
    assert!(
        display.contains("a.ftl") && display.contains("b.ftl"),
        "the report must name both files: {display}"
    );
    match err {
        BuildError::Multiple(errors) => {
            assert_eq!(errors.len(), 2, "{errors:?}");
            assert!(
                errors
                    .iter()
                    .all(|e| matches!(e, BuildError::FtlParse { .. })),
                "{errors:?}",
            );
        }
        other => panic!("expected BuildError::Multiple, got {other:?}"),
    }
}

#[test]
fn every_duplicate_key_is_reported() {
    // Two distinct keys are each defined twice — all four-into-two duplicates
    // must be reported, not just the first.
    let err = LangBundle::from_ftl(
        "hello = One\nhello = Two\nbye = A\nbye = B\n",
        "dup.ftl",
        "en",
        true,
    )
    .expect_err("duplicate keys must fail");
    match err {
        BuildError::Multiple(errors) => {
            assert_eq!(errors.len(), 2, "{errors:?}");
            assert!(
                errors
                    .iter()
                    .all(|e| matches!(e, BuildError::DuplicateKey { .. })),
                "{errors:?}",
            );
        }
        other => panic!("expected BuildError::Multiple, got {other:?}"),
    }
}

#[test]
fn a_parse_error_and_a_duplicate_key_are_reported_together() {
    let dir = "target/test-lint/mixed-errors";
    let en = format!("{dir}/locales/en");
    std::fs::create_dir_all(&en).unwrap();
    std::fs::write(format!("{en}/a.ftl"), "broken { $x }\n").unwrap();
    std::fs::write(format!("{en}/b.ftl"), "dup = One\ndup = Two\n").unwrap();

    let opts = BuildOptions::default()
        .with_locales_folder(&format!("{dir}/locales"))
        .with_output_file_path(&format!("{dir}/l10n.rs"))
        .with_ftl_output(FtlOutputOptions::SingleFile {
            output_ftl_file: format!("{dir}/t.ftl"),
            compressor: None,
        });

    let err = Builder::load(opts)
        .err()
        .expect("both problems must fail the build");
    match err {
        BuildError::Multiple(errors) => {
            assert_eq!(errors.len(), 2, "{errors:?}");
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, BuildError::FtlParse { .. })),
                "{errors:?}",
            );
            assert!(
                errors
                    .iter()
                    .any(|e| matches!(e, BuildError::DuplicateKey { .. })),
                "{errors:?}",
            );
        }
        other => panic!("expected BuildError::Multiple, got {other:?}"),
    }
}
