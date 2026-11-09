//! Q36: a paused drop retains provenance; completed resumption is a control.
use super::rows;
use crate::runtime::profiling::lease;
use crate::runtime::profiling::logical::{Retire, epoch, segments, test_support};
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

fn teardown(
    enabled: &test_support::Enabled,
    moved: bool,
    completed: bool,
) -> (BTreeMap<String, Vec<u64>>, [usize; 5]) {
    epoch::reset_for_lease(enabled.lease());
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    assert!(
        session
            .eval_request("WithCleanup[InputString[\"pause\"], Function[x,x][1]]")
            .is_err()
    );
    assert!(session.pending_input_request().is_some());
    let captured = test_support::epoch();
    let current = captured.lease_generation();
    assert_ne!(current, 0);
    let drop_on_thread = move || {
        if completed {
            assert!(session.resume_eval_request_with_input("done").is_ok());
        }
        let generation = session.logical_profile_generation();
        let before = lease::blocks_unregistered_thread();
        // The last observable pending state is sampled before Session is gone.
        let pending = usize::from(session.pending_input_request().is_some());
        eprintln!(
            "B713 DROP_ENTRY completed={completed} moved={moved} generation={generation:?} current={current} unregistered_before={before} pending={pending}"
        );
        assert_eq!(generation, if completed { None } else { Some(current) });
        assert_eq!(before, moved, "the worker must enter without registration");
        assert_eq!(pending, usize::from(!completed));
        drop(session);
        let after = lease::blocks_unregistered_thread();
        eprintln!("B713 DROP_EXIT completed={completed} moved={moved} unregistered_after={after}");
        assert_eq!(after, moved, "teardown must restore worker registration");
        let active = test_support::active();
        segments::release_thread_registration();
        (pending, active)
    };
    let (pending, active) = if moved {
        std::thread::spawn(drop_on_thread).join().unwrap()
    } else {
        drop_on_thread()
    };
    let observed = rows(&captured);
    let faults = captured.faults.load(Ordering::Relaxed) as usize;
    let contexts = captured.contexts.load(Ordering::Relaxed);
    segments::release_thread_registration();
    let bytes = crate::runtime::profiling::logical::brief713_probe_process_bytes();
    eprintln!("B713 DROP_ROWS completed={completed} moved={moved} rows={observed:?}");
    drop(captured);
    epoch::reset_for_lease(enabled.lease());
    let after_reset = crate::runtime::profiling::logical::brief713_probe_process_bytes();
    eprintln!(
        "B713 DROP_RESET completed={completed} moved={moved} bytes_after_reset={after_reset}"
    );
    assert_eq!(after_reset, 0, "reset must refund every remaining charge");
    (observed, [pending, faults, contexts, active, bytes])
}

fn compare_lifecycle(enabled: &test_support::Enabled, completed: bool) {
    let (owner, owner_teardown) = teardown(enabled, false, completed);
    let (worker, worker_teardown) = teardown(enabled, true, completed);
    let session_drop = 13 + Retire::SessionDrop.index();
    let last_drop = 13 + Retire::LastDrop.index();
    let totals = |observed: &BTreeMap<String, Vec<u64>>| -> [u64; 4] {
        [session_drop, last_drop, 9, 8].map(|index| observed.values().map(|row| row[index]).sum())
    };
    let owner_totals = totals(&owner);
    let worker_totals = totals(&worker);
    eprintln!(
        "B713 DROP_COMPARE completed={completed} own_rows={} moved_rows={} identical={} totals_order=[SessionDrop,LastDrop,open,abandoned] own_totals={owner_totals:?} moved_totals={worker_totals:?} own_teardown={owner_teardown:?} moved_teardown={worker_teardown:?}",
        owner.len(),
        worker.len(),
        owner == worker,
    );
    assert!(!owner.is_empty());
    assert_eq!(
        (owner_teardown[1], worker_teardown[1]),
        (0, 0),
        "STOP Q36: a fixed-tree fault requires director disposition"
    );
    assert_eq!(worker, owner, "all logical rows and abandonment reasons");
    assert_eq!(worker_teardown, owner_teardown);
    assert_eq!(owner_teardown[2..4], [0; 2]);
    assert!(owner.values().all(|row| row[9] == 0));
    if !completed {
        assert!(
            owner_totals[0] > 0,
            "paused drop must attribute SessionDrop"
        );
        assert_eq!(worker_totals[0], owner_totals[0]);
    }
}

#[test]
fn brief713_paused_drop_and_completed_control_preserve_all_rows() {
    let enabled = test_support::enabled();
    compare_lifecycle(&enabled, false);
    compare_lifecycle(&enabled, true);
}
