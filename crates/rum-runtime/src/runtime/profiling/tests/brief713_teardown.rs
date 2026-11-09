//! Q35: cancel, resume and destroy the same paused session on either thread.
//! Q36: this cancelled-lifecycle control does not discriminate the Q35 teardown admissions.
use crate::runtime::profiling::logical::{Epoch, epoch, segments, test_support};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

fn rows(epoch: &Epoch) -> BTreeMap<String, Vec<u64>> {
    epoch
        .rows()
        .into_iter()
        .map(|(key, counts)| {
            let mut row = vec![
                counts.calls,
                counts.ok,
                counts.err,
                counts.some,
                counts.none,
                counts.return_err,
                counts.throw_err,
                counts.cancelled,
                counts.abandoned,
                counts.open,
                counts.pauses,
                counts.input_required,
                counts.segments,
            ];
            row.extend(counts.abandonment_reasons);
            (format!("{key:?}"), row)
        })
        .collect()
}

fn cancelled_teardown(
    enabled: &test_support::Enabled,
    moved: bool,
) -> (BTreeMap<String, Vec<u64>>, [usize; 5]) {
    epoch::reset_for_lease(enabled.lease());
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    let cancel = session.request_cancel_handle();
    assert!(
        session
            .eval_request("WithCleanup[InputString[\"pause\"],Function[x,x][1]]")
            .is_err()
    );
    assert!(session.pending_input_request().is_some());
    let captured = test_support::epoch();
    let teardown = move || {
        cancel.cancel();
        assert!(session.resume_eval_request_with_input("ignored").is_ok());
        let pending = usize::from(session.pending_input_request().is_some());
        eprintln!(
            "B713 TEARDOWN_ENTRY moved={moved} generation_after_cancel={:?} unregistered={}",
            session.logical_profile_generation(),
            crate::runtime::profiling::lease::blocks_unregistered_thread(),
        );
        drop(session);
        let active = test_support::active();
        segments::release_thread_registration();
        (pending, active)
    };
    let (pending, active) = if moved {
        std::thread::spawn(teardown).join().unwrap()
    } else {
        teardown()
    };
    let observed = rows(&captured);
    let faults = captured.faults.load(Ordering::Relaxed) as usize;
    let contexts = captured.contexts.load(Ordering::Relaxed);
    // Release the capture thread's empty TLS header as well. Measure before
    // reset, while the epoch is retained, so reset cannot hide registry charges.
    segments::release_thread_registration();
    let bytes = crate::runtime::profiling::logical::brief713_probe_process_bytes();
    drop(captured);
    epoch::reset_for_lease(enabled.lease());
    assert_eq!(
        crate::runtime::profiling::logical::brief713_probe_process_bytes(),
        0,
        "teardown and the existing reset must refund every remaining charge"
    );
    (observed, [pending, faults, contexts, active, bytes])
}

#[test]
fn brief713_cancel_resume_drop_preserves_all_rows_and_charges() {
    let enabled = test_support::enabled();
    let (owner, owner_teardown) = cancelled_teardown(&enabled, false);
    let (worker, worker_teardown) = cancelled_teardown(&enabled, true);
    eprintln!(
        "B713 TEARDOWN own_rows={} moved_rows={} identical={} own_teardown={owner_teardown:?} moved_teardown={worker_teardown:?}",
        owner.len(),
        worker.len(),
        owner == worker,
    );
    assert!(!owner.is_empty());
    let session_drop = 13 + crate::runtime::profiling::logical::Retire::SessionDrop.index();
    let own_session_drop: u64 = owner.values().map(|row| row[session_drop]).sum();
    let moved_session_drop: u64 = worker.values().map(|row| row[session_drop]).sum();
    eprintln!(
        "B713 TEARDOWN_REASONS own_session_drop={own_session_drop} moved_session_drop={moved_session_drop}"
    );
    assert_eq!(worker, owner, "all logical rows and abandonment reasons");
    assert_eq!(worker_teardown, owner_teardown);
    assert_eq!(owner_teardown[..4], [0; 4]);
    assert!(owner.values().all(|row| row[9] == 0));
}

#[path = "brief713_teardown_paused.rs"]
mod paused;
