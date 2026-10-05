use super::*;

#[test]
fn compiled_matcher_matches_interpreted_matcher_on_supported_subset() {
    let mut session = session_with_test_rewrite_builtins();

    for (pattern_src, value_src, expected) in [
        ("x_Integer", "7", true),
        ("x_String", "7", false),
        ("f[x_Integer, y_String]", "f[7, \"hi\"]", true),
        ("f[x_Integer /; MatchQ[x, 7]]", "f[7]", true),
        ("f[x_Integer /; MatchQ[x, 8]]", "f[7]", false),
        (
            "PatternTest[x_Integer, Function[MatchQ[Slot[1], 7]]]",
            "7",
            true,
        ),
        (
            "PatternTest[x_Integer, Function[MatchQ[Slot[1], 8]]]",
            "7",
            false,
        ),
        ("Except[x_Integer]", "7", false),
        ("Except[x_Integer]", "a", true),
        ("Except[x_String, x_Integer]", "7", true),
        ("Alternatives[x_String, x_Integer]", "7", true),
        ("f[Alternatives[x_String, x_Integer]]", "f[7]", true),
        ("f[Optional[x_Integer, 0]]", "f[]", true),
        ("f[Optional[x_Integer, 0], y_Integer]", "f[1]", true),
        ("f[Repeated[x_Integer]]", "f[1, 2]", true),
        ("f[Repeated[x_Integer, {2}]]", "f[1, 2]", true),
        ("f[x__, y_Integer]", "f[1, 2, 3]", true),
        ("f[x___]", "f[]", true),
    ] {
        let pattern = session
            .lower_syntax_expr(&parse_expr(pattern_src).unwrap())
            .unwrap();
        let pattern = session.normalize_pattern_expr(&pattern).unwrap();
        let value = session
            .lower_syntax_expr(&parse_expr(value_src).unwrap())
            .unwrap();
        let value = session.evaluate(value).unwrap();

        let interpreted = session
            .match_expr_interpreted_owned(pattern.clone(), value.clone(), SymbolExprMap::default())
            .unwrap();
        let compiled = session
            .match_expr_owned(pattern, value, SymbolExprMap::default())
            .unwrap();

        assert_eq!(
            interpreted.matched, expected,
            "interpreted mismatch for {pattern_src}"
        );
        assert_eq!(
            compiled.matched, expected,
            "compiled mismatch for {pattern_src}"
        );
        assert_eq!(
            compiled.bindings, interpreted.bindings,
            "binding mismatch for {pattern_src}"
        );
    }
}

#[test]
fn interpreted_matcher_owned_can_resume_sequence_conditions_across_input() {
    let mut session = session_with_test_rewrite_builtins();
    let pattern = session.lower_syntax_expr(
        &parse_expr(
            "f[x__ /; MatchQ[InputString[\"first: \"], \"yes\"], y__ /; MatchQ[InputString[\"second: \"], \"ok\"]]",
        )
        .unwrap(),
    )
    .unwrap();
    let pattern = session.normalize_pattern_expr(&pattern).unwrap();
    let value = session
        .lower_syntax_expr(&parse_expr("f[1, 2, 3]").unwrap())
        .unwrap();
    let value = session.evaluate(value).unwrap();

    let failure = session
        .match_expr_interpreted_owned(pattern, value, SymbolExprMap::default())
        .expect_err("interpreted matcher should suspend when a sequence condition requests input");

    match failure {
        EvalError::InputRequired { request } => {
            assert_eq!(request.prompt, "first: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_suspended_evaluation("yes")
        .expect_err("resuming the first condition should still allow the second to suspend");

    match second {
        EvalError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let third = session
        .resume_suspended_evaluation("ok")
        .expect_err("backtracking should be able to stage the second condition again");

    match third {
        EvalError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected repeated second input-required failure, got {other:?}"),
    }

    let outcome = CompiledMatchOutcome::from_continuation_expr(
        session
            .resume_suspended_evaluation("ok")
            .expect("final resume should complete interpreted matcher evaluation"),
    );

    assert!(outcome.matched);
}

#[test]
fn inspect_definition_catalog_reports_kinds_and_delayedness() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("x = 42").unwrap();
    session.eval_text("y := x").unwrap();
    session.eval_text("f[z_Integer] := z").unwrap();
    session.eval_text("a /: g[a] := 1").unwrap();
    session.eval_text("h[u_][v_] := pair[u, v]").unwrap();

    let own_entries = session.inspect_definition_catalog("x").unwrap();
    assert_eq!(own_entries.len(), 1);
    assert_eq!(own_entries[0].kind, DefinitionKind::OwnValue);
    assert!(!own_entries[0].delayed);
    assert_eq!(own_entries[0].lhs.to_string(), "x");
    assert_eq!(own_entries[0].rhs.to_string(), "42");

    let delayed_entries = session.inspect_definition_catalog("y").unwrap();
    assert_eq!(delayed_entries.len(), 1);
    assert_eq!(delayed_entries[0].kind, DefinitionKind::DelayedOwnValue);
    assert!(delayed_entries[0].delayed);

    let down_entries = session.inspect_definition_catalog("f").unwrap();
    assert_eq!(down_entries.len(), 1);
    assert_eq!(down_entries[0].kind, DefinitionKind::DownValue);
    assert!(down_entries[0].delayed);
    assert_eq!(
        down_entries[0].lhs.to_string(),
        "f[Pattern[z, Blank[Integer]]]"
    );

    let up_entries = session.inspect_definition_catalog("a").unwrap();
    assert_eq!(up_entries.len(), 1);
    assert_eq!(up_entries[0].kind, DefinitionKind::UpValue);
    assert!(up_entries[0].delayed);
    assert_eq!(up_entries[0].lhs.to_string(), "g[a]");

    let sub_entries = session.inspect_definition_catalog("h").unwrap();
    assert_eq!(sub_entries.len(), 1);
    assert_eq!(sub_entries[0].kind, DefinitionKind::SubValue);
    assert!(sub_entries[0].delayed);
    assert_eq!(
        sub_entries[0].lhs.to_string(),
        "h[Pattern[u, Blank[]]][Pattern[v, Blank[]]]"
    );
}

#[test]
fn set_supports_immediate_pattern_based_downvalues_and_subvalues() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session.eval_text("f[x_] = 1").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(session.eval_text("f[z]").unwrap().value.to_string(), "1");

    assert_eq!(
        session
            .eval_text("g[x_][y_] = pair[x, y]")
            .unwrap()
            .value
            .to_string(),
        "pair[x, y]"
    );
    assert_eq!(
        session.eval_text("g[1][2]").unwrap().value.to_string(),
        "pair[1, 2]"
    );
}

#[test]
fn lowers_named_pattern_assignment() {
    let syntax = parse_expr("x_Integer := f[x]").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(
        lowered.to_string(),
        "SetDelayed[Pattern[x, Blank[Integer]], f[x]]"
    );
}

#[test]
fn lowers_slots_and_pure_functions() {
    let syntax = parse_expr("#1 + #2 &").unwrap();
    let lowered = lower_syntax_expr(&syntax).unwrap();

    assert_eq!(lowered.to_string(), "Function[Plus[Slot[1], Slot[2]]]");

    let sequence_syntax = parse_expr("## + ##2 &").unwrap();
    let sequence_lowered = lower_syntax_expr(&sequence_syntax).unwrap();

    assert_eq!(
        sequence_lowered.to_string(),
        "Function[Plus[SlotSequence[1], SlotSequence[2]]]"
    );

    let named_syntax = parse_expr("#x &").unwrap();
    let named_lowered = lower_syntax_expr(&named_syntax).unwrap();

    assert_eq!(named_lowered.to_string(), "Function[Slot[\"x\"]]");
}

#[test]
fn named_slots_fill_from_rules_and_association_arguments() {
    let mut session = session_with_test_rewrite_builtins();

    for (source, expected) in [
        ("(#dept &)[<|\"dept\" -> \"x\"|>]", "\"x\""),
        ("(#dept &)[<|dept -> \"x\"|>]", "\"x\""),
        ("(#dept &)[<|\"dept\" :> \"x\"|>]", "\"x\""),
        (
            "(#zzz &)[<|\"dept\" -> \"x\"|>]",
            "Missing[\"KeyAbsent\", \"zzz\"]",
        ),
        ("(#dept &)[5]", "Slot[\"dept\"]"),
        ("(#k &)[\"k\" -> 1]", "1"),
        ("(#k &)[k -> 1]", "1"),
        ("(#k &)[\"j\" -> 1, \"k\" -> 2]", "2"),
        ("(#k &)[<|\"k\" -> 1|>, \"k\" -> 9]", "1"),
        ("(#1 &)[5]", "5"),
        ("(f[##] &)[1, 2]", "f[1, 2]"),
    ] {
        assert_eq!(
            session.eval_text(source).unwrap().value.to_string(),
            expected,
            "unexpected named-slot result for {source}"
        );
    }
}

#[test]
fn setdelayed_stores_and_applies_simple_downvalue() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("f[x_Integer] := x")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");

    let info = session.inspect("f").unwrap();
    assert!(info.has_down_values);
}

#[test]
fn matchq_reports_true_for_matching_pattern() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[7, x_Integer]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_operator_form_is_inert_and_applies_to_data() {
    let mut session = session_with_test_rewrite_builtins();

    let inert = session.eval_text("MatchQ[_Integer]").unwrap();
    assert!(inert.messages.is_empty());
    assert_eq!(inert.value.to_string(), "MatchQ[Blank[Integer]]");

    let applied = session.eval_text("MatchQ[_Integer][3]").unwrap();
    assert!(applied.messages.is_empty());
    assert_eq!(applied.value.to_string(), "True");
}

#[test]
fn matchq_reports_false_for_non_matching_pattern() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[\"hi\", x_Integer]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_plain_blank_arguments_under_fixed_heads() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[f[a, b], f[_, _]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[a], f[_, _]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[g[a, b], f[_, _]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_blanksequence_in_argument_lists() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[x__]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[x__]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_blanknullsequence_in_argument_lists() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[x___]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2, 3], f[x__, y_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_supports_conditions_on_sequence_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[1, 2]]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[2, 1]]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_patterntest_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[7, PatternTest[x_Integer, Function[MatchQ[Slot[1], 7]]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[8, PatternTest[x_Integer, Function[MatchQ[Slot[1], 7]]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_alternatives_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[7, Alternatives[x_String, x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[a, Alternatives[x_String, x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[7], f[Alternatives[x_String, x_Integer]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_supports_except_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[7, Except[x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[a, Except[x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[7, Except[x_String, x_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_supports_optional_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[Optional[x_Integer, 0]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1], f[Optional[x_Integer, 0], y_Integer]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[\"hi\"], f[Optional[x_Integer, 0]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn matchq_supports_repeated_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text("MatchQ[f[1, 2], f[Repeated[x_Integer]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[1], f[Repeated[x_Integer, {2}]]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[f[], f[RepeatedNull[x_Integer]]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn matchq_supports_patterntest_on_sequence_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session
            .eval_text(
                "MatchQ[f[1, 2], f[PatternTest[x__, Function[MatchQ[List[SlotSequence[1]], List[1, 2]]]]]]",
            )
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text(
                "MatchQ[f[1, 2], f[PatternTest[x__, Function[MatchQ[List[SlotSequence[1]], List[2, 1]]]]]]",
            )
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn downvalue_matches_after_argument_evaluation() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("f[x_Integer] := x").unwrap();
    session.eval_text("y = 11").unwrap();

    assert_eq!(session.eval_text("f[y]").unwrap().value.to_string(), "11");
}

#[test]
fn downvalue_can_substitute_multiple_named_patterns() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("pair[a_, b_] := {a, b}").unwrap();

    assert_eq!(
        session.eval_text("pair[1, 2]").unwrap().value.to_string(),
        "List[1, 2]"
    );
}

#[test]
fn downvalue_splices_named_blanksequence_into_rhs_calls() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("echo[x__] := g[x]").unwrap();

    assert_eq!(
        session
            .eval_text("echo[1, 2, 3]")
            .unwrap()
            .value
            .to_string(),
        "g[1, 2, 3]"
    );
}

#[test]
fn downvalue_splices_named_blanknullsequence_into_empty_lists() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("pack[x___] := {x}").unwrap();

    assert_eq!(
        session.eval_text("pack[]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("pack[1, 2]").unwrap().value.to_string(),
        "List[1, 2]"
    );
}

#[test]
fn unmatched_downvalue_leaves_call_in_place() {
    let mut session = session_with_test_rewrite_builtins();

    session.eval_text("f[x_Integer] := x").unwrap();

    assert_eq!(
        session.eval_text("f[\"hi\"]").unwrap().value.to_string(),
        "f[\"hi\"]"
    );
}
