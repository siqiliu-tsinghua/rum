use super::{
    decode::{self, Key},
    support,
};
use crate::runtime::profiling::{
    logical::{
        AggregateKey, Call, Epoch, Outcome, Policy, Retire, epoch, publication,
        synchronous::{EventKey, EventValue},
    },
    tests::engine_support,
};
use crate::runtime::{Session, Symbol};

#[test]
fn reset_after_copy_invalidates_captured_receipt() {
    let enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    let old = engine_support::epoch();
    old.event(
        EventKey::Counter("definition_rhs_eval"),
        EventValue::count(7),
        None,
    );
    let copied = std::sync::Arc::new(std::sync::Barrier::new(2));
    let reset = std::sync::Arc::new(std::sync::Barrier::new(2));
    std::thread::scope(|scope| {
        let worker_copied = copied.clone();
        let worker_reset = reset.clone();
        let lease = enabled.lease();
        let worker = scope.spawn(move || {
            lease.allow_current_thread();
            publication::test_hooks::set_after_copy(Box::new(move || {
                worker_copied.wait();
                worker_reset.wait();
            }));
            decode::parse(&publication::render("captured-before-reset"))
        });
        copied.wait();
        epoch::reset_for_lease(enabled.lease());
        let replacement = engine_support::epoch();
        replacement.event(
            EventKey::Counter("definition_rhs_eval"),
            EventValue::count(9),
            None,
        );
        reset.wait();
        let snapshot = worker.join().unwrap();
        let key = Key::Counter("definition_rhs_eval".into());
        assert_eq!(snapshot.epoch, Some(old.id.0));
        assert_eq!(snapshot.validity, "invalid");
        assert_eq!(snapshot.reason, None);
        assert_eq!(snapshot.rows[&key].vector, [7]);
        assert_ne!(old.id, replacement.id);
        let current = support::observe("replacement");
        assert_eq!(current.epoch, Some(replacement.id.0));
        assert_eq!(current.rows[&key].vector, [9]);
    });
    println!(
        "Schema captured-reset verdict: PASS reset_after_both_copies=true captured_old_id=true old_vector=7 replacement_vector=9 validity=invalid torn_copy=false"
    );
}

#[test]
fn collision_fault_and_unsupported_native_rhs() {
    let enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    for alias in [false, true] {
        epoch::reset_for_lease(enabled.lease());
        let session = Session::new();
        let phase = if alias {
            "module.eval.total"
        } else {
            "compound.item.suspend"
        };
        Call::start(
            &session,
            AggregateKey::Runtime {
                phase,
                source: None,
            },
            Policy::Expr,
        )
        .unwrap()
        .finish(Outcome::Ok);
        let e = epoch::existing().unwrap();
        e.event(
            if alias {
                EventKey::Counter("eval_module")
            } else {
                EventKey::Runtime {
                    phase,
                    source: None,
                }
            },
            EventValue::count(1),
            None,
        );
        let snapshot = decode::parse(&publication::render("collision"));
        assert_eq!(snapshot.validity, "invalid");
        assert!(!e.is_valid());
        assert_eq!(snapshot.faults, 1);
        assert_eq!(snapshot.reason, None);
    }
    epoch::reset_for_lease(enabled.lease());
    let session = Session::new();
    Call::start(
        &session,
        AggregateKey::RuleRhs {
            phase: "rhs.instantiate",
            source: None,
        },
        Policy::Expr,
    )
    .unwrap()
    .finish(Outcome::Ok);
    let snapshot = decode::parse(&publication::render("unsupported RHS"));
    assert_eq!(snapshot.validity, "invalid");
    assert_eq!(snapshot.faults, 1);
    assert_eq!(
        snapshot.rows[&Key::Rhs("rhs.instantiate".into(), None)].vector,
        [0, 0, 0]
    );
    println!(
        "Schema collision verdict: PASS native_side_fault=true alias_side_fault=true unsupported_native_rhs_fault=true no_guest_outcome_changed=true"
    );
}

#[test]
fn invalid_cap_retains_rows_and_guest_resume() {
    let enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    let mut session = support::session();
    engine_support::limit(enabled.lease(), 65536);
    assert!(session.eval_request("InputString[\"cap\"]").is_err());
    let e = epoch::existing().unwrap();
    assert!(e.is_valid());
    for i in 0..4096 {
        if !e.is_valid() {
            break;
        }
        e.event(
            EventKey::CacheSymbol {
                event: "builtin_applications",
                symbol: Symbol::from_name(format!("cap{i}")),
            },
            EventValue::count(1),
            None,
        );
    }
    assert!(!e.is_valid());
    let snapshot = decode::parse(&publication::render("cap"));
    assert_eq!(snapshot.validity, "invalid");
    assert_eq!(snapshot.epoch, Some(e.id.0));
    assert_eq!(snapshot.reason, None);
    assert!(!snapshot.rows.is_empty());
    assert!(snapshot.rows.values().all(|row| row.accounting[0] == 0));
    assert_eq!(
        session
            .resume_eval_request_with_input("ok")
            .unwrap()
            .value
            .to_string(),
        "\"ok\""
    );
    assert_eq!(
        decode::parse(&publication::render("still-invalid")).validity,
        "invalid"
    );
    println!(
        "Schema cap verdict: PASS retained_rows=true validity=invalid guest_resume=ok parity_not_certified=true"
    );
}

#[test]
fn sticky_initial_allocation_failure() {
    let enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    let keeper = Epoch::for_test(enabled.lease(), 16 * 1024 * 1024).unwrap();
    let mut storage = keeper.storage(0).unwrap();
    // Find the exact remaining process budget without depending on layouts.
    // Failed resize leaves the retained charge and epoch unchanged.
    let (mut lo, mut hi) = (0, 16 * 1024 * 1024);
    while lo < hi {
        let mid = lo + (hi - lo + 1) / 2;
        if storage.resize(mid) {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    assert!(epoch::current().is_none());
    let failed = decode::parse(&publication::render("initial-cap"));
    assert_eq!(failed.validity, "invalid");
    assert!(failed.epoch.is_some());
    assert_eq!(failed.reason.as_deref(), Some("diagnostic-cap"));
    assert!(failed.rows.is_empty());
    drop(storage);
    drop(keeper);
    assert!(epoch::current().is_none());
    let sticky = decode::parse(&publication::render("freed"));
    assert_eq!(sticky.epoch, failed.epoch);
    assert_eq!(sticky.reason, failed.reason);
    epoch::reset_for_lease(enabled.lease());
    assert!(epoch::current().unwrap().is_valid());
    assert_eq!(support::observe("reset").reason, None);
    println!(
        "Schema initial-cap verdict: PASS failed_id_sticky=true released_budget_does_not_repair=true reset_clears_reason=true"
    );
}

#[test]
fn reset_old_markers_drop_and_fault() {
    let enabled = engine_support::enabled();
    let _flags = support::Flags::mode("lexical");
    let mut session = support::session();
    session
        .eval_text("f[] := Module[{x=3}, (InputString[\"a\"]; x)]")
        .unwrap();
    support::reset_epoch(enabled.lease());
    assert!(session.eval_request("f[]").is_err());
    let old = epoch::existing().unwrap();
    let stamp = std::time::Instant::now();
    let paused = support::observe("old");
    assert!(paused.rows.values().any(|r| r.accounting[0] > 0));
    epoch::reset_for_lease(enabled.lease());
    assert!(!old.is_valid());
    assert!(old.rows().iter().all(|(_, c)| c.open == 0));
    let retired = old
        .rows()
        .iter()
        .map(|(k, c)| (*k, c.calls, c.abandoned))
        .collect::<Vec<_>>();
    assert_eq!(
        session
            .resume_eval_request_with_input("answer")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        retired,
        old.rows()
            .iter()
            .map(|(k, c)| (*k, c.calls, c.abandoned))
            .collect::<Vec<_>>()
    );
    let new = epoch::current().unwrap();
    assert_ne!(old.id, new.id);
    new.event(
        EventKey::Counter("definition_rhs_eval"),
        EventValue::count(111),
        Some(stamp),
    );
    assert!(new.event_rows().iter().all(|(_, v)| v.vector[0] != 111));
    assert_eq!(support::observe("new").epoch, Some(new.id.0));
    assert_eq!(
        session
            .eval_text(
                "Function[{},Module[{x=3},(RumProfileReset[];RumProfileSnapshot[\"nested\"];x)]][]"
            )
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    support::reset_epoch(enabled.lease());
    assert!(session.eval_request("f[]").is_err());
    let e = epoch::existing().unwrap();
    let before = e.rows();
    drop(session);
    let after = decode::parse(&publication::render("drop-before-summary"));
    assert!(after.rows.values().all(|r| r.accounting[0] == 0));
    for (key, c) in before {
        let found = e.rows().into_iter().find(|(k, _)| *k == key).unwrap().1;
        assert_eq!(found.calls, c.calls + c.open);
        assert_eq!(found.ok, c.ok);
    }
    e.fault();
    e.retire(Retire::InstrumentationFault);
    let invalid = decode::parse(&publication::render("fault"));
    assert_eq!(invalid.validity, "invalid");
    assert_eq!(invalid.faults, 1);
    println!(
        "Schema lifecycle verdict: PASS nested_reset_result=3 old_markers_rejected=true old_rows_retired_once=true drop_before_summary=true fault_invalid=true"
    );
}
