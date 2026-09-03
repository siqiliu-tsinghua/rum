use super::support::*;

#[test]
fn parses_large_integer_literal() {
    let parsed = parse_expr("123456789012345678901234567890").unwrap();
    assert_eq!(
        parsed,
        SyntaxExpr::integer("123456789012345678901234567890", z())
    );
}

#[test]
fn parses_simple_decimal_literal_as_numeric_atom() {
    let parsed = parse_expr("0.01").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer("0.01", z()));

    let parsed = parse_expr(".5").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer(".5", z()));

    let parsed = parse_expr("1.").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer("1.", z()));

    let parsed = parse_expr("1.5e3").unwrap();
    assert_eq!(parsed, SyntaxExpr::integer("1.5e3", z()));
}

#[test]
fn parses_lists_and_nested_calls() {
    let parsed = parse_expr("f[{1, 2}, g[x]]").unwrap();
    assert_eq!(parsed.to_string(), "f[{1, 2}, g[x]]");
}

#[test]
fn parses_multiline_string_literals_as_string_atoms() {
    assert_eq!(
        parse_expr("\"\"\"\nab\"\"\"").unwrap(),
        SyntaxExpr::string("ab", z())
    );
    assert_eq!(
        parse_expr("\"\"\"a \" b \"\" c\"\"\"").unwrap(),
        SyntaxExpr::string("a \" b \"\" c", z())
    );
    assert_eq!(
        parse_expr("x = '''\nC:\\path \\t raw\n'''").unwrap(),
        SyntaxExpr::binary(
            BinaryOp::Set,
            SyntaxExpr::symbol("x", z()),
            SyntaxExpr::string("C:\\path \\t raw\n", z()),
            z(),
        )
    );
}

#[test]
fn rejects_too_deep_nested_call_without_panicking() {
    let depth = MAX_DELIMITER_NESTING_DEPTH + 1;
    let mut input = String::with_capacity(depth * 3 + 1);
    for _ in 0..depth {
        input.push_str("f[");
    }
    input.push('x');
    for _ in 0..depth {
        input.push(']');
    }

    let error = parse_expr(&input).expect_err("too-deep call should be a parse error");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_DELIMITER_NESTING_DEPTH,
            ..
        }
    ));
}

#[test]
fn pins_nested_call_parse_depth_boundary() {
    fn nested_call(depth: usize) -> String {
        let mut input = String::with_capacity(depth * 3 + 1);
        for _ in 0..depth {
            input.push_str("f[");
        }
        input.push('x');
        for _ in 0..depth {
            input.push(']');
        }
        input
    }

    let accepted = nested_call(MAX_DELIMITER_NESTING_DEPTH - 1);
    assert_eq!(parse_expr(&accepted).unwrap().to_string(), accepted);

    let rejected = nested_call(MAX_DELIMITER_NESTING_DEPTH + 1);
    let error = parse_expr(&rejected).expect_err("depth should exceed the parser guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_DELIMITER_NESTING_DEPTH,
            ..
        }
    ));
}

#[test]
fn rejects_deep_postfix_blank_chain_without_panicking() {
    // Alternating symbol/blank pairs: consecutive underscores would lex as
    // multi-blank tokens (`___`), which are one chain link each, not three.
    fn blank_chain(links: usize) -> String {
        let mut input = String::from("a");
        for _ in 0..links {
            input.push('_');
            input.push('b');
        }
        input
    }

    let accepted = blank_chain(MAX_OPERATOR_CHAIN_DEPTH);
    assert!(parse_expr(&accepted).is_ok());

    let rejected = blank_chain(MAX_OPERATOR_CHAIN_DEPTH + 1);
    let error = parse_expr(&rejected).expect_err("blank chain should exceed the parser guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_OPERATOR_CHAIN_DEPTH,
            ..
        }
    ));
}

#[test]
fn rejects_deep_right_associative_chain_without_panicking() {
    let mut input = String::with_capacity(32_001);
    input.push('a');
    for _ in 0..16_000 {
        input.push_str("^a");
    }

    let error = parse_expr(&input).expect_err("deep power chain should exceed the parser guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_OPERATOR_CHAIN_DEPTH,
            ..
        }
    ));
}

#[test]
fn rejects_deep_prefix_chain_without_panicking() {
    let input = format!("{}x", "!".repeat(16_000));

    let error = parse_expr(&input).expect_err("deep prefix chain should exceed the parser guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_OPERATOR_CHAIN_DEPTH,
            ..
        }
    ));
}

#[test]
fn pins_power_chain_operator_depth_boundary() {
    fn power_chain(links: usize, rendered: bool) -> String {
        let separator = if rendered { " ^ a" } else { "^a" };
        let mut input = String::with_capacity(links * 4 + 1);
        input.push('a');
        for _ in 0..links {
            input.push_str(separator);
        }
        input
    }

    let accepted = power_chain(MAX_OPERATOR_CHAIN_DEPTH - 1, true);
    let source = power_chain(MAX_OPERATOR_CHAIN_DEPTH - 1, false);
    assert_eq!(parse_expr(&source).unwrap().to_string(), accepted);

    let rejected = power_chain(MAX_OPERATOR_CHAIN_DEPTH + 1, false);
    let error = parse_expr(&rejected).expect_err("chain should exceed the operator-chain guard");
    assert!(matches!(
        error,
        SyntaxError::ParseDepthExceeded {
            max: MAX_OPERATOR_CHAIN_DEPTH,
            ..
        }
    ));
}

#[test]
fn parses_association_literals() {
    let parsed = parse_expr("<|a -> 1, b -> <|c -> 3|>|>").unwrap();
    assert_eq!(parsed.to_string(), "<|a -> 1, b -> <|c -> 3|>|>");
}
