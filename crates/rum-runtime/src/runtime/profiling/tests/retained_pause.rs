//! LP08 retention preserves LP02 explicit observers and physical input observations.
use super::*;
use crate::runtime::{RuntimeInputRequest, Session, Symbol};
use std::sync::atomic::Ordering;

fn key(phase: &'static str) -> AggregateKey {
    AggregateKey::Runtime {
        phase,
        source: None,
    }
}

fn call(session: &Session, phase: &'static str) -> Call {
    Call::start(session, key(phase), Policy::Expr).unwrap()
}

fn row(epoch: &Epoch, key: AggregateKey) -> Counts {
    epoch.rows().into_iter().find(|(k, _)| *k == key).unwrap().1
}

fn own(path: &RecordPath) -> totals::Totals {
    path.state.lock().unwrap().totals
}

fn retain_phase(session: &mut Session, phase: PhaseKey, call: Call) -> Arc<ProfileBoundary> {
    let record = call.inner.record;
    push_phase(phase, call);
    complete_phase(
        session,
        phase,
        super::super::RuntimeProfileOutcome::InputRequired,
        |_| panic!("retention must not invoke terminal completion"),
    );
    let mut stash = collector::take_stash();
    assert_eq!(stash.len(), 1);
    let boundary = stash.pop().unwrap();
    assert_eq!(boundary.record, record);
    boundary
}

fn quiet(epoch: &Epoch) {
    assert_eq!(test_support::active(), 0);
    assert!(segments::running(epoch).is_empty());
    assert_eq!(epoch.faults.load(Ordering::Relaxed), 0);
    epoch.assert_quiescent();
}

#[test]
fn complete_phase_retention_is_silent_for_every_phase() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let epoch = test_support::epoch();
    for phase in PhaseKey::ALL {
        let native = call(&session, "native");
        let parent = native.inner.record;
        let child = Call::start(&session, key(phase.name()), phase.site().policy).unwrap();
        let id = child.inner.record;
        let path = epoch.path(id).unwrap();
        let (owner, prior_parent, policy) = epoch.inspect(id).unwrap();
        let physical = row(&epoch, path.key).input_required;
        let boundary = retain_phase(&mut session, phase, child);
        assert_eq!(own(&path).pauses, 0, "{phase:?}: own pauses");
        assert!(path.is_open(), "{phase:?}: retained record stays open");
        assert_eq!(row(&epoch, path.key).input_required, physical + 1);
        let (after_owner, after_parent, after_policy) = epoch.inspect(id).unwrap();
        assert!(after_owner.same(&owner));
        assert_eq!((prior_parent, after_parent), (Some(parent), Some(parent)));
        assert_eq!((after_policy, boundary.policy()), (policy, Some(policy)));
        assert!(boundary.belongs_to(&session.request_interrupt_control.borrow()));
        assert!(Arc::ptr_eq(&epoch, &boundary.epoch));
        let native_path = epoch.path(parent).unwrap();
        let native_boundary = native.pause().unwrap();
        // A retained phase must not move the cursor past this explicit observer.
        assert_eq!(own(&native_path).pauses, 1);
        boundary.complete(Outcome::Ok);
        boundary.complete(Outcome::Ok);
        native_boundary.complete(Outcome::Ok);
        drop(boundary);
        drop(native_boundary);
        quiet(&epoch);
    }
}

#[test]
fn down_values_finish_retains_identity_before_the_actual_consumer() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let epoch = test_support::epoch();
    let native = call(&session, "native");
    let phase = PhaseKey::CallStageDownValues;
    let child = Call::start(&session, key(phase.name()), phase.site().policy).unwrap();
    let id = child.inner.record;
    let path = epoch.path(id).unwrap();
    let owner = path.owner.clone();
    push_phase(phase, child);
    // Input installs a real production consumer; no substitute frame or hook.
    assert!(matches!(
        session.eval_input(&[]),
        Err(EvalError::InputRequired { .. })
    ));
    let frames = &session.pending_suspension.as_ref().unwrap().frames;
    let consumer = frames.consumer_frontier().unwrap();
    let tail = frames.frontier();
    assert_eq!(tail, Some(consumer));
    pause_call_stage_at_finish(&mut session, phase.name());
    assert_eq!(own(&path).pauses, 0, "down-values own pauses");
    assert_eq!(row(&epoch, path.key).input_required, 1);
    assert!(path.is_open());
    assert!(epoch.inspect(id).unwrap().0.same(&owner));
    assert_eq!(epoch.inspect(id).unwrap().2, Policy::OptionalExpr);
    let frames = &session.pending_suspension.as_ref().unwrap().frames;
    assert_eq!(frames.consumer_frontier(), Some(consumer));
    assert_eq!(
        frames.frontier(),
        tail,
        "boundary is before the exact consumer"
    );
    assert!(collector::take_stash().is_empty());
    let native_path = epoch.path(native.inner.record).unwrap();
    let boundary = native.pause().unwrap();
    assert_eq!(own(&native_path).pauses, 1);
    session.logical_insert_boundary(boundary, None, false);
    // A before-consumer marker succeeds on the raw text even when Input's
    // parser subsequently fails. Moving it after that consumer records Error.
    assert!(matches!(
        session.resume_suspended_evaluation("("),
        Err(EvalError::Syntax(_))
    ));
    let c = row(&epoch, path.key);
    assert_eq!((c.calls, c.some, c.err, c.open), (1, 1, 0, 0));
    assert!(!path.is_open());
    assert!(!epoch.close(id, Ok(Outcome::Ok)));
    quiet(&epoch);
}

#[test]
fn deferred_definition_is_silent_and_flushes_once_with_its_policy() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let mut foreign = Session::new();
    let epoch = test_support::epoch();
    let native = call(&session, "native");
    let definition = AggregateKey::Definition {
        kind: "down_value_rhs_eval",
        symbol: Symbol::new("Global`", "f"),
        index: 0,
    };
    let child = Call::start(&session, definition, Policy::OptionalExpr).unwrap();
    let id = child.inner.record;
    let path = epoch.path(id).unwrap();
    let owner = path.owner.clone();
    child.defer_definition();
    assert_eq!(own(&path).pauses, 0, "definition own pauses");
    assert_eq!(row(&epoch, definition).input_required, 1);
    assert!(path.is_open());
    assert!(epoch.inspect(id).unwrap().0.same(&owner));
    assert!(collector::take_stash().is_empty());
    assert!(
        epoch
            .take_deferred_definition(&foreign.request_interrupt_control.borrow())
            .is_none()
    );
    assert_eq!(epoch.take_deferred_definition(&owner.control), Some(id));
    assert_eq!(epoch.take_deferred_definition(&owner.control), None);
    epoch.defer_definition(id);
    foreign.begin_input_suspension(RuntimeInputRequest::new("foreign"));
    foreign.flush_definition_boundaries();
    session.begin_input_suspension(RuntimeInputRequest::new("definition"));
    session.flush_definition_boundaries();
    let frontier = session
        .pending_suspension
        .as_ref()
        .unwrap()
        .frames
        .frontier();
    assert!(frontier.is_some());
    session.flush_definition_boundaries();
    assert_eq!(
        session
            .pending_suspension
            .as_ref()
            .unwrap()
            .frames
            .frontier(),
        frontier
    );
    assert_eq!(epoch.take_deferred_definition(&owner.control), None);
    assert_eq!(epoch.inspect(id).unwrap().2, Policy::OptionalExpr);
    let native_path = epoch.path(native.inner.record).unwrap();
    let native_boundary = native.pause().unwrap();
    assert_eq!(own(&native_path).pauses, 1);
    assert_eq!(
        session.resume_suspended_evaluation("reply").unwrap(),
        Expr::string("reply")
    );
    let c = row(&epoch, definition);
    assert_eq!((c.calls, c.some, c.err, c.open), (1, 1, 0, 0));
    assert!(!epoch.close(id, Ok(Outcome::Some)));
    native_boundary.complete(Outcome::Ok);
    drop(native_boundary);
    drop(foreign);
    quiet(&epoch);
}

#[test]
fn restored_retained_phase_times_itself_but_pauses_the_old_observer() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let epoch = test_support::epoch();
    let native = call(&session, "native");
    let phase = PhaseKey::EvaluateCallDispatch;
    let child = call(&session, phase.name());
    let path = epoch.path(child.inner.record).unwrap();
    let context = Context::capture(&epoch).unwrap();
    let boundary = retain_phase(&mut session, phase, child);
    let native_boundary = native.pause().unwrap();
    assert_eq!(row(&epoch, key("native")).pauses, 1, "parent event");
    let guard = context.enter().unwrap();
    assert_eq!(segments::ancestry(&epoch).unwrap().record, path.record);
    assert!(
        segments::owner(&path.owner.control)
            .unwrap()
            .1
            .same(&path.owner)
    );
    assert_eq!(segments::running(&epoch).len(), 1);
    assert_eq!(segments::running(&epoch)[0].0, path.record);
    let old_child = call(&session, PhaseKey::CompoundItemEvaluate.name())
        .pause()
        .unwrap();
    guard.pause();
    guard.pause();
    assert_eq!(row(&epoch, key("native")).pauses, 2, "resumed child event");
    assert_eq!(own(&path).pauses, 0);
    let parked = segments::running(&epoch);
    assert_eq!(
        segments::running(&epoch),
        parked,
        "no timer runs after pause"
    );
    drop(guard);
    assert_eq!(own(&path).restored, 1);
    assert_eq!(own(&path).segments, 2);
    old_child.complete(Outcome::Ok);
    boundary.complete(Outcome::Ok);
    native_boundary.complete(Outcome::Ok);
    drop((old_child, boundary, native_boundary, context));
    quiet(&epoch);
}

#[test]
fn restored_retention_without_an_observer_preserves_another_cursor() {
    let _enabled = test_support::enabled();
    let session = Session::new();
    let mut other = Session::new();
    let epoch = test_support::epoch();
    let native = call(&session, "native");
    let native_path = epoch.path(native.inner.record).unwrap();
    let old = call(&session, PhaseKey::CompoundItemSuspend.name());
    let old_context = Context::capture(&epoch).unwrap();
    let old_boundary = old.pause().unwrap();
    let phase = PhaseKey::EvaluateTotal;
    let child = call(&other, phase.name());
    let path = epoch.path(child.inner.record).unwrap();
    assert!(path.parent.as_deref().is_none());
    let context = Context::capture(&epoch).unwrap();
    let boundary = retain_phase(&mut other, phase, child);
    let guard = context.enter().unwrap();
    // Restore an explicit observer above the retained root, then leave its
    // native-parent cursor behind for the root's guard to preserve.
    let old_guard = old_context.enter().unwrap();
    old_guard.pause();
    drop(old_guard);
    guard.pause();
    assert_eq!(own(&path).pauses, 0, "no invented observer");
    let parked = segments::running(&epoch);
    assert_eq!(segments::running(&epoch), parked);
    drop(guard);
    let native_boundary = native.pause().unwrap();
    assert_eq!(own(&native_path).pauses, 0, "unrelated cursor preserved");
    assert_eq!(row(&epoch, key("native")).pauses, 2);
    assert_eq!(own(&path).restored, 1);
    boundary.complete(Outcome::Ok);
    old_boundary.complete(Outcome::Ok);
    native_boundary.complete(Outcome::Ok);
    drop((
        boundary,
        old_boundary,
        native_boundary,
        context,
        old_context,
    ));
    quiet(&epoch);
}

#[test]
fn explicit_local_carriers_still_count_on_restore() {
    let _enabled = test_support::enabled();
    let session = Session::new();
    let epoch = test_support::epoch();
    for phase in [
        PhaseKey::CompoundItemEvaluate,
        PhaseKey::CatchResumeAfterBody,
    ] {
        let old = call(&session, phase.name());
        let path = epoch.path(old.inner.record).unwrap();
        let context = Context::capture(&epoch).unwrap();
        let boundary = old.pause().unwrap();
        assert_eq!(own(&path).pauses, 1);
        let guard = context.enter().unwrap();
        guard.pause();
        assert_eq!(own(&path).pauses, 2, "explicit local observer");
        drop(guard);
        boundary.complete(Outcome::Ok);
        drop((boundary, context));
        quiet(&epoch);
    }
}

#[test]
fn bare_input_preserves_physical_observation_and_logical_outcome() {
    let mut session = Session::new();
    session.eval_text("f[x_] := x").unwrap();
    let _enabled = test_support::enabled();
    assert!(matches!(
        session.eval_text("f[InputString[\"prompt\"]]"),
        Err(EvalError::InputRequired { .. })
    ));
    assert_eq!(
        session.resume_suspended_evaluation("reply").unwrap(),
        Expr::string("reply")
    );
    assert!(
        Session::logical_profile_test_pauses() > 0,
        "physical InputRequired observation"
    );
    let epoch = test_support::epoch();
    assert_eq!(epoch.rows().iter().map(|(_, c)| c.pauses).sum::<u64>(), 0);
    assert!(
        epoch
            .rows()
            .iter()
            .map(|(_, c)| c.input_required)
            .sum::<u64>()
            > 0
    );
    for (_, c) in epoch.rows() {
        assert_eq!((c.calls, c.err, c.abandoned, c.open), (c.ok, 0, 0, 0));
    }
    quiet(&epoch);
}

#[test]
fn repeated_restore_retirement_cap_and_completion_preserve_lifecycle() {
    let enabled = test_support::enabled();
    for reason in [
        None,
        Some(Retire::ProfileReset),
        Some(Retire::DiagnosticCap),
    ] {
        let mut session = Session::new();
        let epoch = test_support::epoch();
        let phase = PhaseKey::PrepareArgumentsTotal;
        let child = call(&session, phase.name());
        let path = epoch.path(child.inner.record).unwrap();
        let owner = path.owner.clone();
        let context = Context::capture(&epoch).unwrap();
        let boundary = retain_phase(&mut session, phase, child);
        for _ in 0..2 {
            let guard = context.enter().unwrap();
            assert_eq!(segments::ancestry(&epoch).unwrap().record, path.record);
            assert!(epoch.inspect(path.record).unwrap().0.same(&owner));
            guard.pause();
            assert_eq!(own(&path).pauses, 0, "repeat remains silent");
            drop(guard);
            assert!(segments::running(&epoch).is_empty());
        }
        assert_eq!(own(&path).restored, 2);
        match reason {
            Some(Retire::ProfileReset) => epoch::reset_for_lease(enabled.lease()),
            Some(Retire::DiagnosticCap) => assert!(epoch.storage(32 * 1024 * 1024).is_none()),
            None => boundary.complete(Outcome::Ok),
            _ => unreachable!(),
        }
        boundary.complete(Outcome::Ok);
        boundary.abandon(Retire::LastDrop);
        let c = row(&epoch, path.key);
        assert_eq!((c.calls, c.open), (1, 0));
        if let Some(reason) = reason {
            assert!(!epoch.is_valid());
            assert_eq!(c.abandonment_reasons[reason.index()], 1);
            assert_eq!(c.abandoned, 1);
            assert!(context.enter().is_none());
        } else {
            assert_eq!((c.ok, c.abandoned), (1, 0));
        }
        drop((boundary, context, path, session));
        quiet(&epoch);
        drop(epoch);
        epoch::reset_for_lease(enabled.lease());
    }
    // Successful no-input completion never enters retention or observes input.
    let mut session = Session::new();
    let epoch = test_support::epoch();
    let phase = PhaseKey::EvaluateInnerUnstableAtom;
    let child = call(&session, phase.name());
    push_phase(phase, child);
    complete_phase(
        &mut session,
        phase,
        super::super::RuntimeProfileOutcome::Ok,
        |call| call.unwrap().finish(Outcome::Ok),
    );
    let c = row(&epoch, key(phase.name()));
    assert_eq!(
        (c.calls, c.ok, c.open, c.pauses, c.input_required),
        (1, 1, 0, 0, 0)
    );
    assert!(collector::take_stash().is_empty());
    quiet(&epoch);
    drop((session, epoch));
    epoch::reset_for_lease(enabled.lease());
    segments::release_thread_registration();
    drop(enabled);
    let _lease = epoch::lease_drained();
    assert_eq!(budget::process_bytes(), 0);
}

#[test]
fn diagnostic_state_layout_and_charge_are_reported() {
    let _lease = test_support::lease();
    eprintln!(
        "LP707 STATE size={} align={} record_path={} diagnostic_charge={}",
        size_of::<totals::State>(),
        align_of::<totals::State>(),
        size_of::<RecordPath>(),
        size_of::<RecordPath>() + 2 * size_of::<usize>()
    );
}

#[test]
fn brief720_queue_join_preserves_foreign_deferred_definition() {
    let enabled = test_support::enabled();
    for (kind, symbol, setup, source) in [
        (
            "down_value_rhs_eval",
            "f",
            r#"f[x_Integer]:=((InputString["r1"];InputString["r2"];83)/;(InputString["g"];True))"#,
            "f[2]",
        ),
        (
            "sub_value_rhs_eval",
            "f",
            r#"f[a_][x_Integer]:=((InputString["r1"];InputString["r2"];83)/;(InputString["g"];True))"#,
            "f[1][2]",
        ),
        (
            "up_value_rhs_eval",
            "a",
            r#"TagSetDelayed[a,f[a,x_Integer],((InputString["r1"];InputString["r2"];83)/;(InputString["g"];True))]"#,
            "f[a,2]",
        ),
    ] {
        let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
        session.eval_text(setup).unwrap();
        epoch::reset_for_lease(enabled.lease());
        let epoch = test_support::epoch();
        assert!(matches!(
            session.eval_text(source),
            Err(EvalError::InputRequired { .. })
        ));
        let mut foreign = Session::new();
        let foreign_key = AggregateKey::Definition {
            kind: "down_value_rhs_eval",
            symbol: Symbol::new("Global`", "foreign720"),
            index: 0,
        };
        let deferred = Call::start(&foreign, foreign_key, Policy::OptionalExpr).unwrap();
        let id = deferred.inner.record;
        let path = epoch.path(id).unwrap();
        let control = Arc::clone(&foreign.request_interrupt_control.borrow());
        deferred.defer_definition();
        foreign.begin_input_suspension(RuntimeInputRequest::new("foreign720"));
        for answer in 0..3 {
            let result = session.resume_suspended_evaluation("answer");
            if answer < 2 {
                assert!(matches!(result, Err(EvalError::InputRequired { .. })));
            } else {
                assert_eq!(result.unwrap().to_string(), "83");
            }
            assert!(
                path.is_open(),
                "another queue must not publish the foreign record"
            );
            assert_eq!(epoch.take_deferred_definition(&control), Some(id));
            epoch.defer_definition(id);
        }
        let own_key = AggregateKey::Definition {
            kind,
            symbol: Symbol::new("Global`", symbol),
            index: 0,
        };
        let own = row(&epoch, own_key);
        assert_eq!((own.calls, own.some, own.open), (1, 1, 0));
        foreign.flush_definition_boundaries();
        foreign.resume_suspended_evaluation("reply").unwrap();
        let found = row(&epoch, foreign_key);
        assert_eq!((found.calls, found.some, found.open), (1, 1, 0));
        assert!(!path.is_open());
        drop((session, foreign, path, control));
        quiet(&epoch);
        eprintln!(
            "BRIEF720C2 FOREIGN PASS kind={kind} join_pauses=2 foreign_deferred_until_own_flush=true own_some=1 foreign_some=1 terminal_open=0"
        );
    }
}
