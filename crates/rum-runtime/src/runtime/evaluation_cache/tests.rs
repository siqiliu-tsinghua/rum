use super::*;

fn session_with_compound_expression() -> Session {
    crate::runtime::test_builtins::session_with_test_compound_expression_builtin()
}

fn call_parts(expr: &Expr) -> (&SharedExpr, &SharedArgs) {
    let Expr::Call { head, args, .. } = expr else {
        panic!("expected a call expression");
    };
    (head, args)
}

fn cache_stable_call(session: &mut Session, expr: &Expr) -> StabilityBloom {
    let (stable, bloom) = session.cached_expr_stability_with_bloom(expr);
    assert!(stable);
    let (head, args) = call_parts(expr);
    assert!(
        session
            .call_stability_cache
            .contains_key(&CallStabilityCacheKey::new(head, args))
    );
    bloom
}

fn association_definition_cache_key(session: &Session, target: &Symbol) -> CallStabilityCacheKey {
    let current = session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.as_ref())
        .expect("the test target must have an Association own value");
    let (head, args) = call_parts(current);
    assert!(is_system_symbol_name(head, "Association"));
    CallStabilityCacheKey::new(head, args)
}

fn association_definition_args_key(session: &Session, target: &Symbol) -> usize {
    let current = session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.as_ref())
        .expect("the test target must have an Association own value");
    call_parts(current).1.ptr_key()
}

fn own_value_text(session: &Session, target: &Symbol) -> String {
    session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.as_ref())
        .expect("the test target must have an own value")
        .to_string()
}

fn seed_association_stability_proof(session: &mut Session, target: &Symbol) {
    let current = session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.clone())
        .expect("the test target must have an Association own value");
    assert!(session.cached_expr_is_evaluation_stable(&current));
    drop(current);
    let key = association_definition_cache_key(session, target);
    assert!(session.has_valid_call_stability_proof_for_key(key));
}

fn association_function_lvalue(target: Symbol, key: Expr) -> Expr {
    Expr::call(Expr::Atom(Atom::Symbol(target)), vec![key])
}

fn session_with_stable_association(name: &str) -> (Session, Symbol) {
    let mut session = Session::default();
    let target = Symbol::new(GLOBAL_CONTEXT, name);
    session.eval_text(&format!("{name} = <|x -> 1|>")).unwrap();
    seed_association_stability_proof(&mut session, &target);
    (session, target)
}

fn association_with_integer_keys(keys: impl IntoIterator<Item = i64>) -> Expr {
    Expr::call(
        system_expr("Association"),
        keys.into_iter()
            .map(|key| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::small_integer(key), Expr::small_integer(key)],
                )
            })
            .collect(),
    )
}

fn session_with_stable_identity_association(name: &str, identity: u64) -> (Session, Symbol) {
    let mut session = Session::default();
    let target = Symbol::with_identity(GLOBAL_CONTEXT, name, identity);
    session.definitions.entry(target).or_default().own_value =
        Some(association_with_integer_keys([1]));
    session.bump_evaluation_epoch_for_symbol(&target);
    seed_association_stability_proof(&mut session, &target);
    (session, target)
}

fn session_with_stable_identity_list(name: &str, identity: u64) -> (Session, Symbol) {
    let mut session = Session::default();
    let target = Symbol::with_identity(GLOBAL_CONTEXT, name, identity);
    session.definitions.entry(target).or_default().own_value =
        Some(Expr::call(system_expr("List"), Vec::new()));
    session.bump_evaluation_epoch_for_symbol(&target);

    let current = session
        .definitions
        .get(&target)
        .and_then(|definition| definition.own_value.clone())
        .expect("the identity target must have a List own value");
    assert!(session.cached_expr_is_evaluation_stable(&current));
    drop(current);
    (session, target)
}

#[test]
fn association_stability_index_preserves_distinct_and_duplicate_verdicts() {
    let mut session = Session::default();
    let indexed_len = ASSOCIATION_INDEX_THRESHOLD as i64 + 1;
    let distinct = association_with_integer_keys(0..indexed_len);
    assert!(session.cached_expr_is_evaluation_stable(&distinct));

    let duplicate = association_with_integer_keys((0..indexed_len).chain([0]));
    assert!(!session.cached_expr_is_evaluation_stable(&duplicate));
}

#[test]
fn append_to_args_become_exclusive_after_definition_take_and_cache_entry_removal() {
    let mut semantic_session = Session::default();
    semantic_session.eval_text("lst = {}").unwrap();
    semantic_session.eval_text("AppendTo[lst, 1]").unwrap();
    assert_eq!(
        own_value_text(&semantic_session, &Symbol::new(GLOBAL_CONTEXT, "lst")),
        "List[1]"
    );

    // The identity-bearing target is dirty only in the generated lane, while
    // the List proof depends only on the regular lane.
    let (mut session, target) = session_with_stable_identity_list("generatedAppendTarget", 2);
    let updated = session
        .try_incremental_append_to_plain_list(
            &Expr::Atom(Atom::Symbol(target)),
            &Expr::small_integer(1),
        )
        .unwrap()
        .expect("the identity target must accept the incremental append");
    assert_eq!(updated.to_string(), "List[1]");
    drop(updated);

    let mut current = session
        .definitions
        .get_mut(&target)
        .and_then(|definition| definition.own_value.take())
        .expect("AppendTo must leave its updated List in the definition");
    let Expr::Call { head, args, .. } = &mut current else {
        panic!("AppendTo must store a call expression");
    };
    let key = CallStabilityCacheKey::new(head, args);

    assert!(
        args.get_mut().is_none(),
        "the exact call-stability entry must initially retain the SharedArgs"
    );
    let proof = session
        .take_call_stability_proof(head, args)
        .expect("the incremental AppendTo result must have an exact call-stability entry");
    assert!(!session.call_stability_cache.contains_key(&key));
    assert_ne!(proof.bloom, StabilityBloom::default());

    assert!(
        args.get_mut().is_some(),
        "cache-entry removal plus definition take must leave exactly one strong SharedArgs owner"
    );
}

#[test]
fn shared_args_make_mut_profiles_only_cow_and_preserves_alias_values() {
    let profiling_enabled = rum_core::repr_profile_enabled();

    let mut unique = SharedArgs::new(vec![Expr::small_integer(1)]);
    let unique_key = unique.ptr_key();
    let before_unique_mutation = rum_core::repr_profile_snapshot();
    assert!(unique.get_mut().is_some());
    unique.make_mut().push(Expr::small_integer(2));
    let after_unique_mutation = rum_core::repr_profile_snapshot();
    assert_eq!(unique.ptr_key(), unique_key);
    assert_eq!(
        after_unique_mutation.shared_args_alloc,
        before_unique_mutation.shared_args_alloc
    );

    let mut aliased = SharedArgs::new(vec![Expr::small_integer(3)]);
    let alias = aliased.clone();
    let alias_key = alias.ptr_key();
    let before_cow = rum_core::repr_profile_snapshot();
    assert!(aliased.get_mut().is_none());
    aliased.make_mut().push(Expr::small_integer(4));
    let after_cow = rum_core::repr_profile_snapshot();

    assert_ne!(aliased.ptr_key(), alias_key);
    assert_eq!(alias.as_ref(), &[Expr::small_integer(3)]);
    assert_eq!(
        aliased.as_ref(),
        &[Expr::small_integer(3), Expr::small_integer(4)]
    );
    assert_eq!(
        after_cow.shared_args_alloc - before_cow.shared_args_alloc,
        u64::from(profiling_enabled)
    );
}

#[test]
fn removing_valid_call_stability_entry_preserves_evaluation_outcome() {
    let mut session = session_with_compound_expression();
    session.eval_text("stableExpr = h[1, 2]").unwrap();
    session.clear_evaluation_caches();
    session.eval_text("stableExpr").unwrap();
    let with_entry = session.eval_text("stableExpr").unwrap();
    let (head, args) = call_parts(&with_entry.value);

    assert!(session.take_call_stability_proof(head, args).is_some());
    let without_entry = session.eval_text("stableExpr").unwrap();

    assert_eq!(without_entry, with_entry);
}

#[test]
fn generation_stale_pointer_matched_entry_is_evicted_without_proof() {
    let mut session = Session::default();
    let expr = Expr::call(system_expr("List"), vec![Expr::small_integer(1)]);
    cache_stable_call(&mut session, &expr);
    let (head, args) = call_parts(&expr);
    let key = CallStabilityCacheKey::new(head, args);
    session.call_stability_generation += 1;

    assert!(session.take_call_stability_proof(head, args).is_none());
    assert!(!session.call_stability_cache.contains_key(&key));
}

#[test]
fn dirty_pointer_matched_entry_is_evicted_without_proof() {
    let mut session = Session::default();
    let observed = Symbol::new(GLOBAL_CONTEXT, "observed");
    let expr = Expr::call(
        Expr::Atom(Atom::Symbol(observed)),
        vec![Expr::small_integer(1)],
    );
    cache_stable_call(&mut session, &expr);
    let (head, args) = call_parts(&expr);
    let key = CallStabilityCacheKey::new(head, args);
    session.bump_evaluation_epoch_for_symbol(&observed);

    assert!(session.take_call_stability_proof(head, args).is_none());
    assert!(!session.call_stability_cache.contains_key(&key));
}

#[test]
fn pointer_mismatched_entry_is_left_untouched() {
    let mut session = Session::default();
    let requested = Expr::call(system_expr("List"), vec![Expr::small_integer(1)]);
    let mismatched = Expr::call(system_expr("List"), vec![Expr::small_integer(2)]);
    let (requested_head, requested_args) = call_parts(&requested);
    let (mismatched_head, mismatched_args) = call_parts(&mismatched);
    let key = CallStabilityCacheKey::new(requested_head, requested_args);
    session.call_stability_cache.insert(
        key,
        CallStabilityCacheEntry {
            generation: session.call_stability_generation,
            head: mismatched_head.clone(),
            args: mismatched_args.clone(),
            stable: true,
            bloom: StabilityBloom::default(),
            dirty_snapshot: session.stability_dirty_bloom,
        },
    );

    assert!(
        session
            .take_call_stability_proof(requested_head, requested_args)
            .is_none()
    );
    assert!(session.take_call_stability_proof_for_key(key).is_none());
    let remaining = session
        .call_stability_cache
        .get(&key)
        .expect("a pointer mismatch must leave the cache entry untouched");
    assert!(SharedExpr::ptr_eq(&remaining.head, mismatched_head));
    assert!(SharedArgs::ptr_eq(&remaining.args, mismatched_args));
}

#[test]
fn valid_call_stability_proof_preflight_is_non_consuming_and_reuses_validity() {
    let mut session = Session::default();
    let expr = Expr::call(system_expr("List"), vec![Expr::small_integer(1)]);
    cache_stable_call(&mut session, &expr);
    let (head, args) = call_parts(&expr);
    let key = CallStabilityCacheKey::new(head, args);

    assert!(session.has_valid_call_stability_proof_for_key(key));
    assert!(session.call_stability_cache.contains_key(&key));

    session.call_stability_generation += 1;
    assert!(!session.has_valid_call_stability_proof_for_key(key));
    assert!(
        session.call_stability_cache.contains_key(&key),
        "preflight must not remove a stale entry"
    );
}

#[test]
fn proof_reinsertion_uses_current_generation_dirty_snapshot_and_bloom() {
    let mut session = Session::default();
    let expr = Expr::call(system_expr("List"), vec![Expr::small_integer(1)]);
    cache_stable_call(&mut session, &expr);
    let (head, args) = call_parts(&expr);
    let mut proof = session
        .take_call_stability_proof(head, args)
        .expect("the stable List must yield a proof");

    let newly_dirty = Symbol::new(GLOBAL_CONTEXT, "newlyDirty");
    session.bump_evaluation_epoch_for_symbol(&newly_dirty);
    session.call_stability_generation += 1;
    proof.bloom.union_assign(StabilityBloom::regular(
        super::stability::symbol_stability_bloom_bit(&newly_dirty),
    ));
    let expected_bloom = proof.bloom;
    let current_generation = session.call_stability_generation;
    let current_dirty_snapshot = session.stability_dirty_bloom;

    session.reinsert_call_stability_proof(head, args, proof);

    let entry = session
        .call_stability_cache
        .get(&CallStabilityCacheKey::new(head, args))
        .expect("reinsertion must seed the exact entry");
    assert_eq!(entry.generation, current_generation);
    assert_eq!(entry.dirty_snapshot, current_dirty_snapshot);
    assert_eq!(entry.bloom, expected_bloom);
    assert!(entry.stable);
    let (stable, hit_bloom) = session.cached_expr_stability_with_bloom(&expr);
    assert!(stable);
    assert_eq!(hit_bloom, expected_bloom);
}

#[test]
fn path_eviction_removes_only_the_named_exact_call_nodes() {
    let mut session = Session::default();
    let leaf = Expr::call(Expr::symbol("leafHead"), vec![Expr::small_integer(1)]);
    let middle = Expr::call(Expr::symbol("middleHead"), vec![leaf]);
    let nested = Expr::call(Expr::symbol("outerHead"), vec![middle]);
    let unrelated = Expr::call(Expr::symbol("unrelatedHead"), vec![Expr::small_integer(2)]);
    cache_stable_call(&mut session, &nested);
    cache_stable_call(&mut session, &unrelated);

    let (outer_head, outer_args) = call_parts(&nested);
    let (middle_head, middle_args) = call_parts(&outer_args[0]);
    let (leaf_head, leaf_args) = call_parts(&middle_args[0]);
    let (unrelated_head, unrelated_args) = call_parts(&unrelated);
    let outer_key = CallStabilityCacheKey::new(outer_head, outer_args);
    let middle_key = CallStabilityCacheKey::new(middle_head, middle_args);
    let leaf_key = CallStabilityCacheKey::new(leaf_head, leaf_args);
    let unrelated_key = CallStabilityCacheKey::new(unrelated_head, unrelated_args);

    session.evict_call_stability_path(&[(outer_head, outer_args), (middle_head, middle_args)]);

    assert!(!session.call_stability_cache.contains_key(&outer_key));
    assert!(!session.call_stability_cache.contains_key(&middle_key));
    assert!(session.call_stability_cache.contains_key(&leaf_key));
    assert!(session.call_stability_cache.contains_key(&unrelated_key));

    cache_stable_call(&mut session, &nested);
    session.evict_call_stability_keys(&[outer_key, middle_key]);

    assert!(!session.call_stability_cache.contains_key(&outer_key));
    assert!(!session.call_stability_cache.contains_key(&middle_key));
    assert!(session.call_stability_cache.contains_key(&leaf_key));
    assert!(session.call_stability_cache.contains_key(&unrelated_key));
}

#[test]
fn append_to_preserves_an_aliased_list_value() {
    let mut session = session_with_compound_expression();
    let outcome = session
        .eval_text("x = {1, 2}; y = x; AppendTo[y, 3]; {x, y}")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[List[1, 2], List[1, 2, 3]]");
    assert!(outcome.messages.is_empty());
}

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

#[test]
fn evaluation_cache_is_invalidated_by_new_down_value() {
    let mut session = Session::default();

    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "f[1]");
    assert!(!session.symbol_call_effect_cache.is_empty());

    session.eval_text("f[x_] := x + 1").unwrap();

    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "2");
}

#[test]
fn evaluation_cache_is_invalidated_by_attribute_change() {
    let mut session = Session::default();

    assert_eq!(
        session.eval_text("f[{a, b}]").unwrap().value.to_string(),
        "f[List[a, b]]"
    );
    assert!(!session.symbol_call_effect_cache.is_empty());

    session.eval_text("SetAttributes[f, Listable]").unwrap();

    assert_eq!(
        session.eval_text("f[{a, b}]").unwrap().value.to_string(),
        "List[f[a], f[b]]"
    );
}

#[test]
fn repeated_writes_to_one_symbol_keep_invalidation_sound() {
    let mut session = session_with_compound_expression();

    session.eval_text("g[] := (s = s + 1; s)").unwrap();
    session.eval_text("s = 0").unwrap();
    assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "1");
    assert_eq!(session.eval_text("g[]").unwrap().value.to_string(), "2");
    assert_eq!(session.eval_text("s").unwrap().value.to_string(), "2");
}

#[test]
fn stability_cache_survives_unrelated_symbol_writes() {
    let mut session = Session::default();

    // Populate the call-stability cache with a stable expression.
    assert_eq!(
        session.eval_text("h[1, 2]").unwrap().value.to_string(),
        "h[1, 2]"
    );

    // Writing an unrelated symbol must not change the stable verdict...
    session.eval_text("unrelated = 5").unwrap();
    assert_eq!(
        session.eval_text("h[1, 2]").unwrap().value.to_string(),
        "h[1, 2]"
    );

    // ...while a write to the observed head must flip it.
    session.eval_text("h[x_, y_] := x + y").unwrap();
    assert_eq!(session.eval_text("h[1, 2]").unwrap().value.to_string(), "3");
}

#[test]
fn repeated_unrelated_writes_do_not_bump_call_stability_generation() {
    let mut session = Session::default();

    session.eval_text("stableExpr = h[1, 2, 3]").unwrap();
    session.clear_evaluation_caches();
    assert_eq!(
        session.eval_text("stableExpr").unwrap().value.to_string(),
        "h[1, 2, 3]"
    );
    assert!(!session.call_stability_cache.is_empty());

    let generation = session.call_stability_generation;
    session.eval_text("unrelatedWrite = 1").unwrap();
    session.eval_text("unrelatedWrite = 2").unwrap();
    session.eval_text("unrelatedWrite = 3").unwrap();

    assert_eq!(session.call_stability_generation, generation);
    assert_eq!(
        session.eval_text("stableExpr").unwrap().value.to_string(),
        "h[1, 2, 3]"
    );
}
