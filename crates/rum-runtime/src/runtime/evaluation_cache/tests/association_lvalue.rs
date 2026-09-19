use super::*;

#[test]
fn association_lvalue_update_preserves_an_aliased_value() {
    let mut session = session_with_compound_expression();
    let outcome = session
        .eval_text("a = <|1 -> 2|>; b = a; b[3] = 4; {a, b}")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[Association[Rule[1, 2]], Association[Rule[1, 2], Rule[3, 4]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof() {
    let (mut semantic_session, semantic_target) =
        session_with_stable_association("directAssociation");

    let replaced = semantic_session
        .eval_text("directAssociation[x] = 2")
        .unwrap();
    assert_eq!(replaced.value.to_string(), "2");

    let appended = semantic_session
        .eval_text("directAssociation[y] = 3")
        .unwrap();
    assert_eq!(appended.value.to_string(), "3");
    assert_eq!(
        semantic_session
            .definitions
            .get(&semantic_target)
            .and_then(|definition| definition.own_value.as_ref())
            .unwrap()
            .to_string(),
        "Association[Rule[x, 2], Rule[y, 3]]"
    );

    // An identity-bearing target writes only the generated lane, while this
    // Association's System symbols write only the regular dependency lane.
    let mut direct_session = Session::default();
    let direct_target = Symbol::with_identity(GLOBAL_CONTEXT, "generatedDirectAssociation", 1);
    direct_session
        .definitions
        .entry(direct_target)
        .or_default()
        .own_value = Some(association_with_integer_keys([1]));
    seed_association_stability_proof(&mut direct_session, &direct_target);
    let before = association_definition_args_key(&direct_session, &direct_target);

    direct_session
        .try_set_association_lvalue(
            association_function_lvalue(direct_target, Expr::small_integer(1)),
            Expr::small_integer(2),
        )
        .unwrap()
        .expect("the generated target must accept the replacement write");
    assert_eq!(
        association_definition_args_key(&direct_session, &direct_target),
        before
    );
    assert!(direct_session.has_valid_call_stability_proof_for_key(
        association_definition_cache_key(&direct_session, &direct_target)
    ));

    direct_session
        .try_set_association_lvalue(
            association_function_lvalue(direct_target, Expr::small_integer(2)),
            Expr::small_integer(3),
        )
        .unwrap()
        .expect("the generated target must accept the append write");
    assert_eq!(
        association_definition_args_key(&direct_session, &direct_target),
        before
    );

    let current = direct_session
        .definitions
        .get(&direct_target)
        .and_then(|definition| definition.own_value.clone())
        .unwrap();
    let (stable, cached_bloom) = direct_session.cached_expr_stability_with_bloom(&current);
    assert!(stable);
    let (head, args) = call_parts(&current);
    let proof = direct_session
        .take_call_stability_proof(head, args)
        .expect("the direct update must reinsert the exact Association proof");
    assert_eq!(proof.bloom, cached_bloom);
}

#[test]
fn association_lvalue_absent_stale_and_dirty_proofs_recover_by_recomputation() {
    let (mut absent_session, absent_target) =
        session_with_stable_association("absentProofAssociation");
    absent_session.clear_evaluation_caches();
    let absent_before = association_definition_args_key(&absent_session, &absent_target);
    let absent_key_before = association_definition_cache_key(&absent_session, &absent_target);
    absent_session
        .eval_text("absentProofAssociation[x] = 2")
        .unwrap();
    assert_eq!(
        association_definition_args_key(&absent_session, &absent_target),
        absent_before
    );
    assert_eq!(
        own_value_text(&absent_session, &absent_target),
        "Association[Rule[x, 2]]"
    );
    // The cache key covers the head as well as the args, so recomputing it
    // here would hide a moved head behind a missing-proof failure. Pin the
    // allocation first, then look the proof up under the key it was seeded
    // with.
    assert_eq!(
        association_definition_cache_key(&absent_session, &absent_target),
        absent_key_before,
        "the recovering write must leave the head and args allocations in place"
    );
    assert!(absent_session.has_valid_call_stability_proof_for_key(absent_key_before));

    let (mut stale_session, stale_target) =
        session_with_stable_association("staleProofAssociation");
    stale_session.call_stability_generation += 1;
    stale_session
        .eval_text("staleProofAssociation[x] = 2")
        .unwrap();
    assert_eq!(
        own_value_text(&stale_session, &stale_target),
        "Association[Rule[x, 2]]"
    );

    let (mut stale_direct_session, stale_direct_target) =
        session_with_stable_identity_association("generatedStaleProofAssociation", 3);
    stale_direct_session.call_stability_generation += 1;
    let stale_before = association_definition_args_key(&stale_direct_session, &stale_direct_target);
    let stale_key_before =
        association_definition_cache_key(&stale_direct_session, &stale_direct_target);
    stale_direct_session
        .try_set_association_lvalue(
            association_function_lvalue(stale_direct_target, Expr::small_integer(1)),
            Expr::small_integer(2),
        )
        .unwrap()
        .expect("the generated target must accept stale-proof recovery");
    assert_eq!(
        association_definition_args_key(&stale_direct_session, &stale_direct_target),
        stale_before
    );
    // The cache key covers the head as well as the args, so recomputing it
    // here would hide a moved head behind a missing-proof failure. Pin the
    // allocation first, then look the proof up under the key it was seeded
    // with.
    assert_eq!(
        association_definition_cache_key(&stale_direct_session, &stale_direct_target),
        stale_key_before,
        "the recovering write must leave the head and args allocations in place"
    );
    assert!(stale_direct_session.has_valid_call_stability_proof_for_key(stale_key_before));

    let (mut dirty_session, dirty_target) =
        session_with_stable_association("dirtyProofAssociation");
    dirty_session.bump_evaluation_epoch_for_symbol(&Symbol::new(GLOBAL_CONTEXT, "x"));
    dirty_session
        .eval_text("dirtyProofAssociation[x] = 2")
        .unwrap();
    assert_eq!(
        own_value_text(&dirty_session, &dirty_target),
        "Association[Rule[x, 2]]"
    );

    let (mut dirty_direct_session, dirty_direct_target) =
        session_with_stable_identity_association("generatedDirtyProofAssociation", 4);
    dirty_direct_session.bump_evaluation_epoch_for_symbol(&Symbol::new(SYSTEM_CONTEXT, "Rule"));
    let dirty_before = association_definition_args_key(&dirty_direct_session, &dirty_direct_target);
    let dirty_key_before =
        association_definition_cache_key(&dirty_direct_session, &dirty_direct_target);
    dirty_direct_session
        .try_set_association_lvalue(
            association_function_lvalue(dirty_direct_target, Expr::small_integer(1)),
            Expr::small_integer(2),
        )
        .unwrap()
        .expect("the generated target must accept dirty-proof recovery");
    assert_eq!(
        association_definition_args_key(&dirty_direct_session, &dirty_direct_target),
        dirty_before
    );
    // The cache key covers the head as well as the args, so recomputing it
    // here would hide a moved head behind a missing-proof failure. Pin the
    // allocation first, then look the proof up under the key it was seeded
    // with.
    assert_eq!(
        association_definition_cache_key(&dirty_direct_session, &dirty_direct_target),
        dirty_key_before,
        "the recovering write must leave the head and args allocations in place"
    );
    assert!(dirty_direct_session.has_valid_call_stability_proof_for_key(dirty_key_before));
}

#[test]
fn association_lvalue_recovery_falls_back_when_data_became_unstable() {
    let (mut session, target) = session_with_stable_association("unstableRecovery");
    session.clear_call_stability_cache_for_size_cap();
    session.eval_text("x = 5").unwrap();
    let before = association_definition_args_key(&session, &target);

    session.eval_text("unstableRecovery[z] = 2").unwrap();

    assert_ne!(
        association_definition_args_key(&session, &target),
        before,
        "a recomputation that finds the stored data unstable must fall back"
    );
    assert!(
        !session.has_valid_call_stability_proof_for_key(association_definition_cache_key(
            &session, &target
        ))
    );
}

#[test]
fn association_lvalue_observable_or_constrained_environments_use_fallback() {
    let (mut tracing_session, tracing_target) =
        session_with_stable_association("tracedAssociation");
    let tracing_before = association_definition_args_key(&tracing_session, &tracing_target);
    tracing_session.tracing_enabled = true;
    tracing_session
        .try_set_association_lvalue(
            association_function_lvalue(tracing_target, Expr::symbol("x")),
            Expr::small_integer(2),
        )
        .unwrap();
    assert_ne!(
        association_definition_args_key(&tracing_session, &tracing_target),
        tracing_before
    );

    let (mut monitor_session, monitor_target) =
        session_with_stable_association("monitoredAssociation");
    let monitor_before = association_definition_args_key(&monitor_session, &monitor_target);
    monitor_session.monitor_scopes.push(MonitorScope {
        display_expr: system_expr("Null"),
        evaluation_monitor: None,
        step_monitor: None,
        last_display: None,
    });
    monitor_session
        .try_set_association_lvalue(
            association_function_lvalue(monitor_target, Expr::symbol("x")),
            Expr::small_integer(2),
        )
        .unwrap();
    assert_ne!(
        association_definition_args_key(&monitor_session, &monitor_target),
        monitor_before
    );

    let (mut time_session, time_target) =
        session_with_stable_association("timeConstrainedAssociation");
    let time_before = association_definition_args_key(&time_session, &time_target);
    time_session.current_time_deadline =
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60));
    time_session
        .try_set_association_lvalue(
            association_function_lvalue(time_target, Expr::symbol("x")),
            Expr::small_integer(2),
        )
        .unwrap();
    assert_ne!(
        association_definition_args_key(&time_session, &time_target),
        time_before
    );

    let (mut memory_session, memory_target) =
        session_with_stable_association("memoryConstrainedAssociation");
    let memory_before = association_definition_args_key(&memory_session, &memory_target);
    memory_session.current_memory_limit = Some(usize::MAX);
    memory_session
        .try_set_association_lvalue(
            association_function_lvalue(memory_target, Expr::symbol("x")),
            Expr::small_integer(2),
        )
        .unwrap();
    assert_ne!(
        association_definition_args_key(&memory_session, &memory_target),
        memory_before
    );

    let (mut up_value_session, up_value_target) =
        session_with_stable_association("upValueAssociation");
    let up_value_before = association_definition_args_key(&up_value_session, &up_value_target);
    up_value_session.has_up_value_definitions = true;
    up_value_session
        .try_set_association_lvalue(
            association_function_lvalue(up_value_target, Expr::symbol("x")),
            Expr::small_integer(2),
        )
        .unwrap();
    assert_ne!(
        association_definition_args_key(&up_value_session, &up_value_target),
        up_value_before
    );
}

#[test]
fn association_lvalue_direct_update_polls_interrupt_before_mutation() {
    let (mut session, target) = session_with_stable_association("interruptedAssociation");
    let original = session
        .definitions
        .get(&target)
        .and_then(|definition| definition.own_value.clone())
        .unwrap();
    session.interrupt_armed = true;
    session.interrupt_poll_counter = 1023;
    let accepted = session
        .accept_cancellable_request()
        .expect("the direct update test should accept a cancellable request");
    let cancel = accepted.cancel_handle();
    session
        .install_accepted_request(accepted)
        .expect("the direct update test ticket should belong to the session");
    cancel.cancel();

    let result = session.try_direct_association_rule_update(
        &target,
        Expr::symbol("x"),
        &Expr::small_integer(2),
    );

    assert!(matches!(result, Err(EvalError::Cancelled)));
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref()),
        Some(&original)
    );
    assert!(
        session.has_valid_call_stability_proof_for_key(association_definition_cache_key(
            &session, &target,
        )),
        "the interrupt poll must happen before the proof is taken"
    );
}

#[test]
fn association_lvalue_nested_multi_and_bare_part_targets_use_fallback() {
    let mut nested_session = Session::default();
    let nested_target = Symbol::new(GLOBAL_CONTEXT, "nestedAssociation");
    nested_session
        .eval_text("nestedAssociation = <|x -> <|y -> 1|>|>")
        .unwrap();
    seed_association_stability_proof(&mut nested_session, &nested_target);
    let nested_before = association_definition_args_key(&nested_session, &nested_target);
    nested_session
        .eval_text("nestedAssociation[x, y] = 2")
        .unwrap();
    assert_ne!(
        association_definition_args_key(&nested_session, &nested_target),
        nested_before
    );

    let mut bare_part_session = Session::default();
    let bare_part_target = Symbol::new(GLOBAL_CONTEXT, "barePartAssociation");
    bare_part_session
        .eval_text("barePartAssociation = <|\"x\" -> 1|>")
        .unwrap();
    seed_association_stability_proof(&mut bare_part_session, &bare_part_target);
    let bare_part_before = association_definition_args_key(&bare_part_session, &bare_part_target);
    bare_part_session
        .eval_text("barePartAssociation[[\"x\"]] = 2")
        .unwrap();
    assert_ne!(
        association_definition_args_key(&bare_part_session, &bare_part_target),
        bare_part_before
    );
    assert_eq!(
        bare_part_session
            .definitions
            .get(&bare_part_target)
            .and_then(|definition| definition.own_value.as_ref())
            .unwrap()
            .to_string(),
        "Association[Rule[\"x\", 2]]"
    );
}

#[test]
fn association_lvalue_direct_update_rolls_back_append_and_replace() {
    let (mut session, target) = session_with_stable_association("rollbackAssociation");
    let original = session
        .definitions
        .get(&target)
        .and_then(|definition| definition.own_value.clone())
        .unwrap();

    session.fail_next_direct_association_update_after_mutation(target);
    let append_outcome = session.eval_text("rollbackAssociation[y] = 2").unwrap();
    assert_eq!(
        append_outcome.termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref()),
        Some(&original)
    );

    seed_association_stability_proof(&mut session, &target);
    session.fail_next_direct_association_update_after_mutation(target);
    let replace_outcome = session.eval_text("rollbackAssociation[x] = 3").unwrap();
    assert_eq!(
        replace_outcome.termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref()),
        Some(&original)
    );
}
