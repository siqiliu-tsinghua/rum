use super::*;

fn branched_fixture(links: usize) -> (SyntaxExpr, Expr) {
    // The unique deepest leaf is a decimal: preserving the syntax atom instead
    // of actually lowering it cannot satisfy the exact Rational assertion.
    let mut source = String::from("f[branch[{0, g[h[0.125]]}]]");
    for index in 1..links {
        source.push_str(&format!("[branch[{{{index}, g[h[{index}]]}}]]"));
    }
    let syntax = parse_expr(&source).unwrap();
    // Build the expected value independently of syntax lowering.
    let branch = |index: usize, leaf: Expr| {
        Expr::call(
            Expr::symbol("branch"),
            vec![canonical_call(
                "List",
                vec![
                    Expr::integer(BigInt::from(index)),
                    Expr::call(
                        Expr::symbol("g"),
                        vec![Expr::call(Expr::symbol("h"), vec![leaf])],
                    ),
                ],
            )],
        )
    };
    let mut value = Expr::call(
        Expr::symbol("f"),
        vec![branch(
            0,
            Expr::rational(Rational::new(BigInt::from(1), BigInt::from(8)).unwrap()),
        )],
    );
    for index in 1..links {
        value = Expr::call(
            value,
            vec![branch(index, Expr::integer(BigInt::from(index)))],
        );
    }
    (syntax, value)
}

struct TestLowered {
    value: Expr,
    used_tail: bool,
}

fn lower_with_front(syntax: &SyntaxExpr, front: usize) -> TestLowered {
    let lowered = lower_syntax_expr_with_front_depth(
        syntax,
        &mut |raw| resolve_unqualified_symbol(raw).or_else(|| Some(symbol_from_raw_name(raw))),
        front,
    )
    .unwrap();
    match lowered {
        LoweredSyntax::Front(value) => TestLowered {
            value,
            used_tail: false,
        },
        LoweredSyntax::Tail(value) => TestLowered {
            value,
            used_tail: true,
        },
    }
}

#[test]
fn branched_lowering_matches_original_exact_result() {
    for links in [4, 24] {
        let (syntax, expected) = branched_fixture(links);
        assert_eq!(lower_syntax_expr(&syntax).unwrap(), expected);
        let tail = lower_with_front(&syntax, 0);
        let front = lower_with_front(&syntax, 16);
        assert_eq!(tail.value, expected, "front zero must lower the whole tree");
        assert_eq!(
            front.value, expected,
            "front sixteen must preserve every branch"
        );
        assert_eq!(tail.value, front.value);
        let mut deepest = &tail.value;
        for _ in 1..links {
            let Expr::Call { head, .. } = deepest else {
                panic!("missing call head")
            };
            deepest = head;
        }
        for index in [0, 0, 1, 0, 0] {
            let Expr::Call { args, .. } = deepest else {
                panic!("missing nested argument")
            };
            deepest = &args[index];
        }
        assert_eq!(
            deepest,
            &Expr::rational(Rational::new(BigInt::from(1), BigInt::from(8)).unwrap()),
            "the unique deepest decimal must become exactly Rational[1, 8]"
        );
    }
}

#[test]
fn branched_lowering_certifies_front_and_tail() {
    for links in [4, 24] {
        let (syntax, expected) = branched_fixture(links);
        let tail = lower_with_front(&syntax, 0);
        let front = lower_with_front(&syntax, 16);
        assert_eq!(tail.value, expected, "front zero exact result");
        assert_eq!(front.value, expected, "front sixteen exact result");
        assert!(tail.used_tail, "front zero must execute the iterative tail");
        assert_eq!(front.used_tail, links > 16, "front sixteen handoff");
    }
}

#[test]
fn tail_preserves_sugar_and_symbol_resolution_order() {
    // Canonical forms from docs/lowering-canonical-forms.md, including family
    // transitions that distinguish syntax flattening from direct calls.
    let sources = [
        "{}",
        "<||>",
        "{a, b}",
        "<|a -> 1, b :> 2|>",
        "{_, __, ___, _Integer, x_Integer, #, #2, #name, ##, ##2}",
        "{+a, -3, -0.125, -x, !x, ++x, --x}",
        "{x &, x++, x--, x!, x!!, f'', x .., x ..., x =.}",
        "{a = b, a := b, a ^= b, a ^:= b, a += b, a -= b, a *= b, a /= b, a //= b}",
        "{a -> b, a :> b, a /; b, a || b, a && b, f @ x, f /@ x, f //@ x, f @@ x, f @@@ x}",
        "{f @* g, a^b, a |-> b, a === b, a =!= b, a == b, a != b, a < b, a > b, a <= b, a >= b}",
        "{a < b <= c == d, a == b == c, (a == b) == c}",
        "{a + b - c + d, a * b / c * d, a / b * c / d, 8 / 2 / 4, 8 * 2 / 4, a / 0}",
        "{Plus[a, b] + c, Times[a, b] * c, a * Times[b, c], a / (b + c)}",
        "{a . b . c, a ** b ** c, a <> b <> c, a ~~ b ~~ c, a | b | c, a ? b ? c}",
        "{a : b : c, a /. b /. c, a //. b //. c, a // f // g, f /* g /* h}",
        "{a::b::c, f[x]::tag, f[x]::\"tag\"}",
        "{a; b; c, (a; b); c, CompoundExpression[a, b]; c}",
        "{f /: g[x_] = rhs, f /: g[x_] := rhs, f /: g[x_] =.}",
    ];
    for source in sources {
        let syntax = parse_expr(source).unwrap();
        let mut schedules = Vec::new();
        let mut results = Vec::new();
        for front in [0, 16] {
            let mut seen = Vec::new();
            let lowered = lower_syntax_expr_with_front_depth(
                &syntax,
                &mut |raw| {
                    seen.push(raw.to_owned());
                    Some(Symbol::new("Certificate`", raw))
                },
                front,
            )
            .unwrap();
            schedules.push(seen);
            results.push(lowered.into_expr());
        }
        assert_eq!(results[0], results[1], "exact sugar lowering: {source}");
        assert_eq!(schedules[0], schedules[1], "resolver schedule: {source}");
    }

    let span = rum_syntax::Span::default();
    let syntax = SyntaxExpr::binary(
        BinaryOp::MessageName,
        SyntaxExpr::call(SyntaxExpr::symbol("f", span), vec![], span),
        SyntaxExpr::call(
            SyntaxExpr::symbol("tag", span),
            vec![SyntaxExpr::symbol("x", span)],
            span,
        ),
        span,
    );
    for front in [0, 16] {
        let mut seen = Vec::new();
        let lowered = lower_syntax_expr_with_front_depth(
            &syntax,
            &mut |raw| {
                seen.push(raw.to_owned());
                Some(Symbol::from_name(raw))
            },
            front,
        )
        .unwrap()
        .into_expr();
        assert_eq!(seen, ["f", "tag", "x", "tag", "x"]);
        assert_eq!(
            lowered,
            canonical_call(
                "MessageName",
                vec![
                    Expr::call(Expr::symbol("f"), vec![]),
                    Expr::call(Expr::symbol("tag"), vec![Expr::symbol("x")]),
                ]
            )
        );
    }
}

// Keep the certificate about lowering. Syntax and Expr destruction are separate
// standing debt, so dismantle the manually built fixtures without recursion.
struct OwnedSyntax(Option<SyntaxExpr>);

impl Drop for OwnedSyntax {
    fn drop(&mut self) {
        let mut pending: Vec<_> = self.0.take().into_iter().collect();
        while let Some(node) = pending.pop() {
            match node.kind {
                SyntaxExprKind::Call { head, args } => {
                    pending.push(*head);
                    pending.extend(args);
                }
                SyntaxExprKind::List(items) | SyntaxExprKind::Association(items) => {
                    pending.extend(items)
                }
                SyntaxExprKind::Blank { head, .. } => pending.extend(head.map(|head| *head)),
                SyntaxExprKind::Pattern { name, pattern } => pending.extend([*name, *pattern]),
                SyntaxExprKind::Prefix { expr, .. } | SyntaxExprKind::Postfix { expr, .. } => {
                    pending.push(*expr)
                }
                SyntaxExprKind::Binary { lhs, rhs, .. } => pending.extend([*lhs, *rhs]),
                SyntaxExprKind::ComparisonChain { operands, .. } => pending.extend(operands),
                _ => {}
            }
        }
    }
}

fn drop_value(value: Expr) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        if let Expr::Call { head, args } = value {
            pending.push(head.into_owned());
            pending.extend(args.into_vec());
        }
    }
}

#[test]
fn public_lowering_paths_reject_manual_trees_above_bound() {
    let span = rum_syntax::Span::default();
    let session = Session::new();
    for shape in 0..3 {
        let mut syntax = SyntaxExpr::symbol("deepest", span);
        for index in 0..6000 {
            syntax = match shape {
                0 => SyntaxExpr::call(syntax, vec![SyntaxExpr::integer("1", span)], span),
                1 => SyntaxExpr::call(SyntaxExpr::symbol("f", span), vec![syntax], span),
                _ => match index % 3 {
                    0 => SyntaxExpr::list(vec![syntax], span),
                    1 => SyntaxExpr::prefix(PrefixOp::Not, syntax, span),
                    _ => SyntaxExpr::binary(
                        BinaryOp::Power,
                        syntax,
                        SyntaxExpr::integer("2", span),
                        span,
                    ),
                },
            };
        }
        let syntax = OwnedSyntax(Some(syntax));
        for result in [
            lower_syntax_expr(syntax.0.as_ref().unwrap()),
            session.lower_syntax_expr(syntax.0.as_ref().unwrap()),
        ] {
            let closed = matches!(&result, Err(EvalError::RecursionLimitExceeded { limit }) if *limit == MAX_LOWERING_DEPTH);
            if let Ok(value) = result {
                drop_value(value);
            }
            assert!(
                closed,
                "both public paths must classify excessive host syntax, shape {shape}"
            );
        }
    }
}

#[test]
fn head_chains_below_bound_preserve_every_element() {
    let span = rum_syntax::Span::default();
    for depth in [4097, 5000] {
        let mut syntax = SyntaxExpr::symbol("deepest", span);
        for index in 0..depth {
            syntax = SyntaxExpr::call(
                syntax,
                vec![SyntaxExpr::integer(index.to_string(), span)],
                span,
            );
        }
        let syntax = OwnedSyntax(Some(syntax));
        let mut value = lower_syntax_expr(syntax.0.as_ref().unwrap()).unwrap();
        for index in (0..depth).rev() {
            let Expr::Call { head, args } = value else {
                panic!("missing head-chain element {index}")
            };
            assert_eq!(args.as_ref(), &[Expr::integer(BigInt::from(index))]);
            value = head.into_owned();
        }
        assert_eq!(value, Expr::symbol("deepest"));
    }
}

#[test]
fn lowering_error_restores_front_budget() {
    let span = rum_syntax::Span::default();
    let invalid = SyntaxExpr::list(vec![SyntaxExpr::integer("invalid", span)], span);
    for front in [0, 16] {
        let result = lower_syntax_expr_with_front_depth(&invalid, &mut |_| None, front);
        assert!(
            matches!(result, Err(EvalError::InvalidInteger { literal }) if literal == "invalid")
        );
        let (syntax, expected) = branched_fixture(4);
        let lowered = lower_with_front(&syntax, 16);
        assert!(!lowered.used_tail);
        assert_eq!(lowered.value, expected);
    }
}
