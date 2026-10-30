use crate::runtime::profiling::logical::{AggregateKey, Call, Policy, Retire, test_support};
use crate::runtime::suspension::{AbandonReason, FrameStorage, SuspendedEvaluation};
use crate::runtime::{EvalError, Expr, RuntimeInputRequest, Session};
use std::mem::size_of;

fn call(session: &Session) -> Call {
    Call::start(
        session,
        AggregateKey::Runtime {
            phase: "queue",
            source: None,
        },
        Policy::Expr,
    )
    .unwrap()
}
fn begin(session: &mut Session) {
    session.begin_input_suspension(RuntimeInputRequest::new("lp02"));
}

#[test]
fn logical_zero_frames_and_after_input_frontier_have_no_guest_storage() {
    for forwarded in [false, true] {
        let _enabled = test_support::enabled();
        let mut session = Session::new();
        if forwarded {
            begin(&mut session);
        }
        let token = call(&session);
        if !forwarded {
            begin(&mut session);
        }
        let frontier = session.logical_frontier();
        let boundary = token.pause().unwrap();
        session.logical_insert_boundary(boundary, frontier, false);
        let pending = session.pending_suspension.as_ref().unwrap();
        assert_eq!(pending.frame_storage_memory_proxy_bytes(), 0);
        assert_eq!(
            pending.occurrence_bytes(),
            crate::runtime::debug_tools::resources::occurrence::Bytes::Exact(size_of::<
                SuspendedEvaluation,
            >())
        );
        assert_eq!(
            session.resume_suspended_evaluation("reply").unwrap(),
            Expr::string("reply")
        );
        let epoch = test_support::epoch();
        let c = epoch.rows()[0].1;
        assert_eq!((c.calls, c.ok, c.open), (1, 1, 0));
        assert_eq!(Session::logical_profile_test_leases(), 0);
        Session::logical_profile_test_assert_quiescent();
        assert_eq!(test_support::active(), 0);
    }
}

#[test]
fn logical_every_structural_route_closes_once_with_its_reason() {
    for reason in [
        AbandonReason::CancelledRequest,
        AbandonReason::FailedSnapshotRestore,
        AbandonReason::HelperBestEffortDiscard,
        AbandonReason::CheckpointRollbackDiscard,
        AbandonReason::SupersededOrQueuedInputDiscard,
        AbandonReason::ContinuationSlotMismatch,
    ] {
        let _enabled = test_support::enabled();
        let mut session = Session::new();
        let token = call(&session);
        begin(&mut session);
        session.logical_insert_boundary(token.pause().unwrap(), None, false);
        session.abandon_pending_suspension(reason);
        session.abandon_pending_suspension(reason);
        let c = test_support::epoch().rows()[0].1;
        assert_eq!(c.calls, 1);
        assert_eq!(c.ok, 0);
        assert_eq!(c.open, 0);
        // This marker-only queue executes no resumed semantic work.
        assert_eq!(c.segments, 1);
        if reason == AbandonReason::CancelledRequest {
            assert_eq!((c.err, c.cancelled), (1, 1));
        } else {
            assert_eq!(c.abandoned, 1);
            assert_eq!(c.abandonment_reasons[Retire::Structural(reason).index()], 1);
        }
        assert_eq!(Session::logical_profile_test_leases(), 0);
        Session::logical_profile_test_assert_quiescent();
        assert_eq!(test_support::active(), 0);
    }
}

#[test]
fn logical_diagnostic_cap_keeps_the_marker_free_guest_capacity() {
    let enabled = test_support::enabled();
    test_support::limit(enabled.lease(), 20 * 1024);
    let mut session = Session::new();
    let token = call(&session);
    begin(&mut session);
    let mut base = FrameStorage::new();
    let context = session.resume_context();
    for _ in 0..200 {
        session.insert_evaluation_return();
        base.push_evaluation_return(context);
    }
    if let Some(boundary) = token.pause() {
        session.logical_insert_boundary(boundary, None, false);
    }
    assert_eq!(
        session
            .pending_suspension
            .as_ref()
            .unwrap()
            .frame_storage_memory_proxy_bytes(),
        base.storage_bytes()
    );
    assert_eq!(
        session.resume_suspended_evaluation("reply").unwrap(),
        Expr::string("reply")
    );
    let epoch = test_support::epoch();
    assert!(!epoch.is_valid());
    let c = epoch.rows()[0].1;
    assert_eq!((c.calls, c.ok, c.abandoned, c.open), (1, 0, 1, 0));
    assert_eq!(Session::logical_profile_test_leases(), 0);
    Session::logical_profile_test_assert_quiescent();
}

#[test]
fn logical_deferred_return_boundary_observes_the_return_resource_error() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let token = call(&session);
    begin(&mut session);
    let boundary = token.pause().unwrap();
    session.logical_insert_boundary(boundary, None, true);
    session.insert_evaluation_return();
    session.current_time_deadline = Some(std::time::Instant::now());
    assert!(matches!(
        session.resume_suspended_evaluation("reply"),
        Err(EvalError::TimeConstraintExceeded)
    ));
    let c = test_support::epoch().rows()[0].1;
    assert_eq!((c.calls, c.err, c.ok, c.open), (1, 1, 0, 0));
    assert_eq!(Session::logical_profile_test_leases(), 0);
    Session::logical_profile_test_assert_quiescent();
}

#[test]
fn logical_driver_slot_fault_abandons_the_rooted_boundary() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let token = call(&session);
    begin(&mut session);
    session.logical_insert_boundary(token.pause().unwrap(), None, false);
    session.logical_slot_fault_for_test();
    let c = test_support::epoch().rows()[0].1;
    assert_eq!((c.calls, c.abandoned, c.ok, c.open), (1, 1, 0, 0));
    assert_eq!(
        c.abandonment_reasons[Retire::Structural(AbandonReason::ContinuationSlotMismatch).index()],
        1
    );
    assert_eq!(Session::logical_profile_test_leases(), 0);
    Session::logical_profile_test_assert_quiescent();
}

#[test]
fn logical_frontier_rejects_another_owner_even_without_semantic_cells() {
    let _enabled = test_support::enabled();
    let mut a = Session::new();
    let mut b = Session::new();
    let token = call(&a);
    begin(&mut a);
    let frontier = a.logical_frontier().unwrap();
    let first = token.pause().unwrap();
    let token = call(&b);
    begin(&mut b);
    let second = token.pause().unwrap();
    b.logical_insert_boundary(second, Some(frontier), false);
    assert!(!test_support::epoch().is_valid());
    drop(first);
    assert_eq!(
        b.resume_suspended_evaluation("unchanged").unwrap(),
        Expr::string("unchanged")
    );
    let c = test_support::epoch().rows()[0].1;
    assert_eq!((c.calls, c.ok, c.abandoned, c.open), (2, 0, 2, 0));
}

#[test]
fn logical_after_consumer_sees_the_input_parser_error() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let token = call(&session);
    let expr = session.lower_text("Input[]").unwrap();
    assert!(matches!(
        session.evaluate(expr),
        Err(EvalError::InputRequired { .. })
    ));
    let frontier = session
        .logical_frontier_after_consumer()
        .expect("the actual Input consumer");
    session.logical_insert_boundary(token.pause().unwrap(), Some(frontier), false);
    assert!(matches!(
        session.resume_suspended_evaluation("("),
        Err(EvalError::Syntax(_))
    ));
    let c = test_support::epoch().rows()[0].1;
    assert_eq!((c.calls, c.err, c.ok, c.open), (1, 1, 0, 0));
    assert_eq!(Session::logical_profile_test_leases(), 0);
    Session::logical_profile_test_assert_quiescent();
}

#[test]
fn logical_queued_frames_have_one_owner_in_both_modes() {
    let _enabled = test_support::enabled();
    let mut ordinary = Session::new();
    let mut profiled = Session::new();
    let mut observations = Vec::new();
    let input = ordinary.lower_text("Input[]").unwrap();
    assert!(matches!(
        ordinary.evaluate(input),
        Err(EvalError::InputRequired { .. })
    ));
    let token = call(&profiled);
    let input = profiled.lower_text("Input[]").unwrap();
    assert!(matches!(
        profiled.evaluate(input),
        Err(EvalError::InputRequired { .. })
    ));
    let boundary = token.pause().unwrap();
    let counts = |s: &Session| {
        s.pending_suspension
            .as_ref()
            .unwrap()
            .frames
            .frame_strong_counts()
    };
    observations.push(("push", counts(&ordinary), counts(&profiled)));
    ordinary.insert_evaluation_return();
    profiled.insert_evaluation_return();
    observations.push(("widen", counts(&ordinary), counts(&profiled)));
    for session in [&mut ordinary, &mut profiled] {
        let mut suffix_session = Session::new();
        let input = suffix_session.lower_text("Input[]").unwrap();
        assert!(matches!(
            suffix_session.evaluate(input),
            Err(EvalError::InputRequired { .. })
        ));
        let mut suffix = std::mem::replace(
            &mut suffix_session.pending_suspension.as_mut().unwrap().frames,
            FrameStorage::new(),
        );
        session
            .pending_suspension
            .as_mut()
            .unwrap()
            .frames
            .append(&mut suffix);
    }
    observations.push(("append", counts(&ordinary), counts(&profiled)));
    profiled.logical_insert_boundary(boundary, None, false);
    observations.push(("marker", counts(&ordinary), counts(&profiled)));
    for (point, ordinary, profiled) in &observations {
        eprintln!("strong-count {point}: ordinary={ordinary:?} profiled={profiled:?}");
        assert!(!ordinary.is_empty());
    }
    assert!(
        observations
            .iter()
            .all(|(_, ordinary, profiled)| ordinary == profiled),
        "{observations:?}"
    );
}

#[test]
fn logical_queue_representation_survives_a_disabled_global_gate() {
    use super::super::state::LEXICAL_PROFILE_ACTIVE;
    use std::sync::atomic::Ordering;
    for abandon in [false, true] {
        let _enabled = test_support::enabled();
        let mut session = Session::new();
        let token = call(&session);
        begin(&mut session);
        session.logical_insert_boundary(token.pause().unwrap(), None, false);
        // 2 is disabled; 0 is uninitialized, and any cold load re-arms it.
        let prior = LEXICAL_PROFILE_ACTIVE.swap(2, Ordering::Relaxed);
        let result = if abandon {
            session.abandon_pending_suspension(AbandonReason::CancelledRequest);
            Ok(Expr::string("reply"))
        } else {
            session.resume_suspended_evaluation("reply")
        };
        LEXICAL_PROFILE_ACTIVE.store(prior, Ordering::Relaxed);
        assert_eq!(result.unwrap(), Expr::string("reply"));
        let c = test_support::epoch().rows()[0].1;
        assert_eq!((c.calls, c.open), (1, 0));
        assert_eq!(c.ok, u64::from(!abandon));
        assert_eq!(c.cancelled, u64::from(abandon));
        assert_eq!(Session::logical_profile_test_leases(), 0);
        Session::logical_profile_test_assert_quiescent();
        assert_eq!(test_support::active(), 0);
    }
}
