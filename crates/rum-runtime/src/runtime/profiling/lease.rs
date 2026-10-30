//! Test-only exclusive lease for the process-wide logical epoch and the lexical counters.
//! Production binaries do not compile this module. The flag-off path of
//! `lexical_profile_enabled` never loads `CURRENT_GEN`.

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};

static TEST_LEASE: Mutex<()> = Mutex::new(());
static NEXT_GEN: AtomicU64 = AtomicU64::new(1);
static CURRENT_GEN: AtomicU64 = AtomicU64::new(0);
/// Read across one admitted lexical write. `lease` takes it for write after
/// publishing its generation, so a lease is current only once every write
/// admitted before it has landed.
static WRITE_GATE: RwLock<()> = RwLock::new(());
/// Leases that have published their generation and wait at `WRITE_GATE`.
static FENCING: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static REGISTERED_GEN: Cell<u64> = const { Cell::new(0) };
}

pub(in crate::runtime) struct TestLease {
    _guard: MutexGuard<'static, ()>,
    generation: u64,
}

pub(in crate::runtime) fn lease() -> TestLease {
    let guard = TEST_LEASE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let generation = NEXT_GEN.fetch_add(1, Ordering::Relaxed);
    CURRENT_GEN.store(generation, Ordering::Release);
    fence_admitted_writes();
    REGISTERED_GEN.with(|cell| cell.set(generation));
    TestLease {
        _guard: guard,
        generation,
    }
}

/// Wait out every lexical write admitted before this generation.
fn fence_admitted_writes() {
    FENCING.fetch_add(1, Ordering::SeqCst);
    drop(
        WRITE_GATE
            .write()
            .unwrap_or_else(|poison| poison.into_inner()),
    );
    FENCING.fetch_sub(1, Ordering::SeqCst);
}

/// One admitted lexical write. Its scope holds atomics, thread-locals and
/// `eprintln!` only: never `admit`, `lease`, or another lock.
pub(in crate::runtime) struct Admitted {
    _gate: RwLockReadGuard<'static, ()>,
}

/// `None` when a lease is current and this thread did not register for it.
#[inline]
pub(in crate::runtime) fn admit() -> Option<Admitted> {
    let gate = WRITE_GATE
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    if blocks_unregistered_thread() {
        return None;
    }
    Some(Admitted { _gate: gate })
}

/// Leases waiting at the fence: the fence falsifier's observable.
#[cfg(test)]
pub(in crate::runtime) fn fencing() -> usize {
    FENCING.load(Ordering::SeqCst)
}

impl TestLease {
    /// Register the calling thread for this lease. Does not lock `TEST_LEASE`.
    #[cfg(test)]
    pub(in crate::runtime) fn allow_current_thread(&self) {
        REGISTERED_GEN.with(|cell| cell.set(self.generation));
    }
}

impl Drop for TestLease {
    fn drop(&mut self) {
        debug_assert_eq!(CURRENT_GEN.load(Ordering::Relaxed), self.generation);
        CURRENT_GEN.store(0, Ordering::Release);
        REGISTERED_GEN.with(|cell| cell.set(0));
    }
}

/// A lease is current and this thread did not register for that generation.
/// When no lease is held this is one acquire load and no mutex.
#[inline]
pub(in crate::runtime) fn blocks_unregistered_thread() -> bool {
    let generation = CURRENT_GEN.load(Ordering::Acquire);
    if generation == 0 {
        return false;
    }
    REGISTERED_GEN.with(|cell| cell.get()) != generation
}
