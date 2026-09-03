use super::*;

fn eval_with_queued_inputs(source: &str, inputs: &[&str]) -> String {
    let mut session = session_with_test_control_builtins();
    for input in inputs {
        session.queue_input_response(*input);
    }

    session
        .eval_request(source)
        .expect("queued inputs should complete evaluation")
        .value
        .to_string()
}

fn eval_by_suspending_and_resuming(source: &str, inputs: &[&str]) -> String {
    let mut session = session_with_test_control_builtins();
    let mut failure = session
        .eval_request(source)
        .expect_err("source should suspend for host input");

    for (index, input) in inputs.iter().enumerate() {
        match &failure.error {
            SessionRequestError::InputRequired { request } => {
                assert!(!request.prompt.is_empty());
            }
            other => panic!("expected input-required failure, got {other:?}"),
        }

        if index + 1 == inputs.len() {
            return session
                .resume_eval_request_with_input(*input)
                .expect("final resume should complete evaluation")
                .value
                .to_string();
        }

        failure = session
            .resume_eval_request_with_input(*input)
            .expect_err("intermediate resume should suspend again");
    }

    panic!("at least one input is required");
}

fn assert_suspend_resume_matches_queued(source: &str, inputs: &[&str], expected: &str) {
    let queued = eval_with_queued_inputs(source, inputs);
    let resumed = eval_by_suspending_and_resuming(source, inputs);

    assert_eq!(queued, expected);
    assert_eq!(resumed, queued);
}

#[test]
fn ordered_single_arg_cursor_resumes_without_replaying_the_matched_prefix() {
    assert_suspend_resume_matches_queued(
        "gMatcherSeen = 0; gMatcherTest[x_] := (gMatcherSeen = gMatcherSeen + 1; True); {MatchQ[f[1, 2, 3], f[x_?gMatcherTest, y_ /; MatchQ[InputString[\"cond: \"], \"yes\"], 3]], gMatcherSeen}",
        &["yes"],
        "List[True, 1]",
    );
}

#[test]
fn position_collection_continuation_roundtrip_matches_uninterrupted_run() {
    assert_suspend_resume_matches_queued(
        "Position[f[1, 2, 3], x_Integer /; If[MatchQ[x, 2], MatchQ[InputString[\"cond: \"], \"yes\"], True], Infinity]",
        &["yes"],
        "List[List[1], List[2], List[3]]",
    );
}

#[test]
fn position_collection_survives_multiple_matcher_suspensions() {
    assert_suspend_resume_matches_queued(
        "Position[f[1, 2, 3], x_Integer /; If[MatchQ[x, 1], MatchQ[InputString[\"cond: \"], \"yes\"], If[MatchQ[x, 2], MatchQ[InputString[\"cond: \"], \"yes\"], True]], Infinity]",
        &["yes", "yes"],
        "List[List[1], List[2], List[3]]",
    );
}

#[test]
fn cases_collection_survives_multiple_matcher_suspensions() {
    assert_suspend_resume_matches_queued(
        "Cases[f[1, 2, 3], x_Integer /; If[MatchQ[x, 1], MatchQ[InputString[\"cond: \"], \"yes\"], If[MatchQ[x, 2], MatchQ[InputString[\"cond: \"], \"yes\"], True]], Infinity]",
        &["yes", "yes"],
        "List[1, 2, 3]",
    );
}

#[test]
fn traversal_count_continuation_roundtrip_matches_uninterrupted_run() {
    assert_suspend_resume_matches_queued(
        "Count[f[1, 2, 3], x_Integer /; If[MatchQ[x, 2], MatchQ[InputString[\"cond: \"], \"yes\"], True], Infinity]",
        &["yes"],
        "3",
    );
}

#[test]
fn sequence_binding_vec_continuation_roundtrip_preserves_multiple_bindings() {
    assert_suspend_resume_matches_queued(
        "MatchQ[f[1, 2, 3], f[x__ /; MatchQ[InputString[\"first: \"], \"yes\"], y__ /; MatchQ[InputString[\"second: \"], \"ok\"]]]",
        &["yes", "ok", "ok"],
        "True",
    );
}

#[test]
fn matcher_binding_numeric_value_survives_suspension() {
    assert_suspend_resume_matches_queued(
        "bindingProbe[x_ /; MatchQ[InputString[\"binding: \"], \"yes\"]] := x; \
         bindingProbe[41]",
        &["yes"],
        "41",
    );
}

#[test]
fn guarded_replacement_numeric_value_survives_suspension() {
    assert_suspend_resume_matches_queued(
        "Replace[5, x_Integer :> ((x + 10) /; \
         MatchQ[InputString[\"replacement: \"], \"yes\"])]",
        &["yes"],
        "15",
    );
}

#[test]
fn nested_rule_application_numeric_value_survives_suspension() {
    assert_suspend_resume_matches_queued(
        "ReplaceAll[f[2], f[x_] :> Replace[x, y_Integer :> ((y * 3) /; \
         MatchQ[InputString[\"nested rule: \"], \"yes\"])]]",
        &["yes"],
        "6",
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_matchq() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("MatchQ[1, x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]]")
        .expect_err("matcher condition should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned MatchQ");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn downvalue_rhs_guard_resumes_at_the_same_candidate_cursor() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "guardResumeCount = 0; f[x_] := \"guarded\" /; (guardResumeCount = guardResumeCount + 1; MatchQ[InputString[\"guard: \"], \"yes\"]); f[x_] := \"fallback\"",
        )
        .unwrap();

    let failure = session
        .eval_request("{f[1], guardResumeCount}")
        .expect_err("rhs guard should suspend when it requests input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("declining resumed guard should continue to the fallback candidate");
    assert_eq!(response.value.to_string(), "List[\"fallback\", 1]");
}

#[test]
fn upvalue_rhs_guard_resumes_at_the_same_candidate_cursor() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "upGuardResumeCount = 0; u /: h[u, x_] := \"guarded\" /; (upGuardResumeCount = upGuardResumeCount + 1; MatchQ[InputString[\"up guard: \"], \"yes\"]); u /: h[u, x_] := \"fallback\"",
        )
        .unwrap();

    let failure = session
        .eval_request("{h[u, 1], upGuardResumeCount}")
        .expect_err("upvalue rhs guard should suspend when it requests input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "up guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("declining resumed upvalue guard should continue to the fallback candidate");
    assert_eq!(response.value.to_string(), "List[\"fallback\", 1]");
}

#[test]
fn subvalue_rhs_guard_resumes_at_the_same_candidate_cursor() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "subGuardResumeCount = 0; s[1][x_] := \"guarded\" /; (subGuardResumeCount = subGuardResumeCount + 1; MatchQ[InputString[\"sub guard: \"], \"yes\"]); s[1][x_] := \"fallback\"",
        )
        .unwrap();

    let failure = session
        .eval_request("{s[1][1], subGuardResumeCount}")
        .expect_err("subvalue rhs guard should suspend when it requests input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "sub guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("declining resumed subvalue guard should continue to the fallback candidate");
    assert_eq!(response.value.to_string(), "List[\"fallback\", 1]");
}

#[test]
fn defaultvalue_rhs_guard_resumes_during_matching_without_replaying_the_candidate() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text(
            "defaultGuardResumeCount = 0; Default[d] := 7 /; (defaultGuardResumeCount = defaultGuardResumeCount + 1; MatchQ[InputString[\"default guard: \"], \"yes\"]); Default[d] := 9; d[x_.] := x",
        )
        .unwrap();

    let failure = session
        .eval_request("{d[], defaultGuardResumeCount}")
        .expect_err("default rhs guard should suspend while optional matching requests input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "default guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("no")
        .expect("declining resumed default guard should continue to the fallback default");
    assert_eq!(response.value.to_string(), "List[9, 1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_sequence_conditioned_matchq() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("MatchQ[f[1, 2], f[x__ /; InputString[\"cond: \"], y_]]")
        .expect_err("sequence-pattern condition should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish sequence-pattern MatchQ");

    assert_eq!(response.value.to_string(), "False");
}

#[test]
fn resume_eval_request_with_input_can_finish_nested_matcher_inside_sequence_conditioned_matchq() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("MatchQ[f[1, 2], f[x__ /; MatchQ[InputString[\"cond: \"], \"yes\"], y_]]")
        .expect_err("nested matcher inside a sequence-pattern condition should suspend");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish sequence-pattern matching with the nested matcher");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_multi_sequence_backtracking_conditions() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "MatchQ[f[1, 2, 3], f[x__ /; MatchQ[InputString[\"first: \"], \"yes\"], y__ /; MatchQ[InputString[\"second: \"], \"ok\"]]]",
        )
        .expect_err("multi-sequence matcher conditions should suspend on the first nested matcher");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "first: ");
        }
        other => panic!("expected first input-required failure, got {other:?}"),
    }

    let second = session.resume_eval_request_with_input("yes").expect_err(
        "after the first sequence condition resumes, the second should still be able to suspend",
    );

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let third = session
        .resume_eval_request_with_input("ok")
        .expect_err("backtracking into a new sequence split should be able to suspend again");

    match &third.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "second: ");
        }
        other => panic!("expected repeated second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resume should finish the original multi-sequence MatchQ after backtracking");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_downvalue_matching() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("f[x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]] := x")
        .unwrap();

    let failure = session
        .eval_request("f[1]")
        .expect_err("conditioned downvalue match should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned downvalue matching");

    assert_eq!(response.value.to_string(), "1");
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_replace_all() {
    let source = "ReplaceAll[1, x_ /; MatchQ[InputString[\"cond: \"], \"yes\"] :> List[x, x]]";
    let uninterrupted = eval_with_queued_inputs(source, &["yes"]);
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(source)
        .expect_err("conditioned ReplaceAll rule should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned ReplaceAll");

    assert_eq!(response.value.to_string(), "List[1, 1]");
    assert_eq!(response.value.to_string(), uninterrupted);
}

#[test]
fn resume_eval_request_with_input_can_finish_orderless_patternsequence_group_matching() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "MatchQ[a + b + c, Plus[PatternSequence[a, x_ /; MatchQ[InputString[\"cond: \"], \"yes\"]], b]]",
        )
        .expect_err("orderless PatternSequence group matching should suspend when its condition requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish orderless PatternSequence group matching");

    assert_eq!(response.value.to_string(), "True");
}

#[test]
fn resume_eval_request_with_input_can_finish_conditioned_cases() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Cases[f[1], x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]]")
        .expect_err("conditioned Cases pattern should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish conditioned Cases");

    assert_eq!(response.value.to_string(), "List[1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_symbol_spec() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("p := x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]")
        .unwrap();

    let failure = session.eval_request("Cases[f[1], p]").expect_err(
        "Cases spec evaluation should suspend when the referenced symbol requests input",
    );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the spec symbol resolves");

    assert_eq!(response.value.to_string(), "List[1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_level_spec() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request("Cases[f[1], _Integer, If[MatchQ[InputString[\"lvl: \"], \"yes\"], 1, 0]]")
        .expect_err("Cases level-spec evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lvl: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the level spec resolves");

    assert_eq!(response.value.to_string(), "List[1]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_third_arg() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "Cases[f[g[1]], x_Symbol, If[MatchQ[InputString[\"heads: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("Cases third-argument traversal config should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the third argument resolves");

    assert_eq!(response.value.to_string(), "List[f]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fourth_arg() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "Cases[f[1, g[2]], x_Symbol, 2, If[MatchQ[InputString[\"heads: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("Cases fourth-argument traversal config should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the fourth argument resolves");

    assert_eq!(response.value.to_string(), "List[f, g]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_match_limit() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "Cases[f[g[1], 2, h[3]], _Integer, Infinity, If[MatchQ[InputString[\"n: \"], \"yes\"], 2, 0]]",
        )
        .expect_err("Cases match-limit evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "n: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the match limit resolves");

    assert_eq!(response.value.to_string(), "List[1, 2]");
}

#[test]
fn resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fifth_arg() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "Cases[f[1, g[2]], x_Symbol, {2}, 1, If[MatchQ[InputString[\"heads: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("Cases fifth-argument traversal config should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Cases after the fifth argument resolves");

    assert_eq!(response.value.to_string(), "List[g]");
}

#[test]
fn resume_eval_request_with_input_can_finish_position_with_suspending_symbol_spec() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("p := x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]")
        .unwrap();

    let failure = session.eval_request("Position[f[1], p]").expect_err(
        "Position pattern evaluation should suspend when the referenced symbol requests input",
    );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Position after the pattern resolves");

    assert_eq!(response.value.to_string(), "List[List[1]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_position_with_suspending_match_limit() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "Position[f[g[1], 2, h[3]], _Integer, Infinity, If[MatchQ[InputString[\"n: \"], \"yes\"], 2, 0]]",
        )
        .expect_err("Position match-limit evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "n: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Position after the match limit resolves");

    assert_eq!(response.value.to_string(), "List[List[1, 1], List[2]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_count_with_suspending_symbol_spec() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("p := x_Integer /; MatchQ[InputString[\"cond: \"], \"yes\"]")
        .unwrap();

    let failure = session
        .eval_request("Count[f[g[1], 2], p, Infinity]")
        .expect_err(
            "Count pattern evaluation should suspend when the referenced symbol requests input",
        );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "cond: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue Count after the pattern resolves");

    assert_eq!(response.value.to_string(), "2");
}

#[test]
fn resume_eval_request_with_input_can_finish_freeq_with_suspending_heads_option() {
    let mut session = session_with_test_control_builtins();

    let failure = session
        .eval_request(
            "FreeQ[f[1], x_Symbol, If[MatchQ[InputString[\"heads: \"], \"yes\"], {0, Infinity}, {1}], If[MatchQ[InputString[\"heads2: \"], \"yes\"], Heads -> True, Heads -> False]]",
        )
        .expect_err("FreeQ traversal option evaluation should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second_failure = session
        .resume_eval_request_with_input("yes")
        .expect_err("FreeQ should continue until the heads option also resolves");

    match &second_failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "heads2: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should continue FreeQ after traversal options resolve");

    assert_eq!(response.value.to_string(), "False");
}
