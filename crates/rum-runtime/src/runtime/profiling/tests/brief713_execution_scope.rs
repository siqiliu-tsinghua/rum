use super::*;
use crate::runtime::profiling::lease;

fn during_foreign_lease<T>(observe: impl FnOnce() -> T) -> T {
    let (ready, receive_ready) = std::sync::mpsc::channel();
    let (release, receive_release) = std::sync::mpsc::channel();
    let holder = std::thread::spawn(move || {
        let _current = crate::runtime::profiling::logical::test_support::enabled();
        ready.send(()).unwrap();
        receive_release.recv().unwrap();
    });
    receive_ready.recv().unwrap();
    let result = observe();
    release.send(()).unwrap();
    holder.join().unwrap();
    result
}

#[test]
fn brief713_execution_scope_cleans_only_its_owned_push() {
    let previous = crate::runtime::profiling::logical::test_support::enabled();
    let session = crate::runtime::Session::new();
    let baseline = EXECUTION_CONTROLS.with_borrow(|stack| stack.len());
    let scope = ExecutionScope::enter(&session);
    let pushed = EXECUTION_CONTROLS.with_borrow(|stack| stack.len());
    assert_eq!(pushed, baseline + 1);
    drop(previous);

    let (unregistered, remaining, stale_control) = during_foreign_lease(|| {
        let unregistered = lease::blocks_unregistered_thread();
        drop(scope);
        let remaining = EXECUTION_CONTROLS.with_borrow(|stack| stack.len());
        let stale_control = execution_control().is_some();
        // Keep a failed cleanup observer from contaminating the second clause.
        EXECUTION_CONTROLS.with_borrow_mut(|stack| stack.truncate(baseline));
        (unregistered, remaining, stale_control)
    });
    eprintln!(
        "B713 SCOPE_CLEANUP unregistered={unregistered} baseline={baseline} pushed={pushed} remaining={remaining} stale_control={stale_control}"
    );

    let previous = crate::runtime::profiling::logical::test_support::enabled();
    let older = ExecutionScope::enter(&session);
    let older_control = execution_control().unwrap();
    let older_depth = EXECUTION_CONTROLS.with_borrow(|stack| stack.len());
    assert_eq!(older_depth, baseline + 1);
    drop(previous);
    let (refused, after_refused, older_preserved) = during_foreign_lease(|| {
        let refused = lease::blocks_unregistered_thread();
        let scope = ExecutionScope::enter(&session);
        drop(scope);
        let after_refused = EXECUTION_CONTROLS.with_borrow(|stack| stack.len());
        let older_preserved =
            execution_control().is_some_and(|control| Arc::ptr_eq(&control, &older_control));
        (refused, after_refused, older_preserved)
    });
    let _registered = crate::runtime::profiling::logical::test_support::enabled();
    drop(older);
    let cleaned = EXECUTION_CONTROLS.with_borrow(|stack| stack.len());
    eprintln!(
        "B713 SCOPE_REFUSED unregistered={refused} older_depth={older_depth} remaining={after_refused} older_preserved={older_preserved} cleaned={cleaned}"
    );

    assert!(unregistered);
    assert_eq!(
        remaining, baseline,
        "owned execution control must unwind during a foreign lease"
    );
    assert!(!stale_control);
    assert!(refused);
    assert_eq!(
        after_refused, older_depth,
        "a refused scope must not pop the older entry"
    );
    assert!(older_preserved);
    assert_eq!(cleaned, baseline);
}
