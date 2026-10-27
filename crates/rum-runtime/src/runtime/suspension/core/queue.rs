//! The guest queue holds only semantic continuation entries.
use super::*;

use crate::runtime::suspension::typed::{Entry, EntryRef};

// The public diagnostic representation keeps the semantic entry type private.
pub(in crate::runtime) struct GuestQueue {
    pub(super) entries: FrameQueue,
}
impl GuestQueue {
    #[inline(always)]
    pub(super) fn new() -> Self {
        Self {
            entries: FrameQueue::new(),
        }
    }
}

pub(in crate::runtime::suspension) enum FrameQueue {
    Legacy(VecDeque<Queued<Arc<DynSuspensionFrame>>>),
    Mixed(VecDeque<Queued<Entry>>),
}

impl FrameQueue {
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Legacy(f) => f.len(),
            Self::Mixed(f) => f.len(),
        }
    }

    #[inline(always)]
    pub(super) fn new() -> Self {
        Self::Legacy(VecDeque::new())
    }

    #[cold]
    #[inline(never)]
    fn widen(&mut self) -> &mut VecDeque<Queued<Entry>> {
        if let Self::Legacy(frames) = self {
            let mut mixed = VecDeque::with_capacity(frames.capacity());
            mixed.extend(frames.drain(..).map(|cell| cell.map(Entry::Legacy)));
            *self = Self::Mixed(mixed);
        }
        let Self::Mixed(frames) = self else {
            unreachable!()
        };
        frames
    }

    #[inline(always)]
    pub(super) fn push_back(&mut self, context: ResumeContext, frame: Arc<DynSuspensionFrame>) {
        match self {
            Self::Legacy(frames) => frames.push_back(Queued {
                context,
                payload: frame,
            }),
            Self::Mixed(frames) => frames.push_back(Queued {
                context,
                payload: Entry::Legacy(frame),
            }),
        }
    }

    #[inline]
    pub(super) fn push_front(&mut self, context: ResumeContext, frame: Arc<DynSuspensionFrame>) {
        match self {
            Self::Legacy(frames) => frames.push_front(Queued {
                context,
                payload: frame,
            }),
            Self::Mixed(frames) => frames.push_front(Queued {
                context,
                payload: Entry::Legacy(frame),
            }),
        }
    }

    #[inline(always)]
    pub(super) fn push_back_typed(&mut self, context: ResumeContext, frame: Arc<dyn ErasedFrame>) {
        let frames = match self {
            Self::Mixed(frames) => frames,
            Self::Legacy(_) => self.widen(),
        };
        frames.push_back(Queued {
            context,
            payload: Entry::Typed(frame),
        });
    }

    pub(super) fn push_front_typed(&mut self, context: ResumeContext, frame: Arc<dyn ErasedFrame>) {
        self.widen().push_front(Queued {
            context,
            payload: Entry::Typed(frame),
        });
    }

    pub(super) fn push_evaluation_return(&mut self, context: ResumeContext) {
        self.widen().push_back(Queued {
            context,
            payload: Entry::EvaluationReturn,
        });
    }

    #[inline]
    pub(super) fn front(&self) -> Option<Queued<EntryRef<'_>>> {
        match self {
            Self::Legacy(frames) => frames.front().map(|cell| Queued {
                context: cell.context,
                payload: EntryRef::Legacy(&cell.payload),
            }),
            Self::Mixed(frames) => frames.front().map(|cell| Queued {
                context: cell.context,
                payload: cell.payload.as_ref(),
            }),
        }
    }

    #[inline]
    pub(super) fn pop_front(&mut self) -> Option<Queued<Entry>> {
        match self {
            Self::Legacy(frames) => frames.pop_front().map(|cell| cell.map(Entry::Legacy)),
            Self::Mixed(frames) => frames.pop_front(),
        }
    }

    #[inline]
    pub(super) fn pop_front_legacy(&mut self) -> Arc<DynSuspensionFrame> {
        match self {
            Self::Legacy(frames) => {
                frames
                    .pop_front()
                    .expect("the resumed suspension frame should remain queued")
                    .payload
            }
            Self::Mixed(frames) => Self::pop_front_mixed_legacy(frames),
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn pop_front_mixed_legacy(
        frames: &mut VecDeque<Queued<Entry>>,
    ) -> Arc<DynSuspensionFrame> {
        match frames
            .pop_front()
            .expect("the resumed suspension frame should remain queued")
            .payload
        {
            Entry::Legacy(frame) => frame,
            Entry::Typed(_) | Entry::EvaluationReturn => {
                unreachable!("legacy dispatch must pop its observed front")
            }
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
                .saturating_mul(size_of::<Queued<Arc<DynSuspensionFrame>>>()),
            Self::Mixed(frames) => frames.capacity().saturating_mul(size_of::<Queued<Entry>>()),
        }
    }

    pub(super) fn projected_growth(&self, typed: bool) -> Option<usize> {
        let (len, capacity, old_size) = match self {
            Self::Legacy(frames) => (
                frames.len(),
                frames.capacity(),
                size_of::<Queued<Arc<DynSuspensionFrame>>>(),
            ),
            Self::Mixed(frames) => (frames.len(), frames.capacity(), size_of::<Queued<Entry>>()),
        };
        let new_size = if typed {
            size_of::<Queued<Entry>>()
        } else {
            old_size
        };
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
                    visit(EntryRef::Legacy(&frame.payload));
                }
            }
            Self::Mixed(frames) => {
                for frame in frames {
                    visit(frame.payload.as_ref());
                }
            }
        }
    }

    #[inline(always)]
    pub(super) fn append(&mut self, suffix: &mut Self) {
        match (&mut *self, &mut *suffix) {
            (Self::Legacy(frames), Self::Legacy(rest)) => frames.append(rest),
            (Self::Mixed(frames), Self::Mixed(rest)) => frames.append(rest),
            _ => self.append_mixed(suffix),
        }
    }

    #[cold]
    #[inline(never)]
    fn append_mixed(&mut self, suffix: &mut Self) {
        let frames = self.widen();
        match suffix {
            Self::Legacy(rest) => frames.extend(rest.drain(..).map(|cell| cell.map(Entry::Legacy))),
            Self::Mixed(rest) => frames.append(rest),
        }
    }
}
