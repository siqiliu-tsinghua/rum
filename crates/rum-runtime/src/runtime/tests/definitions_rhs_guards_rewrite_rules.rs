use super::*;

fn eval_replace_all_unheld(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_rewrite_data_builtin(RewriteDataBuiltin::ReplaceAllUnheld, head, args)
}

fn rewrite_guard_session() -> Session {
    let mut session = session_with_test_control_builtins();
    session.register_builtin(
        BuiltinSpec::special(
            "ReplaceAllUnheld",
            "Test-local ReplaceAllUnheld builtin.",
            eval_replace_all_unheld,
            MemoryGrowthContract::Incremental,
        )
        .protected(),
    );
    session
}

#[test]
fn rewrite_rhs_guards_accept_or_decline_for_every_generic_consumer() {
    let mut session = rewrite_guard_session();
    session
        .eval_text(
            "acceptGuard[1] = True; acceptGuard[-2] = False; \
             guardedRule = x_Integer :> (\"pos\" /; acceptGuard[x])",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "{{1, -2} /. guardedRule, \
                  Replace[1, guardedRule], Replace[-2, guardedRule], \
                  {1, -2} //. guardedRule, \
                  ReplaceList[1, guardedRule], ReplaceList[-2, guardedRule], \
                  Cases[{1, -2, 1}, guardedRule], \
                  ReplaceAt[{1, -2}, guardedRule, {{1}, {2}}], \
                  ReplaceAllUnheld[{1, -2}, guardedRule]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[\"pos\", -2], \"pos\", -2, List[\"pos\", -2], List[\"pos\"], List[], List[\"pos\", \"pos\"], List[\"pos\", -2], List[\"pos\", -2]]"
    );
}

#[test]
fn immediate_and_delayed_rewrite_rules_split_bound_guards() {
    let mut session = rewrite_guard_session();
    session
        .eval_text("acceptGuard[1] = True; acceptGuard[-2] = False")
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "{{1, -2} /. (x_Integer -> (\"immediate\" /; acceptGuard[x])), \
                  {1, -2} /. (x_Integer :> (\"delayed\" /; acceptGuard[x]))}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[\"immediate\", -2], List[\"delayed\", -2]]"
    );
}

#[test]
fn declining_guarded_rules_handoff_in_rule_list_order() {
    let mut session = rewrite_guard_session();

    assert_eq!(
        session
            .eval_text(
                "guardRuns = 0; \
                 first = Replace[1, {x_Integer :> guarded /; (guardRuns = guardRuns + 1; False), x_Integer :> fallback}]; \
                 afterFirst = guardRuns; \
                 second = Replace[1, {x_Integer :> unguarded, x_Integer :> late /; (guardRuns = guardRuns + 1; True)}]; \
                 {first, afterFirst, second, guardRuns}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[fallback, 1, unguarded, 1]"
    );
}

#[test]
fn guarded_exact_rule_preserves_wide_plan_order_and_fallback_index() {
    let mut session = rewrite_guard_session();
    let exact_rules = (2..=32)
        .map(|value| format!("{value} -> {value}"))
        .collect::<Vec<_>>()
        .join(", ");

    assert_eq!(
        session
            .eval_text(&format!(
                "Replace[1, {{1 :> guarded /; False, {exact_rules}, 1 :> fallback}}]",
            ))
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
}

#[test]
fn nested_rewrite_guards_run_left_to_right_and_second_declines() {
    let mut session = rewrite_guard_session();

    assert_eq!(
        session
            .eval_text(
                "guardOrder = 0; \
                 result = Replace[1, {x_Integer :> Condition[guarded, Condition[(guardOrder = guardOrder * 10 + 1; True), (guardOrder = guardOrder * 10 + 2; False)]], x_Integer :> fallback}]; \
                 {result, guardOrder}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[fallback, 12]"
    );
}

#[test]
fn every_non_true_rewrite_guard_result_declines() {
    let mut session = rewrite_guard_session();

    assert_eq!(
        session
            .eval_text(
                "Replace[1, {x_Integer :> symbolResult /; undecided, \
                             x_Integer :> integerResult /; 7, \
                             x_Integer :> callResult /; unknownGuard[x], \
                             x_Integer :> fallback}]",
            )
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
}

#[test]
fn rewrite_guard_side_effect_runs_once_per_candidate_tested() {
    let mut session = rewrite_guard_session();

    assert_eq!(
        session
            .eval_text(
                "guardCount = 0; \
                 result = {1, -2} /. {x_Integer :> guarded /; (guardCount = guardCount + 1; False), x_Integer :> fallback}; \
                 {result, guardCount}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[fallback, fallback], 2]"
    );
}

#[test]
fn replace_list_guard_decline_keeps_later_binding_enumeration() {
    let mut session = rewrite_guard_session();

    assert_eq!(
        session
            .eval_text(
                "bindingGuardCount = 0; \
                 result = ReplaceList[f[a, b, c], \
                   f[x__, y__] :> {List[x], List[y]} /; \
                     (bindingGuardCount = bindingGuardCount + 1; MatchQ[List[x], List[a, b]])]; \
                 {result, bindingGuardCount}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[List[List[a, b], List[c]]], 2]"
    );
}

#[test]
fn replace_repeated_stops_when_the_only_guard_declines() {
    let mut session = rewrite_guard_session();

    assert_eq!(
        session
            .eval_text("ReplaceRepeated[1, x_Integer :> 2 /; False]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
}

#[test]
fn rule_shaped_data_keeps_its_existing_condition_forms() {
    let mut session = rewrite_guard_session();

    assert_eq!(
        session
            .eval_text(
                "{<|1 -> (\"v\" /; True)|>, \
                  Options[dataGuard] = {opt -> (\"v\" /; True)}; Options[dataGuard]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, Condition[\"v\", True]]], List[Rule[opt, Condition[\"v\", True]]]]"
    );
}

#[test]
fn suspending_rewrite_guard_resumes_same_rule_and_position_without_replay() {
    let mut session = rewrite_guard_session();
    session
        .eval_text(
            "rewriteGuardResumeCount = 0; \
             guardedRules = {x_Integer :> \"guarded\" /; \
               (rewriteGuardResumeCount = rewriteGuardResumeCount + 1; \
                MatchQ[InputString[\"rewrite guard: \"], \"yes\"]), \
               x_Integer :> \"fallback\"}",
        )
        .unwrap();

    let failure = session
        .eval_request("{{1} /. guardedRules, rewriteGuardResumeCount}")
        .expect_err("rewrite rhs guard should suspend when it requests input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rewrite guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("declining resumed rewrite guard should continue to the fallback rule");
    assert_eq!(response.value.to_string(), "List[List[\"fallback\"], 1]");
}
