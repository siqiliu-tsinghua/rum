use super::*;
use crate::runtime::test_builtins::register_test_arithmetic_builtins;

fn assert_no_false_part_protection(messages: &[RuntimeMessage]) {
    assert!(
        messages
            .iter()
            .all(|message| !message.text.contains("Part is Protected")),
        "unexpected protected-Part diagnostic: {messages:?}"
    );
}

fn test_key_builtin(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn definition_plain_list_path(
    session: &Session,
    target: &Symbol,
    path: &[usize],
) -> (Vec<CallStabilityCacheKey>, Vec<usize>) {
    let mut selected = session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.as_ref())
        .expect("the Part-assignment test target must have an own value");
    let mut cache_keys = Vec::with_capacity(path.len());
    let mut args_keys = Vec::with_capacity(path.len());
    for &index in path {
        let Expr::Call { head, args } = selected else {
            panic!("the Part-assignment test path must contain only calls");
        };
        assert!(is_system_symbol_name(head, heads::structure::LIST));
        cache_keys.push(CallStabilityCacheKey::new(head, args));
        args_keys.push(args.ptr_key());
        selected = args
            .get(index)
            .expect("the Part-assignment test path must exist");
    }
    (cache_keys, args_keys)
}

fn seed_plain_list_path_stability(
    session: &mut Session,
    target: &Symbol,
    path: &[usize],
) -> (Vec<CallStabilityCacheKey>, Vec<usize>) {
    let current = session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.clone())
        .expect("the Part-assignment test target must have an own value");
    assert!(session.cached_expr_is_evaluation_stable(&current));
    drop(current);

    let (cache_keys, args_keys) = definition_plain_list_path(session, target, path);
    assert!(
        cache_keys
            .iter()
            .all(|key| session.has_valid_call_stability_proof_for_key(*key)),
        "the direct-path precondition must retain every root-to-parent cache entry"
    );
    (cache_keys, args_keys)
}

fn assert_generic_part_assignment_replaces_root_args(session: &mut Session, target: Symbol) {
    let (_, before) = seed_plain_list_path_stability(session, &target, &[0]);
    let lhs = Expr::call(
        system_expr("Part"),
        vec![Expr::Atom(Atom::Symbol(target)), Expr::small_integer(1)],
    );
    let outcome = session
        .try_set_positional_part_lvalue(lhs, Expr::small_integer(9))
        .unwrap();
    assert_eq!(outcome, Some(Expr::small_integer(9)));
    let (_, after) = definition_plain_list_path(session, &target, &[0]);
    assert_ne!(
        after, before,
        "an observable/constrained environment must use the generic rebuilding path"
    );
}

#[test]
fn set_positional_part_supports_flat_nested_and_value_semantics() {
    let mut session = session_with_test_control_builtins();

    let flat = session
        .eval_text("ee = {1, 2, 3}; {ee[[2]] = 99, ee}")
        .unwrap();
    assert_eq!(flat.value.to_string(), "List[99, List[1, 99, 3]]");
    assert!(flat.messages.is_empty());

    let nested = session
        .eval_text("ff = {{1, 2}, {3, 4}}; {ff[[1, 2]] = 9, ff}")
        .unwrap();
    assert_eq!(
        nested.value.to_string(),
        "List[9, List[List[1, 9], List[3, 4]]]"
    );
    assert!(nested.messages.is_empty());

    let aliased = session
        .eval_text("x = {1, 2, 3}; y = x; y[[1]] = 9; {x, y}")
        .unwrap();
    assert_eq!(
        aliased.value.to_string(),
        "List[List[1, 2, 3], List[9, 2, 3]]"
    );
    assert!(aliased.messages.is_empty());
}

#[test]
fn set_positional_part_direct_path_keeps_spine_identity_and_leaves_proofs_evicted() {
    let mut session = Session::default();
    let target = Symbol::new(GLOBAL_CONTEXT, "directPartOwner");
    session
        .eval_text("directPartOwner = {{1, 2}, {3, 4}}")
        .unwrap();
    let (cache_keys, before) = seed_plain_list_path_stability(&mut session, &target, &[0, 1]);

    let first = session.eval_text("directPartOwner[[1, 2]] = 9").unwrap();
    assert_eq!(first.value.to_string(), "9");
    assert!(first.messages.is_empty());
    let (_, after_first) = definition_plain_list_path(&session, &target, &[0, 1]);
    assert_eq!(after_first, before);
    assert!(
        cache_keys
            .iter()
            .all(|key| !session.call_stability_cache.contains_key(key)),
        "changed root-through-parent calls must remain evicted"
    );

    let second = session.eval_text("directPartOwner[[1, 1]] = 8").unwrap();
    assert_eq!(second.value.to_string(), "8");
    assert!(second.messages.is_empty());
    let (_, after_second) = definition_plain_list_path(&session, &target, &[0, 1]);
    assert_eq!(
        after_second, before,
        "a later held Set must remain direct even without reinserted proofs"
    );
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref())
            .unwrap()
            .to_string(),
        "List[List[8, 9], List[3, 4]]"
    );
}

#[test]
fn set_positional_part_cow_preserves_a_root_alias() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "rootOwner = {{1}, {2}}; rootAlias = rootOwner; \
             rootOwner[[1, 1]] = 9; {rootOwner, rootAlias}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[List[9], List[2]], List[List[1], List[2]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn set_positional_part_cow_preserves_an_intermediate_list_alias() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "intermediateOwner = {{{1}}, {{2}}}; \
             intermediateAlias = intermediateOwner[[1, 1]]; \
             intermediateOwner[[1, 1, 1]] = 9; \
             {intermediateOwner, intermediateAlias}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[List[List[9]], List[List[2]]], List[1]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn set_positional_part_cow_preserves_the_selected_child_alias() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "selectedOwner = {{1}, {2}}; selectedAlias = selectedOwner[[1]]; \
             selectedOwner[[1, 1]] = 9; {selectedOwner, selectedAlias}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[List[9], List[2]], List[1]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn set_positional_part_cow_preserves_a_shared_child_at_a_second_position() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "sharedItem = {1}; sharedOwner = {sharedItem, sharedItem}; \
             sharedOwner[[1, 1]] = 9; sharedOwner",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[List[9], List[1]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn set_positional_part_cow_preserves_a_three_level_middle_alias() {
    let mut session = session_with_test_control_builtins();
    let outcome = session
        .eval_text(
            "spineOwner = {{{1}}}; spineMiddle = spineOwner[[1]]; \
             spineOwner[[1, 1, 1]] = 9; {spineOwner, spineMiddle}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[List[List[List[9]]], List[List[1]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn set_positional_part_rederives_after_rhs_and_orders_selectors_left_to_right() {
    let mut session = session_with_test_control_builtins();

    let reassigned = session
        .eval_text("g = {1, 2, 3}; {g[[2]] = (g = {7, 8}; 5), g}")
        .unwrap();
    assert_eq!(reassigned.value.to_string(), "List[5, List[7, 5]]");
    assert!(reassigned.messages.is_empty());

    let ordered = session
        .eval_text(
            "m = {{1, 2}, {3, 4}}; order = {}; \
             {m[[(AppendTo[order, first]; 1), (AppendTo[order, second]; 2)]] = \
                (AppendTo[order, rhs]; 9), order, m}",
        )
        .unwrap();
    assert_eq!(
        ordered.value.to_string(),
        "List[9, List[rhs, first, second], List[List[1, 9], List[3, 4]]]"
    );
    assert!(ordered.messages.is_empty());

    let selector_reassigned = session
        .eval_text("s = {1, 2, 3}; {s[[(s = {7, 8}; 2)]] = 5, s}")
        .unwrap();
    assert_eq!(selector_reassigned.value.to_string(), "List[5, List[7, 5]]");
    assert!(selector_reassigned.messages.is_empty());

    let invalidated_path = session
        .eval_text("short = {1, 2, 3}; {short[[(short = {7}; 2)]] = 5, short}")
        .unwrap();
    assert_eq!(invalidated_path.value.to_string(), "List[5, List[7]]");
    assert!(invalidated_path.messages.iter().any(|message| {
        message.symbol == "Part"
            && message.tag == "partw"
            && message.text == "Part 2 of List[7] does not exist."
    }));
    assert_no_false_part_protection(&invalidated_path.messages);
}

#[test]
fn set_positional_part_refuses_a_target_cleared_by_a_selector() {
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text("c = {1, 2, 3}; {c[[(Clear[c]; 2)]] = 5, c}")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[5, c]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Set"
            && message.tag == "argform"
            && message.text.contains("current List value")
    }));
    assert_no_false_part_protection(&outcome.messages);
}

#[test]
fn set_positional_part_rechecks_protection_after_rhs_and_selector_callbacks() {
    let mut session = session_with_test_control_builtins();

    let rhs_protected = session
        .eval_text("r = {1, 2, 3}; {r[[2]] = (Protect[r]; 5), r}")
        .unwrap();
    assert_eq!(rhs_protected.value.to_string(), "List[5, List[1, 2, 3]]");
    assert!(rhs_protected.messages.iter().any(|message| {
        message.symbol == "Set"
            && message.tag == "wrsym"
            && message.text == "Symbol r is Protected."
    }));
    assert_no_false_part_protection(&rhs_protected.messages);
    session.eval_text("Unprotect[r]").unwrap();

    let selector_protected = session
        .eval_text("p = {1, 2, 3}; {p[[(Protect[p]; 2)]] = 5, p}")
        .unwrap();
    assert_eq!(
        selector_protected.value.to_string(),
        "List[5, List[1, 2, 3]]"
    );
    assert!(selector_protected.messages.iter().any(|message| {
        message.symbol == "Set"
            && message.tag == "wrsym"
            && message.text == "Symbol p is Protected."
    }));
    assert_no_false_part_protection(&selector_protected.messages);
}

#[test]
fn set_positional_part_observable_constrained_and_dynamic_scopes_use_generic_path() {
    let mut tracing_session = Session::default();
    let tracing_target = Symbol::new(GLOBAL_CONTEXT, "tracedPartOwner");
    tracing_session
        .eval_text("tracedPartOwner = {1, 2}")
        .unwrap();
    tracing_session.tracing_enabled = true;
    assert_generic_part_assignment_replaces_root_args(&mut tracing_session, tracing_target);

    let mut monitor_session = Session::default();
    let monitor_target = Symbol::new(GLOBAL_CONTEXT, "monitoredPartOwner");
    monitor_session
        .eval_text("monitoredPartOwner = {1, 2}")
        .unwrap();
    monitor_session.monitor_scopes.push(MonitorScope {
        display_expr: system_expr("Null"),
        evaluation_monitor: None,
        step_monitor: None,
        last_display: None,
    });
    assert_generic_part_assignment_replaces_root_args(&mut monitor_session, monitor_target);

    let mut time_session = Session::default();
    let time_target = Symbol::new(GLOBAL_CONTEXT, "timeConstrainedPartOwner");
    time_session
        .eval_text("timeConstrainedPartOwner = {1, 2}")
        .unwrap();
    time_session.current_time_deadline =
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60));
    assert_generic_part_assignment_replaces_root_args(&mut time_session, time_target);

    let mut memory_session = Session::default();
    let memory_target = Symbol::new(GLOBAL_CONTEXT, "memoryConstrainedPartOwner");
    memory_session
        .eval_text("memoryConstrainedPartOwner = {1, 2}")
        .unwrap();
    memory_session.current_memory_limit = Some(usize::MAX);
    memory_session.note_accounting_arm();
    assert_generic_part_assignment_replaces_root_args(&mut memory_session, memory_target);

    let mut dynamic_session = Session::default();
    let dynamic_target = Symbol::new(GLOBAL_CONTEXT, "dynamicPartOwner");
    dynamic_session
        .eval_text("dynamicPartOwner = {1, 2}")
        .unwrap();
    let dynamic_depth = dynamic_session.push_dynamic_own_value_binding(dynamic_target, None);
    assert_generic_part_assignment_replaces_root_args(&mut dynamic_session, dynamic_target);
    dynamic_session.truncate_dynamic_own_value_bindings(dynamic_depth);
}

#[test]
fn set_positional_part_direct_path_polls_interrupt_before_cache_eviction_or_mutation() {
    let mut session = Session::default();
    let target = Symbol::new(GLOBAL_CONTEXT, "interruptedPartOwner");
    session.eval_text("interruptedPartOwner = {1, 2}").unwrap();
    let (cache_keys, _) = seed_plain_list_path_stability(&mut session, &target, &[0]);
    let original = session
        .definitions
        .get(&target)
        .and_then(|definition| definition.own_value.clone())
        .unwrap();

    session.interrupt_armed = true;
    session.interrupt_poll_counter = 1023;
    let accepted = session
        .accept_cancellable_request()
        .expect("the direct Part test should accept a cancellable request");
    let cancel = accepted.cancel_handle();
    session
        .install_accepted_request(accepted)
        .expect("the direct Part test ticket should belong to the session");
    cancel.cancel();
    let result =
        session.try_direct_positional_part_assignment(&target, &[1], &Expr::small_integer(9));

    assert!(matches!(result, Err(EvalError::Cancelled)));
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref()),
        Some(&original)
    );
    assert!(
        cache_keys
            .iter()
            .all(|key| session.has_valid_call_stability_proof_for_key(*key)),
        "the interrupt poll must precede cache eviction"
    );
}

#[test]
fn set_positional_part_direct_path_rolls_back_slot_and_definition_after_replacement() {
    let mut session = session_with_test_control_builtins();
    let target = Symbol::new(GLOBAL_CONTEXT, "rollbackPartAssignmentOwner");
    session
        .eval_text(
            "rollbackPartAssignmentOwner = {{{1}}}; \
             rollbackRootAlias = rollbackPartAssignmentOwner; \
             rollbackMiddleAlias = rollbackPartAssignmentOwner[[1]]",
        )
        .unwrap();
    seed_plain_list_path_stability(&mut session, &target, &[0, 0, 0]);
    let original = session
        .definitions
        .get(&target)
        .and_then(|definition| definition.own_value.clone())
        .unwrap();
    session.fail_next_positional_part_assignment_after_slot_replacement(target);

    let outcome = session
        .eval_text("rollbackPartAssignmentOwner[[1, 1, 1]] = 9")
        .unwrap();

    assert_eq!(outcome.termination, Some(EvalTerminationReason::Cancelled));
    assert_eq!(
        outcome.value.to_string(),
        "TerminatedEvaluation[\"Cancelled\"]"
    );
    assert_eq!(
        session
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.as_ref()),
        Some(&original),
        "the slot and the whole definition must be restored"
    );
    assert_eq!(
        session
            .eval_text("{rollbackPartAssignmentOwner, rollbackRootAlias, rollbackMiddleAlias}",)
            .unwrap()
            .value
            .to_string(),
        "List[List[List[List[1]]], List[List[List[1]]], List[List[1]]]"
    );
}

#[test]
fn set_positional_part_selector_resume_is_single_pass_and_rederives_the_owner() {
    const SOURCE: &str = "g = {1, 2, 3}; seen = {}; \
        g[[(AppendTo[seen, before]; InputString[\"index: \"]; \
             AppendTo[seen, after]; g = {7, 8}; 2)]] = 9; \
        {g, seen}";

    let mut resumed_session = session_with_test_control_builtins();
    let failure = resumed_session
        .eval_request(SOURCE)
        .expect_err("selector InputString should suspend the request");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "index: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let resumed = resumed_session
        .resume_eval_request_with_input("ready")
        .expect("selector resume should finish the original Part assignment");
    assert_eq!(
        resumed.value.to_string(),
        "List[List[7, 9], List[before, after]]"
    );
    assert!(resumed.messages.is_empty());

    let mut queued_session = session_with_test_control_builtins();
    queued_session.queue_input_response("ready");
    let queued = queued_session
        .eval_request(SOURCE)
        .expect("queued selector input should finish without suspension");

    assert_eq!(queued.value, resumed.value);
    assert_eq!(queued.messages, resumed.messages);
}

#[test]
fn set_positional_part_invalid_targets_report_without_protected_part_fallback() {
    for (source, expected_symbol, expected_tag, expected_value) in [
        (
            "a = {1, 2}; {a[[0]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {1, 2}; {a[[-1]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {1, 2}; {a[[3]] = 9, a}",
            "Part",
            "partw",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {1, 2}; {a[[bad]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
        (
            "a = {f[1]}; {a[[1, 1]] = 9, a}",
            "Set",
            "argform",
            "List[9, List[f[1]]]",
        ),
        (
            "{({1, 2})[[1]] = 9, {1, 2}}",
            "Set",
            "argform",
            "List[9, List[1, 2]]",
        ),
    ] {
        let mut session = session_with_test_control_builtins();
        let outcome = session.eval_text(source).unwrap();

        assert_eq!(outcome.value.to_string(), expected_value, "{source}");
        assert!(
            outcome.messages.iter().any(|message| {
                message.symbol == expected_symbol && message.tag == expected_tag
            }),
            "missing {expected_symbol}::{expected_tag} for {source}: {:?}",
            outcome.messages
        );
        assert_no_false_part_protection(&outcome.messages);
    }
}

#[test]
fn set_positional_part_route_preserves_other_lvalues_and_definition_lhs_forms() {
    let mut session = session_with_test_control_builtins();
    register_test_arithmetic_builtins(&mut session);
    session.register_builtin(BuiltinSpec::ordinary(
        "Key",
        "Test-local association key wrapper.",
        test_key_builtin,
        MemoryGrowthContract::NonAmplifying,
    ));

    let associations = session
        .eval_text(
            "aa = <|1 -> 2|>; bb = <|1 -> 2|>; \
             aa[[Key[1]]] = 99; bb[1] = 99; {aa, bb}",
        )
        .unwrap();
    assert_eq!(
        associations.value.to_string(),
        "List[Association[Rule[1, 99]], Association[Rule[1, 99]]]"
    );
    assert!(associations.messages.is_empty());

    let compound = session
        .eval_text(
            "dd = {1, 2, 3}; cc = {{1}, {2}}; \
             dd[[2]] += 10; AppendTo[cc[[1]], 9]; {dd, cc}",
        )
        .unwrap();
    assert_eq!(
        compound.value.to_string(),
        "List[List[1, 12, 3], List[List[1, 9], List[2]]]"
    );
    assert!(compound.messages.is_empty());

    let ordinary = session
        .eval_text(
            "{left, right} = {1, 2}; \
             f[x_] := x; f[1] = 2; {{left, right}, f[1], f[3]}",
        )
        .unwrap();
    assert_eq!(ordinary.value.to_string(), "List[List[1, 2], 2, 3]");
    assert!(ordinary.messages.is_empty());
}
