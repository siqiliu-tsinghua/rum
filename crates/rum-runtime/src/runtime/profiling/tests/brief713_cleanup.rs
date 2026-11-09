use super::*;
use crate::runtime::profiling::lease;
use std::sync::atomic::Ordering;

#[test]
fn brief713_prior_generation_cleanup_completes() {
    let previous = lease::lease();
    let old = Epoch::for_test(&previous, 1024 * 1024).unwrap();
    let storage = old.storage(32 * 1024).unwrap();
    let path = RecordPath::test_chain(
        &old,
        Owner {
            control: Arc::new(crate::runtime::RequestInterruptState::new()),
            generation: 1,
            root: 1,
        },
        4,
    );
    let probe = old.test_budget_probe();
    drop(old);
    drop(previous);

    // The raw lease deliberately leaves the old charge pending. The existing
    // lease_acquisition_drains_a_prior_epoch_charge test covers pre-enable drain.
    let current = lease::lease();
    let epoch = epoch::current().unwrap();
    let rows = epoch.rows();
    let bytes = probe
        .bytes()
        .expect("old owners must retain their Budget before cleanup");
    eprintln!("B716 OLD_CHARGE present={bytes} minimum={}", 32 * 1024);
    assert!(bytes >= 32 * 1024);
    std::thread::spawn(move || {
        assert!(lease::blocks_unregistered_thread());
        drop(storage);
        drop(path);
    })
    .join()
    .unwrap();
    let remaining = probe.bytes();
    let same_rows = format!("{:?}", epoch.rows()) == format!("{rows:?}");
    let faults = epoch.faults.load(Ordering::Relaxed);
    eprintln!(
        "B713 CLEANUP before={bytes} remaining={remaining:?} rows_unchanged={same_rows} faults={faults}"
    );
    assert_eq!(remaining, None);
    assert!(same_rows);
    assert_eq!(faults, 0);
    epoch::reset_for_lease(&current);
}

#[test]
fn brief713_boundary_storage_layout_is_unchanged() {
    let production = std::mem::size_of::<Storage>();
    let test = std::mem::size_of::<BoundaryStorage>();
    eprintln!("B713 BOUNDARY_LAYOUT production_alias_bytes={production} test_alias_bytes={test}");
    assert_eq!(production, test);
}

#[test]
fn brief713_prior_boundary_settles_without_current_aliases() {
    let previous = lease::lease();
    let old = Epoch::for_test(&previous, 1024 * 1024).unwrap();
    let symbol = crate::runtime::Symbol::system("brief713_old_boundary");
    let record = old
        .open(
            Owner {
                control: Arc::new(crate::runtime::RequestInterruptState::new()),
                generation: 1,
                root: 1,
            },
            None,
            AggregateKey::Runtime {
                phase: "module.eval.total",
                source: Some(symbol),
            },
            Policy::Expr,
        )
        .unwrap();
    let boundary = Arc::new(ProfileBoundary {
        epoch: Arc::clone(&old),
        record,
        _storage: old
            .storage(std::mem::size_of::<ProfileBoundary>() + 16)
            .unwrap()
            .into(),
    });
    drop(previous);
    let current = lease::lease();
    let epoch = epoch::current().unwrap();
    let before = crate::runtime::profiling::logical_profile_snapshot();
    let rows = epoch.rows();
    let bytes = old.test_budget_bytes();
    std::thread::spawn(move || {
        assert!(lease::blocks_unregistered_thread());
        drop(boundary);
    })
    .join()
    .unwrap();
    let refunded = bytes
        .checked_sub(old.test_budget_bytes())
        .expect("owned charges unexpectedly grew in the refund window");
    let after = crate::runtime::profiling::logical_profile_snapshot();
    let counts = old.rows()[0].1;
    let same_rows = format!("{:?}", epoch.rows()) == format!("{rows:?}");
    let same_aliases = before.logical == after.logical
        && before.telemetry.get("symbol_timing_profile_counters")
            == after.telemetry.get("symbol_timing_profile_counters");
    eprintln!(
        "B713 BOUNDARY_CLEANUP refunded={refunded} old_calls={} old_open={} rows_unchanged={same_rows} aliases_unchanged={same_aliases}",
        counts.calls, counts.open
    );
    assert_eq!(
        refunded,
        epoch::RECORD_PATH_STORAGE_BYTES + std::mem::size_of::<ProfileBoundary>() + 16
    );
    assert_eq!((counts.calls, counts.abandoned, counts.open), (1, 1, 0));
    assert!(same_rows && same_aliases);
    assert_eq!(epoch.faults.load(Ordering::Relaxed), 0);
    assert_eq!(old.faults.load(Ordering::Relaxed), 0);
    epoch::reset_for_lease(&current);
}

#[test]
fn brief713_prior_boundary_cannot_settle_current_lease_rows() {
    let previous = lease::lease();
    epoch::reset_for_lease(&previous);
    let old = epoch::current().unwrap();
    let record = old
        .open(
            Owner {
                control: Arc::new(crate::runtime::RequestInterruptState::new()),
                generation: 1,
                root: 1,
            },
            None,
            AggregateKey::Runtime {
                phase: "brief713_prior_current",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    let boundary = Arc::new(ProfileBoundary {
        epoch: Arc::clone(&old),
        record,
        _storage: old
            .storage(std::mem::size_of::<ProfileBoundary>() + 16)
            .unwrap()
            .into(),
    });
    drop(previous);

    // The handoff retires the old record path before a replacement epoch is
    // constructed. The retained boundary still owes its own storage refund.
    let handoff_bytes = old.test_budget_bytes();
    let current = lease::lease();
    let handoff_refunded = handoff_bytes
        .checked_sub(old.test_budget_bytes())
        .expect("owned charges unexpectedly grew in the refund window");
    let epoch = epoch::current().unwrap();
    let same_epoch = Arc::ptr_eq(&old, &epoch);
    let before = format!("{:?}", epoch.rows());
    let bytes = old.test_budget_bytes();
    let refused = std::thread::spawn(move || {
        let refused = lease::blocks_unregistered_thread();
        drop(boundary);
        refused
    })
    .join()
    .unwrap();
    let changed = format!("{:?}", epoch.rows()) != before;
    let counts = old.rows()[0].1;
    let refunded = bytes
        .checked_sub(old.test_budget_bytes())
        .expect("owned charges unexpectedly grew in the refund window");
    let faults = epoch.faults.load(Ordering::Relaxed);
    eprintln!(
        "B713 CURRENT_DROP same_current_epoch={same_epoch} unregistered={refused} current_rows_changed={changed} old_calls={} old_open={} handoff_refunded={handoff_refunded} refunded={refunded} faults={faults}",
        counts.calls, counts.open
    );
    epoch::reset_for_lease(&current);
    assert!(refused);
    assert_eq!((counts.calls, counts.abandoned, counts.open), (1, 1, 0));
    assert_eq!(handoff_refunded, epoch::RECORD_PATH_STORAGE_BYTES);
    assert_eq!(refunded, std::mem::size_of::<ProfileBoundary>() + 16);
    assert_eq!(
        handoff_refunded + refunded,
        epoch::RECORD_PATH_STORAGE_BYTES + std::mem::size_of::<ProfileBoundary>() + 16
    );
    assert_eq!(faults, 0);
    assert!(
        !changed,
        "caller-owned Drop must not reach the current lease's rows"
    );
}
