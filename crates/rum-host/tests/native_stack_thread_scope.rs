use rum_host::{ClosurePrintSink, HostConfig, HostEvalOutcome, HostSession, HostTerminationReason};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

const SOURCE: &str = "kept=42; cleanupCount=0; tail=0; WithCleanup[InputString[\"first: \"]; InputString[\"second: \"], cleanupCount=cleanupCount+1; Print[\"cleanup\"]]; tail=1; Print[\"tail\"]";

const FAIL_OPEN_STACK_SIZE: usize = 16 * 1024 * 1024;
const FAIL_OPEN_SETUP: &str = "dive[n_] := If[n == 50, (Print[\"past-first-poll\"]; Identity[dive[n - 1]]), If[n <= 0, Print[\"cleanup-past-stack-limit\"], Identity[dive[n - 1]]]]";
const FAIL_OPEN_SOURCE: &str = "WithCleanup[InputString[\"prompt\"], dive[60]]";

fn stack_addr() -> usize {
    let marker = 0_u8;
    &marker as *const u8 as usize
}

fn prints(outcome: &HostEvalOutcome) -> String {
    match outcome {
        HostEvalOutcome::Success(s) => {
            format!(
                "Success prints={:?} termination={:?}",
                s.prints, s.termination
            )
        }
        HostEvalOutcome::Failure(f) => format!("Failure code={:?} message={}", f.code, f.message),
        HostEvalOutcome::InputRequired(_) => "InputRequired".to_string(),
        #[allow(unreachable_patterns)]
        _ => "other".to_string(),
    }
}

/// Suspend on thread A, cancel on thread B. `a_alive` keeps A running while B
/// cancels, so B's stack is a fresh mapping rather than A's reused one.
fn suspend_then_cancel(a_alive: bool) -> String {
    let (send_session, receive_session) = mpsc::channel();
    let (release, released) = mpsc::channel::<()>();
    let a = thread::spawn(move || {
        let mut session = HostSession::new();
        let marker = 0_u8;
        eprintln!("DIR628 A stack {:#x}", &marker as *const u8 as usize);
        let outcome = session.eval(SOURCE);
        assert!(
            matches!(outcome, HostEvalOutcome::InputRequired(_)),
            "{}",
            prints(&outcome)
        );
        send_session.send(session).unwrap();
        if a_alive {
            released.recv().unwrap();
        }
    });
    let mut session = receive_session.recv().unwrap();
    let mut a = Some(a);
    if !a_alive {
        a.take().unwrap().join().unwrap();
    }
    let b = thread::spawn(move || {
        let marker = 0_u8;
        eprintln!("DIR628 B stack {:#x}", &marker as *const u8 as usize);
        session.cancel_handle().cancel();
        prints(&session.eval(""))
    });
    let result = b.join().unwrap();
    if let Some(a) = a {
        release.send(()).unwrap();
        a.join().unwrap();
    }
    result
}

fn session_for_fail_open() -> HostSession {
    let mut config = HostConfig::default();
    config.resource_limits.recursion_limit = 4096;
    assert_eq!(config.resource_limits.native_stack_limit, 1_048_576);
    HostSession::builder().with_config(config).build()
}

fn suspend_fail_open_on_current_thread() -> (HostSession, usize) {
    let mut session = session_for_fail_open();
    let setup = session.eval(FAIL_OPEN_SETUP);
    assert!(
        matches!(setup, HostEvalOutcome::Success(_)),
        "expected setup success, got {}",
        prints(&setup)
    );
    let a_stack = stack_addr();
    let outcome = session.eval(FAIL_OPEN_SOURCE);
    assert!(
        matches!(outcome, HostEvalOutcome::InputRequired(_)),
        "expected InputRequired, got {}",
        prints(&outcome)
    );
    (session, a_stack)
}

#[test]
fn same_thread_cancelled_cleanup_control() {
    let mut session = HostSession::new();
    let outcome = session.eval(SOURCE);
    assert!(matches!(outcome, HostEvalOutcome::InputRequired(_)));
    session.cancel_handle().cancel();
    let observed = prints(&session.eval(""));
    eprintln!("DIR628 same-thread: {observed}");
    assert_eq!(
        observed,
        "Success prints=[\"cleanup\"] termination=Some(Cancelled)"
    );
}

#[test]
fn cross_thread_cancelled_cleanup_after_origin_exits_control() {
    let observed = suspend_then_cancel(false);
    eprintln!("DIR628 moved, A exited: {observed}");
    assert_eq!(
        observed,
        "Success prints=[\"cleanup\"] termination=Some(Cancelled)"
    );
}

#[test]
fn cross_thread_cancelled_cleanup_with_origin_alive_observation() {
    eprintln!("DIR628 moved, A alive: {}", suspend_then_cancel(true));
}

#[test]
fn cross_thread_cancelled_cleanup_with_lower_stack_runs_cleanup() {
    let observed = suspend_then_cancel(true);
    eprintln!("BRIEF628 false-positive falsifier observed: {observed}");
    assert_eq!(
        observed, "Success prints=[\"cleanup\"] termination=Some(Cancelled)",
        "false-positive falsifier: cleanup must print \"cleanup\" on B even while A holds its stack"
    );
}

#[test]
fn same_thread_deep_cleanup_enforces_native_stack_limit_control() {
    let (direct_outcome, cancel_outcome, a_stack, b_stack) = thread::Builder::new()
        .name("fail-open-control".into())
        .stack_size(FAIL_OPEN_STACK_SIZE)
        .spawn(|| {
            // 1. Direct evaluation of `dive[60]` at top level must terminate with `StackLimit`.
            let mut direct_session = session_for_fail_open();
            direct_session.eval(FAIL_OPEN_SETUP);
            let direct_outcome = direct_session.eval("dive[60]");

            // 2. Same-thread suspension + cancellation: cleanup starts (`"past-first-poll"` at
            //    depth > 20) and then hits `StackLimitExceeded` before reaching `n == 0`.
            let (mut session, a_stack) = suspend_fail_open_on_current_thread();
            let b_stack = stack_addr();
            session.cancel_handle().cancel();
            let cancel_outcome = session.eval("");
            (direct_outcome, cancel_outcome, a_stack, b_stack)
        })
        .unwrap()
        .join()
        .unwrap();

    eprintln!(
        "BRIEF628 fail-open direct control: A={a_stack:#x} B={b_stack:#x} -> {}",
        prints(&direct_outcome)
    );
    let HostEvalOutcome::Success(direct) = direct_outcome else {
        panic!("expected Success for direct dive[60], got {direct_outcome:?}");
    };
    assert_eq!(
        direct.termination,
        Some(HostTerminationReason::StackLimit),
        "direct dive[60] must exceed the 1 MiB native stack limit on the same thread"
    );
    assert_eq!(
        direct.prints,
        vec!["past-first-poll".to_string()],
        "direct dive[60] must pass the first poll and stop before the leaf print"
    );

    eprintln!(
        "BRIEF628 fail-open same-thread control: A={a_stack:#x} B={b_stack:#x} -> {}",
        prints(&cancel_outcome)
    );
    let HostEvalOutcome::Success(cancel) = cancel_outcome else {
        panic!("expected Success for cancelled control, got {cancel_outcome:?}");
    };
    assert_eq!(cancel.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(
        cancel.prints,
        vec!["past-first-poll".to_string()],
        "same-thread cleanup must fire the native stack guard before reaching the leaf print"
    );
}

#[test]
fn cross_thread_cancelled_cleanup_with_higher_stack_enforces_native_stack_limit() {
    let (tx_b_ready, rx_b_ready) = mpsc::channel();
    let (tx_to_b, rx_to_b) = mpsc::channel::<(HostSession, usize)>();
    let (tx_a_release, rx_a_release) = mpsc::channel();

    // Spawn B first with a 16 MiB stack and keep it waiting so Linux mmap places
    // B's stack at a higher virtual address than A's stack.
    let b = thread::Builder::new()
        .name("fail-open-b-high".into())
        .stack_size(FAIL_OPEN_STACK_SIZE)
        .spawn(move || {
            let b_ready_stack = stack_addr();
            tx_b_ready.send(b_ready_stack).unwrap();
            let (mut session, a_stack) = rx_to_b.recv().unwrap();
            let b_stack = stack_addr();
            session.cancel_handle().cancel();
            let outcome = session.eval("");
            (outcome, a_stack, b_stack)
        })
        .unwrap();

    let b_ready_stack = rx_b_ready.recv().unwrap();

    // Spawn A second while B is alive so A's stack is mapped below B's stack.
    let a = thread::Builder::new()
        .name("fail-open-a-low".into())
        .stack_size(FAIL_OPEN_STACK_SIZE)
        .spawn(move || {
            let (session, a_stack) = suspend_fail_open_on_current_thread();
            tx_to_b.send((session, a_stack)).unwrap();
            rx_a_release.recv().unwrap();
        })
        .unwrap();

    let (outcome, a_stack, b_stack) = b.join().unwrap();
    tx_a_release.send(()).unwrap();
    a.join().unwrap();

    assert!(
        b_stack > a_stack,
        "expected B's stack ({b_stack:#x}, ready {b_ready_stack:#x}) to lie above A's stack ({a_stack:#x})"
    );
    let delta_mib = (b_stack - a_stack) as f64 / (1024.0 * 1024.0);
    eprintln!(
        "BRIEF628 fail-open moved (B above A by {delta_mib:.2} MiB): A={a_stack:#x} B={b_stack:#x} -> {}",
        prints(&outcome)
    );

    let HostEvalOutcome::Success(s) = outcome else {
        panic!("expected Success, got {outcome:?}");
    };
    assert_eq!(s.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(
        s.prints,
        vec!["past-first-poll".to_string()],
        "fail-open falsifier: cleanup on B (stack {b_stack:#x} above A's stale base {a_stack:#x}) \
         must still enforce the 1 MiB native stack limit and stop before printing \"cleanup-past-stack-limit\""
    );
}

#[test]
fn same_thread_cancelled_cleanup_after_deep_suspension_resets_check_threshold() {
    let (direct_outcome, stale_depth_outcome) = thread::Builder::new()
        .name("stale-check-depth".into())
        .stack_size(FAIL_OPEN_STACK_SIZE)
        .spawn(|| {
            // Build a cleanup expression with 60 nested `Module[{x = 1}, ...]` frames (~64
            // evaluation_depth steps total). At depth 16 it prints `"past-first-poll"`, and at
            // the leaf (depth ~64) it prints `"cleanup-past-stack-limit"`.
            let cleanup_expr = format!(
                "{}(Print[\"past-first-poll\"]; {}Print[\"cleanup-past-stack-limit\"]{}){}",
                "Module[{x = 1}, ".repeat(16),
                "Module[{x = 1}, ".repeat(44),
                "]".repeat(44),
                "]".repeat(16),
            );

            // 1. Control: `cleanup_expr` evaluated directly at depth 0 exceeds 1 MiB and stops at `StackLimit`.
            let mut control = session_for_fail_open();
            let direct_outcome = control.eval(cleanup_expr.as_str());

            // 2. Same-thread stale check threshold:
            //    80 nested `u[...]` calls suspend at evaluation_depth ~83 (< 1 MiB), leaving
            //    the threshold at 88. When cancelled on the same thread, `WithCleanup` restores
            //    `evaluation_depth = 2` and must reset the threshold so `cleanup_expr` (max depth
            //    ~66 < 88, > 1 MiB of native stack) polls and stops at the native stack limit.
            let mut session = session_for_fail_open();
            let deep_input = format!("{}InputString[\"deep\"]{}", "u[".repeat(80), "]".repeat(80));
            let source = format!("WithCleanup[{deep_input}, {cleanup_expr}]");
            let suspended = session.eval(source.as_str());
            assert!(
                matches!(suspended, HostEvalOutcome::InputRequired(_)),
                "expected InputRequired from deep input, got {}",
                prints(&suspended)
            );
            session.cancel_handle().cancel();
            let stale_depth_outcome = session.eval("");
            (direct_outcome, stale_depth_outcome)
        })
        .unwrap()
        .join()
        .unwrap();

    eprintln!(
        "BRIEF628 stale-check-depth direct control: {}",
        prints(&direct_outcome)
    );
    let HostEvalOutcome::Success(direct) = direct_outcome else {
        panic!("expected Success for direct cleanup_expr, got {direct_outcome:?}");
    };
    assert_eq!(
        direct.termination,
        Some(HostTerminationReason::StackLimit),
        "direct cleanup_expr must exceed the 1 MiB native stack limit"
    );
    assert_eq!(direct.prints, vec!["past-first-poll".to_string()]);

    eprintln!(
        "BRIEF628 stale-check-depth same-thread cancelled: {}",
        prints(&stale_depth_outcome)
    );
    let HostEvalOutcome::Success(stale) = stale_depth_outcome else {
        panic!("expected Success for cancelled stale-depth probe, got {stale_depth_outcome:?}");
    };
    assert_eq!(stale.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(
        stale.prints,
        vec!["past-first-poll".to_string()],
        "stale native_stack_next_check_depth falsifier: cleanup after a deep suspension on the \
         same thread must still poll and stop before printing \"cleanup-past-stack-limit\""
    );
}

#[test]
fn nested_session_in_host_callback_shares_outermost_thread_stack_base() {
    let (solo_12, solo_24, outer_outcome, inner_outcome) = thread::Builder::new()
        .name("nested-session-probe".into())
        .stack_size(FAIL_OPEN_STACK_SIZE)
        .spawn(|| {
            let dive_setup =
                "dive[n_] := If[n == 8, (Print[\"past-first-poll\"]; Identity[dive[n - 1]]), \
                              If[n <= 0, Print[\"leaf-reached\"], Identity[dive[n - 1]]]]";

            // 1. Controls: `dive[12]` alone stays under 1 MiB (`leaf-reached`), while `dive[24]`
            //    alone exceeds 1 MiB (`StackLimit`).
            let mut solo = session_for_fail_open();
            solo.eval(dive_setup);
            let solo_12 = solo.eval("dive[12]");
            let solo_24 = solo.eval("dive[24]");

            // 2. Nested sessions on one thread: outer Session 1 consumes ~650 KiB via `outerDive[15]`
            //    and at `n == 0` prints `"trigger-inner"`, whose `PrintSink` callback evaluates
            //    `dive[12]` (~650 KiB) in inner Session 2 on the same thread (~1.3 MiB total).
            let inner_observed: Arc<Mutex<Option<HostEvalOutcome>>> = Arc::new(Mutex::new(None));
            let inner_slot = Arc::clone(&inner_observed);

            let mut outer = session_for_fail_open();
            outer.eval(
                "outerDive[n_] := If[n <= 0, Print[\"trigger-inner\"], Identity[outerDive[n - 1]]]",
            );
            outer.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
                if line == "trigger-inner" {
                    let mut inner = session_for_fail_open();
                    inner.eval(dive_setup);
                    let outcome = inner.eval("dive[12]");
                    *inner_slot.lock().unwrap() = Some(outcome);
                }
            }));

            let outer_outcome = outer.eval("outerDive[15]");
            let inner_outcome = inner_observed
                .lock()
                .unwrap()
                .take()
                .expect("PrintSink callback must have invoked inner Session 2");
            (solo_12, solo_24, outer_outcome, inner_outcome)
        })
        .unwrap()
        .join()
        .unwrap();

    eprintln!("BRIEF628 nested control dive[12]: {}", prints(&solo_12));
    eprintln!("BRIEF628 nested control dive[24]: {}", prints(&solo_24));
    eprintln!(
        "BRIEF628 nested outer outerDive[15]: {}",
        prints(&outer_outcome)
    );
    eprintln!("BRIEF628 nested inner dive[12]: {}", prints(&inner_outcome));

    let HostEvalOutcome::Success(s12) = solo_12 else {
        panic!("expected Success for solo dive[12]");
    };
    assert_eq!(s12.termination, None);
    assert_eq!(
        s12.prints,
        vec!["past-first-poll".to_string(), "leaf-reached".to_string()]
    );

    let HostEvalOutcome::Success(s24) = solo_24 else {
        panic!("expected Success for solo dive[24]");
    };
    assert_eq!(s24.termination, Some(HostTerminationReason::StackLimit));
    assert_eq!(s24.prints, vec!["past-first-poll".to_string()]);

    let HostEvalOutcome::Success(inner) = inner_outcome else {
        panic!("expected Success for inner dive[12]");
    };
    assert_eq!(
        inner.termination,
        Some(HostTerminationReason::StackLimit),
        "nested-session falsifier: inner Session 2 on the same thread must measure against the \
         thread's outermost rum entry and fire StackLimit before reaching \"leaf-reached\""
    );
}
