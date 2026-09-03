use super::*;
use rum_syntax::{FormatterOptions, MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS, Span, format_parsed_source};

const AFFECTED_OPERATORS: [(BinaryOp, &str); 16] = [
    (BinaryOp::Add, "+"),
    (BinaryOp::Subtract, "-"),
    (BinaryOp::Times, "*"),
    (BinaryOp::Divide, "/"),
    (BinaryOp::Dot, "."),
    (BinaryOp::NonCommutativeMultiply, "**"),
    (BinaryOp::StringJoin, "<>"),
    (BinaryOp::MessageName, "::"),
    (BinaryOp::ReplaceAll, "/."),
    (BinaryOp::ReplaceRepeated, "//."),
    (BinaryOp::Pattern, ":"),
    (BinaryOp::Alternatives, "|"),
    (BinaryOp::PatternTest, "?"),
    (BinaryOp::StringExpression, "~~"),
    (BinaryOp::PostfixApply, "//"),
    (BinaryOp::RightComposition, "/*"),
];

#[test]
fn left_chain_lowering_preserves_cross_family_shape_and_resolver_order() {
    let syntax = parse_expr("a . b * c + d /. r //. s")
        .expect("cross-family left-chain source should parse");
    let mut calls = Vec::new();
    let lowered = lower_syntax_expr_with(&syntax, &mut |raw| {
        calls.push(raw.to_owned());
        None
    })
    .expect("cross-family left-chain source should lower");

    assert_eq!(calls, ["a", "b", "c", "d", "r", "s"]);
    assert_eq!(
        lowered.to_string(),
        "ReplaceRepeated[ReplaceAll[Plus[Times[Dot[a, b], c], d], r], s]"
    );
}

#[test]
fn left_chain_lowering_preserves_message_name_double_resolution_order() {
    let syntax = parse_expr("owner::first::tag[inner]").expect("message-name chain should parse");
    let mut calls = Vec::new();
    let lowered = lower_syntax_expr_with(&syntax, &mut |raw| {
        calls.push(raw.to_owned());
        None
    })
    .expect("message-name chain should lower");

    assert_eq!(calls, ["owner", "first", "tag", "inner", "tag", "inner"]);
    assert_eq!(lowered.to_string(), "MessageName[owner::first, tag[inner]]");
}

#[test]
fn left_chain_lowering_keeps_direct_canonical_calls_opaque() {
    let cases = [
        ("Plus[a, b] + c + d", "Plus[Plus[a, b], c, d]"),
        ("a + Plus[b, c] + d", "Plus[a, Plus[b, c], d]"),
        ("Times[a, b] * c * d", "Times[Times[a, b], c, d]"),
        ("a * Times[b, c] * d", "Times[a, Times[b, c], d]"),
        (
            "RightComposition[f, g] /* h /* k",
            "RightComposition[RightComposition[f, g], h, k]",
        ),
        ("Dot[a, b] . c . d", "Dot[Dot[a, b], c, d]"),
        (
            "StringJoin[a, b] <> c <> d",
            "StringJoin[StringJoin[a, b], c, d]",
        ),
        (
            "Alternatives[a, b] | c | d",
            "Alternatives[Alternatives[a, b], c, d]",
        ),
        (
            "StringExpression[a, b] ~~ c ~~ d",
            "StringExpression[StringExpression[a, b], c, d]",
        ),
    ];

    for (source, expected) in cases {
        let syntax = parse_expr(source).expect("direct-call chain should parse");
        let lowered = lower_syntax_expr(&syntax).expect("direct-call chain should lower");
        assert_eq!(lowered.to_string(), expected, "{source}");
    }
}

#[test]
fn held_sameq_explicitly_observes_flat_chain_shapes() {
    let mut session = Session::new();
    crate::runtime::test_builtins::register_test_sameq_builtin(&mut session);

    let cases = [
        (
            "SameQ[HoldComplete[a + b + c], HoldComplete[Plus[a, b, c]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[a + b + c], HoldComplete[Plus[Plus[a, b], c]]]",
            "False",
        ),
        (
            "SameQ[HoldComplete[a * b * c], HoldComplete[Times[a, b, c]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[a * b * c], HoldComplete[Times[Times[a, b], c]]]",
            "False",
        ),
        (
            "SameQ[HoldComplete[f /* g /* h], HoldComplete[System`RightComposition[f, g, h]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[f /* g /* h], HoldComplete[System`RightComposition[System`RightComposition[f, g], h]]]",
            "False",
        ),
    ];

    for (source, expected) in cases {
        let value = session
            .eval_text(source)
            .expect("held SameQ case should evaluate");
        assert_eq!(value.value.to_string(), expected, "{source}");
    }
}

#[test]
fn held_sameq_explicitly_observes_variadic_non_flat_chain_shapes() {
    let mut session = Session::new();
    crate::runtime::test_builtins::register_test_sameq_builtin(&mut session);

    let cases = [
        (
            "SameQ[HoldComplete[a . b . c], HoldComplete[System`Dot[a, b, c]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[a . b . c], HoldComplete[System`Dot[System`Dot[a, b], c]]]",
            "False",
        ),
        (
            "SameQ[HoldComplete[a <> b <> c], HoldComplete[System`StringJoin[a, b, c]]]",
            "True",
        ),
        (
            "SameQ[HoldComplete[a <> b <> c], HoldComplete[System`StringJoin[System`StringJoin[a, b], c]]]",
            "False",
        ),
    ];

    for (source, expected) in cases {
        let value = session
            .eval_text(source)
            .expect("held variadic non-Flat SameQ case should evaluate");
        assert_eq!(value.value.to_string(), expected, "{source}");
    }
}

#[test]
fn variadic_non_flat_chain_matching_shapes_are_explicit() {
    let mut session = session_with_test_rewrite_builtins();
    let cases = [
        ("MatchQ[b, a | b | c]", "True"),
        ("MatchQ[c, a | b | c]", "True"),
        ("MatchQ[d, a | b | c]", "False"),
        ("MatchQ[b, Alternatives[Alternatives[a, b], c]]", "True"),
        ("MatchQ[c, Alternatives[Alternatives[a, b], c]]", "True"),
        ("MatchQ[d, Alternatives[Alternatives[a, b], c]]", "False"),
        (
            "MatchQ[HoldComplete[a <> b <> c], HoldComplete[Verbatim[System`StringJoin][_, _]]]",
            "False",
        ),
        (
            "MatchQ[HoldComplete[a <> b <> c], HoldComplete[Verbatim[System`StringJoin][_, _, _]]]",
            "True",
        ),
        (
            "MatchQ[HoldComplete[a ~~ b ~~ c], HoldComplete[Verbatim[System`StringExpression][_, _]]]",
            "False",
        ),
        (
            "MatchQ[HoldComplete[a ~~ b ~~ c], HoldComplete[Verbatim[System`StringExpression][_, _, _]]]",
            "True",
        ),
    ];

    for (source, expected) in cases {
        let value = session
            .eval_text(source)
            .expect("variadic non-Flat matching case should evaluate");
        assert_eq!(value.value.to_string(), expected, "{source}");
    }
}

#[test]
fn flat_chain_evaluated_results_remain_unchanged() {
    let mut session = session_with_test_rewrite_builtins();
    let cases = [
        ("1 + 2 + 3 + 4", "10"),
        ("2 * 3 * 4", "24"),
        ("10 - 3 - 2", "5"),
        ("100 / 5 / 2", "10"),
        ("1 + 2 - 3 + 4 - 5", "-1"),
        ("(f /* g)[x]", "g[f[x]]"),
        ("1/2 + 1/3 + 1/6", "1"),
        ("2 * 3 / 4 * 5", "15/2"),
        ("84 / 7 / 3 / 2 / 2", "1"),
        ("(f /* g /* h /* k)[x]", "k[h[g[f[x]]]]"),
    ];

    for (source, expected) in cases {
        let value = session
            .eval_text(source)
            .expect("flat-chain result case should evaluate");
        assert_eq!(value.value.to_string(), expected, "{source}");
    }
}

#[test]
fn left_chain_lowering_accepts_the_boundary_for_all_operators() {
    const LINK_COUNT: usize = MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS;

    for (op, token) in AFFECTED_OPERATORS {
        let source = repeated_chain_source(token, LINK_COUNT);
        let syntax = parse_expr(&source)
            .unwrap_or_else(|error| panic!("boundary {token:?} left chain should parse: {error}"));
        let lowered = match lower_syntax_expr(&syntax) {
            Ok(lowered) => lowered,
            Err(error) => {
                drop_left_binary_chain(syntax);
                panic!("boundary {token:?} left chain should lower: {error}");
            }
        };

        assert_homogeneous_chain_shape(&lowered, op, LINK_COUNT);
        drop_left_binary_chain(syntax);
        drop_expr_iteratively(lowered);
    }
}

#[test]
fn left_chain_parser_rejects_20_000_links_gracefully_for_all_operators() {
    const LINK_COUNT: usize = 20_000;

    for (_, token) in AFFECTED_OPERATORS {
        let source = repeated_chain_source(token, LINK_COUNT);
        let error = parse_expr(&source)
            .expect_err("20,000-link source chain should be rejected without aborting");
        assert!(matches!(
            error,
            SyntaxError::LeftAssociativeChainTooLong {
                max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
                ..
            }
        ));
        assert!(!error.is_incomplete(), "{token:?}");
    }
}

#[test]
fn left_chain_lowering_accepts_a_boundary_hand_built_alternating_ast() {
    const LINK_COUNT: usize = MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS;

    let mut syntax = SyntaxExpr::symbol("a", Span::default());
    for index in 0..LINK_COUNT {
        syntax = SyntaxExpr::binary(
            AFFECTED_OPERATORS[index % AFFECTED_OPERATORS.len()].0,
            syntax,
            SyntaxExpr::symbol("a", Span::default()),
            Span::default(),
        );
    }

    let lowered = lower_syntax_expr(&syntax).expect("alternating direct AST should lower");
    let mut current = &lowered;
    for _ in 0..(LINK_COUNT / AFFECTED_OPERATORS.len()) {
        for (op, _) in AFFECTED_OPERATORS[4..].iter().rev() {
            current = assert_link_shape(current, *op, false);
        }
        current = assert_flat_multiplicative_pair(current);
        current = assert_flat_additive_pair(current);
    }
    assert_global_a(current);

    drop_left_binary_chain(syntax);
    drop_expr_iteratively(lowered);
}

#[test]
fn left_chain_lowering_rejects_a_20_000_link_hand_built_ast_gracefully() {
    const LINK_COUNT: usize = 20_000;

    let mut syntax = SyntaxExpr::symbol("a", Span::default());
    for index in 0..LINK_COUNT {
        syntax = SyntaxExpr::binary(
            AFFECTED_OPERATORS[index % AFFECTED_OPERATORS.len()].0,
            syntax,
            SyntaxExpr::symbol("a", Span::point(index)),
            Span::new(index, index + 1),
        );
    }

    let error = lower_syntax_expr(&syntax)
        .expect_err("20,000-link direct AST should be rejected without aborting");
    let rejected_index = LINK_COUNT - MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS - 1;
    assert!(matches!(
        error,
        EvalError::Syntax(SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            span,
        }) if span == Span::new(rejected_index, rejected_index + 1)
    ));

    drop_left_binary_chain(syntax);
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort() {
    let accepted = run_on_two_mib_thread("left-chain-accepted", || {
        let source = repeated_chain_source("+", MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
        let parsed = rum_syntax::parse_with_diagnostics(source)
            .expect("4,096-link source should parse on a 2 MiB stack");
        let lowered = lower_syntax_expr(parsed.syntax())
            .expect("4,096-link syntax should lower on a 2 MiB stack");
        assert_homogeneous_chain_shape(&lowered, BinaryOp::Add, MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
        let formatted = format_parsed_source(&parsed, FormatterOptions::default());
        assert!(!formatted.is_empty());

        drop_expr_iteratively(lowered);
        drop(formatted);
        drop(parsed);
        "2 MiB accepted: 4096 links parsed, lowered, formatted, and dropped".to_string()
    });
    println!("{accepted}");

    for link_count in [MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1, 20_000] {
        let result = run_on_two_mib_thread("left-chain-rejected", move || {
            let source = repeated_chain_source("+", link_count);
            let diagnostic = match rum_syntax::parse_with_diagnostics(source) {
                Ok(parsed) => {
                    drop(parsed);
                    panic!("{link_count}-link source should be rejected");
                }
                Err(diagnostic) => diagnostic,
            };
            let span = match diagnostic.error() {
                SyntaxError::LeftAssociativeChainTooLong {
                    max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
                    span,
                } => *span,
                error => panic!("unexpected {link_count}-link error: {error}"),
            };
            assert_eq!(diagnostic.source().text(span), Some("+"));
            let rendered = diagnostic.render();
            assert!(
                rendered.contains("error: left-associative chain exceeded limit 4096 at bytes")
            );
            assert!(
                rendered
                    .lines()
                    .last()
                    .is_some_and(|line| line.ends_with('^'))
            );

            drop(rendered);
            drop(diagnostic);
            format!("2 MiB rejected: {link_count} links diagnosed and dropped")
        });
        println!("{result}");
    }
}

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB probe thread should spawn")
        .join()
        .unwrap_or_else(|_| panic!("2 MiB probe thread {name:?} panicked"))
}

fn repeated_chain_source(token: &str, link_count: usize) -> String {
    let mut source = String::with_capacity(1 + link_count * (token.len() + 3));
    source.push('a');
    for _ in 0..link_count {
        source.push(' ');
        source.push_str(token);
        source.push_str(" a");
    }
    source
}

fn assert_homogeneous_chain_shape(expr: &Expr, op: BinaryOp, link_count: usize) {
    if matches!(
        op,
        BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Times
            | BinaryOp::Divide
            | BinaryOp::Dot
            | BinaryOp::StringJoin
            | BinaryOp::Alternatives
            | BinaryOp::StringExpression
            | BinaryOp::RightComposition
    ) {
        assert_flat_homogeneous_chain_shape(expr, op, link_count);
        return;
    }

    let mut current = expr;
    for index in (0..link_count).rev() {
        current = assert_link_shape(current, op, index == 0);
    }
    assert_global_a(current);
}

fn assert_flat_homogeneous_chain_shape(expr: &Expr, op: BinaryOp, link_count: usize) {
    let Expr::Call { head, args } = expr else {
        panic!("flat left chain should lower to a call");
    };
    let expected_head = match op {
        BinaryOp::Add | BinaryOp::Subtract => heads::arithmetic::PLUS,
        BinaryOp::Times | BinaryOp::Divide => heads::arithmetic::TIMES,
        BinaryOp::Dot => "Dot",
        BinaryOp::StringJoin => "StringJoin",
        BinaryOp::Alternatives => "Alternatives",
        BinaryOp::StringExpression => "StringExpression",
        BinaryOp::RightComposition => "RightComposition",
        _ => unreachable!("only flattened operators reach this helper"),
    };
    assert!(is_system_symbol_name(head, expected_head));
    assert_eq!(args.len(), link_count + 1);
    assert_global_a(&args[0]);

    for arg in &args[1..] {
        match op {
            BinaryOp::Add
            | BinaryOp::Times
            | BinaryOp::Dot
            | BinaryOp::StringJoin
            | BinaryOp::Alternatives
            | BinaryOp::StringExpression
            | BinaryOp::RightComposition => {
                assert_global_a(arg);
            }
            BinaryOp::Subtract => assert_negated_global_a(arg),
            BinaryOp::Divide => assert_reciprocal_global_a(arg),
            _ => unreachable!("only flattened operators reach this helper"),
        }
    }
}

fn assert_flat_additive_pair(expr: &Expr) -> &Expr {
    let Expr::Call { head, args } = expr else {
        panic!("additive pair should lower to a call");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::PLUS));
    assert_eq!(args.len(), 3);
    assert_global_a(&args[1]);
    assert_negated_global_a(&args[2]);
    &args[0]
}

fn assert_flat_multiplicative_pair(expr: &Expr) -> &Expr {
    let Expr::Call { head, args } = expr else {
        panic!("multiplicative pair should lower to a call");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::TIMES));
    assert_eq!(args.len(), 3);
    assert_global_a(&args[1]);
    assert_reciprocal_global_a(&args[2]);
    &args[0]
}

fn assert_negated_global_a(expr: &Expr) {
    let Expr::Call { head, args } = expr else {
        panic!("subtracted term should lower to Times[-1, a]");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::TIMES));
    assert_eq!(args.len(), 2);
    assert_eq!(args[0], Expr::integer(BigInt::from(-1)));
    assert_global_a(&args[1]);
}

fn assert_reciprocal_global_a(expr: &Expr) {
    let Expr::Call { head, args } = expr else {
        panic!("divisor should lower to Power[a, -1]");
    };
    assert!(is_system_symbol_name(head, heads::arithmetic::POWER));
    assert_eq!(args.len(), 2);
    assert_global_a(&args[0]);
    assert_eq!(args[1], Expr::integer(BigInt::from(-1)));
}

fn assert_link_shape(expr: &Expr, op: BinaryOp, pattern_lhs_was_symbol: bool) -> &Expr {
    let Expr::Call { head, args } = expr else {
        panic!("left-chain link should lower to a call");
    };

    if op == BinaryOp::PostfixApply {
        assert_global_a(head);
        assert_eq!(args.len(), 1);
        return &args[0];
    }

    let expected_head = match op {
        BinaryOp::Add | BinaryOp::Subtract => heads::arithmetic::PLUS,
        BinaryOp::Times | BinaryOp::Divide => heads::arithmetic::TIMES,
        BinaryOp::Dot => "Dot",
        BinaryOp::NonCommutativeMultiply => "NonCommutativeMultiply",
        BinaryOp::StringJoin => "StringJoin",
        BinaryOp::MessageName => heads::messages::MESSAGE_NAME,
        BinaryOp::ReplaceAll => heads::rules::REPLACE_ALL,
        BinaryOp::ReplaceRepeated => heads::rules::REPLACE_REPEATED,
        BinaryOp::Pattern if pattern_lhs_was_symbol => heads::patterns::PATTERN,
        BinaryOp::Pattern => "Optional",
        BinaryOp::Alternatives => "Alternatives",
        BinaryOp::PatternTest => "PatternTest",
        BinaryOp::StringExpression => "StringExpression",
        BinaryOp::RightComposition => "RightComposition",
        _ => panic!("unexpected left-chain operator {op:?}"),
    };

    assert!(is_system_symbol_name(head, expected_head));
    assert_eq!(args.len(), 2);
    &args[0]
}

fn assert_global_a(expr: &Expr) {
    assert!(matches!(
        expr,
        Expr::Atom(Atom::Symbol(symbol))
            if symbol.context() == GLOBAL_CONTEXT && symbol.name() == "a"
    ));
}

fn drop_left_binary_chain(mut expr: SyntaxExpr) {
    loop {
        match expr.kind {
            SyntaxExprKind::Binary { lhs, rhs, .. } => {
                drop(rhs);
                expr = *lhs;
            }
            _ => return,
        }
    }
}

fn drop_expr_iteratively(expr: Expr) {
    let mut work = vec![expr];
    while let Some(expr) = work.pop() {
        if let Some((head, args)) = expr.into_call_parts() {
            work.push(head);
            work.extend(args);
        }
    }
}

/// The 2 MiB round trip above uses `+`, which lowers to one flat `Plus` and so
/// never exercises a recursive `Expr` drop. The seven operators that stay
/// nested build a 4,096-deep `Expr`, and that is what a host actually drops at
/// the accepted boundary — so it is dropped naturally here, not dismantled.
#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary() {
    for token in ["/.", "//.", "//", "?", "::", "**", ":"] {
        let message = run_on_two_mib_thread("nested-natural-drop", move || {
            let source = repeated_chain_source(token, MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
            let parsed = rum_syntax::parse_with_diagnostics(source)
                .expect("4,096-link source should parse on a 2 MiB stack");
            let lowered = lower_syntax_expr(parsed.syntax())
                .expect("4,096-link syntax should lower on a 2 MiB stack");
            let formatted = format_parsed_source(&parsed, FormatterOptions::default());
            assert!(!formatted.is_empty());

            drop(lowered);
            drop(formatted);
            drop(parsed);
            format!(
                "2 MiB nested `{token}`: 4096 links parsed, lowered, formatted, dropped naturally"
            )
        });
        println!("{message}");
    }
}
