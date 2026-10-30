//! C4′ concurrency contract for the one process-wide epoch (brief-633).
//! Rows are keyed by `AggregateKey`. Two sessions that record different keys
//! do not cross. Two sessions that record one key add. `Session` drop
//! abandons only that session's owner.

use super::super::logical::{AggregateKey, Call, Counts, Outcome, Policy, Retire, epoch};
use crate::runtime::Session;
use std::sync::{Arc, Barrier};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stable {
    calls: u64,
    ok: u64,
    err: u64,
    some: u64,
    none: u64,
    return_err: u64,
    throw_err: u64,
    cancelled: u64,
    abandoned: u64,
    open: u64,
    pauses: u64,
    segments: u64,
    reasons: [u64; 11],
}

fn stable(counts: &Counts) -> Stable {
    Stable {
        calls: counts.calls,
        ok: counts.ok,
        err: counts.err,
        some: counts.some,
        none: counts.none,
        return_err: counts.return_err,
        throw_err: counts.throw_err,
        cancelled: counts.cancelled,
        abandoned: counts.abandoned,
        open: counts.open,
        pauses: counts.pauses,
        segments: counts.segments,
        reasons: counts.abandonment_reasons,
    }
}

fn add(left: Stable, right: Stable) -> Stable {
    Stable {
        calls: left.calls + right.calls,
        ok: left.ok + right.ok,
        err: left.err + right.err,
        some: left.some + right.some,
        none: left.none + right.none,
        return_err: left.return_err + right.return_err,
        throw_err: left.throw_err + right.throw_err,
        cancelled: left.cancelled + right.cancelled,
        abandoned: left.abandoned + right.abandoned,
        open: left.open + right.open,
        pauses: left.pauses + right.pauses,
        segments: left.segments + right.segments,
        reasons: std::array::from_fn(|index| left.reasons[index] + right.reasons[index]),
    }
}

fn published() -> Vec<(AggregateKey, Stable)> {
    epoch::current()
        .map(|epoch| {
            epoch
                .rows()
                .into_iter()
                .map(|(key, counts)| (key, stable(&counts)))
                .collect()
        })
        .unwrap_or_default()
}

fn row(phase: &'static str) -> Stable {
    let matched: Vec<Stable> = published()
        .into_iter()
        .filter_map(|(key, counts)| match key {
            AggregateKey::Runtime {
                phase: found,
                source: None,
            } if found == phase => Some(counts),
            _ => None,
        })
        .collect();
    assert_eq!(
        matched.len(),
        1,
        "phase {phase} should be one row, published {:?}",
        published()
    );
    matched[0]
}

fn join_registered(
    enabled: &super::engine_support::Enabled,
    left: impl FnOnce() + Send,
    right: impl FnOnce() + Send,
) {
    std::thread::scope(|scope| {
        scope.spawn(|| {
            enabled.lease().allow_current_thread();
            left();
        });
        scope.spawn(|| {
            enabled.lease().allow_current_thread();
            right();
        });
    });
}

fn open(session: &Session, phase: &'static str) -> Call {
    Call::start(
        session,
        AggregateKey::Runtime {
            phase,
            source: None,
        },
        Policy::Expr,
    )
    .expect("span opens")
}

/// Different keys, both calls open together. Each row matches its solo row.
#[test]
fn no_cross_attribution() {
    let solo_a = {
        let _enabled = super::engine_support::enabled();
        let session = Session::new();
        open(&session, "owner-a").finish(Outcome::Ok);
        row("owner-a")
    };
    let solo_b = {
        let _enabled = super::engine_support::enabled();
        let session = Session::new();
        open(&session, "owner-b").finish(Outcome::Ok);
        row("owner-b")
    };

    let enabled = super::engine_support::enabled();
    let gate = Arc::new(Barrier::new(2));
    let left_gate = Arc::clone(&gate);
    join_registered(
        &enabled,
        move || {
            let session = Session::new();
            let call = open(&session, "owner-a");
            left_gate.wait();
            call.finish(Outcome::Ok);
        },
        move || {
            let session = Session::new();
            let call = open(&session, "owner-b");
            gate.wait();
            call.finish(Outcome::Ok);
        },
    );

    let found = published();
    assert_eq!(found.len(), 2, "no other row changes, published {found:?}");
    assert_eq!(row("owner-a"), solo_a, "owner-a crossed or dropped");
    assert_eq!(row("owner-b"), solo_b, "owner-b crossed or dropped");
}

/// One key, unequal sides, both calls open together. The row is the sum.
#[test]
fn same_key_additivity() {
    let solo_ok = {
        let _enabled = super::engine_support::enabled();
        let session = Session::new();
        open(&session, "owner-sum").finish(Outcome::Ok);
        row("owner-sum")
    };
    let solo_err = {
        let _enabled = super::engine_support::enabled();
        let session = Session::new();
        open(&session, "owner-sum").finish(Outcome::Error);
        row("owner-sum")
    };
    assert_ne!(solo_ok, solo_err, "the two sides must differ");

    let enabled = super::engine_support::enabled();
    let gate = Arc::new(Barrier::new(2));
    let left_gate = Arc::clone(&gate);
    join_registered(
        &enabled,
        move || {
            let session = Session::new();
            let call = open(&session, "owner-sum");
            left_gate.wait();
            call.finish(Outcome::Ok);
        },
        move || {
            let session = Session::new();
            let call = open(&session, "owner-sum");
            gate.wait();
            call.finish(Outcome::Error);
        },
    );

    let found = published();
    assert_eq!(found.len(), 1, "one key, published {found:?}");
    assert_eq!(
        row("owner-sum"),
        add(solo_ok, solo_err),
        "concurrent row is not the field-by-field sum"
    );
}

/// Session drop reaches `abandon_owner` for that owner only.
/// Path: `session_state.rs` `Drop for Session` ->
/// `profiling/summary.rs` `print_lexical_profile_summary_once` ->
/// `logical/epoch.rs` `drop_session` -> `Epoch::abandon_owner` with
/// `Retire::SessionDrop`.
#[test]
fn owner_scoped_abandonment() {
    let solo_abandoned = {
        let _enabled = super::engine_support::enabled();
        let session = Session::new();
        let call = open(&session, "owner-a");
        drop(session);
        drop(call);
        let stable = row("owner-a");
        assert_eq!(stable.reasons[Retire::SessionDrop.index()], 1);
        assert_eq!(stable.abandoned, 1);
        assert_eq!(stable.ok, 0);
        stable
    };
    let solo_ok = {
        let _enabled = super::engine_support::enabled();
        let session = Session::new();
        open(&session, "owner-b").finish(Outcome::Ok);
        let stable = row("owner-b");
        assert_eq!(stable.ok, 1);
        assert_eq!(stable.abandoned, 0);
        stable
    };

    let enabled = super::engine_support::enabled();
    let opened = Arc::new(Barrier::new(2));
    let abandoned = Arc::new(Barrier::new(2));
    let left_opened = Arc::clone(&opened);
    let left_abandoned = Arc::clone(&abandoned);
    join_registered(
        &enabled,
        move || {
            let session = Session::new();
            let call = open(&session, "owner-a");
            left_opened.wait();
            drop(session);
            left_abandoned.wait();
            drop(call);
        },
        move || {
            let session = Session::new();
            let call = open(&session, "owner-b");
            opened.wait();
            abandoned.wait();
            call.finish(Outcome::Ok);
        },
    );

    let found = published();
    assert_eq!(
        found.len(),
        2,
        "abandonment touched another key, published {found:?}"
    );
    assert_eq!(
        row("owner-a"),
        solo_abandoned,
        "A lost its abandonment reason"
    );
    assert_eq!(row("owner-b"), solo_ok, "B was abandoned with A");
    Session::logical_profile_test_assert_quiescent();
}
