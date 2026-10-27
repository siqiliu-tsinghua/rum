//! Stable TLS slots, links, and geometrically charged storage.
use super::*;

impl Stack {
    pub(super) fn frame(&self, index: usize) -> Option<&Active> {
        match self.frames.get(index)? {
            ActiveSlot::Live(active) => Some(active),
            ActiveSlot::Free(_) => None,
        }
    }
    pub(super) fn frame_mut(&mut self, index: usize) -> Option<&mut Active> {
        match self.frames.get_mut(index)? {
            ActiveSlot::Live(active) => Some(active),
            ActiveSlot::Free(_) => None,
        }
    }
    fn reserve(&mut self, epoch: &Arc<Epoch>) -> Option<()> {
        if self.free.is_some() || self.frames.len() < self.frames.capacity() {
            return Some(());
        }
        let old = self.frames.capacity();
        let capacity = old.checked_mul(2)?.max(4);
        let peak = old
            .checked_add(capacity)?
            .checked_mul(size_of::<ActiveSlot>())?;
        if let Some(storage) = &mut self.storage {
            if !storage.resize(peak) {
                return None;
            }
        } else {
            self.storage = Some(epoch.storage(peak)?);
        }
        if self
            .frames
            .try_reserve_exact(capacity - self.frames.len())
            .is_err()
        {
            if let Some(storage) = &mut self.storage {
                storage.resize(old * size_of::<ActiveSlot>());
            }
            return None;
        }
        if !self
            .storage
            .as_mut()?
            .resize(self.frames.capacity() * size_of::<ActiveSlot>())
        {
            return None;
        }
        Some(())
    }
    pub(super) fn push(
        &mut self,
        epoch: &Arc<Epoch>,
        path: Arc<RecordPath>,
        restored: bool,
    ) -> Option<(usize, SegmentId)> {
        self.reserve(epoch)?;
        let index = self.free.unwrap_or(self.frames.len());
        let segment = SegmentId(epoch::next_id());
        let previous_record = {
            let mut state = path.state.lock().unwrap();
            let previous = state.active.map(|id| id.index);
            state.active = Some(ActiveId {
                thread: std::thread::current().id(),
                index,
                segment,
            });
            previous
        };
        let now = Instant::now();
        if let Some(top) = self.top {
            let parent = self.frame_mut(top)?;
            parent.park(now);
            parent.next = Some(index);
        }
        let active = Active {
            epoch: Arc::clone(epoch),
            path,
            segment,
            start: Some(now),
            elapsed: 0,
            paused: false,
            restored,
            previous: self.top,
            next: None,
            previous_record,
            next_record: None,
        };
        if let Some(previous) = previous_record {
            self.frame_mut(previous)?.next_record = Some(index);
        }
        if let Some(free) = self.free {
            let Some(ActiveSlot::Free(next)) = self.frames.get(free) else {
                return None;
            };
            self.free = *next;
            self.frames[index] = ActiveSlot::Live(active);
        } else {
            self.frames.push(ActiveSlot::Live(active));
        }
        self.top = Some(index);
        self.count += 1;
        ACTIVE_STATE.with(|state| state.set(self.count + 1));
        PAUSE_PARENT.with(|p| p.set(None));
        Some((index, segment))
    }
    pub(super) fn take(&mut self, index: usize) -> Option<Active> {
        let active = self.frame(index)?;
        let (previous, next, previous_record, next_record) = (
            active.previous,
            active.next,
            active.previous_record,
            active.next_record,
        );
        if let Some(previous) = previous {
            self.frame_mut(previous)?.next = next;
        }
        if let Some(next) = next {
            self.frame_mut(next)?.previous = previous;
        } else {
            self.top = previous;
        }
        if let Some(previous) = previous_record {
            self.frame_mut(previous)?.next_record = next_record;
        }
        if let Some(next) = next_record {
            self.frame_mut(next)?.previous_record = previous_record;
        } else {
            let head = previous_record.and_then(|index| {
                self.frame(index).map(|a| ActiveId {
                    thread: std::thread::current().id(),
                    index,
                    segment: a.segment,
                })
            });
            self.frame(index)?.path.state.lock().unwrap().active = head;
        }
        let slot = self.frames.get_mut(index)?;
        let active = match std::mem::replace(slot, ActiveSlot::Free(self.free)) {
            ActiveSlot::Live(active) => active,
            free => {
                *slot = free;
                return None;
            }
        };
        self.free = Some(index);
        self.count -= 1;
        ACTIVE_STATE.with(|state| state.set(self.count + 1));
        if self.count == 0 {
            self.frames = Vec::new();
            self.free = None;
            self.storage = None;
        }
        Some(active)
    }
}
