use crate::runtime::RequestInterruptState;
use crate::runtime::profiling::logical::*;
use std::sync::Arc;

fn chain() -> (Arc<Epoch>, Owner, [RecordId; 3]) {
    let epoch = Epoch::new(1024 * 1024).unwrap();
    let owner = Owner {
        control: Arc::new(RequestInterruptState::new()),
        generation: 1,
        root: 1,
    };
    let mut ids = Vec::new();
    for (phase, nanos) in [("a-parent", 1), ("b-child", 2), ("c-leaf", 4)] {
        let id = epoch
            .open(
                owner.clone(),
                ids.last().copied(),
                AggregateKey::Runtime {
                    phase,
                    source: None,
                },
                Policy::Expr,
            )
            .unwrap();
        // These are separate events while each innermost record is open.
        epoch.pause(id);
        epoch.interval(id, nanos, 0);
        ids.push(id);
    }
    (epoch, owner, ids.try_into().unwrap())
}

fn accumulated(epoch: &Epoch) {
    let rows = epoch.rows();
    assert_eq!(rows.len(), 3);
    for ((_, c), expected) in rows.into_iter().zip([(7, 1, 3), (6, 2, 2), (4, 4, 1)]) {
        assert_eq!((c.nanos, c.self_nanos, c.pauses), expected);
        assert_eq!(c.segments, 1);
    }
}

#[test]
fn logical_teardown_rejects_an_unsettled_child() {
    let (epoch, _, ids) = chain();
    assert!(epoch.close(ids[0], Ok(Outcome::Ok)));
    let failure = std::panic::catch_unwind(|| epoch.assert_quiescent()).unwrap_err();
    let message = failure.downcast_ref::<String>().unwrap();
    assert!(message.contains("logical teardown: pending children"));
    assert!(message.contains("(2, 1, 2, 0, 0)"));
    assert!(epoch.close(ids[1], Ok(Outcome::Ok)));
    assert!(epoch.close(ids[2], Ok(Outcome::Ok)));
    epoch.assert_quiescent();
}

#[test]
fn logical_late_child_totals_survive_every_terminal_order() {
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let (epoch, _, ids) = chain();
        accumulated(&epoch);
        for (closed, position) in order.into_iter().enumerate() {
            assert!(epoch.close(ids[position], Err(Retire::LastDrop)));
            assert!(!epoch.close(ids[position], Ok(Outcome::Ok)));
            accumulated(&epoch);
            assert_eq!(
                epoch.rows().iter().map(|(_, c)| c.calls).sum::<u64>(),
                closed as u64 + 1
            );
            assert_eq!(
                epoch.rows().iter().map(|(_, c)| c.open).sum::<u64>(),
                2 - closed as u64
            );
        }
        assert!(epoch.is_valid());
        assert_eq!(epoch.faults.load(std::sync::atomic::Ordering::Relaxed), 0);
        epoch.assert_quiescent();
    }
}

#[test]
fn logical_bulk_retirement_preserves_late_child_totals() {
    for retire in [false, true] {
        let (epoch, owner, _) = chain();
        if retire {
            epoch.retire(Retire::ProfileReset);
        } else {
            epoch.abandon_owner(&owner.control, Retire::SessionDrop);
        }
        accumulated(&epoch);
        for (_, counts) in epoch.rows() {
            assert_eq!((counts.calls, counts.abandoned, counts.open), (1, 1, 0));
        }
        assert_eq!(epoch.faults.load(std::sync::atomic::Ordering::Relaxed), 0);
        epoch.assert_quiescent();
    }
}

#[test]
fn logical_context_restores_only_one_record() {
    let (epoch, _, ids) = chain();
    let mut native: Vec<_> = ids
        .iter()
        .map(|id| segments::enter(&epoch, *id).unwrap())
        .collect();
    let context = Context::capture(&epoch).unwrap();
    while let Some(segment) = native.pop() {
        drop(segment);
    }
    assert_eq!(segments::active_count(), 0);
    let guard = context.enter().unwrap();
    assert_eq!(segments::active_count(), 1);
    guard.pause();
    drop(guard);
    assert_eq!(segments::active_count(), 0);
    assert_eq!(
        epoch
            .rows()
            .iter()
            .map(|(_, c)| c.pauses)
            .collect::<Vec<_>>(),
        [4, 3, 2]
    );
    for id in ids.into_iter().rev() {
        assert!(epoch.close(id, Ok(Outcome::Ok)));
    }
    drop(context);
    assert_eq!(epoch.contexts.load(std::sync::atomic::Ordering::Relaxed), 0);
    assert!(epoch.is_valid());
    epoch.assert_quiescent();
}
