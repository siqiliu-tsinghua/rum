use super::*;
use crate::runtime::profiling::lease;

#[test]
fn brief713_refused_timing_completion_cleans_owned_frames() {
    let (ready, prepared) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let symbol = Symbol::system("brief713_timing_cleanup");
        EVALUATION_PROFILE_SOURCE_STACK.with_borrow_mut(|stack| stack.push(symbol));
        EVALUATION_PROFILE_TIMING_STACK.with_borrow_mut(|stack| {
            for name in [
                "parent",
                "definition_rhs_eval_by_source",
                "eval_module_by_source",
            ] {
                stack.push(SymbolTimingFrame {
                    name,
                    symbol,
                    child_nanos: 0,
                });
            }
        });
        CALL_STABILITY_TIMING_STACK.with_borrow_mut(|stack| {
            for _ in 0..2 {
                stack.push(CallStabilityTimingFrame {
                    start: Instant::now(),
                    child_nanos: 0,
                });
            }
        });
        let start = Instant::now();
        ready.send(()).unwrap();
        released.recv().unwrap();
        assert!(lease::blocks_unregistered_thread());
        record_profile_eval_module_by_source(Some(start), true);
        record_profile_definition_rhs_eval_by_source(Some(start));
        record_call_stability_source_head_timing(
            &Expr::Atom(Atom::Symbol(symbol)),
            CallStabilityProfileOutcome::Hit,
        );
        let symbols = EVALUATION_PROFILE_TIMING_STACK
            .with_borrow(|stack| (stack.len(), stack[0].child_nanos));
        let calls =
            CALL_STABILITY_TIMING_STACK.with_borrow(|stack| (stack.len(), stack[0].child_nanos));
        (symbols, calls)
    });
    prepared.recv().unwrap();
    let _lease = lease::lease();
    release.send(()).unwrap();
    let (symbols, calls) = worker.join().unwrap();
    eprintln!(
        "B713 TIMING_CLEANUP symbol_frames={} call_frames={} symbol_parent_credited={} call_parent_credited={}",
        symbols.0,
        calls.0,
        symbols.1 > 0,
        calls.1 > 0
    );
    assert_eq!((symbols.0, calls.0), (1, 1));
    assert!(symbols.1 > 0 && calls.1 > 0);
}
