//! Only this module changes guest storage and its diagnostic metadata.
use super::queue::GuestQueue;
use super::*;
use crate::runtime::profiling::logical::{
    Context, ContextGuard, Epoch, ProfileBoundary, Retire, Storage,
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
    root: Option<CellId>,
    arena: Arc<QueueArena>,
    epoch: Arc<Epoch>,
    _storage: Storage,
}
struct Cell {
    kind: Kind,
    context: Option<Arc<Context>>,
    deferred: bool,
}
enum Kind {
    Guest { consumer: bool },
    Boundary(Arc<ProfileBoundary>),
}
impl Cell {
    fn consumer(&self) -> bool {
        matches!(self.kind, Kind::Guest { consumer: true })
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
    pub(super) fn new(guest: GuestQueue, epoch: Arc<Epoch>) -> Result<Box<Self>, GuestQueue> {
        let Some(storage) = epoch.storage(size_of::<Self>()) else {
            return Err(guest);
        };
        let Some(arena) = QueueArena::shared(&epoch) else {
            return Err(guest);
        };
        let mut result = Self {
            guest,
            root: None,
            arena,
            epoch,
            _storage: storage,
        };
        if result.guest.entries.len() != 0 {
            let Some(context) = Context::capture(&result.epoch) else {
                return Err(result.into_guest());
            };
            let mut data = result.arena.data.lock().unwrap();
            let mut failed = false;
            result.guest.entries.visit(|entry| {
                if failed {
                    return;
                }
                let cell = Cell {
                    kind: Kind::Guest {
                        consumer: !matches!(entry, EntryRef::EvaluationReturn),
                    },
                    context: Some(Arc::clone(&context)),
                    deferred: false,
                };
                if let Some(id) = data.allocate(cell) {
                    let tail = data.last(result.root);
                    let root = data.insert(result.root, tail, id);
                    failed = root.is_none();
                    result.root = root;
                } else {
                    failed = true;
                }
            });
            drop(data);
            if failed {
                result.epoch.retire(Retire::DiagnosticCap);
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
    pub(super) fn epoch(&self) -> &Arc<Epoch> {
        &self.epoch
    }
    #[cold]
    #[inline(never)]
    pub(super) fn is_empty(&self) -> bool {
        self.root.is_none() && self.guest.entries.is_empty()
    }
    pub(super) fn frontier(&self) -> Option<CellId> {
        self.arena.data.lock().unwrap().last(self.root)
    }
    pub(super) fn consumer_frontier(&self) -> Option<CellId> {
        self.arena.data.lock().unwrap().last_consumer(self.root)
    }

    fn add(&mut self, cell: Cell, after: Option<CellId>) -> bool {
        if !self.epoch.is_valid() {
            return false;
        }
        let mut data = self.arena.data.lock().unwrap();
        if after.is_some_and(|id| !data.belongs(self.root, id)) {
            drop(data);
            self.epoch.fault();
            return false;
        }
        let Some(id) = data.allocate(cell) else {
            drop(data);
            self.epoch.retire(Retire::DiagnosticCap);
            return false;
        };
        if let Some(root) = data.insert(self.root, after, id) {
            self.root = Some(root);
            true
        } else {
            let cell = data.release(id);
            drop(data);
            drop(cell);
            self.epoch.fault();
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
        let Some(context) = Context::capture(&self.epoch) else {
            return false;
        };
        let after = if front { None } else { self.frontier() };
        if !self.add(
            Cell {
                kind: Kind::Guest { consumer },
                context: Some(context),
                deferred: false,
            },
            after,
        ) {
            return false;
        }
        consumer || self.return_inserted()
    }
    pub(super) fn insert(
        &mut self,
        _context: ResumeContext,
        boundary: Arc<ProfileBoundary>,
        after: Option<CellId>,
        deferred: bool,
    ) -> bool {
        let Some(context) = Context::capture(&self.epoch) else {
            return false;
        };
        self.add(
            Cell {
                kind: Kind::Boundary(boundary),
                context: Some(context),
                deferred,
            },
            after,
        )
    }
    fn return_inserted(&mut self) -> bool {
        let mut data = self.arena.data.lock().unwrap();
        while let Some(id) = data.first_deferred(self.root) {
            let Some(root) = data.cut(self.root, id) else {
                drop(data);
                self.epoch.fault();
                return false;
            };
            self.root = root;
            if data.clear_deferred(id).is_none() {
                drop(data);
                self.epoch.fault();
                return false;
            }
            let after = data.last(self.root);
            if let Some(root) = data.insert(self.root, after, id) {
                self.root = Some(root);
            } else {
                drop(data);
                self.epoch.fault();
                return false;
            }
        }
        true
    }
    pub(super) fn enter_front(&self) -> Option<ContextGuard> {
        let context = {
            let data = self.arena.data.lock().unwrap();
            data.cell(data.first(self.root)?)?.context.clone()
        };
        context.and_then(|c| c.enter())
    }
    #[cold]
    #[inline(never)]
    pub(super) fn front(&self) -> Option<Queued<EntryRef<'_>>> {
        self.guest.entries.front()
    }
    fn take_cell(&mut self) -> Option<Cell> {
        let mut data = self.arena.data.lock().unwrap();
        let id = data.first(self.root)?;
        self.root = data.cut(self.root, id)?;
        data.release(id)
    }
    fn pop_metadata(&mut self) {
        if !matches!(self.take_cell().map(|c| c.kind), Some(Kind::Guest { .. })) {
            self.epoch.fault();
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
        let data = self.arena.data.lock().unwrap();
        data.first(self.root)
            .and_then(|id| data.cell(id))
            .is_some_and(|c| matches!(c.kind, Kind::Boundary(_)))
    }
    pub(super) fn observe_boundaries(&mut self, outcome: &Result<Slot, EvalError>) {
        while self.boundary_at_front() {
            if let Some(cell) = self.take_cell() {
                let _context = cell.context.as_ref().and_then(Context::enter);
                if let Kind::Boundary(boundary) = cell.kind {
                    boundary.observe(outcome);
                }
            }
        }
    }
    pub(super) fn abandon_boundaries(&mut self, reason: AbandonReason) {
        while self.boundary_at_front() {
            if let Some(cell) = self.take_cell() {
                let _context = cell.context.as_ref().and_then(Context::enter);
                if let Kind::Boundary(boundary) = cell.kind {
                    boundary.abandon(Retire::Structural(reason));
                }
            }
        }
    }
    pub(super) fn append(&mut self, suffix: &mut Self) -> bool {
        if !Arc::ptr_eq(&self.epoch, &suffix.epoch) || !Arc::ptr_eq(&self.arena, &suffix.arena) {
            self.epoch.fault();
            suffix.epoch.fault();
            return false;
        }
        if !self.epoch.is_valid() {
            return false;
        }
        let mut data = self.arena.data.lock().unwrap();
        self.root = data.join(self.root, suffix.root);
        suffix.root = None;
        self.guest.entries.append(&mut suffix.guest.entries);
        true
    }
}
impl Drop for ProfileQueue {
    fn drop(&mut self) {
        // Release each diagnostic owner after unlocking the arena. Marker drop
        // may retire records in the epoch; the registry owns no arena strongly.
        while let Some(cell) = self.take_cell() {
            drop(cell);
        }
    }
}
