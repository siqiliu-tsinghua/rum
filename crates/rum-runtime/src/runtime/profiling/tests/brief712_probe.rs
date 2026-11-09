//! Pre-fix mechanism falsifiers. Scratch-only; unchanged expectations for a future repair.
use super::*;
use std::sync::{Arc, Barrier};

#[test]
fn brief712_unregistered_stash_theft_leaves_open_row() {
    let enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    assert!(matches!(
        session.eval_text("InputString[\"probe\"]"),
        Err(EvalError::InputRequired { .. })
    ));
    let key = AggregateKey::Runtime {
        phase: "evaluate.once_unstable",
        source: None,
    };
    let boundary = Call::start(&session, key, Policy::Expr)
        .unwrap()
        .pause()
        .unwrap();
    collector::stash_boundary(boundary);
    let taken = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker = std::thread::Builder::new()
        .name("brief712-outsider".into())
        .spawn({
            let taken = taken.clone();
            let release = release.clone();
            move || {
                assert!(crate::runtime::profiling::lease::blocks_unregistered_thread());
                let stolen = collector::take_stash();
                let count = stolen.len();
                taken.wait();
                release.wait();
                collector::restore_stash(stolen);
                count
            }
        })
        .unwrap();
    taken.wait();
    // The leased owner consumes every available boundary while the outsider is parked.
    // This is the exact take/complete contract; the full-lib trace covers guest-driver attachment.
    for boundary in collector::take_stash() {
        boundary.complete(Outcome::Ok);
    }
    let outcome = session.resume_suspended_evaluation("reply");
    let epoch = epoch::current().unwrap();
    let open = epoch
        .rows()
        .into_iter()
        .find(|(k, _)| *k == key)
        .unwrap()
        .1
        .open;
    release.wait();
    let stolen = worker.join().unwrap();
    assert_eq!(outcome.unwrap().to_string(), "\"reply\"");
    let faults = epoch.faults.load(std::sync::atomic::Ordering::Relaxed);
    eprintln!("B712 DETERMINISTIC theft={stolen} open={open} faults={faults}");
    epoch::reset_for_lease(enabled.lease());
    assert_eq!(
        (stolen, open, faults),
        (0, 0, 0),
        "foreign stash transaction must leave no row or fault"
    );
}

#[test]
fn brief712_unregistered_stale_attach_must_not_fault() {
    let enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    assert!(matches!(
        session.eval_text("InputString[\"probe\"]"),
        Err(EvalError::InputRequired { .. })
    ));
    let boundary = Call::start(
        &session,
        AggregateKey::Runtime {
            phase: "brief712.attach",
            source: None,
        },
        Policy::Expr,
    )
    .unwrap()
    .pause()
    .unwrap();
    collector::stash_boundary(boundary);
    let taken = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let worker = std::thread::Builder::new()
        .name("brief712-outsider".into())
        .spawn({
            let taken = taken.clone();
            let release = release.clone();
            move || {
                assert!(crate::runtime::profiling::lease::blocks_unregistered_thread());
                let stolen = collector::take_stash();
                taken.wait();
                release.wait();
                collector::restore_stash(stolen);
                std::panic::catch_unwind(collector::drain_attached).is_ok()
            }
        })
        .unwrap();
    taken.wait();
    epoch::reset_for_lease(enabled.lease());
    release.wait();
    let no_fault = worker.join().unwrap();
    eprintln!("B712 DETERMINISTIC stale_attach_no_fault={no_fault}");
    assert!(
        no_fault,
        "unregistered drain must not inspect a stale foreign boundary before owner lookup"
    );
}
