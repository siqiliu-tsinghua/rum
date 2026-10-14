//! Keep legacy queue buffers unchanged; widen only an owner containing typed entries.
use super::*;
use crate::runtime::suspension::typed::{Entry, EntryRef};

pub(in crate::runtime::suspension) enum FrameQueue {
    Legacy(VecDeque<Arc<DynSuspensionFrame>>),
    Mixed(VecDeque<Entry>),
}

impl FrameQueue {
    #[inline]
    pub(super) fn new() -> Self {
        Self::Legacy(VecDeque::new())
    }

    #[cold]
    #[inline(never)]
    fn widen(&mut self) -> &mut VecDeque<Entry> {
        if let Self::Legacy(frames) = self {
            let mut mixed = VecDeque::with_capacity(frames.capacity());
            mixed.extend(frames.drain(..).map(Entry::Legacy));
            *self = Self::Mixed(mixed);
        }
        let Self::Mixed(frames) = self else {
            unreachable!()
        };
        frames
    }

    #[inline]
    pub(super) fn push_back(&mut self, frame: Arc<DynSuspensionFrame>) {
        match self {
            Self::Legacy(frames) => frames.push_back(frame),
            Self::Mixed(frames) => frames.push_back(Entry::Legacy(frame)),
        }
    }

    #[inline]
    pub(super) fn push_front(&mut self, frame: Arc<DynSuspensionFrame>) {
        match self {
            Self::Legacy(frames) => frames.push_front(frame),
            Self::Mixed(frames) => frames.push_front(Entry::Legacy(frame)),
        }
    }

    pub(super) fn push_back_typed(&mut self, frame: Arc<dyn ErasedFrame>) {
        self.widen().push_back(Entry::Typed(frame));
    }

    pub(super) fn push_front_typed(&mut self, frame: Arc<dyn ErasedFrame>) {
        self.widen().push_front(Entry::Typed(frame));
    }

    #[inline]
    pub(super) fn front(&self) -> Option<EntryRef<'_>> {
        match self {
            Self::Legacy(frames) => frames.front().map(EntryRef::Legacy),
            Self::Mixed(frames) => frames.front().map(Entry::as_ref),
        }
    }

    #[inline]
    pub(super) fn pop_front(&mut self) -> Option<Entry> {
        match self {
            Self::Legacy(frames) => frames.pop_front().map(Entry::Legacy),
            Self::Mixed(frames) => frames.pop_front(),
        }
    }

    #[inline]
    pub(super) fn pop_front_legacy(&mut self) -> Arc<DynSuspensionFrame> {
        match self {
            Self::Legacy(frames) => frames
                .pop_front()
                .expect("the resumed suspension frame should remain queued"),
            Self::Mixed(frames) => Self::pop_front_mixed_legacy(frames),
        }
    }

    #[cold]
    #[inline(never)]
    fn pop_front_mixed_legacy(frames: &mut VecDeque<Entry>) -> Arc<DynSuspensionFrame> {
        match frames
            .pop_front()
            .expect("the resumed suspension frame should remain queued")
        {
            Entry::Legacy(frame) => frame,
            Entry::Typed(_) => unreachable!("legacy dispatch must pop its observed front"),
        }
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        match self {
            Self::Legacy(frames) => frames.is_empty(),
            Self::Mixed(frames) => frames.is_empty(),
        }
    }

    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        match self {
            Self::Legacy(frames) => frames.capacity(),
            Self::Mixed(frames) => frames.capacity(),
        }
    }

    pub(super) fn storage_bytes(&self) -> usize {
        match self {
            Self::Legacy(frames) => frames
                .capacity()
                .saturating_mul(size_of::<Arc<DynSuspensionFrame>>()),
            Self::Mixed(frames) => frames.capacity().saturating_mul(size_of::<Entry>()),
        }
    }

    pub(super) fn projected_growth(&self, typed: bool) -> Option<usize> {
        let (len, capacity, old_size) = match self {
            Self::Legacy(frames) => (
                frames.len(),
                frames.capacity(),
                size_of::<Arc<DynSuspensionFrame>>(),
            ),
            Self::Mixed(frames) => (frames.len(), frames.capacity(), size_of::<Entry>()),
        };
        let new_size = if typed { size_of::<Entry>() } else { old_size };
        let next_capacity = if len == capacity {
            capacity.checked_mul(2)?.max(8)
        } else {
            capacity
        };
        next_capacity
            .checked_mul(new_size)?
            .checked_sub(capacity.checked_mul(old_size)?)
    }

    pub(in crate::runtime::suspension) fn visit(&self, mut visit: impl FnMut(EntryRef<'_>)) {
        match self {
            Self::Legacy(frames) => {
                for frame in frames {
                    visit(EntryRef::Legacy(frame));
                }
            }
            Self::Mixed(frames) => {
                for frame in frames {
                    visit(frame.as_ref());
                }
            }
        }
    }

    #[inline]
    pub(super) fn append(&mut self, suffix: &mut Self) {
        if let (Self::Legacy(frames), Self::Legacy(rest)) = (&mut *self, &mut *suffix) {
            frames.append(rest);
            return;
        }
        self.append_mixed(suffix);
    }

    #[cold]
    #[inline(never)]
    fn append_mixed(&mut self, suffix: &mut Self) {
        let frames = self.widen();
        match suffix {
            Self::Legacy(rest) => frames.extend(rest.drain(..).map(Entry::Legacy)),
            Self::Mixed(rest) => frames.append(rest),
        }
    }
}
