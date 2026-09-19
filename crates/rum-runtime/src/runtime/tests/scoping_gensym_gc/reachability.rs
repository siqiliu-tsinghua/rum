use super::*;

#[test]
fn own_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "own value",
        "ClearAll[gcOwn078]; Module[{x}, x = 7; gcOwn078 = Hold[x]; 0]",
        "First[gcOwn078]",
        "ClearAll[gcOwn078]",
    );
}

#[test]
fn delayed_own_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "delayed own value",
        "ClearAll[gcDelayedOwn078]; Module[{x}, x = 7; gcDelayedOwn078 := Hold[x]; 0]",
        "First[gcDelayedOwn078]",
        "ClearAll[gcDelayedOwn078]",
    );
}

#[test]
fn down_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "down value",
        "ClearAll[gcDown078]; Module[{x}, x = 7; gcDown078[] := Hold[x]; 0]",
        "First[gcDown078[]]",
        "ClearAll[gcDown078]",
    );
}

#[test]
fn up_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "up value",
        "ClearAll[gcUpTag078, gcUpHead078]; Module[{x}, x = 7; gcUpTag078 /: gcUpHead078[gcUpTag078[]] := Hold[x]; 0]",
        "First[gcUpHead078[gcUpTag078[]]]",
        "ClearAll[gcUpTag078, gcUpHead078]",
    );
}

#[test]
fn sub_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "sub value",
        "ClearAll[gcSub078]; Module[{x}, x = 7; gcSub078[a_][b_] := Hold[x]; 0]",
        "First[gcSub078[1][2]]",
        "ClearAll[gcSub078]",
    );
}

#[test]
fn option_root_escape_keeps_candidate_until_options_are_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "option root",
        "ClearAll[gcOptions078, gcOption078]; Module[{x}, x = 7; Options[gcOptions078] = {gcOption078 -> Hold[x]}; 0]",
        "First[OptionValue[gcOptions078, gcOption078]]",
        "ClearAll[gcOptions078, gcOption078]",
    );
}

#[test]
fn syntax_information_root_escape_keeps_candidate_until_syntax_info_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "syntax information root",
        "ClearAll[gcSyntax078]; Module[{x}, x = 7; SyntaxInformation[gcSyntax078] = {\"LocalVariables\" -> Hold[x]}; 0]",
        "First[\"LocalVariables\" /. SyntaxInformation[gcSyntax078]]",
        "ClearAll[gcSyntax078]",
    );
}

#[test]
fn message_root_escape_keeps_candidate_until_message_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "message root",
        "ClearAll[gcMessage078]; Module[{x}, x = 7; gcMessage078::usage = Hold[x]; 0]",
        "First[MessageDefinitions[gcMessage078][[1]][[2]]]",
        "ClearAll[gcMessage078]",
    );
}

#[test]
fn returned_expr_extra_root_keeps_candidate_definition_live() {
    let mut session = session_with_test_control_builtins();

    let outcome = session.eval_text("Module[{x}, x = 7; Hold[x]]").unwrap();
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);

    session.collect_and_sweep_dead_gensyms(std::slice::from_ref(&outcome.value));

    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate)),
        "returned Hold[x$] must keep x$ live while it is an extra root"
    );

    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(
        candidates
            .iter()
            .all(|candidate| !session.definitions.contains_key(candidate)),
        "candidate should sweep once the returned value is no longer a root"
    );
}

#[test]
fn module_result_root_retains_only_its_transitive_local_chain() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(
            "Module[{gcChainA = Hold[gcChainB], gcChainB = Hold[gcChainC], gcChainC = 7, gcUnused = 9}, Hold[gcChainA]]",
        )
        .unwrap();
    let candidates = retained_candidates(&session);

    assert_eq!(
        candidates.len(),
        3,
        "the unrelated local should be reclaimed"
    );
    assert_candidate_definition_count(&session, &candidates, 3);

    session.collect_and_sweep_dead_gensyms(std::slice::from_ref(&outcome.value));
    assert_candidate_definition_count(&session, &candidates, 3);

    session.collect_and_sweep_dead_gensyms(&[]);
    assert_candidate_definition_count(&session, &candidates, 0);
}

#[test]
fn outcome_extra_roots_keep_success_return_throw_and_cleanup_candidates_live() {
    assert_outcome_roots_keep_then_drop_candidates("Ok", 1, |exprs| Ok(exprs[0].clone()));
    assert_outcome_roots_keep_then_drop_candidates("Return", 1, |exprs| {
        Err(EvalError::Return {
            value: exprs[0].clone(),
        })
    });
    assert_outcome_roots_keep_then_drop_candidates("Thrown", 2, |exprs| {
        Err(EvalError::Thrown {
            value: exprs[0].clone(),
            tag: exprs[1].clone(),
            explicit_tag: true,
        })
    });
    assert_outcome_roots_keep_then_drop_candidates("ThrownBypassingCatch", 4, |exprs| {
        Err(EvalError::ThrownBypassingCatch {
            value: exprs[0].clone(),
            tag: exprs[1].clone(),
            explicit_tag: true,
            cleanup_value: exprs[2].clone(),
            cleanup_tag: exprs[3].clone(),
            cleanup_explicit_tag: true,
        })
    });
}

#[test]
fn outcome_extra_roots_are_empty_for_non_value_errors_and_candidate_is_reclaimed() {
    let mut session = session_with_test_control_builtins();
    let candidate = allocate_retained_candidate(&mut session, 1);
    let outcome = Err(EvalError::Cancelled);
    let roots = gensym_gc_extra_roots_for_outcome(&outcome);

    assert!(roots.is_empty());
    assert_candidate_definition_count(&session, &[candidate], 1);

    session.collect_and_sweep_dead_gensyms(&roots);

    assert_candidate_definition_count(&session, &[candidate], 0);
    assert!(!session.gc_candidate_gensyms.contains(&candidate));
}

#[test]
fn module_unreferenced_error_paths_reclaim_gensyms_immediately() {
    let mut throw_session = session_with_test_control_builtins();
    assert_eq!(
        eval_string(
            &mut throw_session,
            "Catch[Module[{gcThrow220 = 1}, Throw[fail]]]"
        ),
        "fail"
    );
    assert_module_gensym_reclaimed(&throw_session, "gcThrow220$1");

    let mut abort_session = session_with_test_control_builtins();
    assert_eq!(
        eval_string(
            &mut abort_session,
            "CheckAbort[Module[{gcAbort220 = 1}, Abort[]], caught]"
        ),
        "caught"
    );
    assert_module_gensym_reclaimed(&abort_session, "gcAbort220$1");

    let mut return_session = session_with_test_control_builtins();
    assert_eq!(
        eval_string(
            &mut return_session,
            "Function[Module[{gcReturn220 = 1}, Return[fail]]][]"
        ),
        "fail"
    );
    assert_module_gensym_reclaimed(&return_session, "gcReturn220$1");
}

#[test]
fn module_throw_payload_root_retains_its_gensym_definition() {
    let mut session = session_with_test_first();

    assert_eq!(
        eval_string(
            &mut session,
            "First[Catch[Module[{gcPayload220 = 1}, Throw[Hold[gcPayload220]]]]]"
        ),
        "1"
    );

    let candidate = Symbol::from_name("gcPayload220$1");
    assert!(session.definitions.contains_key(&candidate));
    assert!(session.gc_candidate_gensyms.contains(&candidate));
}

#[test]
fn nested_module_error_reclamation_mirrors_success_conservatism() {
    let mut success_session = session_with_test_control_builtins();
    assert_eq!(
        eval_string(
            &mut success_session,
            "Module[{gcOuterOk220 = 1}, Module[{gcInnerOk220 = 2}, 42]]"
        ),
        "42"
    );
    assert_module_gensym_deferred(&success_session, "gcOuterOk220$1");
    assert_module_gensym_reclaimed(&success_session, "gcInnerOk220$2");

    let mut error_session = session_with_test_control_builtins();
    assert_eq!(
        eval_string(
            &mut error_session,
            "Catch[Module[{gcOuterErr220 = 1}, Module[{gcInnerErr220 = 2}, Throw[fail]]]]"
        ),
        "fail"
    );
    assert_module_gensym_deferred(&error_session, "gcOuterErr220$1");
    assert_module_gensym_reclaimed(&error_session, "gcInnerErr220$2");
}

#[test]
fn module_error_path_definition_escape_stays_deferred() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        eval_string(
            &mut session,
            "Catch[Module[{gcGuard220 = 1}, gcGuardFn220[y_] := gcGuard220 + y; Throw[fail]]]"
        ),
        "fail"
    );

    assert_module_gensym_deferred(&session, "gcGuard220$1");
}

#[test]
fn downvalue_rhs_guard_keeps_its_escaped_module_binding_live() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        eval_string(
            &mut session,
            "Catch[Module[{gcRhsGuard348 = 1}, gcRhsGuardFn348[y_] := y /; MatchQ[gcRhsGuard348, 1]; Throw[fail]]]"
        ),
        "fail"
    );

    assert_module_gensym_deferred(&session, "gcRhsGuard348$1");
    assert_eq!(eval_string(&mut session, "gcRhsGuardFn348[7]"), "7");
}

#[test]
fn upvalue_rhs_guard_keeps_its_escaped_module_binding_live() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        eval_string(
            &mut session,
            "Catch[Module[{gcUpRhsGuard349 = 1}, gcUpTag349 /: gcUpFn349[gcUpTag349, y_] := y /; MatchQ[gcUpRhsGuard349, 1]; Throw[fail]]]"
        ),
        "fail"
    );

    assert_module_gensym_deferred(&session, "gcUpRhsGuard349$1");
    assert_eq!(eval_string(&mut session, "gcUpFn349[gcUpTag349, 7]"), "7");
}

#[test]
fn subvalue_rhs_guard_keeps_its_escaped_module_binding_live() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        eval_string(
            &mut session,
            "Catch[Module[{gcSubRhsGuard349 = 1}, gcSubFn349[1][y_] := y /; MatchQ[gcSubRhsGuard349, 1]; Throw[fail]]]"
        ),
        "fail"
    );

    assert_module_gensym_deferred(&session, "gcSubRhsGuard349$1");
    assert_eq!(eval_string(&mut session, "gcSubFn349[1][7]"), "7");
}

#[test]
fn global_root_keeps_escaped_candidate_until_root_is_cleared() {
    let mut session = session_with_test_first();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcRoot050]; Module[{x}, x = 7; gcRoot050 = Hold[x]; 0]"
        ),
        "0"
    );
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, "First[gcRoot050]"), "7");
    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate))
    );

    session.eval_text("Clear[gcRoot050]").unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(
        candidates
            .iter()
            .all(|candidate| !session.definitions.contains_key(candidate))
    );
}

#[test]
fn transitive_definition_kind_chain_keeps_all_reachable_candidates_live() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcChainRoot078, gcChainHead078]; Module[{a, b}, b = 11; a /: gcChainHead078[a] := b; gcChainRoot078[] := a; 0]"
        ),
        "0"
    );
    let candidates = retained_candidates(&session);
    assert_eq!(candidates.len(), 2);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(
        eval_string(&mut session, "gcChainHead078[gcChainRoot078[]]"),
        "11"
    );
    assert_candidate_definition_count(&session, &candidates, 2);

    session
        .eval_text("ClearAll[gcChainRoot078, gcChainHead078]")
        .unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &candidates, 0);
}

#[test]
fn live_candidate_definition_marks_transitive_candidate_dependencies() {
    let mut session = session_with_test_first();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcTrans050]; Module[{a, b}, b = 11; a := b; gcTrans050 = Hold[a]; 0]"
        ),
        "0"
    );
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 2);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, "First[gcTrans050]"), "11");
    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate)),
        "rooted candidate a$ should keep candidate b$ live through a$'s definition"
    );
}

#[test]
fn empty_candidate_set_sweep_is_a_noop() {
    let mut session = session_with_test_control_builtins();

    session
        .eval_text("ClearAll[gcNoop078]; gcNoop078 = 17")
        .unwrap();
    assert!(session.gc_candidate_gensyms.is_empty());

    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(session.gc_candidate_gensyms.is_empty());
    assert_eq!(eval_string(&mut session, "gcNoop078"), "17");
}

#[test]
fn nested_module_activations_keep_their_candidates_until_each_activation_is_removed() {
    let mut session = session_with_test_control_builtins();
    let outer_candidate = allocate_retained_candidate(&mut session, 7);
    let inner_candidate = allocate_retained_candidate(&mut session, 11);

    session.module_activations.push(ModuleActivation {
        gensyms: vec![outer_candidate],
        escaped: false,
        side_table_stamp: session.current_side_table_stamp(),
    });
    session.module_activations.push(ModuleActivation {
        gensyms: vec![inner_candidate],
        escaped: false,
        side_table_stamp: session.current_side_table_stamp(),
    });

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[outer_candidate, inner_candidate], 2);

    session.module_activations.pop();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[outer_candidate, inner_candidate], 1);
    assert!(session.gc_candidate_gensyms.contains(&outer_candidate));
    assert!(!session.gc_candidate_gensyms.contains(&inner_candidate));

    session.module_activations.clear();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[outer_candidate], 0);
    assert!(!session.gc_candidate_gensyms.contains(&outer_candidate));
}

#[test]
fn suspended_computation_keeps_candidate_live_until_resume_finishes() {
    let mut session = session_with_test_control_builtins();
    let candidate = allocate_retained_candidate(&mut session, 7);
    let source = format!("List[{candidate}, List[InputString[\"gc: \"]]]");

    let failure = session
        .eval_request(&source)
        .expect_err("InputString should suspend the candidate-holding call");
    assert!(matches!(
        failure.error,
        SessionRequestError::InputRequired { .. }
    ));

    let suspension = session.pending_suspension.as_ref().unwrap();
    let candidates = std::collections::HashSet::from([candidate]);
    let mut live = std::collections::HashSet::new();
    let mut worklist = Vec::new();
    for entry in &suspension.active_stack {
        seed_live_gensyms_from_expr(&entry.expr, &candidates, &mut live, &mut worklist);
    }
    assert!(
        live.contains(&candidate),
        "the candidate must be held by the suspension's traversed active stack"
    );

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[candidate], 1);
    assert!(session.gc_candidate_gensyms.contains(&candidate));

    let response = session
        .resume_eval_request_with_input("ready")
        .expect("the suspended candidate-holding call should resume");
    assert_eq!(response.value.to_string(), "List[7, List[\"ready\"]]");
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[candidate], 0);
    assert!(!session.gc_candidate_gensyms.contains(&candidate));
}

#[test]
fn attribute_state_on_dead_candidate_is_cleared_during_reclaim() {
    let mut session = session_with_test_control_builtins();
    let candidate = allocate_retained_candidate(&mut session, 7);
    let mut attributes = session.attribute_set_for_symbol(&candidate);
    attributes.insert(Attribute::HoldFirst);
    session.set_attribute_set_for_symbol(candidate, attributes);

    assert!(session.attributes.contains_key(&candidate));

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[candidate], 0);
    assert!(!session.gc_candidate_gensyms.contains(&candidate));
    assert!(!session.attributes.contains_key(&candidate));
}

#[test]
fn unique_generated_symbols_are_not_module_gc_candidates() {
    let mut session = session_with_test_control_builtins();

    let generated = eval_string(&mut session, "Unique[gcUnique050]");
    assert_eq!(session.gc_candidate_gensyms.len(), 0);

    assert_eq!(
        eval_string(&mut session, &format!("{generated} = 17")),
        "17"
    );
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, &generated), "17");
    assert_eq!(session.gc_candidate_gensyms.len(), 0);
}

#[test]
fn recursive_module_sweep_reclaims_dead_retained_gensyms() {
    const STACK_BYTES: usize = 8 * 1024 * 1024;

    std::thread::Builder::new()
        .name("recursive-module-sweep".to_string())
        .stack_size(STACK_BYTES)
        .spawn(|| run_recursive_module_sweep_reclaims_dead_retained_gensyms(STACK_BYTES))
        .expect("recursive Module sweep thread should start")
        .join()
        .expect("recursive Module sweep should finish");
}

fn run_recursive_module_sweep_reclaims_dead_retained_gensyms(stack_bytes: usize) {
    let mut session = session_with_test_gc_probe_builtins();
    session.set_native_stack_limit(
        native_stack_limit_for_declared_stack(stack_bytes)
            .expect("recursive Module sweep stack should satisfy the supported minimum"),
    );

    session
        .eval_text(
            "ClearAll[gcRec050]; gcRec050[n_] := Module[{r}, If[SameQ[n, 0], 0, r = gcRec050[n - 1] + 1; r]]",
        )
        .unwrap();

    for _ in 0..4 {
        assert_eq!(eval_string(&mut session, "Do[gcRec050[12], {15}]"), "Null");
        assert!(
            session.retained_candidate_definition_count() > 0,
            "recursive Module workload should retain escaped gensyms before sweep"
        );

        session.collect_and_sweep_dead_gensyms(&[]);

        assert_eq!(session.retained_candidate_definition_count(), 0);
        assert_eq!(session.gc_candidate_gensym_count(), 0);
    }

    assert_eq!(eval_string(&mut session, "gcRec050[5]"), "5");
}
