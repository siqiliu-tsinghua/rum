//! A moved session inherits only the generation of its retained profiler state.
use super::*;
use std::collections::BTreeMap;

fn logical_rows(epoch: &Epoch) -> BTreeMap<String, Vec<u64>> {
    epoch
        .rows()
        .into_iter()
        .map(|(key, c)| {
            // All logical counts, including physical pause observations and
            // every abandonment reason. Monotonic durations are not counts.
            let mut counts = vec![
                c.calls,
                c.ok,
                c.err,
                c.some,
                c.none,
                c.return_err,
                c.throw_err,
                c.cancelled,
                c.abandoned,
                c.open,
                c.pauses,
                c.input_required,
                c.segments,
            ];
            counts.extend(c.abandonment_reasons);
            (format!("{key:?}"), counts)
        })
        .collect()
}

fn cancel_on_thread(
    enabled: &test_support::Enabled,
    moved: bool,
) -> (BTreeMap<String, Vec<u64>>, [usize; 5]) {
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    epoch::reset_for_lease(enabled.lease());
    let cancel = session.request_cancel_handle();
    assert!(
        session
            .eval_request("WithCleanup[InputString[\"pause\"],Function[x,x][1]]")
            .is_err()
    );
    assert!(session.pending_input_request().is_some());
    cancel.cancel();
    let resume = move || {
        assert!(session.resume_eval_request_with_input("ignored").is_ok());
        session
    };
    let session = if moved {
        std::thread::spawn(resume).join().unwrap()
    } else {
        resume()
    };
    let pending = usize::from(session.pending_input_request().is_some());
    drop(session);
    let epoch = test_support::epoch();
    let rows = logical_rows(&epoch);
    let faults = epoch.faults.load(Ordering::Relaxed) as usize;
    let contexts = epoch.contexts.load(Ordering::Relaxed);
    let active = test_support::active();
    drop(epoch);
    // The empty per-thread segment header normally lives until lease release.
    // Use that same teardown here while retaining exclusion for the byte read.
    segments::release_thread_registration();
    epoch::reset_for_lease(enabled.lease());
    let bytes = budget::process_bytes();
    (rows, [pending, faults, contexts, active, bytes])
}

#[test]
fn brief713_moved_cancel_preserves_every_logical_row() {
    let enabled = test_support::enabled();
    let (own, own_teardown) = cancel_on_thread(&enabled, false);
    let (moved, moved_teardown) = cancel_on_thread(&enabled, true);
    eprintln!(
        "B713 MOVED_CANCEL own_rows={} moved_rows={} identical={} own_teardown={own_teardown:?} moved_teardown={moved_teardown:?}",
        own.len(),
        moved.len(),
        own == moved,
    );
    assert!(!own.is_empty());
    assert!(
        own.keys()
            .any(|key| key.contains("function.apply.evaluate"))
    );
    assert_eq!(
        moved, own,
        "every session row must survive a moved cancellation"
    );
    assert!(own.values().all(|counts| counts[9] == 0));
    assert_eq!(own_teardown, [0; 5]);
    assert_eq!(moved_teardown, [0; 5]);
}

struct ProfileFlags([u8; 2]);

impl ProfileFlags {
    fn enabled() -> Self {
        use crate::runtime::profiling::{lease, state};
        let _publication = lease::publish().expect("the current lease owns flag publication");
        Self([
            state::LEXICAL_PROFILE_ENABLED.swap(1, Ordering::Relaxed),
            state::LEXICAL_PROFILE_ACTIVE.swap(1, Ordering::Relaxed),
        ])
    }
}

impl Drop for ProfileFlags {
    fn drop(&mut self) {
        use crate::runtime::profiling::{lease, state};
        let _publication = lease::publish().expect("the current lease owns flag restoration");
        state::LEXICAL_PROFILE_ENABLED.store(self.0[0], Ordering::Relaxed);
        state::LEXICAL_PROFILE_ACTIVE.store(self.0[1], Ordering::Relaxed);
    }
}

#[test]
fn brief713_stale_session_and_context_do_not_enter_next_generation() {
    use crate::runtime::profiling::{lease, logical_profile_snapshot};
    let first = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    assert!(
        session
            .eval_request("WithCleanup[InputString[\"old\"],Function[x,x][1]]")
            .is_err()
    );
    assert!(session.pending_input_request().is_some());
    let old_epoch = test_support::epoch();
    let context = Context::capture(&old_epoch).unwrap();
    drop(first);

    // Raw lease acquisition intentionally does not wait for the retained old
    // queue/context. They are caller-owned and must be safe across this handoff.
    let second = lease::lease();
    let _flags = ProfileFlags::enabled();
    let current = test_support::epoch();
    assert_ne!(old_epoch.lease_generation(), current.lease_generation());
    let before = logical_profile_snapshot().logical;
    let (span_refused, context_refused, restored, completed) = std::thread::spawn(move || {
        assert!(lease::blocks_unregistered_thread());
        let scope = ExecutionScope::enter(&session);
        let span_refused = !scope.pushed && execution_control().is_none();
        crate::runtime::profiling::record_profile_arg_prefilter(true);
        drop(scope);
        let guard = context.enter_with_control(Some(Arc::clone(
            &session.request_interrupt_control.borrow(),
        )));
        let context_refused = guard.is_none();
        crate::runtime::profiling::record_profile_arg_prefilter(true);
        drop(guard);
        let completed = session.resume_eval_request_with_input("stale").is_ok();
        let restored = lease::blocks_unregistered_thread() && execution_control().is_none();
        (span_refused, context_refused, restored, completed)
    })
    .join()
    .unwrap();
    let unchanged = before == logical_profile_snapshot().logical;
    let faults = current.faults.load(Ordering::Relaxed);
    let old_contexts = old_epoch.contexts.load(Ordering::Relaxed);
    eprintln!(
        "B713 STALE_GENERATION span_refused={span_refused} context_refused={context_refused} restored={restored} completed={completed} current_unchanged={unchanged} faults={faults} old_contexts={old_contexts}"
    );
    assert_eq!(
        (
            span_refused,
            context_refused,
            restored,
            completed,
            unchanged
        ),
        (true, true, true, true, true),
    );
    assert_eq!((faults, old_contexts), (0, 0));
    drop(old_epoch);
    drop(current);
    epoch::reset_for_lease(&second);
}
