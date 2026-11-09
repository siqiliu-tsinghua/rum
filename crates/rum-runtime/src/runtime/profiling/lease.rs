//! Test-only exclusive lease for the process-wide logical epoch and the lexical counters.
//! Production binaries do not compile this module. The flag-off path of
//! `lexical_profile_enabled` never loads `CURRENT_GEN`.

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

static TEST_LEASE: Mutex<()> = Mutex::new(());
static NEXT_GEN: AtomicU64 = AtomicU64::new(1);
static CURRENT_GEN: AtomicU64 = AtomicU64::new(0);
/// Read across one admitted lexical write. `lease` takes it for write after
/// publishing its generation, so a lease is current only once every write
/// admitted before it has landed.
static WRITE_GATE: RwLock<()> = RwLock::new(());
/// Leases that have published their generation and wait at `WRITE_GATE`.
static FENCING: AtomicUsize = AtomicUsize::new(0);
/// Exclusive resets waiting at `WRITE_GATE`, separate from publication fences.
static RESET_WAITING: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static REGISTERED_GEN: Cell<u64> = const { Cell::new(0) };
    static ADMITTED_DEPTH: Cell<usize> = const { Cell::new(0) };
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
    let lease = TestLease {
        _guard: guard,
        generation,
    };
    super::logical::epoch::reset_for_lease(&lease);
    lease
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

/// One admitted lexical write. Its scope never calls `admit` or `lease`. It
/// takes only locks that no thread holds while calling either: `std::env`'s (a
/// cold load), stderr's, and in the reset the logical epoch's slot, data and
/// record-path state locks.
pub(in crate::runtime) struct Admitted {
    _gate: AdmittedGate,
}

enum AdmittedGate {
    Shared {
        _gate: RwLockReadGuard<'static, ()>,
    },
    Exclusive {
        _gate: RwLockWriteGuard<'static, ()>,
    },
}

/// A global reset root. Shared admission cannot construct this proof.
pub(in crate::runtime) struct ExclusiveAdmitted {
    access: Admitted,
}

impl ExclusiveAdmitted {
    pub(in crate::runtime) fn admitted(&self) -> &Admitted {
        &self.access
    }
}

/// `None` when a lease is current and this thread did not register for it.
#[inline]
pub(in crate::runtime) fn admit() -> Option<Admitted> {
    ADMITTED_DEPTH.with(|depth| debug_assert_eq!(depth.get(), 0, "nested profiling admission"));
    let gate = WRITE_GATE
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    if blocks_unregistered_thread() {
        return None;
    }
    ADMITTED_DEPTH.with(|depth| depth.set(depth.get() + 1));
    Some(Admitted {
        _gate: AdmittedGate::Shared { _gate: gate },
    })
}

/// A session or context may carry the registration of its retained epoch.
/// The provenance is immutable caller-owned data; admission still fences every
/// access to current state before comparing the published generation.
pub(in crate::runtime) fn admit_generation(
    generation: Option<u64>,
) -> Option<(Admitted, ThreadRegistration)> {
    ADMITTED_DEPTH.with(|depth| debug_assert_eq!(depth.get(), 0, "nested profiling admission"));
    let gate = WRITE_GATE
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    let current = CURRENT_GEN.load(Ordering::Acquire);
    let registered = REGISTERED_GEN.with(Cell::get);
    if current != 0 && registered != current && generation != Some(current) {
        return None;
    }
    ADMITTED_DEPTH.with(|depth| depth.set(depth.get() + 1));
    let mut registration = ThreadRegistration::inactive();
    if registered != current {
        if let Some(generation) = generation {
            registration.previous = Some(REGISTERED_GEN.with(|cell| cell.replace(generation)));
        }
    }
    Some((
        Admitted {
            _gate: AdmittedGate::Shared { _gate: gate },
        },
        registration,
    ))
}

impl Admitted {
    /// A publication can already be waiting behind this permit. Record the
    /// generation admitted on this thread, never the waiting publication's.
    pub(in crate::runtime) fn generation(&self) -> u64 {
        REGISTERED_GEN.with(Cell::get)
    }
}

/// Restores TLS only if this scope installed a registration. The Rc marker
/// keeps restoration on the entering thread, including during unwinding.
pub(in crate::runtime) struct ThreadRegistration {
    previous: Option<u64>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl ThreadRegistration {
    pub(in crate::runtime) fn inactive() -> Self {
        Self {
            previous: None,
            _thread: std::marker::PhantomData,
        }
    }
}

impl Drop for ThreadRegistration {
    fn drop(&mut self) {
        if let Some(previous) = self.previous {
            REGISTERED_GEN.with(|cell| cell.set(previous));
        }
    }
}

pub(in crate::runtime) fn admit_exclusive() -> Option<ExclusiveAdmitted> {
    let gate = acquire_write(&RESET_WAITING)?;
    Some(ExclusiveAdmitted {
        access: Admitted {
            _gate: AdmittedGate::Exclusive { _gate: gate },
        },
    })
}

impl Drop for Admitted {
    fn drop(&mut self) {
        ADMITTED_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

/// Waits out admitted readers. The caller stores flags while this is held,
/// and acquires it before taking any payload lock.
#[cfg(test)]
pub(in crate::runtime) struct Publication {
    _gate: RwLockWriteGuard<'static, ()>,
}

#[cfg(test)]
pub(in crate::runtime) fn publish() -> Option<Publication> {
    let gate = acquire_write(&FENCING)?;
    Some(Publication { _gate: gate })
}

fn acquire_write(waiting: &AtomicUsize) -> Option<RwLockWriteGuard<'static, ()>> {
    ADMITTED_DEPTH.with(|depth| debug_assert_eq!(depth.get(), 0, "nested profiling admission"));
    waiting.fetch_add(1, Ordering::SeqCst);
    let gate = WRITE_GATE
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    waiting.fetch_sub(1, Ordering::SeqCst);
    if blocks_unregistered_thread() {
        return None;
    }
    ADMITTED_DEPTH.with(|depth| depth.set(depth.get() + 1));
    Some(gate)
}

#[cfg(test)]
impl Drop for Publication {
    fn drop(&mut self) {
        ADMITTED_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

#[cfg(test)]
#[path = "tests/brief713_depth.rs"]
mod brief713_depth;

/// Leases waiting at the fence: the fence falsifier's observable.
#[cfg(test)]
pub(in crate::runtime) fn fencing() -> usize {
    FENCING.load(Ordering::SeqCst)
}

#[cfg(test)]
pub(in crate::runtime) fn reset_waiting() -> usize {
    RESET_WAITING.load(Ordering::SeqCst)
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
        super::logical::segments::release_thread_registration();
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
