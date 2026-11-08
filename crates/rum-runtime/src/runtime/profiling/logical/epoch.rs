use super::index::Index;
use super::*;
use budget::{Budget, Storage};
use record::Record;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
mod snapshot;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static CURRENT: OnceLock<Mutex<Current>> = OnceLock::new();

#[derive(Default)]
struct Current {
    epoch: Option<Arc<Epoch>>,
    failed: Option<EpochId>,
}

pub(in crate::runtime) fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
fn current_slot() -> &'static Mutex<Current> {
    CURRENT.get_or_init(|| Mutex::new(Current::default()))
}

pub(in crate::runtime) struct Epoch {
    pub(in crate::runtime) id: EpochId,
    pub(in crate::runtime) valid: AtomicBool,
    pub(in crate::runtime) contexts: AtomicUsize,
    pub(in crate::runtime) faults: AtomicU64,
    pub(in crate::runtime) queue_arena: Mutex<
        Option<(
            std::sync::Weak<crate::runtime::suspension::QueueArena>,
            Storage,
        )>,
    >,
    budget: Arc<Budget>,
    data: Mutex<Data>,
    _storage: Storage,
}
struct Data {
    records: Index<RecordId, Record>,
    rows: Index<AggregateKey, Counts>,
    deferred: Index<RecordId, RecordId>,
}

impl Epoch {
    pub(in crate::runtime) fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Relaxed)
    }
    fn new_unleased(limit: usize) -> Option<Arc<Self>> {
        let budget = Budget::new(limit)?;
        let storage =
            budget.claim(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())?;
        let records_storage = budget.claim(0)?;
        let rows_storage = budget.claim(0)?;
        let deferred_storage = budget.claim(0)?;
        Some(Arc::new(Self {
            id: EpochId(next_id()),
            valid: AtomicBool::new(true),
            contexts: AtomicUsize::new(0),
            faults: AtomicU64::new(0),
            budget,
            queue_arena: Mutex::new(None),
            data: Mutex::new(Data {
                records: Index::new(records_storage),
                rows: Index::new(rows_storage),
                deferred: Index::new(deferred_storage),
            }),
            _storage: storage,
        }))
    }

    /// Tests construct an epoch only while they hold the process lease.
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    pub(in crate::runtime) fn for_test(
        lease: &crate::runtime::profiling::lease::TestLease,
        limit: usize,
    ) -> Option<Arc<Self>> {
        let _ = lease;
        Self::new_unleased(limit)
    }

    pub(in crate::runtime) fn storage(&self, bytes: usize) -> Option<Storage> {
        // An unregistered thread must not charge this epoch while a test
        // lease is current. The header lives in that thread's TLS until the
        // thread exits, which outlives the test. Production builds omit this.
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        if crate::runtime::profiling::lease::blocks_unregistered_thread() {
            return None;
        }
        if !self.valid.load(Ordering::Relaxed) {
            return None;
        }
        let claim = self.budget.claim(bytes);
        if claim.is_none() {
            self.retire(Retire::DiagnosticCap);
        }
        claim
    }

    pub(in crate::runtime) fn open(
        &self,
        owner: Owner,
        parent: Option<RecordId>,
        key: AggregateKey,
        policy: Policy,
    ) -> Option<RecordId> {
        // Refuse before taking the data lock. A refusal must not retire the
        // epoch: the caller is not this lease's thread.
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        if crate::runtime::profiling::lease::blocks_unregistered_thread() {
            return None;
        }
        let mut data = self.data.lock().unwrap();
        if !self.valid.load(Ordering::Relaxed) {
            return None;
        }
        if parent.is_some_and(|id| {
            !data
                .records
                .get(id)
                .is_some_and(|r| r.owner.same(&owner) && r.path.is_open())
        }) {
            drop(data);
            self.fault();
            return None;
        }
        if data.rows.get(key).is_none() && !data.rows.insert(key, Counts::default()) {
            drop(data);
            self.retire(Retire::DiagnosticCap);
            return None;
        }
        let id = RecordId(next_id());
        let parent_path = parent.and_then(|id| data.records.get(id).map(|r| Arc::clone(&r.path)));
        let Some(storage) = self
            .budget
            .claim(std::mem::size_of::<RecordPath>() + 2 * std::mem::size_of::<usize>())
        else {
            drop(data);
            self.retire(Retire::DiagnosticCap);
            return None;
        };
        let path = Arc::new(RecordPath {
            record: id,
            depth: parent_path.as_ref().map_or(1, |p| p.depth + 1),
            parent: crate::runtime::accumulator_chain::AccumulatorChain::new(parent_path),
            owner: owner.clone(),
            key,
            state: Mutex::new(super::module::initial_state(key)),
            _storage: storage,
        });
        let parent_path = path.parent.clone();
        if !data.records.insert(
            id,
            Record {
                owner,
                parent,
                policy,
                path,
            },
        ) {
            drop(data);
            self.retire(Retire::DiagnosticCap);
            return None;
        }
        if let Some(parent) = parent_path.as_deref() {
            parent.state.lock().unwrap().children += 1;
        }
        if let Some(counts) = data.rows.get_mut(key) {
            counts.open += 1;
        }
        Some(id)
    }

    pub(in crate::runtime) fn close(&self, id: RecordId, outcome: Result<Outcome, Retire>) -> bool {
        let mut data = self.data.lock().unwrap();
        let Some(path) = data.records.get(id).map(|r| Arc::clone(&r.path)) else {
            return false;
        };
        data.deferred.remove(id);
        let Data { records, rows, .. } = &mut *data;
        if !super::totals::terminal(&path, rows, outcome) {
            return false;
        }
        super::totals::settle(path, rows, |id| {
            records.remove(id);
        });
        true
    }

    pub(in crate::runtime) fn input_required(&self, id: RecordId) {
        let mut data = self.data.lock().unwrap();
        let Some(key) = data.records.get(id).map(|record| record.path.key) else {
            return;
        };
        if let Some(counts) = data.rows.get_mut(key) {
            counts.input_required = counts.input_required.saturating_add(1);
        }
    }

    /// Mark a record that only LP08 retention keeps open. Physical
    /// InputRequired observation is preserved for the total test accessor.
    pub(super) fn observe_silent_retention(&self, record: RecordId) {
        let Some(path) = self.path(record) else {
            return;
        };
        path.state.lock().unwrap().pause_transparent = true;
        // Release record-state before taking epoch-data in input_required.
        self.input_required(record);
    }

    pub(in crate::runtime) fn inspect(
        &self,
        id: RecordId,
    ) -> Option<(Owner, Option<RecordId>, Policy)> {
        self.data
            .lock()
            .unwrap()
            .records
            .get(id)
            .filter(|r| r.path.is_open())
            .map(|r| (r.owner.clone(), r.parent, r.policy))
    }

    pub(in crate::runtime) fn path(&self, id: RecordId) -> Option<Arc<RecordPath>> {
        self.data
            .lock()
            .unwrap()
            .records
            .get(id)
            .map(|r| Arc::clone(&r.path))
    }

    #[cfg(test)]
    pub(in crate::runtime) fn interval(&self, id: RecordId, nanos: u64, child: u64) {
        if let Some(path) = self.path(id) {
            path.interval(nanos.saturating_sub(child), false);
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) fn pause(&self, id: RecordId) {
        if let Some(path) = self.path(id) {
            path.pause();
        }
    }

    pub(in crate::runtime) fn retire(&self, reason: Retire) {
        if !self.valid.swap(false, Ordering::Relaxed) {
            return;
        }
        let mut data = self.data.lock().unwrap();
        for _ in data.deferred.remove_matching(|_| true) {}
        let Data { records, rows, .. } = &mut *data;
        for record in records.remove_matching(|_| true) {
            super::totals::terminal(&record.path, rows, Err(reason));
            super::totals::settle(record.path, rows, |_| {});
        }
        self.contexts.store(0, Ordering::Relaxed);
        // Reasons are a closed enum; printing does not retain an unbounded log.
        eprintln!(
            "rum-profile logical epoch={} valid=false reason={reason:?}",
            self.id.0
        );
    }

    pub(in crate::runtime) fn fault(&self) {
        self.faults.fetch_add(1, Ordering::Relaxed);
        self.retire(Retire::InstrumentationFault);
    }

    pub(in crate::runtime) fn abandon_owner(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
        reason: Retire,
    ) {
        let mut data = self.data.lock().unwrap();
        {
            let Data {
                records, deferred, ..
            } = &mut *data;
            for _ in deferred.remove_matching(|id| {
                records
                    .get(*id)
                    .is_some_and(|r| Arc::ptr_eq(&r.owner.control, control))
            }) {}
        }
        let Data { records, rows, .. } = &mut *data;
        for record in records.remove_matching(|r| Arc::ptr_eq(&r.owner.control, control)) {
            super::totals::terminal(&record.path, rows, Err(reason));
            super::totals::settle(record.path, rows, |_| {});
        }
    }
}

#[cfg(test)]
pub(in crate::runtime) fn test_limit(
    lease: &crate::runtime::profiling::lease::TestLease,
    limit: usize,
) {
    reset_for_lease(lease);
    current_slot().lock().unwrap().epoch = Epoch::new_unleased(limit);
}

/// The leased epoch when one already exists. Unlike `current`, this does not
/// construct an epoch: a direct definition flush has no active segment, and
/// creating one here would attach the deferred record to the wrong owner.
pub(in crate::runtime) fn existing() -> Option<Arc<Epoch>> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    if crate::runtime::profiling::lease::blocks_unregistered_thread() {
        return None;
    }
    current_slot().lock().unwrap().epoch.clone()
}

pub(in crate::runtime) fn current() -> Option<Arc<Epoch>> {
    let mut slot = current_slot().lock().unwrap();
    // Re-check under the slot lock, including when an epoch already exists.
    // An unregistered thread must neither construct nor borrow the leased
    // epoch. Production builds omit this check.
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    if crate::runtime::profiling::lease::blocks_unregistered_thread() {
        return None;
    }
    if slot.epoch.is_none() && slot.failed.is_none() {
        slot.epoch = Epoch::new_unleased(16 * 1024 * 1024);
        if slot.epoch.is_none() {
            let id = EpochId(next_id());
            slot.failed = Some(id);
            eprintln!(
                "rum-profile logical epoch={} valid=false reason=diagnostic-cap",
                id.0
            );
        }
    }
    slot.epoch.clone()
}

fn reset_unleased() {
    let mut slot = current_slot().lock().unwrap();
    slot.failed = None;
    if let Some(old) = slot.epoch.take() {
        super::collector::clear_registrations();
        old.retire(Retire::ProfileReset);
        // Records are already retired. Dropping a leftover boundary closes
        // nothing and cannot fault a missing record.
        super::collector::clear_stash();
    }
}

/// Production counter reset. Callers outside `profiling::reset` cannot mint
/// `ProductionReset`, so a test cannot reach this without the lease.
pub(in crate::runtime::profiling) fn reset_production(
    _token: super::super::reset::ProductionReset,
) {
    reset_unleased();
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
pub(in crate::runtime) fn reset_for_lease(lease: &crate::runtime::profiling::lease::TestLease) {
    let _ = lease;
    reset_unleased();
}

pub(in crate::runtime) fn drop_session(session: &crate::runtime::Session) {
    let control = Arc::clone(&session.request_interrupt_control.borrow());
    super::collector::drop_belonging(&control);
    if let Some(epoch) = current_slot().lock().unwrap().epoch.as_ref() {
        epoch.abandon_owner(&control, Retire::SessionDrop);
    }
}

pub(in crate::runtime) fn summary() {
    let slot = current_slot().lock().unwrap();
    if let Some(epoch) = slot.epoch.as_ref() {
        eprintln!(
            "rum-profile logical epoch={} valid={} faults={} context_leases={}",
            epoch.id.0,
            epoch.valid.load(Ordering::Relaxed),
            epoch.faults.load(Ordering::Relaxed),
            epoch.contexts.load(Ordering::Relaxed)
        );
    } else if let Some(id) = slot.failed {
        eprintln!(
            "rum-profile logical epoch={} valid=false reason=diagnostic-cap",
            id.0
        );
    }
}

/// After resetting under the lease, wait for retired owners (including TLS
/// destructors) to release their charges before a test takes its baseline.
/// The current lease prevents unregistered threads from adding new charges.
#[cfg(test)]
pub(in crate::runtime) fn drain_for_lease(_lease: &crate::runtime::profiling::lease::TestLease) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while budget::process_bytes() != 0 {
        super::lease_release::release_deferred_charge();
        assert!(
            std::time::Instant::now() < deadline,
            "diagnostic drain stalled at {} bytes",
            budget::process_bytes()
        );
        std::thread::yield_now();
    }
}

impl Epoch {
    pub(super) fn defer_definition(&self, record: RecordId) {
        if !self.is_valid() {
            return;
        }
        let ok = self.data.lock().unwrap().deferred.insert(record, record);
        if !ok {
            self.retire(Retire::DiagnosticCap);
        }
    }
    pub(super) fn take_deferred_definition(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
    ) -> Option<RecordId> {
        let mut data = self.data.lock().unwrap();
        let Data {
            records, deferred, ..
        } = &mut *data;
        deferred
            .remove_matching(|id| {
                records.get(*id).is_some_and(|r| {
                    Arc::ptr_eq(&r.owner.control, control)
                        && r.owner.generation == control.logical_generation()
                })
            })
            .next()
    }
}
