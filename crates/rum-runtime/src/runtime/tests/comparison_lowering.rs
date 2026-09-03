use super::*;
use rum_syntax::{MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS, Span};

#[test]
fn comparison_lowering_preserves_resolver_call_count_and_order() {
    let syntax = parse_expr("(f[a] < b) <= (c == g[d])")
        .expect("resolver-order comparison source should parse");
    let mut calls = Vec::new();
    let lowered = lower_syntax_expr_with(&syntax, &mut |raw| {
        calls.push(raw.to_owned());
        None
    })
    .expect("resolver-order comparison source should lower");

    assert_eq!(calls.len(), 6);
    assert_eq!(
        calls,
        vec![
            "f".to_owned(),
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "g".to_owned(),
            "d".to_owned(),
        ]
    );
    assert_eq!(
        lowered.to_string(),
        "LessEqual[Less[f[a], b], Equal[c, g[d]]]"
    );
}

#[test]
fn comparison_lowering_accepts_a_uniform_chain_at_the_boundary() {
    const LINK_COUNT: usize = MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS;

    let source = comparison_source(LINK_COUNT);
    let syntax = parse_expr(&source).expect("boundary comparison source should parse");
    let lowered = lower_syntax_expr(&syntax).expect("boundary comparison source should lower");

    let Expr::Call { head, args } = lowered else {
        panic!("boundary comparison should lower to a call");
    };
    assert!(is_system_symbol_name(&head, "Equal"));
    assert_eq!(args.len(), LINK_COUNT + 1);
    assert!(args.iter().all(is_global_a));
}

#[test]
fn comparison_parser_rejects_a_long_uniform_chain_gracefully() {
    const LINK_COUNT: usize = 50_000;

    let source = comparison_source(LINK_COUNT);
    let error = parse_expr(&source)
        .expect_err("long comparison source should be rejected without aborting");

    assert!(matches!(
        error,
        SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            ..
        }
    ));
    assert!(!error.is_incomplete());
}

#[test]
fn comparison_lowering_bounds_direct_chain_nodes() {
    let accepted = hand_built_comparison_chain(MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS, 1_000);
    let lowered = lower_syntax_expr(&accepted)
        .expect("a direct comparison chain at the parser limit should lower");
    let Expr::Call { head, args } = &lowered else {
        panic!("direct comparison tree should lower to a call");
    };
    assert!(is_system_symbol_name(head, "Equal"));
    assert_eq!(args.len(), MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    drop(accepted);
    drop(lowered);

    let rejected = hand_built_comparison_chain(MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1, 20_000);
    let error = lower_syntax_expr(&rejected)
        .expect_err("a direct comparison chain above the parser limit should be rejected");
    assert!(matches!(
        error,
        EvalError::Syntax(SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            ..
        })
    ));
    drop(rejected);
}

#[test]
fn comparison_lowering_keeps_parenthesized_subtrees_independent() {
    let side = comparison_source(2_048);
    let source = format!("({side}) == ({side})");
    let syntax =
        parse_expr(&source).expect("independent parenthesized comparison spines should parse");
    let lowered = lower_syntax_expr(&syntax)
        .expect("independent parenthesized chains should not share a link budget");

    let Expr::Call { head, args } = &lowered else {
        panic!("outer parenthesized comparison should lower to a call");
    };
    assert!(is_system_symbol_name(head, "Equal"));
    assert_eq!(args.len(), 2);
    for arg in args {
        let Expr::Call {
            head: inner_head,
            args: inner_args,
        } = arg
        else {
            panic!("parenthesized chain should remain an inner call");
        };
        assert!(is_system_symbol_name(inner_head, "Equal"));
        assert_eq!(inner_args.len(), 2_049);
    }
}

fn comparison_source(link_count: usize) -> String {
    let mut source = String::with_capacity(link_count * 5 + 1);
    for index in 0..=link_count {
        if index != 0 {
            source.push_str(" == ");
        }
        source.push('a');
    }
    source
}

fn hand_built_comparison_chain(link_count: usize, span_start: usize) -> SyntaxExpr {
    let operands = (0..=link_count)
        .map(|index| SyntaxExpr::symbol("a", Span::point(span_start + index)))
        .collect();
    SyntaxExpr::comparison_chain(
        operands,
        vec![BinaryOp::Equal; link_count],
        Span::new(span_start, span_start + link_count + 1),
    )
}

fn is_global_a(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Atom(Atom::Symbol(symbol))
            if symbol.context() == GLOBAL_CONTEXT && symbol.name() == "a"
    )
}
