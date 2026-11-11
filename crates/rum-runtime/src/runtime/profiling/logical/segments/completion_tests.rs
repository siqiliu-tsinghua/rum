use super::super::test_support;
use super::*;

fn opened(epoch: &Arc<Epoch>, label: &'static str) -> (RecordId, Segment) {
    let owner = Owner {
        control: Arc::new(crate::runtime::RequestInterruptState::new()),
        generation: 1,
        root: 1,
    };
    let id = epoch
        .open(
            owner,
            None,
            AggregateKey::Runtime {
                phase: label,
                source: None,
            },
            Policy::OptionalExpr,
        )
        .unwrap();
    (id, enter(epoch, id).unwrap())
}
fn boundary(epoch: &Arc<Epoch>, record: RecordId) -> ProfileBoundary {
    ProfileBoundary {
        epoch: Arc::clone(epoch),
        record,
        _storage: epoch
            .storage(size_of::<ProfileBoundary>() + 16)
            .unwrap()
            .into(),
    }
}

#[test]
fn completed_guard_repeated_pause_reuse_and_outcomes() {
    let enabled = test_support::enabled();
    for outcome in [
        Outcome::None,
        Outcome::Some,
        Outcome::Error,
        Outcome::Return,
        Outcome::Throw,
        Outcome::Cancelled,
    ] {
        let epoch = Epoch::for_test(enabled.lease(), 1024 * 1024).unwrap();
        let (id, segment) = opened(&epoch, "completion-completed");
        let context = Context::capture(&epoch).unwrap();
        drop(segment);
        let guard = context.enter().unwrap();
        let old_index = ACTIVE.with_borrow(|stack| stack.top.unwrap());
        let b = boundary(&epoch, id);
        b.complete(outcome);
        let snapshot = epoch.rows();
        guard.pause();
        guard.pause();
        assert!(
            epoch.is_valid(),
            "completed_context_repeated_pause_keeps_epoch_valid"
        );
        assert_eq!(
            epoch.faults.load(std::sync::atomic::Ordering::Relaxed),
            0,
            "completed_context_repeated_pause_has_zero_faults"
        );
        assert_eq!(format!("{:?}", epoch.rows()), format!("{snapshot:?}"));
        let (other, live) = opened(&epoch, "completion-reused");
        assert_eq!(
            live.index, old_index,
            "active slot is reused by a new generation"
        );
        guard.pause();
        assert!(epoch.is_valid());
        assert_eq!(active_count(), 1);
        drop(guard);
        assert_eq!(active_count(), 1);
        ACTIVE.with_borrow(|stack| {
            let active = stack.frame(live.index).unwrap();
            assert_eq!(active.segment, live.segment);
            assert_eq!(active.path.record, other);
            assert!(!active.paused);
        });
        drop(live);
        epoch.close(other, Ok(Outcome::Some));
        assert_eq!(epoch.faults.load(std::sync::atomic::Ordering::Relaxed), 0);
        drop((context, b));
        epoch.assert_quiescent();
    }
    eprintln!(
        "COMPLETED SEGMENT PASS outcomes=None,Some,Error,Return,Throw,Cancelled repeated_pause=true slot_reuse=true zero_faults=true"
    );
}

#[test]
fn missing_slot_without_explicit_finish_still_faults() {
    let enabled = test_support::enabled();
    let epoch = Epoch::for_test(enabled.lease(), 1024 * 1024).unwrap();
    let (_, segment) = opened(&epoch, "completion-no-certificate");
    segment.end();
    segment.pause();
    assert!(!epoch.is_valid());
    assert_eq!(epoch.faults.load(std::sync::atomic::Ordering::Relaxed), 1);
    eprintln!(
        "COMPLETED SEGMENT PASS missing_slot_without_explicit_finish=fault invalid_epoch=true"
    );
}

#[test]
fn displaced_live_context_still_faults() {
    let enabled = test_support::enabled();
    let epoch = Epoch::for_test(enabled.lease(), 1024 * 1024).unwrap();
    let (_, a) = opened(&epoch, "completion-live-a");
    let (_, b) = opened(&epoch, "completion-live-b");
    a.pause();
    assert!(!epoch.is_valid());
    assert_eq!(epoch.faults.load(std::sync::atomic::Ordering::Relaxed), 1);
    drop(b);
    drop(a);
    assert_eq!(active_count(), 0);
    eprintln!("COMPLETED SEGMENT PASS displaced_live_context=fault invalid_epoch=true");
}

#[test]
fn cross_thread_context_completion_and_pause() {
    let enabled = test_support::enabled();
    let epoch = Epoch::for_test(enabled.lease(), 1024 * 1024).unwrap();
    let (id, segment) = opened(&epoch, "completion-thread");
    let context = Context::capture(&epoch).unwrap();
    drop(segment);
    let b = boundary(&epoch, id);
    let lease = enabled.lease();
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                lease.allow_current_thread();
                let guard = context.enter().unwrap();
                b.complete(Outcome::None);
                guard.pause();
                assert!(epoch.is_valid());
                drop(guard);
                release_thread_registration();
            })
            .join()
            .unwrap();
    });
    drop((context, b));
    epoch.assert_quiescent();
    eprintln!(
        "COMPLETED SEGMENT PASS cross_thread_context=true new_thread_segment=true completion_pause_valid=true"
    );
}

#[test]
fn cap_reset_and_storage_drop_stay_invalid_or_drained() {
    let enabled = test_support::enabled();
    for reason in [Retire::ProfileReset, Retire::DiagnosticCap] {
        let epoch = Epoch::for_test(enabled.lease(), 1024 * 1024).unwrap();
        let (id, segment) = opened(&epoch, "completion-retired");
        epoch.retire(reason);
        finish_active(&epoch, id);
        assert!(!epoch.is_valid());
        drop(segment);
        epoch.assert_quiescent();
    }
    release_thread_registration();
    let epoch = Epoch::for_test(enabled.lease(), 1024 * 1024).unwrap();
    let probe = epoch.test_budget_probe();
    let (id, segment) = opened(&epoch, "completion-storage");
    finish_active(&epoch, id);
    epoch.close(id, Ok(Outcome::None));
    release_thread_registration();
    drop(epoch);
    assert!(
        probe.bytes().is_some(),
        "last segment retains the epoch and its charged storage"
    );
    drop(segment);
    assert_eq!(probe.bytes(), None);
    eprintln!(
        "COMPLETED SEGMENT PASS reset_invalid=true cap_invalid=true receipt_storage_last_drop_released=true"
    );
}

#[test]
fn layout_receipt() {
    eprintln!(
        "COMPLETED SEGMENT LAYOUT Segment={} OptionSegment={} ContextGuard={} OptionContextGuard={} Call={} Active={} ActiveSlot={} ActivePointer={}",
        size_of::<Segment>(),
        size_of::<Option<Segment>>(),
        size_of::<ContextGuard>(),
        size_of::<Option<ContextGuard>>(),
        size_of::<Call>(),
        size_of::<Active>(),
        size_of::<ActiveSlot>(),
        size_of::<Box<Active>>()
    );
}
