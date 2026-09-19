use super::*;

#[test]
fn split_script_into_forms_stops_hard_syntax_errors_from_poisoning_later_forms() {
    let forms = split_script_into_forms("1 2\n3").unwrap();

    let texts: Vec<&str> = forms.iter().map(|form| form.text()).collect();
    assert_eq!(texts, vec!["1 2", "3"]);
}

#[test]
fn split_script_into_forms_skips_initial_shebang_trivia() {
    let forms = split_script_into_forms("#!/usr/bin/env rum\n\n1\n2").unwrap();

    let texts: Vec<&str> = forms.iter().map(|form| form.text()).collect();
    assert_eq!(texts, vec!["1", "2"]);
}

#[test]
fn split_script_into_forms_treats_multiline_comments_as_incomplete_trivia() {
    let forms = split_script_into_forms("1\n(* multi\n   line *)\n2").unwrap();

    let texts: Vec<&str> = forms.iter().map(|form| form.text()).collect();
    assert_eq!(texts, vec!["1", "2"]);
}

/// Corpus for the linear-vs-line-driven splitter differential. Every source
/// the line-driven splitter can see should split identically under the
/// boundary-aware prefix parse.
const SCRIPT_FORM_DIFFERENTIAL_CORPUS: &[&str] = &[
    // Basics, including the pinned hard-error and juxtaposition splits.
    "",
    "   \n\n",
    "(* only a comment *)",
    "#!/usr/bin/env rum\n",
    "1\n2\n3",
    "a = 1\nb = a + 1\na + b",
    "1 2\n3",
    "x\ny",
    "{a\nb}",
    "  1  \n   2  ",
    "x = 1   \n   y = 2",
    // Multi-line lists, associations, definitions.
    "data = {\n  1,\n  2,\n  3\n};\nPrint[Length[data]]",
    "assoc = <|\n  \"a\" -> 1,\n  \"b\" -> 2\n|>;\nassoc[\"a\"]",
    "f[x_] := x + 1\ng[x_, y_] := f[x] f[y]\nf[1] + g[2, 3]",
    "h[1][2][3] = 4\nh[1][2][3]",
    "f[x_] /: g[x_] := x\nz = 1",
    // Trailing operators keep a form open across lines.
    "1 +\n2 +\n3",
    "2 *\n3 +\n4",
    "x = 1 +\n2;\ny = 3",
    // Unterminated strings/comments take the lex-failure fallback.
    "a = 1\nb = \"unterminated",
    "x = 1\n(* unterminated comment\ny = 2",
    "\"open\n1 + 1",
    // Hard errors at top level.
    "1 !\n3",
    "q = 1\nw = 1 ! 2",
    "1 ! 2\n3\n4",
    "w = 1 ! 2\nz = 3",
    "x\n=.",
    // Hard errors inside unclosed delimiters.
    "f[1, 2\n! 3\n4",
    "f[1, 2, ]\n3",
    "{1, 2}\n[3]",
    // Blank lines inside and around forms.
    "1\n\n\n2",
    "\n\n1 +\n\n2\n\n\n3\n\n",
    "  \n\t\n1\n",
    // Shebang.
    "#!/usr/bin/env rum\n\n1\n2",
    "#!/usr/bin/env rum\nx = 1 ! 2\ny",
    // Nested brackets.
    "a = {f[g[1, {2, 3}]],\n   h[4]}\na",
    "x = ((1 +\n2) *\n3)\nx",
    "m = {{1, 2},\n     {3, 4}}\nm[[2, 1]]",
    // Strings containing delimiters and newlines.
    "s = \"a[b;c]\nd\"\ns",
    "t = \"[ ] ; { } ( )\"\nt",
    "u = \"line1\nline2\";\nu",
    "v = \"\"\nv",
    "1 ! \"a\nb\" x\ny",
    // Comments containing delimiters, and comment-only accumulation.
    "(* [ } ; *) 1\n2",
    "1 + (* { ; *) 2\n3",
    "(* a\nb *)\nx = 1",
    "1\n(* multi\n   line *)\n2",
    "1 +\n(* keeps\n   accumulating *) 2",
    // Incomplete tails at EOF.
    "x = 1\ny +",
    "data = {",
    "f[x_] :=\n",
    "1 +\n2 *",
    "g[1,\n2,",
    "g[1, 2\n3",
    // Span sugar boundaries.
    "a;;\nb",
    "a;;b\n;;c",
    "lst[[1;;2]] = 9\nlst",
    // Compound-expression boundaries.
    "a; b\n; c",
    "a;\nb",
    "a; ; b\nc;;d",
    "1; 2; 3\n4; 5",
    // Function/postfix/operator continuations.
    "f &\n@ g",
    "x\n& y",
    "x++\ny++",
    "f[x]\n[y]",
    "p = #1 + #2 &\nq = #3 &\np",
    "x_.\ny_",
    "x_\nInteger",
    "f[x_] /;\nx > 0 := x\nf[1]",
    "a~f~b\nc~g~d",
    "x @\ny + 1",
    "x @@\ny",
    "msg::tag = \"text\"\nPrint[msg::tag]",
    "r = {a -> 1,\n     b :> 2}\nr",
    "w = 1 ! 2\nv = 2 ! 3\nu = 3",
];

#[test]
fn parse_script_forms_matches_the_line_driven_splitter_over_the_corpus() {
    let mut corpus: Vec<String> = SCRIPT_FORM_DIFFERENTIAL_CORPUS
        .iter()
        .map(|source| (*source).to_string())
        .collect();

    // CRLF variants of a representative subset (lists, trailing operators,
    // hard errors, multi-line strings, incomplete tails, span sugar).
    for source in [
        "1\n2\n3",
        "data = {\n  1,\n  2\n};\nPrint[Length[data]]",
        "1 +\n2 +\n3",
        "q = 1\nw = 1 ! 2",
        "s = \"a[b;c]\nd\"\ns",
        "x = 1\ny +",
        "a;;\nb",
        "a; b\n; c",
    ] {
        corpus.push(source.replace('\n', "\r\n"));
    }

    // Deterministic generated sources: a small LCG stitches pieces from a
    // token pool with spaces, newlines, and CRLFs, so line boundaries land
    // inside and between every construct the pool can build.
    const POOL: &[&str] = &[
        "1",
        "2",
        "x",
        "y",
        "f",
        "g",
        "+",
        "*",
        "^",
        "=",
        ":=",
        "->",
        ";",
        ",",
        "&",
        "!",
        "++",
        ";;",
        "_",
        "__",
        "#1",
        "~",
        "@",
        "/@",
        "[",
        "]",
        "{",
        "}",
        "(",
        ")",
        "<|",
        "|>",
        "[[",
        "]]",
        "/;",
        "=: ",
        ".",
        "=.",
        "\"s\"",
        "\"a\nb\"",
        "(* c *)",
        "(* a\nb *)",
        "\"",
        "(*",
    ];
    let mut state = 0x2545_f491_4f6c_dd1d_u64;

    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    for _ in 0..3000 {
        let length = 3 + (next() % 12) as usize;
        let mut source = String::new();

        for index in 0..length {
            if index > 0 {
                source.push_str(match next() % 4 {
                    0 => "\n",
                    1 => "\r\n",
                    _ => " ",
                });
            }

            source.push_str(POOL[(next() % POOL.len() as u64) as usize]);
        }

        corpus.push(source);
    }

    for source in &corpus {
        let linear = split_script_into_forms(source);
        let legacy = split_script_into_forms_line_driven(source);

        match (&linear, &legacy) {
            (Ok(linear_forms), Ok(legacy_forms)) => {
                let linear_texts: Vec<&str> = linear_forms.iter().map(|form| form.text()).collect();
                assert_eq!(
                    linear_texts, *legacy_forms,
                    "splitter disagreement on {source:?}"
                );

                // A Parsed form is lowered without ever re-parsing its text,
                // so that text must parse standalone: if it ever failed,
                // evaluation would silently stop surfacing the per-form
                // syntax error the old always-parse loop reported.
                for form in linear_forms {
                    if let rum_syntax::ScriptForm::Parsed { text, .. } = form {
                        assert!(
                            rum_syntax::parse_expr(text).is_ok(),
                            "parsed form text does not parse standalone: {text:?}"
                        );
                    }
                }
            }
            (Ok(linear_forms), Err(legacy_err)) => panic!(
                "linear splitter returned {linear_forms:?} but line-driven \
                 failed with {legacy_err:?} on {source:?}"
            ),
            (Err(linear_err), Ok(legacy_forms)) => panic!(
                "linear splitter failed with {linear_err:?} but line-driven \
                 returned {legacy_forms:?} on {source:?}"
            ),
            (Err(linear_err), Err(legacy_err)) => assert_eq!(
                format!("{linear_err:?}"),
                format!("{legacy_err:?}"),
                "tail error disagreement on {source:?}"
            ),
        }

        // A source whose whole-source lex succeeds must take the linear
        // path; the fallback exists for lex failures only.
        if rum_syntax::lex_with_trivia(source).is_ok() {
            assert!(
                !matches!(
                    rum_syntax::parse_script_forms(source),
                    Err(rum_syntax::ScriptFormsError::LexFailed(_))
                ),
                "clean source took the lex-failure fallback: {source:?}"
            );
        }
    }
}

#[test]
fn eval_script_pins_the_second_form_syntax_error_message() {
    // A hard-error form stays text-only, so evaluation still fails at the
    // same token with the same message. As of brief-385, its span is the
    // token's file coordinate; pin it so two-state form handling cannot
    // silently alter diagnostics or restore the former form-local anchor.
    let mut session = session_with_test_control_builtins();

    let error = session
        .eval_script("x = 1\nw = 2 !")
        .expect_err("the second form has a trailing-input syntax error");

    assert!(matches!(error, EvalError::Syntax(_)));
    assert_eq!(error.to_string(), "trailing input at bytes 12..13");
    // The first form still ran before the error surfaced.
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
}

#[test]
fn lex_fallback_preserves_normalized_crlf_values_and_file_error_spans() {
    let mut session = session_with_test_control_builtins();

    let error = session
        .eval_script("s = \"a\r\nb\"\r\n\\")
        .expect_err("the final form contains an unexpected character");

    assert_eq!(
        error.to_string(),
        "unexpected character `\\` at bytes 12..13"
    );
    assert_eq!(session.eval_text("s").unwrap().value, Expr::string("a\nb"));
}

#[test]
fn eval_script_preserves_messages_emitted_before_a_later_error() {
    let mut session = session_with_test_control_builtins();

    let error = session
        .eval_script("FindFile[\"Missing`\"]\n1 ! 2")
        .expect_err("expected a later evaluation error");

    assert!(matches!(error, EvalError::Syntax(_)));
    assert!(
        session
            .take_messages()
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
}
