use crate::runtime::RequestInterruptState;
use crate::runtime::profiling::logical::segments;
use crate::runtime::profiling::logical::*;
use std::sync::Arc;
fn owner() -> Owner {
    Owner {
        control: Arc::new(RequestInterruptState::new()),
        generation: 1,
        root: 1,
    }
}
#[test]
fn logical_terminal_transitions_are_idempotent() {
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    for outcome in [
        Outcome::Ok,
        Outcome::Some,
        Outcome::None,
        Outcome::Error,
        Outcome::Return,
        Outcome::Throw,
        Outcome::Cancelled,
    ] {
        let id = epoch
            .open(
                owner(),
                None,
                AggregateKey::Runtime {
                    phase: "terminal",
                    source: None,
                },
                Policy::Expr,
            )
            .unwrap();
        assert!(epoch.close(id, Ok(outcome)));
        assert!(!epoch.close(id, Ok(Outcome::Ok)));
    }
    let c = epoch.rows()[0].1;
    assert_eq!(
        (
            c.calls,
            c.ok,
            c.err,
            c.some,
            c.none,
            c.return_err,
            c.throw_err,
            c.cancelled,
            c.open
        ),
        (7, 3, 4, 1, 1, 1, 1, 1, 0)
    );
}
#[test]
fn logical_epoch_cap_invalidates_without_success() {
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 4096).unwrap();
    let _ = epoch
        .open(
            owner(),
            None,
            AggregateKey::Runtime {
                phase: "cap",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    assert!(epoch.storage(1024 * 1024).is_none());
    assert!(!epoch.is_valid());
    for (_, c) in epoch.rows() {
        assert_eq!(c.open, 0);
        assert_eq!(c.ok, 0);
        assert_eq!(c.calls, c.abandoned);
    }
}
#[test]
fn logical_equal_keys_are_distinct_identities() {
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    let owner = owner();
    let key = AggregateKey::Runtime {
        phase: "recursive",
        source: None,
    };
    let a = epoch.open(owner.clone(), None, key, Policy::Expr).unwrap();
    let outer = segments::enter(&epoch, a).unwrap();
    let b = epoch.open(owner, Some(a), key, Policy::Expr).unwrap();
    let inner = segments::enter(&epoch, b).unwrap();
    assert_ne!(a, b);
    outer.end();
    assert!(!epoch.is_valid());
    assert_eq!(segments::active_count(), 1);
    inner.end();
    assert_eq!(segments::active_count(), 0);
}
#[test]
fn logical_reset_does_not_pop_a_new_epoch() {
    let lease = crate::runtime::profiling::lease::lease();
    let old = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    let id = old
        .open(
            owner(),
            None,
            AggregateKey::Runtime {
                phase: "old",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    let token = segments::enter(&old, id).unwrap();
    old.retire(Retire::ProfileReset);
    let new = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    let id = new
        .open(
            owner(),
            None,
            AggregateKey::Runtime {
                phase: "new",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    let inner = segments::enter(&new, id).unwrap();
    drop(token);
    assert_eq!(segments::active_count(), 1);
    assert!(new.is_valid());
    drop(inner);
    new.close(id, Ok(Outcome::Ok));
    assert_eq!(segments::active_count(), 0);
}

#[test]
fn logical_marker_refuses_an_active_recursive_peer() {
    let lease = crate::runtime::profiling::lease::lease();
    for abandon in [false, true] {
        let epoch = Epoch::for_test(&lease, 1024 * 1024).unwrap();
        let owner = owner();
        let key = AggregateKey::Runtime {
            phase: "marker-lifo",
            source: None,
        };
        let a = epoch.open(owner.clone(), None, key, Policy::Expr).unwrap();
        let outer = segments::enter(&epoch, a).unwrap();
        let b = epoch.open(owner, Some(a), key, Policy::Expr).unwrap();
        let inner = segments::enter(&epoch, b).unwrap();
        let marker = Arc::new(ProfileBoundary {
            epoch: Arc::clone(&epoch),
            record: a,
            _storage: epoch
                .storage(std::mem::size_of::<ProfileBoundary>() + 2 * std::mem::size_of::<usize>())
                .unwrap(),
        });
        if abandon {
            marker.abandon(Retire::LastDrop);
        } else {
            marker.complete(Outcome::Ok);
        }
        assert!(!epoch.is_valid());
        assert_eq!(segments::active_count(), 1);
        let counts = epoch.rows()[0].1;
        assert_eq!(
            (counts.calls, counts.abandoned, counts.ok, counts.open),
            (2, 2, 0, 0)
        );
        assert_eq!(
            counts.abandonment_reasons[Retire::InstrumentationFault.index()],
            2
        );
        drop(inner);
        drop(outer);
        assert_eq!(segments::active_count(), 0);
    }
}

#[test]
fn logical_active_abandonment_finishes_the_interval_once() {
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    let id = epoch
        .open(
            owner(),
            None,
            AggregateKey::Runtime {
                phase: "active-abandon",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    let segment = segments::enter(&epoch, id).unwrap();
    let marker = Arc::new(ProfileBoundary {
        epoch: Arc::clone(&epoch),
        record: id,
        _storage: epoch
            .storage(std::mem::size_of::<ProfileBoundary>() + 16)
            .unwrap(),
    });
    marker.abandon(Retire::LastDrop);
    assert_eq!(segments::active_count(), 0);
    let counts = epoch.rows()[0].1;
    assert_eq!(
        (counts.calls, counts.abandoned, counts.segments, counts.open),
        (1, 1, 1, 0)
    );
    marker.abandon(Retire::LastDrop);
    drop(segment);
    assert_eq!(epoch.rows()[0].1.segments, 1);
}
