//! Diagnostic allocations have a process ceiling independent of guest memory.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const PROCESS_LIMIT: usize = 16 * 1024 * 1024;
static PROCESS_BYTES: AtomicUsize = AtomicUsize::new(0);

pub(super) struct Budget {
    bytes: AtomicUsize,
    limit: usize,
}

pub(in crate::runtime) struct Storage {
    budget: Arc<Budget>,
    bytes: usize,
}

fn reserve(counter: &AtomicUsize, limit: usize, bytes: usize) -> bool {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
            old.checked_add(bytes).filter(|new| *new <= limit)
        })
        .is_ok()
}

impl Budget {
    pub(super) fn new(limit: usize) -> Option<Arc<Self>> {
        let bytes = std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>();
        if !reserve(&PROCESS_BYTES, PROCESS_LIMIT, bytes) {
            return None;
        }
        Some(Arc::new(Self {
            bytes: AtomicUsize::new(bytes),
            limit: limit.min(PROCESS_LIMIT),
        }))
    }

    pub(super) fn claim(self: &Arc<Self>, bytes: usize) -> Option<Storage> {
        if !reserve(&self.bytes, self.limit, bytes) {
            return None;
        }
        if !reserve(&PROCESS_BYTES, PROCESS_LIMIT, bytes) {
            self.bytes.fetch_sub(bytes, Ordering::Relaxed);
            return None;
        }
        Some(Storage {
            budget: Arc::clone(self),
            bytes,
        })
    }
}

impl Drop for Budget {
    fn drop(&mut self) {
        let own = std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>();
        debug_assert_eq!(self.bytes.load(Ordering::Relaxed), own);
        PROCESS_BYTES.fetch_sub(own, Ordering::Relaxed);
    }
}

impl Storage {
    pub(in crate::runtime) fn resize(&mut self, bytes: usize) -> bool {
        if bytes > self.bytes {
            let extra = bytes - self.bytes;
            let Some(mut claim) = self.budget.claim(extra) else {
                return false;
            };
            claim.bytes = 0;
        } else {
            let released = self.bytes - bytes;
            self.budget.bytes.fetch_sub(released, Ordering::Relaxed);
            PROCESS_BYTES.fetch_sub(released, Ordering::Relaxed);
        }
        self.bytes = bytes;
        true
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        self.budget.bytes.fetch_sub(self.bytes, Ordering::Relaxed);
        PROCESS_BYTES.fetch_sub(self.bytes, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_initial_cap_failure_stays_invalid_until_reset() {
        use crate::runtime::profiling::logical::{epoch, test_support};
        let enabled = test_support::enabled();
        let budget = Budget::new(PROCESS_LIMIT).unwrap();
        let reserved = budget
            .claim(PROCESS_LIMIT - PROCESS_BYTES.load(Ordering::Relaxed))
            .unwrap();
        assert!(epoch::current().is_none());
        drop(reserved);
        drop(budget);
        // Freeing old diagnostic owners must not silently repair lost receipts.
        assert!(epoch::current().is_none());
        epoch::reset_for_lease(enabled.lease());
        assert!(epoch::current().unwrap().is_valid());
    }
}

#[cfg(test)]
#[test]
fn logical_thread_metadata_stays_charged_until_thread_exit() {
    use crate::runtime::Session;
    use crate::runtime::profiling::logical::{
        AggregateKey, Call, Outcome, Policy, epoch, test_support,
    };
    let enabled = test_support::enabled();
    let before = PROCESS_BYTES.load(Ordering::Relaxed);
    let (report, received) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel();
    let lease = enabled.lease();
    // Scope's implicit wait fires when the worker closure returns, before
    // that thread's TLS destructors. Join the handle so the header charge
    // is released before the parent samples.
    let pinned = std::thread::scope(|scope| {
        let worker = scope.spawn(move || {
            lease.allow_current_thread();
            let session = Session::new();
            Call::start(
                &session,
                AggregateKey::Runtime {
                    phase: "thread-metadata",
                    source: None,
                },
                Policy::Expr,
            )
            .unwrap()
            .finish(Outcome::Ok);
            drop(session);
            epoch::reset_for_lease(lease);
            assert_eq!(test_support::active(), 0);
            report.send(PROCESS_BYTES.load(Ordering::Relaxed)).unwrap();
            let _ = released.recv();
        });
        let pinned = received.recv().unwrap();
        release.send(()).unwrap();
        worker.join().unwrap();
        pinned
    });
    assert!(
        pinned > before,
        "TLS metadata must stay reserved while its thread lives"
    );
    assert_eq!(PROCESS_BYTES.load(Ordering::Relaxed), before);
}

#[cfg(test)]
#[test]
fn logical_expired_queue_arena_stays_charged_until_weak_drop() {
    use crate::runtime::profiling::logical::Epoch;
    use crate::runtime::suspension::{FrameStorage, QueueArena};
    let before = PROCESS_BYTES.load(Ordering::Relaxed);
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 1024 * 1024).unwrap();
    let initial = PROCESS_BYTES.load(Ordering::Relaxed);
    let mut queue = FrameStorage::new();
    queue.enable_profile(Arc::clone(&epoch));
    drop(queue);
    let parked = PROCESS_BYTES.load(Ordering::Relaxed);
    assert_eq!(
        Arc::strong_count(&epoch),
        1,
        "the arena must not retain a strong cycle"
    );
    assert_eq!(
        parked - initial,
        std::mem::size_of::<QueueArena>() + 2 * std::mem::size_of::<usize>()
    );
    drop(epoch);
    assert_eq!(PROCESS_BYTES.load(Ordering::Relaxed), before);
}
