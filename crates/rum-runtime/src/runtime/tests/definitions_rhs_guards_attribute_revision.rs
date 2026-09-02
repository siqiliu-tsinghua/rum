use super::*;

fn revision_guard_session() -> Session {
    session_with_test_control_builtins()
}

fn install_every_guarded_revision_lane(session: &mut Session) {
    session
        .eval_text(
            "revisionDown[x_] := \"down\" /; MatchQ[x, 1]; \
             revisionUpTag /: revisionUpHead[revisionUpTag, x_] := \"up\" /; MatchQ[x, 1]; \
             revisionSub[1][x_] := \"sub\" /; MatchQ[x, 1]; \
             Default[revisionDefault] := \"default\" /; revisionDefaultGuard; \
             revisionDefault[x_.] := x; \
             revisionRule = x_Integer :> (\"rewrite\" /; MatchQ[x, 1])",
        )
        .unwrap();
}

fn assert_every_guarded_revision_lane(session: &mut Session) {
    session.eval_text("revisionDefaultGuard = True").unwrap();
    let accepted = session
        .eval_text(
            "{revisionDown[1], revisionUpHead[revisionUpTag, 1], \
              revisionSub[1][1], revisionDefault[], Replace[1, revisionRule]}",
        )
        .unwrap();
    assert_eq!(
        accepted.value.to_string(),
        "List[\"down\", \"up\", \"sub\", \"default\", \"rewrite\"]"
    );
    assert!(accepted.messages.is_empty());

    session.eval_text("revisionDefaultGuard = False").unwrap();
    let declined = session
        .eval_text(
            "{revisionDown[-1], revisionUpHead[revisionUpTag, -1], \
              revisionSub[1][-1], revisionDefault[], Replace[-1, revisionRule]}",
        )
        .unwrap();
    assert_eq!(
        declined.value.to_string(),
        "List[revisionDown[-1], revisionUpHead[revisionUpTag, -1], revisionSub[1][-1], revisionDefault[], -1]"
    );
    assert!(declined.messages.is_empty());
}

#[test]
fn unrelated_set_and_clear_attributes_preserve_every_guarded_lane() {
    let mut session = revision_guard_session();
    install_every_guarded_revision_lane(&mut session);

    session
        .eval_text("SetAttributes[revisionUnrelated, HoldFirst]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut session);

    session
        .eval_text("ClearAttributes[revisionUnrelated, HoldFirst]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut session);
}

#[test]
fn attributes_on_the_guarded_symbol_preserve_every_guarded_lane() {
    let mut session = revision_guard_session();
    install_every_guarded_revision_lane(&mut session);

    session
        .eval_text("SetAttributes[revisionDown, HoldFirst]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut session);

    session
        .eval_text("ClearAttributes[revisionDown, HoldFirst]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut session);
}

#[test]
fn valueq_rollback_preserves_every_guarded_lane_and_post_trigger_value() {
    let mut session = revision_guard_session();
    install_every_guarded_revision_lane(&mut session);

    assert_eq!(
        session
            .eval_text("ValueQ[revisionDown[-1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_every_guarded_revision_lane(&mut session);

    assert_eq!(
        session
            .eval_text("ValueQ[revisionDown[1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_every_guarded_revision_lane(&mut session);
}

#[test]
fn eval_file_rollback_preserves_every_guarded_lane() {
    let dir = TempDirGuard::new("rhs-guard-attribute-revision-rollback");
    let file = dir.write(
        "broken.rum",
        "SetAttributes[fileRollbackOnly, HoldFirst]\n1 ! 2",
    );
    let mut session = revision_guard_session();
    install_test_file_io(&mut session);
    install_every_guarded_revision_lane(&mut session);

    let error = session
        .eval_file(file)
        .expect_err("the invalid file must restore its loader snapshot");
    assert!(matches!(error, EvalError::Syntax(_)));
    assert_every_guarded_revision_lane(&mut session);
}

#[test]
fn repeated_attribute_bumps_preserve_every_guarded_lane() {
    let mut session = revision_guard_session();
    install_every_guarded_revision_lane(&mut session);

    assert_eq!(
        session
            .eval_text(
                "SetAttributes[revisionBumpOne, HoldFirst]; \
                 SetAttributes[revisionBumpTwo, HoldFirst]",
            )
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_every_guarded_revision_lane(&mut session);
}

#[test]
fn matcher_ownership_guard_is_enforced_after_attribute_revision_invalidation() {
    let mut session = revision_guard_session();
    session
        .eval_text(
            "ownershipRevision[x_] := 91 /; MatchQ[x, 7]; \
             SetAttributes[ownershipRevisionBump, HoldFirst]",
        )
        .unwrap();

    let outcome = session
        .eval_text("{ownershipRevision[7], ownershipRevision[8]}")
        .unwrap();
    assert_eq!(outcome.value.to_string(), "List[91, ownershipRevision[8]]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn listable_guarded_downvalue_recompiles_and_enforces_its_guard() {
    let mut session = revision_guard_session();
    session
        .eval_text("listableGuard[x_] := \"accepted\" /; MatchQ[x, 1]")
        .unwrap();
    session
        .eval_text("SetAttributes[listableGuard, Listable]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("listableGuard[{1, -1}]")
            .unwrap()
            .value
            .to_string(),
        "List[\"accepted\", listableGuard[-1]]"
    );
}

#[test]
fn interleaved_guarded_downvalue_definitions_survive_later_bumps() {
    let mut session = revision_guard_session();
    session
        .eval_text(
            "interleavedGuard[x_] := \"one\" /; MatchQ[x, 1]; \
             SetAttributes[interleavedBumpOne, HoldFirst]; \
             interleavedGuard[x_] := \"two\" /; MatchQ[x, 2]; \
             SetAttributes[interleavedBumpTwo, HoldFirst]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text("{interleavedGuard[1], interleavedGuard[2], interleavedGuard[3]}")
            .unwrap()
            .value
            .to_string(),
        "List[\"one\", \"two\", interleavedGuard[3]]"
    );
}

#[test]
fn remaining_attribute_revision_sources_preserve_every_guarded_lane() {
    let mut registered = revision_guard_session();
    install_every_guarded_revision_lane(&mut registered);
    registered.register_builtin(BuiltinSpec::with_hold(
        "RevisionBumpBuiltin",
        "Test-local attribute revision bump.",
        HoldAttribute::HoldFirst,
        MemoryGrowthContract::NonAmplifying,
    ));
    assert_every_guarded_revision_lane(&mut registered);

    let mut clear_all = revision_guard_session();
    clear_all
        .eval_text("SetAttributes[revisionClearAllTarget, HoldFirst]")
        .unwrap();
    install_every_guarded_revision_lane(&mut clear_all);
    clear_all
        .eval_text("ClearAll[revisionClearAllTarget]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut clear_all);

    let mut block_restore = revision_guard_session();
    block_restore
        .eval_text("SetAttributes[revisionBlockTarget, HoldFirst]")
        .unwrap();
    install_every_guarded_revision_lane(&mut block_restore);
    block_restore
        .eval_text("Block[{revisionBlockTarget}, Null]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut block_restore);

    let mut iterator_restore = revision_guard_session();
    iterator_restore
        .eval_text("SetAttributes[revisionIteratorTarget, HoldFirst]")
        .unwrap();
    install_every_guarded_revision_lane(&mut iterator_restore);
    iterator_restore
        .eval_text("Do[Null, {revisionIteratorTarget, 1, 1}]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut iterator_restore);

    let mut dynamic_cleanup = revision_guard_session();
    let dynamic_symbol = Symbol::new("Global`", "revisionDynamicCleanup");
    dynamic_cleanup.set_attribute_set_for_symbol(
        dynamic_symbol,
        AttributeSet::from_hold(HoldAttribute::HoldFirst),
    );
    install_every_guarded_revision_lane(&mut dynamic_cleanup);
    dynamic_cleanup.clear_dynamic_scope_symbol_side_effects(&dynamic_symbol);
    assert_every_guarded_revision_lane(&mut dynamic_cleanup);

    let mut unique = revision_guard_session();
    install_every_guarded_revision_lane(&mut unique);
    unique
        .eval_text("Unique[\"revisionUnique\", HoldFirst]")
        .unwrap();
    assert_every_guarded_revision_lane(&mut unique);
}
