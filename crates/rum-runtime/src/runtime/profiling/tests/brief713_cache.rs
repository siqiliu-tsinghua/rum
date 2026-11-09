//! Cache boundary probes, mounted inside evaluation_cache::profile.
use super::*;
use crate::runtime::profiling::{lease, logical::test_support};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;

impl Session {
    pub(in crate::runtime) fn brief713_cache_flag_reads() -> [(&'static str, bool); 3] {
        [
            (
                "EVALUATION_CACHE_PROFILE_ENABLED",
                evaluation_cache_profile_configured(),
            ),
            (
                "EVALUATION_CACHE_PROFILE_ACTIVE",
                evaluation_cache_profile_active(),
            ),
            (
                "EVALUATION_CACHE_PROFILE_INCREMENTAL_ENABLED",
                evaluation_cache_profile_incremental_enabled(),
            ),
        ]
    }
}

#[test]
fn brief713_flag_swap_cannot_be_observed_across_publication() {
    let held = lease::lease();
    let saved = Session::test_cache_profile_flags([2; 3]);
    let (parked, parking) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let published = AtomicBool::new(false);
    let (observed, fenced) = std::thread::scope(|scope| {
        let held = &held;
        let reader = scope.spawn(move || {
            held.allow_current_thread();
            // admit has read the generation, but the flag load has not run.
            let access = lease::admit().unwrap();
            parked.send(()).unwrap();
            released.recv().unwrap();
            let observed = EVALUATION_CACHE_PROFILE_ENABLED.load(Ordering::Relaxed) == 1;
            drop(access);
            observed
        });
        parking.recv().unwrap();
        let publisher = scope.spawn(|| {
            held.allow_current_thread();
            Session::test_cache_profile_flags([1; 3]);
            published.store(true, Ordering::SeqCst);
        });
        while lease::fencing() == 0 && !published.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        let fenced = lease::fencing() > 0;
        release.send(()).unwrap();
        let observed = reader.join().unwrap();
        publisher.join().unwrap();
        (observed, fenced)
    });
    Session::test_cache_profile_flags(saved);
    eprintln!("B713 FLAG_SWAP observed_swap={observed} publication_fenced={fenced}");
    assert_eq!((observed, fenced), (false, true));
}

#[test]
fn brief713_cache_write_from_unregistered_thread_is_refused() {
    let _enabled = test_support::enabled();
    let saved = Session::test_cache_profile_flags([1, 1, 2]);
    let epoch = crate::runtime::profiling::logical::epoch::current().unwrap();
    let before = crate::runtime::profiling::logical_profile_snapshot();
    std::thread::spawn(|| {
        let symbol = Symbol::system("brief713_cache");
        record_evaluation_cache_counter(&EVALUATE_ENTRY_COUNT, "evaluate_entries");
        record_symbol_profile_counter("down_value_scans", &symbol);
        record_invalidation_site("brief713_cache", None, Some(&symbol), Location::caller());
        Session::module_source_alias_completion(symbol);
        Session::module_source_alias_duration(symbol, 17, 11);
    })
    .join()
    .unwrap();
    let after = crate::runtime::profiling::logical_profile_snapshot();
    Session::test_cache_profile_flags(saved);
    let counter = "cache|EVALUATE_ENTRY_COUNT";
    let counter_changed = before.logical.get(counter) != after.logical.get(counter);
    let maps = [
        "symbol_profile_counters",
        "invalidation_site_profile_counters",
        "invalidation_family_profile_counters",
        "symbol_timing_profile_counters",
    ];
    let changed: Vec<_> = maps
        .into_iter()
        .filter(|name| before.telemetry.get(*name) != after.telemetry.get(*name))
        .collect();
    let faults = epoch.faults.load(Ordering::Relaxed);
    eprintln!(
        "B713 CACHE_WRITE counter_changed={counter_changed} maps_changed={changed:?} faults={faults}"
    );
    assert!(!counter_changed);
    assert!(changed.is_empty());
    assert_eq!(faults, 0);
}
