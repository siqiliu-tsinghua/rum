//! Only the innermost executing interval runs. Tokens retain stable TLS slots.
use super::totals::ActiveId;
use super::*;
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;
use std::time::Instant;

struct Active {
    epoch: Arc<Epoch>,
    path: Arc<RecordPath>,
    segment: SegmentId,
    start: Option<Instant>,
    elapsed: u64,
    paused: bool,
    restored: bool,
    previous: Option<usize>,
    next: Option<usize>,
    previous_record: Option<usize>,
    next_record: Option<usize>,
}
impl Active {
    fn park(&mut self, now: Instant) {
        if let Some(start) = self.start.take() {
            let nanos = now
                .duration_since(start)
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64;
            self.elapsed = self.elapsed.saturating_add(nanos);
        }
    }
}
enum ActiveSlot {
    Live(Active),
    Free(Option<usize>),
}
struct Stack {
    frames: Vec<ActiveSlot>,
    free: Option<usize>,
    top: Option<usize>,
    count: usize,
    storage: Option<Storage>,
    // TLS registration stays charged until this thread is destroyed.
    header: Option<Storage>,
}
thread_local! {
    static ACTIVE_STATE: Cell<usize> = const { Cell::new(0) };
    // A single InputRequired unwinds native callers and then the queue guard.
    // Only the first (innermost) observer emits its pause; callers advance this
    // cursor. The next execution entry starts a new event.
    static PAUSE_PARENT: Cell<Option<(EpochId, RecordId)>> = const { Cell::new(None) };
    static ACTIVE: RefCell<Stack> = const { RefCell::new(Stack {
        frames: Vec::new(), free: None, top: None, count: 0,
        storage: None, header: None,
    }) };
}
pub(in crate::runtime) struct Segment {
    pub(in crate::runtime) epoch: Arc<Epoch>,
    pub(in crate::runtime) record: RecordId,
    segment: SegmentId,
    index: usize,
    _native_thread: PhantomData<Rc<()>>,
}

mod stack;

pub(in crate::runtime) fn ancestry(epoch: &Arc<Epoch>) -> Option<Arc<RecordPath>> {
    if active_len() == 0 {
        return None;
    }
    ACTIVE.with_borrow(|stack| {
        let active = stack.frame(stack.top?)?;
        (active.epoch.id == epoch.id).then(|| Arc::clone(&active.path))
    })
}
pub(in crate::runtime) fn owner(
    control: &Arc<crate::runtime::RequestInterruptState>,
) -> Option<(Arc<Epoch>, Owner, RecordId)> {
    if active_len() == 0 {
        return None;
    }
    ACTIVE.with_borrow(|stack| {
        let active = stack.frame(stack.top?)?;
        let owner = &active.path.owner;
        (active.epoch.is_valid() && active.path.is_open() && Arc::ptr_eq(control, &owner.control))
            .then(|| (Arc::clone(&active.epoch), owner.clone(), active.path.record))
    })
}
pub(in crate::runtime) fn has_active() -> bool {
    active_len() != 0
}
pub(in crate::runtime) fn active_len() -> usize {
    ACTIVE_STATE.with(Cell::get).saturating_sub(1)
}
pub(in crate::runtime) fn executing_epoch() -> Option<Arc<Epoch>> {
    if active_len() == 0 {
        return None;
    }
    ACTIVE.with_borrow(|stack| Some(Arc::clone(&stack.frame(stack.top?)?.epoch)))
}
pub(in crate::runtime) fn enter(epoch: &Arc<Epoch>, record: RecordId) -> Option<Segment> {
    let path = epoch.path(record)?;
    let parent = path.parent.as_deref().map(|p| p.record);
    let matches = if active_len() == 0 {
        parent.is_none()
    } else {
        ACTIVE.with_borrow(
            |stack| match (stack.top.and_then(|i| stack.frame(i)), parent) {
                (None, None) => true,
                (Some(active), Some(parent)) => {
                    active.epoch.id == epoch.id && active.path.record == parent
                }
                (_, None) => true,
                _ => false,
            },
        )
    };
    if !matches {
        epoch.fault();
        return None;
    }
    install(epoch, path, false)
}
pub(super) fn restore(epoch: &Arc<Epoch>, path: &Arc<RecordPath>) -> Option<Segment> {
    // Queue contexts must be consumed before their captured record's boundary.
    // A stale context is a visible instrumentation fault, never silent loss.
    if !path.is_open() {
        epoch.fault();
        return None;
    }
    install(epoch, Arc::clone(path), true)
}
fn install(epoch: &Arc<Epoch>, path: Arc<RecordPath>, restored: bool) -> Option<Segment> {
    let head = path.state.lock().unwrap().active;
    if head.is_some_and(|id| id.thread != std::thread::current().id()) {
        epoch.fault();
        return None;
    }
    let header = if ACTIVE_STATE.with(Cell::get) == 0 {
        Some(epoch.storage(size_of::<RefCell<Stack>>() + 4096)?)
    } else {
        None
    };
    let record = path.record;
    let appended = ACTIVE.with_borrow_mut(|stack| {
        if let Some(header) = header {
            stack.header = Some(header);
            ACTIVE_STATE.with(|s| s.set(1));
        }
        stack.push(epoch, path, restored)
    });
    let Some((index, segment)) = appended else {
        epoch.retire(Retire::DiagnosticCap);
        return None;
    };
    Some(Segment {
        epoch: Arc::clone(epoch),
        record,
        segment,
        index,
        _native_thread: PhantomData,
    })
}
impl Segment {
    pub(in crate::runtime) fn pause(&self) {
        ACTIVE.with_borrow_mut(|stack| {
            if stack.top != Some(self.index) {
                self.epoch.fault();
                return;
            }
            let Some(active) = stack.frame_mut(self.index) else {
                return;
            };
            if active.segment != self.segment || active.paused {
                return;
            }
            active.park(Instant::now());
            active.paused = true;
            PAUSE_PARENT.with(|cursor| {
                if cursor.get() != Some((self.epoch.id, self.record)) {
                    active.path.pause();
                }
                cursor.set(
                    active
                        .path
                        .parent
                        .as_deref()
                        .map(|p| (self.epoch.id, p.record)),
                );
            });
        });
    }
    pub(in crate::runtime) fn end(&self) {
        if active_len() == 0 {
            return;
        }
        ACTIVE.with_borrow_mut(|stack| {
            let Some(active) = stack.frame(self.index) else {
                return;
            };
            if active.segment != self.segment
                || active.epoch.id != self.epoch.id
                || active.path.record != self.record
            {
                return;
            }
            let ordered = stack.top == Some(self.index);
            if !ordered {
                self.epoch.fault();
            }
            let Some(mut active) = stack.take(self.index) else {
                self.epoch.fault();
                return;
            };
            if !ordered {
                return;
            }
            let now = Instant::now();
            active.park(now);
            active.path.interval(active.elapsed, active.restored);
            if let Some(parent) = stack.top.and_then(|i| stack.frame_mut(i)) {
                if !parent.paused && parent.epoch.is_valid() {
                    parent.start = Some(now);
                }
            }
        });
    }
}
impl Drop for Segment {
    fn drop(&mut self) {
        self.end();
    }
}
pub(in crate::runtime) fn finish_active(epoch: &Arc<Epoch>, record: RecordId) {
    if active_len() == 0 {
        return;
    }
    let Some(path) = epoch.path(record) else {
        return;
    };
    let Some(id) = path.state.lock().unwrap().active else {
        return;
    };
    if id.thread != std::thread::current().id() {
        epoch.fault();
        return;
    }
    let token = Segment {
        epoch: Arc::clone(epoch),
        record,
        segment: id.segment,
        index: id.index,
        _native_thread: PhantomData,
    };
    token.end();
}
#[cfg(test)]
pub(in crate::runtime) fn active_count() -> usize {
    active_len()
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
pub(super) fn running(epoch: &Epoch) -> Vec<(RecordId, u64)> {
    if active_len() == 0 {
        return Vec::new();
    }
    let now = Instant::now();
    ACTIVE.with_borrow(|stack| {
        stack
            .frames
            .iter()
            .filter_map(|slot| {
                let ActiveSlot::Live(active) = slot else {
                    return None;
                };
                if active.epoch.id != epoch.id || !active.path.is_open() {
                    return None;
                }
                let running = active.start.map_or(0, |start| {
                    now.duration_since(start)
                        .as_nanos()
                        .min(u128::from(u64::MAX)) as u64
                });
                Some((active.path.record, active.elapsed.saturating_add(running)))
            })
            .collect()
    })
}
