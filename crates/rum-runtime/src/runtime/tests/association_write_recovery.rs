use super::*;

fn association_entries<'a>(session: &'a Session, target: &Symbol) -> &'a SharedArgs {
    let current = session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.as_ref())
        .expect("the write-recovery target must have an Association own value");
    let Expr::Call { head, args, .. } = current else {
        panic!("the write-recovery target must store an Association");
    };
    assert!(is_system_symbol_name(head, "Association"));
    args
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

fn install_association(session: &mut Session, target: Symbol, keys: impl IntoIterator<Item = i64>) {
    session.definitions.entry(target).or_default().own_value =
        Some(association_with_integer_keys(keys));
}

fn seed_association_stability_proof(session: &mut Session, target: &Symbol) {
    let current = session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.clone())
        .expect("the write-recovery target must have an Association own value");
    assert!(session.cached_expr_is_evaluation_stable(&current));
}

fn association_function_lvalue(target: Symbol, key: Expr) -> Expr {
    Expr::call(Expr::Atom(Atom::Symbol(target)), vec![key])
}

#[test]
fn association_incremental_writes_preserve_ordinary_symbol_semantics_past_the_cache_size_cap() {
    let mut session = session_with_test_control_builtins();
    let target = Symbol::new(GLOBAL_CONTEXT, "bigAssoc");
    install_association(&mut session, target, 1..=16000);
    seed_association_stability_proof(&mut session, &target);

    session
        .eval_text("Do[bigAssoc[i] = i, {i, 16001, 16500}]")
        .unwrap();

    let entries = association_entries(&session, &target);
    assert_eq!(entries.len(), 16500);
    for expected in [1_i64, 16001, 16385, 16500] {
        let key = Expr::small_integer(expected);
        assert!(
            entries.iter().any(|entry| {
                let Expr::Call { head, args, .. } = entry else {
                    return false;
                };
                is_system_symbol_name(head, "Rule") && args.len() == 2 && args[0] == key
            }),
            "the association must contain the key {expected}"
        );
    }
}

/// Incremental writes must not fall off the direct path once the session
/// crosses the call-stability cache's entry cap. Seeding a 16 000-key
/// Association puts the cache near `CALL_STABILITY_CACHE_MAX_ENTRIES`
/// (1 << 14); a few hundred more writes push the per-write rule entries past
/// it, the size-cap clear wipes the proof mid-loop, and the recovery must
/// re-derive it. The args allocation chosen before the crossing is still the
/// one in place at the end — a write that fell back would store a rebuilt
/// value under a fresh allocation and change that key permanently. (The
/// end-to-end 20 000-key loop is covered by the release-CLI timings in this
/// brief; this compressed shape keeps the debug runtime tolerable.)
#[test]
fn association_incremental_writes_stay_direct_past_the_cache_size_cap() {
    let mut session = session_with_test_control_builtins();
    let target = Symbol::with_identity(GLOBAL_CONTEXT, "generatedBigAssoc", 6);
    install_association(&mut session, target, 1..=16000);
    // The identity-bearing target writes only the generated lane, while this
    // Association's System symbols and integer keys depend only on the regular
    // lane. Mark the target dirty before seeding so the loop exercises the
    // rewritten-dirty case after the cache-cap recovery.
    session.bump_evaluation_epoch_for_symbol(&target);
    seed_association_stability_proof(&mut session, &target);
    let seeded_args = association_entries(&session, &target).ptr_key();

    for key in 16001_i64..=16500 {
        session
            .try_set_association_lvalue(
                association_function_lvalue(target, Expr::small_integer(key)),
                Expr::small_integer(key),
            )
            .unwrap()
            .expect("the generated target must accept the incremental write");
    }

    let entries = association_entries(&session, &target);
    assert_eq!(
        entries.ptr_key(),
        seeded_args,
        "a fallback after the size-cap clear would have replaced the args allocation"
    );
    assert!(
        session.association_lvalue_has_valid_direct_proof(&target),
        "the loop must end with a live direct-write proof"
    );
}
