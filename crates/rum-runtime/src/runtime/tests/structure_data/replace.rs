use super::*;

#[test]
fn replacepart_deep_path_shallow_companion_pins_three_level_rebuild() {
    let mut session = Session::new();

    let outcome = session
        .eval_text("ReplacePart[{{{0, 1}, {2, 3}}, {{4, 5}, {6, 7}}}, {1, 2, 1} -> 9]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[List[List[0, 1], List[9, 3]], List[List[4, 5], List[6, 7]]]"
    );
    assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
}

#[test]
fn replacepart_supports_immediate_delayed_and_head_replacements() {
    let mut session = Session::new();

    session.eval_text("n = 1").unwrap();
    session.eval_text("immediate = 2 -> n").unwrap();
    session.eval_text("delayed = 2 :> n").unwrap();
    session.eval_text("n = 2").unwrap();

    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], immediate]")
            .unwrap()
            .value
            .to_string(),
        "f[a, 1]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], delayed]")
            .unwrap()
            .value
            .to_string(),
        "f[a, 2]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {0} -> g]")
            .unwrap()
            .value
            .to_string(),
        "g[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {{1, 1} -> x, {2, 1} -> y}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[x], h[y]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a, b], h[c, d]], {All, 2} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a, b], h[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c, d], Span[2, All, 2] -> y]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y, c, y]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], All -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {1 ;; 2} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[x, x]"
    );

    session.eval_text("delayedAll = {All, 2} :> n").unwrap();
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a, b], h[c, d]], delayedAll]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a, b], h[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c, d], 2 ;; ;; 2 -> y]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y, c, y]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], 0 ;; -1 -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], All -> x, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], 0 -> x, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[1, 0 -> x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[1, All -> x, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[1, 0 ;; 0 -> x]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {{}} -> x]")
            .unwrap()
            .value
            .to_string(),
        "x"
    );
}

#[test]
fn replacepart_spans_reuse_part_selection_and_fail_closed_on_malformed_shapes() {
    let mut session = Session::new();

    for (input, expected) in [
        (
            "ReplacePart[{a, b, c, d}, (2 ;; 4 ;; 2) -> x]",
            "List[a, x, c, x]",
        ),
        (
            "ReplacePart[{a, b, c, d}, (2 ;; 3) -> x]",
            "List[a, x, x, d]",
        ),
        ("ReplacePart[{a, b, c, d}, (;;) -> x]", "List[x, x, x, x]"),
        ("ReplacePart[{a, b, c, d}, (2 ;;) -> x]", "List[a, x, x, x]"),
        (
            "ReplacePart[{a, b, c, d}, (-3 ;; -1 ;; 2) -> x]",
            "List[a, x, c, x]",
        ),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{input}");
        assert!(outcome.messages.is_empty(), "{input}");
    }

    for input in [
        "ReplacePart[{a, b, c, d}, (1 ;; 4 ;; 0) -> x]",
        "ReplacePart[{a, b, c, d}, (a ;; 3) -> x]",
        "ReplacePart[{a, b, c, d}, Span[1, 2, 3, 4] -> x]",
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "ReplacePart" && message.tag == "span"),
            "{input}"
        );
        assert!(
            outcome.value.to_string().starts_with("ReplacePart["),
            "{input}"
        );
    }
}

#[test]
fn replacepart_span_preserves_lenient_missing_positions_and_raw_delayed_rhs() {
    let mut session = Session::new();

    for input in [
        "ReplacePart[{a, b, c, d}, 9 -> x]",
        "ReplacePart[{a, b, c, d}, (9 ;; 10) -> x]",
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), "List[a, b, c, d]", "{input}");
        assert!(outcome.messages.is_empty(), "{input}");
    }

    session.eval_text("n = 0").unwrap();
    let outcome = session
        .eval_text("ReplacePart[Hold[a, b, c, d], (2 ;; 4 ;; 2) :> (n = n + 1; x)]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Hold[a, CompoundExpression[Set[n, Plus[n, 1]], x], c, CompoundExpression[Set[n, Plus[n, 1]], x]]"
    );
    assert_eq!(session.eval_text("n").unwrap().value.to_string(), "0");
}

#[test]
fn replacepart_supports_pattern_position_specs_and_rhs_bindings() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c], i_ -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[x, x, x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b, c], {i_} -> i]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {i_, 0} -> i]")
            .unwrap()
            .value
            .to_string(),
        "f[1[a], 2[b]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {i_, 0} -> x, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a], h[b]]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], i_ -> x, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "x[x, x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[a, b], {{i_}, {0}} -> x]")
            .unwrap()
            .value
            .to_string(),
        "x[x, x]"
    );
}

#[test]
fn replacepart_handles_duplicate_and_overlapping_targets_more_stably() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplacePart[f[a], {1 -> x, 1 -> y}]")
            .unwrap()
            .value
            .to_string(),
        "f[y]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a]], {{1} -> x, {1, 1} -> y}]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a]], {{1, 1} -> y, {1} -> x}]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("ReplacePart[f[g[a], h[b]], {All, 0} -> x]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a], h[b]]"
    );
}

#[test]
fn replacepart_rule_lists_preserve_sequential_conflicts_and_missing_positions() {
    let mut session = Session::new();

    for (input, expected) in [
        (
            "ReplacePart[{1, 2, 3}, {{1} -> \"a\", {1} -> \"b\"}]",
            "List[\"b\", 2, 3]",
        ),
        (
            "ReplacePart[{{1, 2}, {3, 4}}, {{1} -> x, {1, 2} -> y}]",
            "List[x, List[3, 4]]",
        ),
        ("ReplacePart[{1, 2}, {{5} -> z}]", "List[1, 2]"),
        (
            "ReplacePart[{a}, {{1} -> f[b, c], {1, 2} -> d}]",
            "List[f[b, d]]",
        ),
    ] {
        let outcome = session.eval_text(input).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{input}");
        assert!(outcome.messages.is_empty(), "{input}");
    }
}

#[test]
fn replacepart_handles_twenty_thousand_structural_rules_in_one_walk() {
    let count = 20_000;
    let list = (1..=count)
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let rules = (1..=count)
        .map(|index| format!("{{{index}}} -> 0"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("ReplacePart[{{{list}}}, {{{rules}}}]");

    let outcome = Session::new().eval_text(&source).unwrap();
    let Expr::Call { head, args } = outcome.value else {
        panic!("ReplacePart should return a list, got {}", outcome.value);
    };
    assert!(is_system_symbol_name(&head, "List"));
    assert_eq!(args.len(), count);
    let zero = Expr::integer(BigInt::from(0_u8));
    assert!(args.iter().all(|arg| *arg == zero));
    assert!(outcome.messages.is_empty());
}

#[test]
fn replaceat_supports_exact_positions_operator_form_and_rule_lists() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("ReplaceAt[f[a, b], x_ -> y, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[x_ -> y, 2][f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, y]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[f[1], {x_String -> z, x_Integer -> y}, 1]")
            .unwrap()
            .value
            .to_string(),
        "f[y]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[f[a, b], x_ -> y, {}]")
            .unwrap()
            .value
            .to_string(),
        "y"
    );
}
