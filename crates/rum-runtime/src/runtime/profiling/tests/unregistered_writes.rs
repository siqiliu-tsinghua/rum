//! Lexical counter writes from threads that never registered with the
//! current profiling test lease (brief-635 amendment 1).

use super::super::state::LEXICAL_PROFILE_SUMMARY_PRINTED;
use super::super::timing::lexical_profile_start;
use super::super::{
    print_lexical_profile_snapshot, print_lexical_profile_summary_once,
    record_profile_arg_list_start, record_profile_arg_prefilter,
    record_profile_definition_rhs_eval, record_profile_lexical_substitution_finish,
    record_profile_module_resume_entry, record_profile_rewrite_rule_match_outcome,
    record_profile_rewrite_rule_prefilter_candidate,
};
use crate::runtime::Session;
use crate::runtime::profiling::lease;
use crate::runtime::profiling::logical::test_support;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Instant;

/// Every `AtomicU64` in `state.rs` and the summary once-flag, by name, as the
/// observer reads them. The cold-load flags are `cold_load.rs`'s.
fn process_wide() -> BTreeMap<String, String> {
    let snapshot = crate::runtime::profiling::logical_profile_snapshot();
    let mut out = BTreeMap::new();
    for line in include_str!("../state.rs").lines() {
        let Some(rest) = line.split("static ").nth(1) else {
            continue;
        };
        let name = rest.split(':').next().unwrap();
        let value = if rest.contains(": AtomicU64") {
            let row = format!("counter|{}", name.to_lowercase().trim_end_matches("_count"));
            snapshot
                .telemetry
                .get(name)
                .cloned()
                .or_else(|| snapshot.logical.get(&row).map(|v| format!("{v:?}")))
        } else if name == "LEXICAL_PROFILE_SUMMARY_PRINTED" {
            snapshot.flags.get(name).map(u8::to_string)
        } else {
            continue;
        };
        let value = value.unwrap_or_else(|| panic!("the observer does not read {name}"));
        assert!(out.insert(name.to_string(), value).is_none());
    }
    assert_eq!(out.len(), 48, "47 counters and the once-flag");
    out
}

/// One write site per entry. Each runs on a fresh thread that never
/// registered, while this test holds the lease with both flags on.
#[test]
fn unregistered_writes_do_not_reach_a_held_window() {
    let _enabled = test_support::enabled();
    let printed = LEXICAL_PROFILE_SUMMARY_PRINTED.load(Ordering::Relaxed);
    let entries: [(&str, fn()); 9] = [
        ("record_timing", || {
            record_profile_definition_rhs_eval(Some(Instant::now()))
        }),
        ("record_profile_lexical_timing_elapsed", || {
            record_profile_lexical_substitution_finish(Some(Instant::now()), true)
        }),
        ("record_counter", || record_profile_arg_prefilter(true)),
        ("record_add", || record_profile_arg_list_start(1 << 40)),
        ("record_max", || record_profile_module_resume_entry(1 << 40)),
        ("rewrite_rule_prefilter_candidate", || {
            record_profile_rewrite_rule_prefilter_candidate(true)
        }),
        ("rewrite_rule_match_outcome", || {
            record_profile_rewrite_rule_match_outcome(true, true)
        }),
        ("summary_once", || {
            print_lexical_profile_summary_once(&Session::new())
        }),
        ("snapshot", || {
            print_lexical_profile_snapshot("unregistered")
        }),
    ];
    let mut wrote = Vec::new();
    for (site, entry) in entries {
        LEXICAL_PROFILE_SUMMARY_PRINTED.store(0, Ordering::Relaxed);
        let before = process_wide();
        std::thread::spawn(entry).join().expect(site);
        let after = process_wide();
        let moved: Vec<_> = before
            .iter()
            .filter(|(name, value)| after[*name] != **value)
            .map(|(name, _)| name.as_str())
            .collect();
        if !moved.is_empty() {
            wrote.push(format!("{site}: {moved:?}"));
        }
    }
    LEXICAL_PROFILE_SUMMARY_PRINTED.store(printed, Ordering::Relaxed);
    assert!(
        wrote.is_empty(),
        "unregistered threads wrote while a lease was held: {wrote:#?}"
    );
}

/// A start taken under one lease's flag, recorded under the next lease: the
/// shape of the 32-thread red at stage 1 (`definition_rhs_eval` 25 -> 26).
#[test]
fn unregistered_timing_started_under_one_lease_does_not_land_in_the_next() {
    let (started_tx, started) = mpsc::channel();
    let (go, go_rx) = mpsc::channel::<()>();
    let first = test_support::enabled();
    let worker = std::thread::spawn(move || {
        let start = lexical_profile_start();
        started_tx.send(start.is_some()).unwrap();
        go_rx.recv().unwrap();
        record_profile_definition_rhs_eval(start);
    });
    assert!(
        started.recv().unwrap(),
        "the worker's start was not taken under the flag"
    );
    drop(first);
    let _second = lease::lease();
    let before = process_wide();
    go.send(()).unwrap();
    worker.join().expect("worker");
    assert_eq!(
        process_wide(),
        before,
        "a start taken under one lease recorded into the next"
    );
}

/// A lease that becomes current waits for every lexical write admitted
/// before its generation was published.
#[test]
fn a_new_lease_waits_for_an_admitted_write() {
    let held = lease::lease();
    let admitted = lease::admit().expect("the holder's own thread is admitted");
    let returned = Arc::new(AtomicBool::new(false));
    let next = std::thread::spawn({
        let returned = Arc::clone(&returned);
        move || {
            let _next = lease::lease();
            returned.store(true, Ordering::SeqCst);
        }
    });
    drop(held);
    while lease::fencing() == 0 && !returned.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }
    assert!(
        !returned.load(Ordering::SeqCst),
        "a lease became current while an admitted write was in flight"
    );
    drop(admitted);
    next.join().expect("next lease");
}
