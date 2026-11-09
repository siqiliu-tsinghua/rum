use super::index::Index;
use super::synchronous::{EventKey, EventValue};
use super::*;
use budget::{Budget, Storage};
use record::Record;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::time::Instant;

mod snapshot;

pub(in crate::runtime) const RECORD_PATH_STORAGE_BYTES: usize =
    std::mem::size_of::<RecordPath>() + 2 * std::mem::size_of::<usize>();

#[cfg(test)]
#[path = "../tests/synchronous_families.rs"]
mod synchronous_families;

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
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    lease_generation: u64,
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
    events: Index<EventKey, EventValue>,
    event_birth: Instant,
}

impl Epoch {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    pub(in crate::runtime) fn lease_generation(&self) -> u64 {
        self.lease_generation
    }

    pub(in crate::runtime) fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Relaxed)
    }
    fn new_unleased(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
        limit: usize,
    ) -> Option<Arc<Self>> {
        let budget = Budget::new(limit)?;
        let storage =
            budget.claim(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())?;
        let records_storage = budget.claim(0)?;
        let rows_storage = budget.claim(0)?;
        let deferred_storage = budget.claim(0)?;
        let event_storage = budget.claim(0)?;
        Some(Arc::new(Self {
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            lease_generation: _access.generation(),
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
                events: Index::new(event_storage),
                event_birth: Instant::now(),
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
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return None;
        };
        let _ = lease;
        Self::new_unleased(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            limit,
        )
    }

    pub(in crate::runtime) fn storage(&self, bytes: usize) -> Option<Storage> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return None;
        };
        self.storage_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            bytes,
        )
    }

    pub(in crate::runtime) fn storage_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
        bytes: usize,
    ) -> Option<Storage> {
        // An unregistered thread must not charge this epoch while a test
        // lease is current. The header lives in that thread's TLS until the
        // thread exits, which outlives the test. Production builds omit this.
        if !self.valid.load(Ordering::Relaxed) {
            return None;
        }
        let claim = self.budget.claim(bytes);
        if claim.is_none() {
            self.retire_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
                Retire::DiagnosticCap,
            );
        }
        claim
    }

    pub(in crate::runtime) fn event(
        &self,
        key: EventKey,
        delta: EventValue,
        start: Option<Instant>,
    ) {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        self.event_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            key,
            delta,
            start,
        );
    }

    pub(in crate::runtime) fn event_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
        key: EventKey,
        delta: EventValue,
        start: Option<Instant>,
    ) {
        let mut data = self.data.lock().unwrap();
        if !self.is_valid() || start.is_some_and(|start| start < data.event_birth) {
            return;
        }
        // Equal clock readings cannot prove membership in this epoch.
        if start.is_some_and(|start| start == data.event_birth) {
            drop(data);
            self.fault_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
            );
            return;
        }
        if data.events.get(key).is_none() && !data.events.insert(key, EventValue::default()) {
            drop(data);
            self.retire_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
                Retire::DiagnosticCap,
            );
            return;
        }
        data.events.get_mut(key).unwrap().add(delta);
    }

    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    pub(in crate::runtime) fn event_rows(&self) -> Vec<(EventKey, EventValue)> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return Vec::new();
        };
        self.event_rows_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
        )
    }

    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    pub(in crate::runtime) fn event_rows_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
    ) -> Vec<(EventKey, EventValue)> {
        self.data
            .lock()
            .unwrap()
            .events
            .iter()
            .map(|(key, value)| (key, *value))
            .collect()
    }

    /// Bounded side-index copy for publication; encoding happens after unlock.
    pub(in crate::runtime) fn events(&self) -> Vec<(EventKey, EventValue)> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return Vec::new();
        };
        self.events_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
        )
    }

    pub(in crate::runtime) fn events_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
    ) -> Vec<(EventKey, EventValue)> {
        self.data
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .events
            .iter()
            .map(|(key, value)| (key, *value))
            .collect()
    }

    pub(in crate::runtime) fn open(
        &self,
        owner: Owner,
        parent: Option<RecordId>,
        key: AggregateKey,
        policy: Policy,
    ) -> Option<RecordId> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return None;
        };
        self.open_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            owner,
            parent,
            key,
            policy,
        )
    }

    pub(in crate::runtime) fn open_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,

        owner: Owner,
        parent: Option<RecordId>,
        key: AggregateKey,
        policy: Policy,
    ) -> Option<RecordId> {
        // Refuse before taking the data lock. A refusal must not retire the
        // epoch: the caller is not this lease's thread.

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
            self.fault_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
            );
            return None;
        }
        if data.rows.get(key).is_none() && !data.rows.insert(key, Counts::default()) {
            drop(data);
            self.retire_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
                Retire::DiagnosticCap,
            );
            return None;
        }
        let id = RecordId(next_id());
        let parent_path = parent.and_then(|id| data.records.get(id).map(|r| Arc::clone(&r.path)));
        let Some(storage) = self.budget.claim(RECORD_PATH_STORAGE_BYTES) else {
            drop(data);
            self.retire_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
                Retire::DiagnosticCap,
            );
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
            self.retire_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
                Retire::DiagnosticCap,
            );
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
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let access = crate::runtime::profiling::lease::admit();
        self.close_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            access.as_ref(),
            id,
            outcome,
        )
    }

    pub(in crate::runtime) fn close_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: Option<&crate::runtime::profiling::lease::Admitted>,
        id: RecordId,
        outcome: Result<Outcome, Retire>,
    ) -> bool {
        let mut data = self.data.lock().unwrap();
        let Some(path) = data.records.get(id).map(|r| Arc::clone(&r.path)) else {
            return false;
        };
        data.deferred.remove(id);
        let Data { records, rows, .. } = &mut *data;
        if !super::totals::terminal(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            &path,
            rows,
            outcome,
        ) {
            return false;
        }
        super::totals::settle(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            path,
            rows,
            |id| {
                records.remove(id);
            },
        );
        true
    }

    pub(in crate::runtime) fn input_required(&self, id: RecordId) {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        self.input_required_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            id,
        )
    }

    pub(in crate::runtime) fn input_required_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
        id: RecordId,
    ) {
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
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        self.observe_silent_retention_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            record,
        )
    }

    pub(super) fn observe_silent_retention_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
        record: RecordId,
    ) {
        let Some(path) = self.path_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            Some(_access),
            record,
        ) else {
            return;
        };
        path.state.lock().unwrap().pause_transparent = true;
        // Release record-state before taking epoch-data in input_required.
        self.input_required_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            record,
        );
    }

    pub(in crate::runtime) fn inspect(
        &self,
        id: RecordId,
    ) -> Option<(Owner, Option<RecordId>, Policy)> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return None;
        };
        self.inspect_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            id,
        )
    }

    pub(in crate::runtime) fn inspect_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,

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
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return None;
        };
        self.path_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            Some(&access),
            id,
        )
    }

    pub(in crate::runtime) fn path_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: Option<&crate::runtime::profiling::lease::Admitted>,
        id: RecordId,
    ) -> Option<Arc<RecordPath>> {
        self.data
            .lock()
            .unwrap()
            .records
            .get(id)
            .map(|r| Arc::clone(&r.path))
    }

    #[cfg(test)]
    pub(in crate::runtime) fn interval(&self, id: RecordId, nanos: u64, child: u64) {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        if let Some(path) = self.path_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            Some(&access),
            id,
        ) {
            path.interval(nanos.saturating_sub(child), false);
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) fn pause(&self, id: RecordId) {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        if let Some(path) = self.path_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            Some(&access),
            id,
        ) {
            path.pause();
        }
    }

    pub(in crate::runtime) fn retire(&self, reason: Retire) {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let access = crate::runtime::profiling::lease::admit();
        self.retire_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            access.as_ref(),
            reason,
        )
    }

    pub(in crate::runtime) fn retire_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: Option<&crate::runtime::profiling::lease::Admitted>,
        reason: Retire,
    ) {
        if !self.valid.swap(false, Ordering::Relaxed) {
            return;
        }
        let mut data = self.data.lock().unwrap();
        for _ in data.deferred.remove_matching(|_| true) {}
        let Data { records, rows, .. } = &mut *data;
        for record in records.remove_matching(|_| true) {
            super::totals::terminal(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _access,
                &record.path,
                rows,
                Err(reason),
            );
            super::totals::settle(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _access,
                record.path,
                rows,
                |_| {},
            );
        }
        self.contexts.store(0, Ordering::Relaxed);
        // Reasons are a closed enum; printing does not retain an unbounded log.
        eprintln!(
            "rum-profile logical epoch={} valid=false reason={reason:?}",
            self.id.0
        );
    }

    pub(in crate::runtime) fn fault(&self) {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let access = crate::runtime::profiling::lease::admit();
        self.fault_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            access.as_ref(),
        )
    }

    pub(in crate::runtime) fn fault_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: Option<&crate::runtime::profiling::lease::Admitted>,
    ) {
        self.faults.fetch_add(1, Ordering::Relaxed);
        self.retire_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            Retire::InstrumentationFault,
        );
    }

    pub(in crate::runtime) fn abandon_owner(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
        reason: Retire,
    ) {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let access = crate::runtime::profiling::lease::admit();
        self.abandon_owner_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            access.as_ref(),
            control,
            reason,
        )
    }

    pub(in crate::runtime) fn abandon_owner_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: Option<&crate::runtime::profiling::lease::Admitted>,

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
            super::totals::terminal(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _access,
                &record.path,
                rows,
                Err(reason),
            );
            super::totals::settle(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _access,
                record.path,
                rows,
                |_| {},
            );
        }
    }
}

#[cfg(test)]
pub(in crate::runtime) fn test_limit(
    lease: &crate::runtime::profiling::lease::TestLease,
    limit: usize,
) {
    reset_for_lease(lease);
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    current_slot().lock().unwrap().epoch = Epoch::new_unleased(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        limit,
    );
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
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return None;
    };
    existing_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

pub(in crate::runtime) fn existing_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> Option<Arc<Epoch>> {
    current_slot().lock().unwrap().epoch.clone()
}

/// Capture the current identity, including sticky initial failure, without
/// creating an epoch or borrowing a replacement during publication.
pub(in crate::runtime) fn publication_state() -> (Option<Arc<Epoch>>, Option<u64>) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return (None, None);
    };
    publication_state_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

pub(in crate::runtime) fn publication_state_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> (Option<Arc<Epoch>>, Option<u64>) {
    let slot = current_slot()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());

    (slot.epoch.clone(), slot.failed.map(|id| id.0))
}

pub(in crate::runtime) fn current() -> Option<Arc<Epoch>> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return None;
    };
    current_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

pub(in crate::runtime) fn current_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) -> Option<Arc<Epoch>> {
    let mut slot = current_slot().lock().unwrap();

    if slot.epoch.is_none() && slot.failed.is_none() {
        slot.epoch = Epoch::new_unleased(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            16 * 1024 * 1024,
        );
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

fn reset_unleased(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::ExclusiveAdmitted,
) {
    let mut slot = current_slot().lock().unwrap();
    slot.failed = None;
    super::collector::clear_registrations_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access.admitted(),
    );
    if let Some(old) = slot.epoch.take() {
        old.retire_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            Some(_access.admitted()),
            Retire::ProfileReset,
        );
    }
    // Records are already retired. Dropping a leftover boundary closes
    // nothing and cannot fault a missing record.
    super::collector::clear_stash_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access.admitted(),
    );
}

/// Production counter reset. Callers outside `profiling::reset` cannot mint
/// `ProductionReset`, so a test cannot reach this without the lease.
pub(in crate::runtime::profiling) fn reset_production(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::ExclusiveAdmitted,

    _token: super::super::reset::ProductionReset,
) {
    reset_unleased(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    );
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
pub(in crate::runtime) fn reset_for_lease(lease: &crate::runtime::profiling::lease::TestLease) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit_exclusive() else {
        return;
    };
    let _ = lease;
    reset_unleased(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
}

pub(in crate::runtime) fn drop_session(session: &crate::runtime::Session) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    drop_session_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        session,
    )
}

pub(in crate::runtime) fn drop_session_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    session: &crate::runtime::Session,
) {
    let control = Arc::clone(&session.request_interrupt_control.borrow());
    super::collector::drop_belonging_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        &control,
    );
    if let Some(epoch) = current_slot().lock().unwrap().epoch.as_ref() {
        epoch.abandon_owner_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            Some(_access),
            &control,
            Retire::SessionDrop,
        );
    }
}

pub(in crate::runtime) fn summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
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

/// Retire the prior current epoch at handoff before waiting for retained owners.
/// A prior owner may need to reacquire the lease to release its retained charge.
#[cfg(test)]
pub(in crate::runtime) fn lease_drained() -> crate::runtime::profiling::lease::TestLease {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let lease = crate::runtime::profiling::lease::lease();
        reset_for_lease(&lease);
        if budget::process_bytes() == 0 {
            return lease;
        }
        drop(lease);
        drain_for_lease(deadline);
        std::thread::yield_now();
    }
}

#[cfg(test)]
fn drain_for_lease(deadline: std::time::Instant) {
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
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        self.defer_definition_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            record,
        )
    }

    pub(super) fn defer_definition_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
        record: RecordId,
    ) {
        if !self.is_valid() {
            return;
        }
        let ok = self.data.lock().unwrap().deferred.insert(record, record);
        if !ok {
            self.retire_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(_access),
                Retire::DiagnosticCap,
            );
        }
    }
    pub(super) fn take_deferred_definition(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
    ) -> Option<RecordId> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return None;
        };
        self.take_deferred_definition_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            control,
        )
    }

    pub(super) fn take_deferred_definition_admitted(
        &self,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,

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

#[cfg(test)]
#[path = "../tests/brief713_context_drop.rs"]
mod brief713_context_drop;

#[cfg(test)]
impl Epoch {
    pub(in crate::runtime) fn test_budget_bytes(&self) -> usize {
        self.budget.test_bytes()
    }

    pub(in crate::runtime) fn test_budget_probe(&self) -> BudgetProbe {
        self.budget.test_probe()
    }
}
