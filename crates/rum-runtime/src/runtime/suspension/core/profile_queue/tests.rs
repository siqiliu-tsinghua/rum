use super::*;

#[test]
fn logical_sequence_preserves_order_join_and_stale_handle_rejection() {
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 4 * 1024 * 1024).unwrap();
    let mut arena = Arena::new(epoch.storage(0).unwrap());
    let mut root = None;
    let mut expected = Vec::new();
    let mut seed = 19u64;
    for i in 0..4096 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let position = (seed >> 32) as usize % (expected.len() + 1);
        let after = position.checked_sub(1).map(|i| expected[i]);
        let id = arena
            .allocate(Cell {
                kind: if i % 3 == 0 {
                    Kind::Guest
                } else {
                    Kind::EvaluationReturn
                },
                context: None,
                deferred: false,
            })
            .unwrap();
        root = Some(arena.insert(root, after, id).unwrap());
        expected.insert(position, id);
        if i % 137 == 0 {
            assert_eq!(arena.ordered_and_balanced(root), expected);
        }
        assert_eq!(arena.first(root), expected.first().copied());
        assert_eq!(arena.last(root), expected.last().copied());
        assert_eq!(
            arena.last_consumer(root),
            expected
                .iter()
                .rev()
                .copied()
                .find(|id| matches!(arena.cell(*id).unwrap().kind, Kind::Guest))
        );
    }
    for _ in 0..1000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let position = (seed >> 32) as usize % expected.len();
        let id = expected.remove(position);
        root = arena.cut(root, id).unwrap();
        drop(arena.release(id).unwrap());
        assert!(!arena.belongs(root, id));
        let replacement = arena
            .allocate(Cell {
                kind: Kind::Guest,
                context: None,
                deferred: false,
            })
            .unwrap();
        assert_ne!(id, replacement);
        root = Some(arena.insert(root, None, replacement).unwrap());
        expected.insert(0, replacement);
    }
    let mut suffix = None;
    for _ in 0..100 {
        let id = arena
            .allocate(Cell {
                kind: Kind::EvaluationReturn,
                context: None,
                deferred: false,
            })
            .unwrap();
        let tail = arena.last(suffix);
        suffix = Some(arena.insert(suffix, tail, id).unwrap());
        expected.push(id);
    }
    root = arena.join(root, suffix);
    assert_eq!(arena.ordered_and_balanced(root), expected);
    for id in expected {
        assert_eq!(arena.first(root), Some(id));
        root = arena.cut(root, id).unwrap();
        drop(arena.release(id).unwrap());
    }
    assert!(root.is_none());
}

#[test]
fn logical_deferred_markers_keep_sequence_order_after_a_return() {
    use crate::runtime::profiling::logical::{AggregateKey, Owner, Policy};
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 4 * 1024 * 1024).unwrap();
    let context = ResumeContext {
        stack_len: 0,
        depth: 0,
    };
    let mut guest = GuestQueue::new();
    for _ in 0..12 {
        guest.entries.push_evaluation_return(context);
    }
    let mut queue = match ProfileQueue::new(guest, Arc::clone(&epoch)) {
        Ok(queue) => queue,
        Err(_) => panic!("diagnostic fixture capacity"),
    };
    let mut deferred = Vec::new();
    for i in 0..24 {
        let before = queue
            .arena
            .data
            .lock()
            .unwrap()
            .ordered_and_balanced(queue.root);
        let after = if i % 4 == 0 {
            None
        } else {
            Some(before[(i * 7) % before.len()])
        };
        let record = epoch
            .open(
                Owner {
                    control: Arc::new(crate::runtime::RequestInterruptState::new()),
                    generation: 1,
                    root: 1,
                },
                None,
                AggregateKey::Runtime {
                    phase: "deferred-order",
                    source: None,
                },
                Policy::Expr,
            )
            .unwrap();
        let boundary = Arc::new(ProfileBoundary {
            epoch: Arc::clone(&epoch),
            record,
            _storage: epoch.storage(size_of::<ProfileBoundary>() + 16).unwrap(),
        });
        assert!(queue.insert(context, boundary, after, i % 3 != 0));
        let now = queue
            .arena
            .data
            .lock()
            .unwrap()
            .ordered_and_balanced(queue.root);
        if i % 3 != 0 {
            deferred.push(*now.iter().find(|id| !before.contains(id)).unwrap());
        }
    }
    let before = queue
        .arena
        .data
        .lock()
        .unwrap()
        .ordered_and_balanced(queue.root);
    assert!(queue.push(context, Entry::EvaluationReturn, false));
    let after = queue
        .arena
        .data
        .lock()
        .unwrap()
        .ordered_and_balanced(queue.root);
    let added = *after.iter().find(|id| !before.contains(id)).unwrap();
    let mut expected: Vec<_> = before
        .iter()
        .copied()
        .filter(|id| !deferred.contains(id))
        .collect();
    expected.push(added);
    expected.extend(before.iter().copied().filter(|id| deferred.contains(id)));
    assert_eq!(after, expected);
    drop(queue);
    assert!(epoch.rows().iter().all(|(_, counts)| counts.open == 0));
}
