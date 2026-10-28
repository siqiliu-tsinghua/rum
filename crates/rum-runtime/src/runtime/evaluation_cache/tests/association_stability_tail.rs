use super::super::stability::{
    CallStabilityCacheTraceEvent, call_stability_cache_max_entries,
    start_call_stability_cache_trace, take_call_stability_cache_trace,
};
use super::*;

fn rule(key: Expr, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![key, value])
}

fn rule_delayed(key: Expr, value: Expr) -> Expr {
    Expr::call(system_expr("RuleDelayed"), vec![key, value])
}

fn association(entries: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Association"), entries)
}

fn traced_stability(
    session: &mut Session,
    expr: &Expr,
    recursive_front_depth: usize,
) -> ((bool, StabilityBloom), Vec<CallStabilityCacheTraceEvent>) {
    let _entry = crate::runtime::native_stack_guard::NativeStackEntry::enter();
    start_call_stability_cache_trace();
    let result =
        session.association_rooted_cached_expr_stability_with_bloom(expr, recursive_front_depth);
    (result, take_call_stability_cache_trace())
}

fn assert_recursive_and_tail_traces_match(
    expr: &Expr,
    prepare: impl Fn(&mut Session),
) -> ((bool, StabilityBloom), Vec<CallStabilityCacheTraceEvent>) {
    let mut recursive_session = Session::default();
    let mut tail_session = Session::default();
    prepare(&mut recursive_session);
    prepare(&mut tail_session);

    let recursive = traced_stability(&mut recursive_session, expr, 64);
    let tail = traced_stability(&mut tail_session, expr, 0);
    assert_eq!(tail, recursive);
    tail
}

fn fill_cache_past_size_cap(session: &mut Session) {
    for value in 0..=call_stability_cache_max_entries() {
        let expr = Expr::call(
            Expr::symbol("stabilityCapSeed"),
            vec![Expr::small_integer(value as i64)],
        );
        let (head, args) = call_parts(&expr);
        session.call_stability_cache.insert(
            CallStabilityCacheKey::new(head, args),
            CallStabilityCacheEntry {
                generation: session.call_stability_generation,
                head: head.clone(),
                args: args.clone(),
                stable: true,
                bloom: StabilityBloom::default(),
                dirty_snapshot: session.stability_dirty_bloom,
            },
        );
    }
    assert!(session.call_stability_cache.len() > call_stability_cache_max_entries());
}

fn interleaved_association_subtree(wrapper_count: usize) -> Expr {
    let mut value = Expr::string("interleaved-association-tail");
    for _ in 0..wrapper_count {
        let wrapped_value = Expr::call(Expr::symbol("ff"), vec![value]);
        value = association(vec![rule(Expr::string("k"), wrapped_value)]);
    }
    value
}

#[test]
fn recursive_front_exhaustion_owns_intervening_ordinary_calls() {
    let expr = interleaved_association_subtree(20);
    let mut recursive_session = Session::default();
    let recursive = traced_stability(&mut recursive_session, &expr, 128);

    let mut bounded_session = Session::default();
    bounded_session.reset_association_stability_tail_entry_count();
    let bounded = traced_stability(&mut bounded_session, &expr, 16);

    assert_eq!(bounded, recursive);
    assert_eq!(
        bounded_session.association_stability_tail_entry_count(),
        1,
        "one exhausted tail must own the rest of the rooted subtree without resetting at nested associations"
    );
}

#[test]
fn duplicate_key_cache_trace_matches_recursive_front() {
    let first_value = Expr::call(
        Expr::symbol("firstDuplicateValue"),
        vec![Expr::small_integer(1)],
    );
    let skipped_value = Expr::call(
        Expr::symbol("skippedDuplicateValue"),
        vec![Expr::small_integer(2)],
    );
    let first_rule = rule(Expr::string("duplicate"), first_value.clone());
    let expr = association(vec![
        first_rule.clone(),
        rule(Expr::string("duplicate"), skipped_value.clone()),
    ]);

    let (result, trace) = assert_recursive_and_tail_traces_match(&expr, |_| {});
    assert!(!result.0);
    assert_eq!(
        trace,
        vec![
            CallStabilityCacheTraceEvent::Insert(first_value),
            CallStabilityCacheTraceEvent::Insert(first_rule),
        ],
        "the duplicate must be detected only after the preceding entry is cached"
    );
}

#[test]
fn rule_delayed_cache_trace_matches_recursive_front() {
    let skipped_rhs = Expr::call(
        system_expr("Set"),
        vec![Expr::symbol("skippedDelayedRhs"), Expr::small_integer(1)],
    );
    let delayed_rule = rule_delayed(Expr::string("delayed"), skipped_rhs.clone());
    let later_value = Expr::call(
        Expr::symbol("laterImmediateValue"),
        vec![Expr::small_integer(2)],
    );
    let later_rule = rule(Expr::string("later"), later_value.clone());
    let expr = association(vec![delayed_rule.clone(), later_rule.clone()]);

    let (result, trace) = assert_recursive_and_tail_traces_match(&expr, |_| {});
    assert!(result.0);
    assert_eq!(
        trace,
        vec![
            CallStabilityCacheTraceEvent::Insert(delayed_rule),
            CallStabilityCacheTraceEvent::Insert(later_value),
            CallStabilityCacheTraceEvent::Insert(later_rule),
            CallStabilityCacheTraceEvent::Insert(expr),
        ],
        "RuleDelayed must cache after its lhs without consulting its rhs"
    );
    assert!(
        trace.iter().all(|event| {
            !matches!(event, CallStabilityCacheTraceEvent::Insert(inserted) if inserted == &skipped_rhs)
        }),
        "the delayed rhs must remain unvisited"
    );
}

#[test]
fn dirty_and_generation_cache_traces_match_recursive_front() {
    let dirty_symbol = Symbol::new(GLOBAL_CONTEXT, "dirtyStabilityTraceSymbol");
    let value = Expr::call(
        Expr::symbol("dirtyTraceValue"),
        vec![Expr::Atom(Atom::Symbol(dirty_symbol))],
    );
    let rule = rule(Expr::string("dirty"), value.clone());
    let expr = association(vec![rule.clone()]);
    let expected = vec![
        CallStabilityCacheTraceEvent::Insert(value),
        CallStabilityCacheTraceEvent::Insert(rule),
        CallStabilityCacheTraceEvent::Insert(expr.clone()),
    ];

    let (dirty_result, dirty_trace) = assert_recursive_and_tail_traces_match(&expr, |session| {
        assert!(
            session
                .association_rooted_cached_expr_stability_with_bloom(&expr, 64)
                .0
        );
        session.bump_evaluation_epoch_for_symbol(&dirty_symbol);
    });
    assert!(dirty_result.0);
    assert_eq!(dirty_trace, expected);

    let (generation_result, generation_trace) =
        assert_recursive_and_tail_traces_match(&expr, |session| {
            assert!(
                session
                    .association_rooted_cached_expr_stability_with_bloom(&expr, 64)
                    .0
            );
            session.call_stability_generation += 1;
        });
    assert!(generation_result.0);
    assert_eq!(generation_trace, expected);
}

#[test]
fn unstable_early_child_cache_trace_matches_recursive_front() {
    let unstable_symbol = Symbol::new(GLOBAL_CONTEXT, "unstableEarlyTraceValue");
    let early_key = Expr::call(
        Expr::symbol("evaluatedEarlyTraceKey"),
        vec![Expr::small_integer(1)],
    );
    let later_value = Expr::call(
        Expr::symbol("mustRemainUnvisited"),
        vec![Expr::small_integer(2)],
    );
    let expr = association(vec![
        rule(early_key.clone(), Expr::Atom(Atom::Symbol(unstable_symbol))),
        rule(Expr::string("later"), later_value),
    ]);

    let (result, trace) = assert_recursive_and_tail_traces_match(&expr, |session| {
        session
            .definitions
            .entry(unstable_symbol)
            .or_default()
            .own_value = Some(Expr::small_integer(7));
    });
    assert!(!result.0);
    assert_eq!(
        trace,
        vec![CallStabilityCacheTraceEvent::Insert(early_key)],
        "the first unstable value must prevent every later child visit"
    );
}

#[test]
fn cache_cap_postorder_insertion_trace_matches_recursive_front() {
    let leaf = Expr::call(
        Expr::symbol("associationCapLeaf"),
        vec![Expr::small_integer(1)],
    );
    let inner_rule = rule(Expr::string("k"), leaf.clone());
    let inner = association(vec![inner_rule.clone()]);
    let outer_rule = rule(Expr::string("k"), inner.clone());
    let expr = association(vec![outer_rule.clone()]);

    let (result, trace) = assert_recursive_and_tail_traces_match(&expr, |session| {
        fill_cache_past_size_cap(session);
    });
    assert!(result.0);
    assert_eq!(
        trace,
        vec![
            CallStabilityCacheTraceEvent::ClearForSizeCap,
            CallStabilityCacheTraceEvent::Insert(leaf),
            CallStabilityCacheTraceEvent::Insert(inner_rule),
            CallStabilityCacheTraceEvent::Insert(inner),
            CallStabilityCacheTraceEvent::Insert(outer_rule),
            CallStabilityCacheTraceEvent::Insert(expr),
        ]
    );
}
