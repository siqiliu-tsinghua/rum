use super::*;

fn wait_for_reset(reset_done: &std::sync::atomic::AtomicBool) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while !reset_done.load(Ordering::Acquire)
        && crate::runtime::profiling::lease::reset_waiting() == 0
    {
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::yield_now();
    }
    true
}

#[test]
fn brief713_summary_existing_helpers_at_depth_zero() {
    let _enabled = crate::runtime::profiling::logical::test_support::enabled();
    print_lexical_profile_summary_body();
    eprintln!("B713 SUMMARY_BRIDGE held=false no_nested_admission=true");
}

#[test]
fn brief713_summary_existing_helpers_under_permit() {
    let _enabled = crate::runtime::profiling::logical::test_support::enabled();
    let result = std::panic::catch_unwind(|| {
        let access = crate::runtime::profiling::lease::admit().unwrap();
        print_lexical_profile_summary_body_admitted(&access);
    });
    let no_nested_admission = result.is_ok();
    eprintln!("B713 SUMMARY_BRIDGE held=true no_nested_admission={no_nested_admission}");
    assert!(
        no_nested_admission,
        "summary reporting helper needs an admitted bridge"
    );
}

#[test]
fn brief713_summary_permit_excludes_registered_reset() {
    let enabled = crate::runtime::profiling::logical::test_support::enabled();
    let lease = enabled.lease();
    let reset_done = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let access = crate::runtime::profiling::lease::admit().unwrap();
        DEFINITION_RHS_EVAL_COUNT.store(7, Ordering::Relaxed);
        print_lexical_profile_summary_body_admitted(&access);
        let before = load(&DEFINITION_RHS_EVAL_COUNT);
        let worker = scope.spawn(|| {
            lease.allow_current_thread();
            reset_lexical_profile_counters();
            reset_done.store(true, Ordering::Release);
        });
        let signaled = wait_for_reset(&reset_done);
        let reset_completed_while_held = reset_done.load(Ordering::Acquire);
        let while_held = load(&DEFINITION_RHS_EVAL_COUNT);
        let reset_fenced = crate::runtime::profiling::lease::reset_waiting() > 0;
        drop(access);
        assert!(
            signaled,
            "reset wait timed out: reset_completed_while_held={reset_completed_while_held} reset_fenced={reset_fenced} before={before} while_held={while_held}"
        );
        worker.join().unwrap();
        let after = load(&DEFINITION_RHS_EVAL_COUNT);
        eprintln!(
            "B713 SUMMARY_RESET before={before} while_held={while_held} reset_completed_while_held={reset_completed_while_held} reset_fenced={reset_fenced} after={after}"
        );
        assert_eq!(
            while_held, before,
            "registered reset crossed the summary permit"
        );
        assert!(!reset_completed_while_held);
        assert!(reset_fenced);
        assert_eq!(after, 0, "registered reset must complete after release");
    });
}

#[test]
fn brief713_summary_permit_excludes_registered_epoch_reset() {
    let enabled = crate::runtime::profiling::logical::test_support::enabled();
    let lease = enabled.lease();
    let before = logical::epoch::current().unwrap().id;
    let reset_done = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        let access = crate::runtime::profiling::lease::admit().unwrap();
        let worker = scope.spawn(|| {
            lease.allow_current_thread();
            logical::epoch::reset_for_lease(lease);
            reset_done.store(true, Ordering::Release);
        });
        let signaled = wait_for_reset(&reset_done);
        let reset_completed_while_held = reset_done.load(Ordering::Acquire);
        let reset_fenced = crate::runtime::profiling::lease::reset_waiting() > 0;
        drop(access);
        assert!(
            signaled,
            "epoch reset wait timed out: reset_completed_while_held={reset_completed_while_held} reset_fenced={reset_fenced} before={before:?}"
        );
        worker.join().unwrap();
        let after = logical::epoch::current().unwrap().id;
        let epoch_changed = before != after;
        eprintln!(
            "B713 EPOCH_RESET before={} after={} reset_completed_while_held={reset_completed_while_held} reset_fenced={reset_fenced} epoch_changed={epoch_changed}",
            before.0, after.0
        );
        assert!(!reset_completed_while_held);
        assert!(reset_fenced);
        assert!(
            epoch_changed,
            "registered epoch reset must complete after release"
        );
    });
}
