//! Only this module changes guest storage and its diagnostic metadata.
use super::queue::GuestQueue;
use super::*;
use crate::runtime::profiling::logical::{
    Context, ContextGuard, Epoch, Policy, ProfileBoundary, Retire, Storage,
};
use std::sync::Mutex;

mod arena;
#[cfg(test)]
mod tests;
use arena::Arena;
pub(in crate::runtime) use arena::CellId;

pub(in crate::runtime) struct QueueArena {
    data: Mutex<Arena>,
    // The registry holds only a Weak. Its charge owner must outlive this data.
    _epoch: Arc<Epoch>,
}

pub(in crate::runtime) struct ProfileQueue {
    guest: GuestQueue,
    // Exact identity returned by the most recent production Guest allocation.
    // Only the cold pause-after-push path takes it; never an arena search.
    inserted_consumer: Option<CellId>,
    // Shared with the epoch collector so a session-less insert updates the
    // same AVL root the driver reads.
    root: Arc<Mutex<Option<CellId>>>,
    arena: Option<Arc<QueueArena>>,
    epoch: Option<Arc<Epoch>>,
    _storage: Option<Storage>,
}
struct Cell {
    kind: Kind,
    context: Option<Arc<Context>>,
    deferred: bool,
}
enum Kind {
    Guest,
    EvaluationReturn,
    Boundary(Arc<ProfileBoundary>),
    DownValuesBoundary(Box<DownValuesView>),
}

// The existing diagnostic queue owns this binding. Neither the guest frame nor
// the adapter carries it; its complete allocation is charged before creation.
struct DownValuesView {
    boundary: Arc<ProfileBoundary>,
    consumer: CellId,
    _storage: Storage,
}

impl DownValuesView {
    fn new(boundary: Arc<ProfileBoundary>, consumer: CellId) -> Option<Box<Self>> {
        let storage = boundary.epoch.storage(size_of::<Self>())?;
        Some(Box::new(Self {
            boundary,
            consumer,
            _storage: storage,
        }))
    }

    fn observe(&self, queue: &ProfileQueue, outcome: &Result<Slot, EvalError>) {
        let boundary = &self.boundary;
        // A retired epoch or a closed record leaves a stale boundary; the
        // generic boundary path ignores it. Only a live boundary with the
        // wrong policy, or a displaced consumer in the same live epoch, is an
        // instrumentation fault.
        if !boundary.epoch.is_valid() {
            return;
        }
        let Some(policy) = boundary.policy() else {
            return;
        };
        if !matches!(policy, Policy::OptionalExpr) {
            boundary.epoch.fault();
            return;
        }
        let Some(epoch) = queue.epoch.as_ref() else {
            return;
        };
        if !epoch.is_valid() || !Arc::ptr_eq(epoch, &boundary.epoch) {
            return;
        }
        let ordered = queue.arena.as_ref().is_some_and(|arena| {
            let root = *queue.root.lock().unwrap();
            let data = arena.data.lock().unwrap();
            data.first(root) == Some(self.consumer)
                && data.belongs(root, self.consumer)
                && data
                    .cell(self.consumer)
                    .is_some_and(|cell| matches!(cell.kind, Kind::Guest))
        });
        if !ordered {
            boundary.epoch.fault();
            return;
        }
        if let Ok(Slot::Value(expr)) = outcome {
            let decoded = <Option<Expr> as ContinuationValue>::from_continuation_expr(expr.clone());
            let view = Ok(Slot::State(ContinuationState::OptionalExpr(decoded)));
            boundary.observe(&view);
        } else {
            boundary.observe(outcome);
        }
    }
}

impl QueueArena {
    fn shared(epoch: &Arc<Epoch>) -> Option<Arc<Self>> {
        let mut slot = epoch.queue_arena.lock().unwrap();
        if let Some(arena) = slot.as_ref().and_then(|(weak, _)| weak.upgrade()) {
            return Some(arena);
        }
        // Release the expired weak allocation before releasing its charge.
        slot.take();
        let storage = epoch.storage(size_of::<Self>() + 2 * size_of::<usize>())?;
        let buffer = epoch.storage(0)?;
        let arena = Arc::new(Self {
            data: Mutex::new(Arena::new(buffer)),
            _epoch: Arc::clone(epoch),
        });
        *slot = Some((Arc::downgrade(&arena), storage));
        Some(arena)
    }
}
impl ProfileQueue {
    pub(super) fn with_control(guest: GuestQueue, epoch: Option<Arc<Epoch>>) -> Box<Self> {
        let guest = match &epoch {
            Some(epoch) => match Self::new(guest, Arc::clone(epoch)) {
                Ok(queue) => return queue,
                Err(guest) => guest,
            },
            None => guest,
        };
        Box::new(Self {
            guest,
            inserted_consumer: None,
            root: Arc::new(Mutex::new(None)),
            arena: None,
            epoch,
            _storage: None,
        })
    }
    pub(super) fn retire_diagnostics(&mut self) {
        while let Some(cell) = self.take_cell() {
            drop(cell);
        }
        self.inserted_consumer = None;
        self.arena = None;
        self._storage = None;
    }

    pub(super) fn new(guest: GuestQueue, epoch: Arc<Epoch>) -> Result<Box<Self>, GuestQueue> {
        let Some(storage) = epoch.storage(size_of::<Self>()) else {
            return Err(guest);
        };
        let Some(arena) = QueueArena::shared(&epoch) else {
            return Err(guest);
        };
        let result = Self {
            guest,
            inserted_consumer: None,
            root: Arc::new(Mutex::new(None)),
            arena: Some(arena),
            epoch: Some(epoch),
            _storage: Some(storage),
        };
        if result.guest.entries.len() != 0 {
            let Some(context) = Context::capture(result.epoch.as_ref().unwrap()) else {
                return Err(result.into_guest());
            };
            let mut root = result.root.lock().unwrap();
            let mut data = result.arena.as_ref().unwrap().data.lock().unwrap();
            let mut failed = false;
            result.guest.entries.visit(|entry| {
                if failed {
                    return;
                }
                let cell = Cell {
                    kind: if matches!(entry, EntryRef::EvaluationReturn) {
                        Kind::EvaluationReturn
                    } else {
                        Kind::Guest
                    },
                    context: Some(Arc::clone(&context)),
                    deferred: false,
                };
                if let Some(id) = data.allocate(cell) {
                    let tail = data.last(*root);
                    let next = data.insert(*root, tail, id);
                    failed = next.is_none();
                    *root = next;
                } else {
                    failed = true;
                }
            });
            drop(data);
            drop(root);
            if failed {
                result.epoch.as_ref().unwrap().retire(Retire::DiagnosticCap);
                return Err(result.into_guest());
            }
        }
        Ok(Box::new(result))
    }
    pub(super) fn guest(&self) -> &GuestQueue {
        &self.guest
    }
    pub(super) fn into_guest(mut self) -> GuestQueue {
        std::mem::replace(&mut self.guest, GuestQueue::new())
    }
    pub(super) fn epoch(&self) -> Option<&Arc<Epoch>> {
        self.epoch.as_ref()
    }
    pub(super) fn root_id(&self) -> Option<CellId> {
        *self.root.lock().unwrap()
    }
    #[cold]
    #[inline(never)]
    pub(super) fn is_empty(&self) -> bool {
        self.root_id().is_none() && self.guest.entries.is_empty()
    }
    pub(super) fn frontier(&self) -> Option<CellId> {
        let arena = self.arena.as_ref()?;
        let root = *self.root.lock().unwrap();
        arena.data.lock().unwrap().last(root)
    }
    #[cfg(test)]
    pub(super) fn consumer_frontier(&self) -> Option<CellId> {
        let arena = self.arena.as_ref()?;
        let root = *self.root.lock().unwrap();
        arena.data.lock().unwrap().last_consumer(root)
    }

    fn add(&mut self, cell: Cell, after: Option<CellId>, before: Option<CellId>) -> bool {
        if !self.epoch.as_ref().is_some_and(|e| e.is_valid()) {
            return false;
        }
        let mut root = self.root.lock().unwrap();
        let mut data = self.arena.as_ref().unwrap().data.lock().unwrap();
        if after.is_some_and(|id| !data.belongs(*root, id))
            || before.is_some_and(|id| {
                !data.belongs(*root, id)
                    || !data
                        .cell(id)
                        .is_some_and(|cell| matches!(cell.kind, Kind::Guest))
            })
        {
            drop(data);
            drop(root);
            self.epoch.as_ref().unwrap().fault();
            return false;
        }
        let consumer = matches!(cell.kind, Kind::Guest);
        let Some(id) = data.allocate(cell) else {
            drop(data);
            drop(root);
            self.epoch.as_ref().unwrap().retire(Retire::DiagnosticCap);
            return false;
        };
        let next = match before {
            Some(consumer) => data.insert_before(*root, consumer, id),
            None => data.insert(*root, after, id),
        };
        if let Some(next) = next {
            *root = Some(next);
            if consumer {
                self.inserted_consumer = Some(id);
            }
            true
        } else {
            let cell = data.release(id);
            drop(data);
            drop(root);
            drop(cell);
            self.epoch.as_ref().unwrap().fault();
            false
        }
    }
    #[cold]
    #[inline(never)]
    pub(super) fn push(&mut self, context: ResumeContext, entry: Entry, front: bool) -> bool {
        let consumer = !matches!(entry, Entry::EvaluationReturn);
        match entry {
            Entry::Legacy(frame) if front => self.guest.entries.push_front(context, frame),
            Entry::Legacy(frame) => self.guest.entries.push_back(context, frame),
            Entry::Typed(frame) if front => self.guest.entries.push_front_typed(context, frame),
            Entry::Typed(frame) => self.guest.entries.push_back_typed(context, frame),
            Entry::EvaluationReturn => self.guest.entries.push_evaluation_return(context),
        }
        if self.arena.is_none() {
            return true;
        }
        let Some(context) = Context::capture(self.epoch.as_ref().unwrap()) else {
            return false;
        };
        let after = if front { None } else { self.frontier() };
        if !self.add(
            Cell {
                kind: if consumer {
                    Kind::Guest
                } else {
                    Kind::EvaluationReturn
                },
                context: Some(context),
                deferred: false,
            },
            after,
            None,
        ) {
            return false;
        }
        consumer || self.return_inserted()
    }
    // Only the existing Profiled typed-push arm calls this after success.
    // The registry identifies this queue; foreign controls stay stashed.
    #[cold]
    #[inline(never)]
    pub(super) fn publish_stashed_after_consumer(&self) {
        let control = {
            let owners = owners().lock().unwrap();
            owners
                .iter()
                .find(|owner| {
                    Arc::ptr_eq(&owner.root, &self.root)
                        && self.epoch.as_ref().is_some_and(|epoch| {
                            Arc::ptr_eq(epoch, &owner.epoch) && epoch.is_valid()
                        })
                })
                .map(|owner| Arc::clone(&owner.control))
        };
        if let Some(control) = control {
            crate::runtime::profiling::logical::drain_attached_for(&control);
        }
    }

    pub(super) fn insert(
        &mut self,
        _context: ResumeContext,
        boundary: Arc<ProfileBoundary>,
        after: Option<CellId>,
        deferred: bool,
    ) -> bool {
        if self.arena.is_none() {
            return false;
        }
        let Some(context) = Context::capture(self.epoch.as_ref().unwrap()) else {
            return false;
        };
        self.add(
            Cell {
                kind: Kind::Boundary(boundary),
                context: Some(context),
                deferred,
            },
            after,
            None,
        )
    }
    pub(super) fn insert_before_consumer(&mut self, boundary: Arc<ProfileBoundary>) -> bool {
        let Some(epoch) = self.epoch.as_ref() else {
            boundary.epoch.fault();
            return false;
        };
        if !Arc::ptr_eq(epoch, &boundary.epoch) {
            boundary.epoch.fault();
            return false;
        }
        let Some(consumer) = self.inserted_consumer.take() else {
            boundary.epoch.fault();
            return false;
        };
        let Some(context) = Context::capture(epoch) else {
            return false;
        };
        let Some(view) = DownValuesView::new(boundary, consumer) else {
            return false;
        };
        self.add(
            Cell {
                kind: Kind::DownValuesBoundary(view),
                context: Some(context),
                deferred: false,
            },
            None,
            Some(consumer),
        )
    }
    fn return_inserted(&mut self) -> bool {
        let mut root = self.root.lock().unwrap();
        let mut data = self.arena.as_ref().unwrap().data.lock().unwrap();
        while let Some(id) = data.first_deferred(*root) {
            let Some(next) = data.cut(*root, id) else {
                drop(data);
                drop(root);
                self.epoch.as_ref().unwrap().fault();
                return false;
            };
            *root = next;
            if data.clear_deferred(id).is_none() {
                drop(data);
                drop(root);
                self.epoch.as_ref().unwrap().fault();
                return false;
            }
            let after = data.last(*root);
            if let Some(next) = data.insert(*root, after, id) {
                *root = Some(next);
            } else {
                drop(data);
                drop(root);
                self.epoch.as_ref().unwrap().fault();
                return false;
            }
        }
        true
    }
    pub(super) fn enter_front_with_control(
        &self,
        control: Arc<crate::runtime::RequestInterruptState>,
    ) -> Option<ContextGuard> {
        if self.arena.is_none() {
            return Some(ContextGuard::enter_control(control));
        }
        let context = {
            let root = *self.root.lock().unwrap();
            let data = self.arena.as_ref().unwrap().data.lock().unwrap();
            data.cell(data.first(root)?)?.context.clone()
        };
        context
            .and_then(|c| c.enter_with_control(Some(Arc::clone(&control))))
            .or_else(|| Some(ContextGuard::enter_control(control)))
    }
    pub(super) fn enter_front(&self) -> Option<ContextGuard> {
        self.arena.as_ref()?;
        let context = {
            let root = *self.root.lock().unwrap();
            let data = self.arena.as_ref().unwrap().data.lock().unwrap();
            data.cell(data.first(root)?)?.context.clone()
        };
        context.and_then(|c| c.enter())
    }
    #[cold]
    #[inline(never)]
    pub(super) fn front(&self) -> Option<Queued<EntryRef<'_>>> {
        self.guest.entries.front()
    }
    fn take_cell(&mut self) -> Option<Cell> {
        let arena = self.arena.as_ref()?;
        let mut root = self.root.lock().unwrap();
        let mut data = arena.data.lock().unwrap();
        let id = data.first(*root)?;
        *root = data.cut(*root, id)?;
        if self.inserted_consumer == Some(id) {
            self.inserted_consumer = None;
        }
        data.release(id)
    }
    fn pop_metadata(&mut self) {
        if self.arena.is_none() {
            return;
        }
        if !matches!(
            self.take_cell().map(|c| c.kind),
            Some(Kind::Guest | Kind::EvaluationReturn)
        ) {
            self.epoch.as_ref().unwrap().fault();
        }
    }
    #[cold]
    #[inline(never)]
    pub(super) fn pop(&mut self) -> Option<Queued<Entry>> {
        let guest = self.guest.entries.pop_front()?;
        self.pop_metadata();
        Some(guest)
    }
    #[cold]
    #[inline(never)]
    pub(super) fn pop_legacy(&mut self) -> Arc<DynSuspensionFrame> {
        self.pop_metadata();
        self.guest.entries.pop_front_legacy()
    }
    fn boundary_at_front(&self) -> bool {
        if self.arena.is_none() {
            return false;
        }
        let root = *self.root.lock().unwrap();
        let data = self.arena.as_ref().unwrap().data.lock().unwrap();
        data.first(root)
            .and_then(|id| data.cell(id))
            .is_some_and(|c| matches!(c.kind, Kind::Boundary(_) | Kind::DownValuesBoundary(_)))
    }
    pub(super) fn observe_boundaries(&mut self, outcome: &Result<Slot, EvalError>) {
        while self.boundary_at_front() {
            if let Some(cell) = self.take_cell() {
                let _context = cell.context.as_ref().and_then(Context::enter);
                match cell.kind {
                    Kind::Boundary(boundary) => boundary.observe(outcome),
                    Kind::DownValuesBoundary(view) => view.observe(self, outcome),
                    _ => continue,
                }
            }
        }
    }
    pub(super) fn abandon_boundaries(&mut self, reason: AbandonReason) {
        while self.boundary_at_front() {
            if let Some(cell) = self.take_cell() {
                let _context = cell.context.as_ref().and_then(Context::enter);
                match cell.kind {
                    Kind::Boundary(boundary) => boundary.abandon(Retire::Structural(reason)),
                    Kind::DownValuesBoundary(view) => {
                        view.boundary.abandon(Retire::Structural(reason))
                    }
                    _ => continue,
                }
            }
        }
    }
    pub(super) fn append(&mut self, suffix: &mut Self) -> bool {
        // All pause-after-push insertions finish before the driver joins queues.
        // Unclaimed insertion handles must not be inherited by a later owner.
        self.inserted_consumer = None;
        suffix.inserted_consumer = None;
        if self.arena.is_none() || suffix.arena.is_none() {
            self.retire_diagnostics();
            suffix.retire_diagnostics();
            self.guest.entries.append(&mut suffix.guest.entries);
            return true;
        }

        if !self
            .epoch
            .as_ref()
            .zip(suffix.epoch.as_ref())
            .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
            || !self
                .arena
                .as_ref()
                .zip(suffix.arena.as_ref())
                .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
        {
            self.epoch.as_ref().unwrap().fault();
            suffix.epoch.as_ref().unwrap().fault();
            return false;
        }
        if !self.epoch.as_ref().is_some_and(|e| e.is_valid()) {
            return false;
        }
        let mut own = self.root.lock().unwrap();
        let mut other = suffix.root.lock().unwrap();
        let mut data = self.arena.as_ref().unwrap().data.lock().unwrap();
        *own = data.join(*own, *other);
        *other = None;
        drop(data);
        drop(other);
        drop(own);
        self.guest.entries.append(&mut suffix.guest.entries);
        true
    }
}
impl Drop for ProfileQueue {
    fn drop(&mut self) {
        unregister_root(&self.root);
        // Release each diagnostic owner after unlocking the arena. Marker drop
        // may retire records in the epoch; the registry owns no arena strongly.
        while let Some(cell) = self.take_cell() {
            drop(cell);
        }
    }
}

struct OwnerReg {
    control: Arc<crate::runtime::RequestInterruptState>,
    root: Arc<Mutex<Option<CellId>>>,
    arena: Arc<QueueArena>,
    epoch: Arc<Epoch>,
}

fn owners() -> &'static Mutex<Vec<OwnerReg>> {
    static OWNERS: std::sync::OnceLock<Mutex<Vec<OwnerReg>>> = std::sync::OnceLock::new();
    OWNERS.get_or_init(|| Mutex::new(Vec::new()))
}

pub(in crate::runtime) fn register_owner(
    control: Arc<crate::runtime::RequestInterruptState>,
    queue: &ProfileQueue,
) {
    let (Some(arena), Some(epoch)) = (queue.arena.clone(), queue.epoch.clone()) else {
        return;
    };
    crate::runtime::profiling::logical::set_hooks(
        attach_boundary,
        clear_owners,
        unregister_control,
    );
    let mut owners = owners().lock().unwrap();
    owners.retain(|owner| !Arc::ptr_eq(&owner.control, &control));
    owners.push(OwnerReg {
        control,
        root: Arc::clone(&queue.root),
        arena,
        epoch,
    });
}

fn clear_owners() {
    owners().lock().unwrap().clear();
}

fn unregister_control(control: &Arc<crate::runtime::RequestInterruptState>) {
    owners()
        .lock()
        .unwrap()
        .retain(|owner| !Arc::ptr_eq(&owner.control, control));
}

fn unregister_root(root: &Arc<Mutex<Option<CellId>>>) {
    owners()
        .lock()
        .unwrap()
        .retain(|owner| !Arc::ptr_eq(&owner.root, root));
}

fn attach_boundary(boundary: &Arc<ProfileBoundary>) -> bool {
    debug_assert!(matches!(
        boundary.policy(),
        Some(
            Policy::Expr
                | Policy::OptionalExpr
                | Policy::Bool
                | Policy::Prepared
                | Policy::Match
                | Policy::ApplicableExpr
                | Policy::Incoming(_)
        )
    ));
    let found = {
        let owners = owners().lock().unwrap();
        owners
            .iter()
            .find(|owner| {
                Arc::ptr_eq(&owner.epoch, &boundary.epoch) && boundary.belongs_to(&owner.control)
            })
            .map(|owner| {
                (
                    Arc::clone(&owner.root),
                    Arc::clone(&owner.arena),
                    Arc::clone(&owner.epoch),
                )
            })
    };
    let Some((root, arena, epoch)) = found else {
        return false;
    };
    if !epoch.is_valid() {
        // The retired epoch must not stay alive through a stashed boundary.
        return true;
    }
    let Some(context) = Context::capture(&epoch) else {
        return false;
    };
    let mut root = root.lock().unwrap();
    let mut data = arena.data.lock().unwrap();
    let after = data.last(*root);
    if after.is_some_and(|id| !data.belongs(*root, id)) {
        drop(data);
        drop(root);
        epoch.fault();
        return true;
    }
    let cell = Cell {
        kind: Kind::Boundary(Arc::clone(boundary)),
        context: Some(context),
        deferred: false,
    };
    let Some(id) = data.allocate(cell) else {
        drop(data);
        drop(root);
        epoch.retire(Retire::DiagnosticCap);
        return true;
    };
    if let Some(next) = data.insert(*root, after, id) {
        *root = Some(next);
        true
    } else {
        let cell = data.release(id);
        drop(data);
        drop(root);
        drop(cell);
        epoch.fault();
        true
    }
}
