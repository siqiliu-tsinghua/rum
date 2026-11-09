use super::*;

thread_local! {
    static REGISTRATION_PAUSE: std::cell::RefCell<Option<(
        std::sync::mpsc::Sender<bool>,
        std::sync::mpsc::Receiver<()>,
    )>> = const { std::cell::RefCell::new(None) };
    static REGISTRATION_OBSERVER: std::cell::RefCell<Option<(
        Arc<std::sync::atomic::AtomicBool>, bool,
    )>> = const { std::cell::RefCell::new(None) };
}

pub(super) fn pause_registration() {
    let pause = REGISTRATION_PAUSE.with(|slot| slot.borrow_mut().take());
    if let Some((parked, release)) = pause {
        parked.send(true).unwrap();
        release.recv().unwrap();
    }
}

pub(super) fn observe_registration() {
    REGISTRATION_OBSERVER.with_borrow_mut(|slot| {
        if let Some((acquired, crossed)) = slot {
            *crossed = acquired.load(std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[test]
fn brief713_registration_crossing_publication_is_fenced() {
    use crate::runtime::profiling::lease;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    let previous = lease::lease();
    let epoch = Epoch::for_test(&previous, 1024 * 1024).unwrap();
    let queue = ProfileQueue::new(GuestQueue::new(), epoch).ok().unwrap();
    let control = Arc::new(crate::runtime::RequestInterruptState::new());
    let (parked, parking) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let acquired = Arc::new(AtomicBool::new(false));
    drop(previous);
    let worker = std::thread::spawn({
        let acquired = Arc::clone(&acquired);
        move || {
            REGISTRATION_PAUSE.with(|slot| *slot.borrow_mut() = Some((parked.clone(), released)));
            REGISTRATION_OBSERVER.with_borrow_mut(|slot| *slot = Some((acquired, false)));
            register_owner(Arc::clone(&control), &queue);
            let crossed = REGISTRATION_OBSERVER.with_borrow_mut(|slot| slot.take().unwrap().1);
            let refused = REGISTRATION_PAUSE.with(|slot| slot.borrow_mut().take().is_some());
            if refused {
                parked.send(false).unwrap();
            }
            (crossed, refused)
        }
    });
    let paused = parking.recv().unwrap();
    let (crossed, refused, fenced) = if paused {
        let (release_holder, holder_released) = mpsc::channel();
        let (publication, published) = mpsc::channel();
        let holder = std::thread::spawn({
            let acquired = Arc::clone(&acquired);
            move || {
                let _held = lease::lease();
                acquired.store(true, Ordering::SeqCst);
                publication.send(()).unwrap();
                holder_released.recv().unwrap();
            }
        });
        while lease::fencing() == 0 && !acquired.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        // A transient fence entry is not a wait; give an unfenced publisher
        // the opportunity to acknowledge acquisition before releasing the writer.
        let _ = published.recv_timeout(std::time::Duration::from_millis(100));
        let fenced = lease::fencing() > 0;
        release.send(()).unwrap();
        let (crossed, refused) = worker.join().unwrap();
        release_holder.send(()).unwrap();
        holder.join().unwrap();
        (crossed, refused, fenced)
    } else {
        let (crossed, refused) = worker.join().unwrap();
        (crossed, refused, false)
    };
    eprintln!("B713 REGISTRATION crossed_publication={crossed} refused={refused} fenced={fenced}");
    assert!(!crossed);
    assert!(refused || fenced);
}

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
            .unwrap_or_else(|cell| {
                drop(cell);
                panic!("diagnostic fixture capacity");
            });
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
            .unwrap_or_else(|cell| {
                drop(cell);
                panic!("diagnostic fixture capacity");
            });
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
            .unwrap_or_else(|cell| {
                drop(cell);
                panic!("diagnostic fixture capacity");
            });
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
fn brief713_failed_arena_allocation_returns_boundary_before_settlement() {
    use crate::runtime::profiling::{lease, logical};
    use logical::{AggregateKey, Owner};
    let lease = lease::lease();
    let limit = 1024 * 1024;
    let epoch = Epoch::for_test(&lease, limit).unwrap();
    let mut queue = ProfileQueue::new(GuestQueue::new(), Arc::clone(&epoch))
        .ok()
        .unwrap();
    let arena = Arc::clone(queue.arena.as_ref().unwrap());
    let record = epoch
        .open(
            Owner {
                control: Arc::new(crate::runtime::RequestInterruptState::new()),
                generation: 1,
                root: 1,
            },
            None,
            AggregateKey::Runtime {
                phase: "brief713-allocation-failure",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    let boundary = Arc::new(ProfileBoundary {
        epoch: Arc::clone(&epoch),
        record,
        _storage: epoch
            .storage(size_of::<ProfileBoundary>() + 16)
            .unwrap()
            .into(),
    });
    let weak = Arc::downgrade(&boundary);
    assert_eq!(Arc::strong_count(&boundary), 1);
    let cell = Cell {
        kind: Kind::Boundary(boundary),
        context: None,
        deferred: false,
    };
    // Fill the real per-epoch budget without retiring it. A failed resize
    // refunds its attempted growth, so binary search leaves no spare byte.
    let mut ballast = epoch.storage(0).unwrap();
    let (mut low, mut high) = (0, limit);
    assert!(!ballast.resize(high));
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if ballast.resize(middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    assert!(ballast.resize(low));
    assert!(!ballast.resize(low + 1));
    assert!(epoch.is_valid());
    let before = epoch.test_budget_bytes();
    let allocation = {
        let mut data = arena.data.lock().unwrap();
        data.allocate(cell)
    };
    let cell = allocation.expect_err("the full budget must refuse arena growth");
    let Kind::Boundary(boundary) = &cell.kind else {
        panic!("allocation failure must preserve the boundary cell");
    };
    assert_eq!(Arc::strong_count(boundary), 1);
    assert_eq!(Arc::as_ptr(boundary), weak.as_ptr());
    assert!(epoch.inspect(record).is_some());
    assert_eq!(epoch.rows()[0].1.calls, 0);
    assert_eq!(epoch.rows()[0].1.open, 1);
    // Peak reservation fails before reserve/slot insertion. Its unchanged
    // charge and empty sequence witness the unchanged arena on this path.
    assert_eq!(epoch.test_budget_bytes(), before);
    assert!(
        arena
            .data
            .lock()
            .unwrap()
            .ordered_and_balanced(None)
            .is_empty()
    );
    assert!(!queue.add(cell, None, None));
    assert!(weak.upgrade().is_none());
    assert!(queue.root_id().is_none());
    assert!(
        arena
            .data
            .lock()
            .unwrap()
            .ordered_and_balanced(None)
            .is_empty()
    );
    let rows = epoch.rows();
    assert_eq!(rows.len(), 1);
    let counts = rows[0].1;
    assert_eq!((counts.calls, counts.open, counts.abandoned), (1, 0, 1));
    assert_eq!(counts.abandonment_reasons[Retire::LastDrop.index()], 1);
    assert_eq!(counts.abandonment_reasons[Retire::DiagnosticCap.index()], 0);
    assert!(!epoch.is_valid());
    assert_eq!(epoch.faults.load(std::sync::atomic::Ordering::Relaxed), 0);
    drop(queue);
    let after_drop = epoch.rows()[0].1;
    assert_eq!(
        (after_drop.calls, after_drop.open, after_drop.abandoned),
        (1, 0, 1)
    );
    eprintln!(
        "B713 ALLOCATION_FAILURE returned_sole_boundary=true added=false calls=1 open=0 abandoned=1 last_drop=1 diagnostic_cap=0 arena_charge_unchanged=true sequence_empty=true"
    );
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
            .as_ref()
            .unwrap()
            .data
            .lock()
            .unwrap()
            .ordered_and_balanced(queue.root_id());
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
            _storage: epoch
                .storage(size_of::<ProfileBoundary>() + 16)
                .unwrap()
                .into(),
        });
        assert!(queue.insert(context, boundary, after, i % 3 != 0));
        let now = queue
            .arena
            .as_ref()
            .unwrap()
            .data
            .lock()
            .unwrap()
            .ordered_and_balanced(queue.root_id());
        if i % 3 != 0 {
            deferred.push(*now.iter().find(|id| !before.contains(id)).unwrap());
        }
    }
    let before = queue
        .arena
        .as_ref()
        .unwrap()
        .data
        .lock()
        .unwrap()
        .ordered_and_balanced(queue.root_id());
    assert!(queue.push(context, Entry::EvaluationReturn, false));
    let after = queue
        .arena
        .as_ref()
        .unwrap()
        .data
        .lock()
        .unwrap()
        .ordered_and_balanced(queue.root_id());
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

thread_local! {
    static ATTACHMENT_PAUSE: std::cell::RefCell<Option<(
        std::sync::mpsc::Sender<()>,
        std::sync::mpsc::Receiver<()>,
    )>> = const { std::cell::RefCell::new(None) };
}

pub(super) fn pause_attachment_after_owner() {
    let pause = ATTACHMENT_PAUSE.with(|slot| slot.borrow_mut().take());
    if let Some((found, retired)) = pause {
        eprintln!("B713 COLLECTOR_UNWIND owner_found=true");
        found.send(()).unwrap();
        retired
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
    }
}

#[test]
fn brief713_collector_unwind_releases_owned_boundary() {
    use crate::runtime::profiling::{lease, logical};
    use logical::{AggregateKey, Owner};
    let lease = lease::lease();
    let mut foreign = logical::budget_witness::ForeignRefund::charged(&lease, 4096);
    let epoch = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    let control = Arc::new(crate::runtime::RequestInterruptState::new());
    let queue = ProfileQueue::new(GuestQueue::new(), Arc::clone(&epoch))
        .ok()
        .unwrap();
    register_owner(Arc::clone(&control), &queue);
    let record = epoch
        .open(
            Owner {
                control: Arc::clone(&control),
                generation: control.logical_generation(),
                root: 1,
            },
            None,
            AggregateKey::Runtime {
                phase: "brief713-collector-unwind",
                source: None,
            },
            Policy::Expr,
        )
        .unwrap();
    let boundary_bytes = size_of::<ProfileBoundary>() + 16;
    let boundary = Arc::new(ProfileBoundary {
        epoch: Arc::clone(&epoch),
        record,
        _storage: epoch.storage(boundary_bytes).unwrap().into(),
    });
    let weak = Arc::downgrade(&boundary);
    eprintln!(
        "B713 COLLECTOR_UNWIND references_before_stash={}",
        Arc::strong_count(&boundary)
    );
    assert_eq!(Arc::strong_count(&boundary), 1);
    logical::brief713_probe_stash(boundary);
    let (found, observed) = std::sync::mpsc::channel();
    let (retired, complete) = std::sync::mpsc::channel();
    ATTACHMENT_PAUSE.with(|slot| *slot.borrow_mut() = Some((found, complete)));
    let bytes = epoch.test_budget_bytes();
    foreign.refund_and_join();
    let (result, retired_rows) = std::thread::scope(|scope| {
        let lease = &lease;
        let epoch = &epoch;
        let worker = scope.spawn(move || {
            lease.allow_current_thread();
            observed
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            epoch.retire(Retire::ProfileReset);
            assert!(!epoch.is_valid());
            assert!(epoch.inspect(record).is_none());
            let rows = format!("{:?}", epoch.rows());
            eprintln!("B713 COLLECTOR_UNWIND real_retire_complete=true record_absent=true");
            retired.send(()).unwrap();
            rows
        });
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(logical::drain_attached));
        (result, worker.join().unwrap())
    });
    let original_assertion = result.err().is_some_and(|panic| {
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("");
        message.contains("assertion failed: matches!(") && message.contains("Policy::Expr")
    });
    let refunded = bytes
        .checked_sub(epoch.test_budget_bytes())
        .expect("owned charges unexpectedly grew in the refund window");
    let expected = logical::epoch::RECORD_PATH_STORAGE_BYTES + boundary_bytes;
    let disposed = weak.upgrade().is_none();
    let rows_unchanged = format!("{:?}", epoch.rows()) == retired_rows;
    eprintln!(
        "B713 COLLECTOR_UNWIND original_assertion_caught={original_assertion} no_nested_admission={original_assertion} disposed={disposed} refunded={refunded} expected={expected} rows_unchanged_since_retire={rows_unchanged}"
    );
    assert!(
        original_assertion,
        "the unchanged attach policy assertion must be caught"
    );
    assert!(disposed);
    assert_eq!(refunded, expected);
    assert!(
        rows_unchanged,
        "cleanup after retirement must not settle the row again"
    );
    assert_eq!(epoch.faults.load(std::sync::atomic::Ordering::Relaxed), 0);
    drop(queue);
}
