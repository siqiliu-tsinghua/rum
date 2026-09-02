use super::*;

fn guard_test_session() -> Session {
    let mut session = session_with_test_control_builtins();
    session.register_builtin(BuiltinSpec::ordinary(
        "Greater",
        "Test-local exact-integer comparison.",
        test_integer_greater,
        MemoryGrowthContract::NonAmplifying,
    ));
    session.register_builtin(BuiltinSpec::ordinary(
        "Less",
        "Test-local exact-integer comparison.",
        test_integer_less,
        MemoryGrowthContract::NonAmplifying,
    ));
    session
}

fn test_integer_greater(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    test_integer_order(head, args, |left, right| left > right)
}

fn test_integer_less(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    test_integer_order(head, args, |left, right| left < right)
}

fn test_integer_order(
    head: &Expr,
    args: &[Expr],
    accepts: impl FnOnce(BigInt, BigInt) -> bool,
) -> Result<Expr, EvalError> {
    let [left, right] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let (Some(left), Some(right)) = (expr_as_exact_integer(left), expr_as_exact_integer(right))
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    Ok(system_expr(if accepts(left, right) {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    }))
}

#[test]
fn delayed_downvalue_rhs_guard_accepts_or_declines_without_messages() {
    let mut session = guard_test_session();

    let installed = session.eval_text("b[x_] := x^2 /; x > 0").unwrap();
    assert!(installed.messages.is_empty());

    let accepted = session.eval_text("b[3]").unwrap();
    assert_eq!(accepted.value.to_string(), "9");
    assert!(accepted.messages.is_empty());

    let declined = session.eval_text("b[-3]").unwrap();
    assert_eq!(declined.value.to_string(), "b[-3]");
    assert!(declined.messages.is_empty());
}

#[test]
fn guarded_and_unguarded_downvalues_coexist_in_installation_order() {
    let mut session = guard_test_session();

    session.eval_text("f[x_] := \"positive\" /; x > 0").unwrap();
    session.eval_text("f[x_] := \"other\"").unwrap();

    assert_eq!(
        session.eval_text("f[3]").unwrap().value.to_string(),
        "\"positive\""
    );
    assert_eq!(
        session.eval_text("f[-3]").unwrap().value.to_string(),
        "\"other\""
    );
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], Condition[\"positive\", Greater[x, 0]]], RuleDelayed[HoldPattern[f[Pattern[x, Blank[]]]], \"other\"]]"
    );
    assert_eq!(
        session
            .eval_text("Definition[f]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[SetDelayed[f[Pattern[x, Blank[]]], Condition[\"positive\", Greater[x, 0]]], SetDelayed[f[Pattern[x, Blank[]]], \"other\"]]"
    );
}

#[test]
fn explicit_true_guard_is_distinct_from_an_unguarded_downvalue() {
    let mut session = guard_test_session();

    session.eval_text("f[x_] := guarded /; True").unwrap();
    session.eval_text("f[x_] := unguarded").unwrap();

    assert_eq!(
        session.eval_text("f[1]").unwrap().value.to_string(),
        "guarded"
    );
    let Expr::Call { args, .. } = session.eval_text("DownValues[f]").unwrap().value else {
        panic!("DownValues[f] should be a List");
    };
    assert_eq!(args.len(), 2);
}

#[test]
fn same_alpha_canonical_lhs_and_guards_redefine_in_place() {
    let mut session = guard_test_session();

    session
        .eval_text("f[x_] := \"old positive\" /; x > 0")
        .unwrap();
    session.eval_text("f[z_] := \"negative\" /; z < 0").unwrap();
    session
        .eval_text("f[y_] := \"new positive\" /; y > 0")
        .unwrap();

    assert_eq!(
        session.eval_text("f[3]").unwrap().value.to_string(),
        "\"new positive\""
    );
    assert_eq!(
        session.eval_text("f[-3]").unwrap().value.to_string(),
        "\"negative\""
    );
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[HoldPattern[f[Pattern[y, Blank[]]]], Condition[\"new positive\", Greater[y, 0]]], RuleDelayed[HoldPattern[f[Pattern[z, Blank[]]]], Condition[\"negative\", Less[z, 0]]]]"
    );
}

#[test]
fn nested_rhs_guards_flatten_in_order_for_both_spellings() {
    let mut session = guard_test_session();

    session
        .eval_text("right[x_] := Condition[\"right\", Condition[x > 0, x < 10]]")
        .unwrap();
    session
        .eval_text("left[x_] := Condition[Condition[\"left\", x > 0], x < 10]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("{right[5], right[-1], right[11], left[5], left[-1], left[11]}")
            .unwrap()
            .value
            .to_string(),
        "List[\"right\", right[-1], right[11], \"left\", left[-1], left[11]]"
    );

    session
        .eval_text(
            "ordered[x_] := Condition[\"guarded\", Condition[(guardOrder = guardOrder * 10 + 1; True), (guardOrder = guardOrder * 10 + 2; False)]]",
        )
        .unwrap();
    session.eval_text("ordered[x_] := \"fallback\"").unwrap();
    assert_eq!(
        session
            .eval_text("guardOrder = 0; {ordered[1], guardOrder}")
            .unwrap()
            .value
            .to_string(),
        "List[\"fallback\", 12]"
    );
}

#[test]
fn every_non_true_guard_result_declines_to_the_next_candidate() {
    let mut session = guard_test_session();

    session
        .eval_text("f[x_] := symbolResult /; undecided")
        .unwrap();
    session.eval_text("f[x_] := integerResult /; 7").unwrap();
    session
        .eval_text("f[x_] := callResult /; unknownGuard[x]")
        .unwrap();
    session.eval_text("f[x_] := fallback").unwrap();

    let outcome = session.eval_text("f[1]").unwrap();
    assert_eq!(outcome.value.to_string(), "fallback");
    assert!(outcome.messages.is_empty());

    let Expr::Call { args, .. } = session.eval_text("DownValues[f]").unwrap().value else {
        panic!("DownValues[f] should be a List");
    };
    assert_eq!(args.len(), 4);
}

#[test]
fn guard_errors_propagate_like_pattern_position_condition_errors() {
    let mut session = guard_test_session();

    assert_eq!(
        session
            .eval_text("Catch[MatchQ[1, x_ /; Throw[patternGuardError]]]")
            .unwrap()
            .value
            .to_string(),
        "patternGuardError"
    );

    session
        .eval_text("f[x_] := guarded /; Throw[definitionGuardError]")
        .unwrap();
    session.eval_text("f[x_] := fallback").unwrap();
    assert_eq!(
        session.eval_text("Catch[f[1]]").unwrap().value.to_string(),
        "definitionGuardError"
    );
}

#[test]
fn rhs_guard_side_effects_run_before_the_body_is_selected() {
    let mut session = guard_test_session();

    session
        .eval_text("f[x_] := \"hit\" /; (Print[x]; True)")
        .unwrap();
    let outcome = session.eval_text("f[3]").unwrap();

    assert_eq!(outcome.value.to_string(), "\"hit\"");
    assert_eq!(outcome.prints, vec!["3"]);
}

#[test]
fn immediate_downvalue_rhs_evaluates_completely_before_guard_splitting() {
    let mut session = guard_test_session();

    let installed = session
        .eval_text(
            "guardSetCount = 0; f[x_] = (guardSetCount = guardSetCount + 1; x) /; (guardSetCount = guardSetCount + 1; x > 0)",
        )
        .unwrap();
    assert_eq!(installed.value.to_string(), "Condition[x, Greater[x, 0]]");
    assert_eq!(
        session
            .eval_text("{guardSetCount, f[1], f[-1], guardSetCount}")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, f[-1], 2]"
    );
}

#[test]
fn unset_lhs_removes_every_rhs_guard_variant() {
    let mut session = guard_test_session();

    session.eval_text("b[x_] := positive /; x > 0").unwrap();
    session.eval_text("b[x_] := negative /; x < 0").unwrap();
    session.eval_text("b[x_] := other").unwrap();
    session.eval_text("Unset[b[x_]]").unwrap();

    assert_eq!(
        session
            .eval_text("DownValues[b]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(session.eval_text("b[1]").unwrap().value.to_string(), "b[1]");
}

#[test]
fn downvalues_guard_reconstruction_round_trips_behavior() {
    let mut session = guard_test_session();

    session
        .eval_text("f[x_] := \"positive\" /; x > 0; f[x_] := \"other\"; saved = DownValues[f]")
        .unwrap();
    let before = session
        .eval_text("DownValues[f]")
        .unwrap()
        .value
        .to_string();

    let restored = session
        .eval_text("Clear[f]; DownValues[f] = saved")
        .unwrap();
    assert!(restored.messages.is_empty());
    assert_eq!(
        session
            .eval_text("{f[3], f[-3]}")
            .unwrap()
            .value
            .to_string(),
        "List[\"positive\", \"other\"]"
    );
    assert_eq!(
        session
            .eval_text("DownValues[f]")
            .unwrap()
            .value
            .to_string(),
        before
    );
}

#[test]
fn rhs_guards_and_bodies_share_the_definition_option_scope() {
    let mut session = guard_test_session();

    session.eval_text("Options[f] = {accept -> False}").unwrap();
    session
        .eval_text(
            "f[x_, OptionsPattern[]] := guarded[x, OptionValue[accept]] /; OptionValue[accept]",
        )
        .unwrap();
    session
        .eval_text("f[x_, OptionsPattern[]] := fallback[x, OptionValue[accept]]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("{f[1], f[1, accept -> True]}")
            .unwrap()
            .value
            .to_string(),
        "List[fallback[1, False], guarded[1, True]]"
    );
}

#[test]
fn wide_exact_dispatch_preserves_guard_variant_order() {
    let mut session = guard_test_session();

    session
        .eval_text(
            "f[1] := guarded /; False; f[1] := fallback; Do[With[{k = i}, f[k] = k], {i, 2, 32}]",
        )
        .unwrap();

    assert_eq!(
        session.eval_text("f[1]").unwrap().value.to_string(),
        "fallback"
    );
    let Expr::Call { args, .. } = session.eval_text("DownValues[f]").unwrap().value else {
        panic!("DownValues[f] should be a List");
    };
    assert_eq!(args.len(), 33);
}
