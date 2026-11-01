//! The representation selects ordinary or diagnostic execution.
use super::profile_queue::{CellId, ProfileQueue};
use super::queue::GuestQueue;
use super::*;
use crate::runtime::profiling::logical::{Epoch, ProfileBoundary};

pub(in crate::runtime) enum FrameStorage {
    Guest(GuestQueue),
    Profiled(Box<ProfileQueue>),
}

impl FrameStorage {
    #[inline]
    pub(in crate::runtime) fn new() -> Self {
        Self::Guest(GuestQueue::new())
    }
    #[inline]
    pub(super) fn is_profiled(&self) -> bool {
        matches!(self, Self::Profiled(_))
    }

    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn enable_profile(&mut self, epoch: Arc<Epoch>) {
        if matches!(self, Self::Profiled(_)) {
            return;
        }
        let old = std::mem::replace(self, Self::new());
        *self = match old {
            Self::Guest(guest) => match ProfileQueue::new(guest, epoch) {
                Ok(profile) => Self::Profiled(profile),
                Err(guest) => Self::Guest(guest),
            },
            profile => profile,
        };
    }
    #[cold]
    #[inline(never)]
    fn deprofile(&mut self) {
        let old = std::mem::replace(self, Self::new());
        *self = match old {
            Self::Profiled(profile) => Self::Guest(profile.into_guest()),
            ordinary => ordinary,
        };
    }
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn frontier(&self) -> Option<CellId> {
        match self {
            Self::Profiled(p) => p.frontier(),
            Self::Guest(_) => None,
        }
    }
    #[cfg(test)]
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn consumer_frontier(&self) -> Option<CellId> {
        match self {
            Self::Profiled(p) => p.consumer_frontier(),
            Self::Guest(_) => None,
        }
    }
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn profile_boundary(
        &mut self,
        context: ResumeContext,
        boundary: Arc<ProfileBoundary>,
        after: Option<CellId>,
        deferred: bool,
    ) {
        self.enable_profile(Arc::clone(&boundary.epoch));
        if let Self::Profiled(p) = self {
            if !p.insert(context, boundary, after, deferred) {
                self.deprofile();
            }
        }
    }
    #[inline(always)]
    pub(super) fn push_back(&mut self, context: ResumeContext, frame: Arc<DynSuspensionFrame>) {
        match self {
            Self::Guest(q) => q.entries.push_back(context, frame),
            Self::Profiled(p) => {
                if !p.push(context, Entry::Legacy(frame), false) {
                    self.deprofile();
                }
            }
        }
    }
    #[inline]
    pub(super) fn push_front(&mut self, context: ResumeContext, frame: Arc<DynSuspensionFrame>) {
        match self {
            Self::Guest(q) => q.entries.push_front(context, frame),
            Self::Profiled(p) => {
                if !p.push(context, Entry::Legacy(frame), true) {
                    self.deprofile();
                }
            }
        }
    }
    #[inline]
    pub(super) fn push_back_typed(&mut self, context: ResumeContext, frame: Arc<dyn ErasedFrame>) {
        match self {
            Self::Guest(q) => q.entries.push_back_typed(context, frame),
            Self::Profiled(p) => {
                if !p.push(context, Entry::Typed(frame), false) {
                    self.deprofile();
                }
            }
        }
    }
    #[inline]
    pub(super) fn push_front_typed(&mut self, context: ResumeContext, frame: Arc<dyn ErasedFrame>) {
        match self {
            Self::Guest(q) => q.entries.push_front_typed(context, frame),
            Self::Profiled(p) => {
                if !p.push(context, Entry::Typed(frame), true) {
                    self.deprofile();
                }
            }
        }
    }
    #[inline]
    pub(in crate::runtime) fn push_evaluation_return(&mut self, context: ResumeContext) {
        match self {
            Self::Guest(q) => q.entries.push_evaluation_return(context),
            Self::Profiled(p) => {
                if !p.push(context, Entry::EvaluationReturn, false) {
                    self.deprofile();
                }
            }
        }
    }
    #[inline]
    pub(super) fn front(&self) -> Option<Queued<EntryRef<'_>>> {
        match self {
            Self::Guest(q) => q.entries.front(),
            Self::Profiled(p) => p.front(),
        }
    }
    #[inline]
    pub(super) fn pop_front(&mut self) -> Option<Queued<Entry>> {
        match self {
            Self::Guest(q) => q.entries.pop_front(),
            Self::Profiled(p) => p.pop(),
        }
    }
    #[inline]
    pub(super) fn pop_front_legacy(&mut self) -> Arc<DynSuspensionFrame> {
        match self {
            Self::Guest(q) => q.entries.pop_front_legacy(),
            Self::Profiled(p) => p.pop_legacy(),
        }
    }
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        match self {
            Self::Guest(q) => q.entries.is_empty(),
            Self::Profiled(p) => p.is_empty(),
        }
    }
    #[cfg(test)]
    pub(super) fn capacity(&self) -> usize {
        match self {
            Self::Guest(q) => q.entries.capacity(),
            Self::Profiled(p) => p.guest().entries.capacity(),
        }
    }

    pub(in crate::runtime) fn storage_bytes(&self) -> usize {
        match self {
            Self::Guest(q) => q.entries.storage_bytes(),
            Self::Profiled(p) => p.guest().entries.storage_bytes(),
        }
    }

    pub(super) fn projected_growth(&self, typed: bool) -> Option<usize> {
        match self {
            Self::Guest(q) => q.entries.projected_growth(typed),
            Self::Profiled(p) => p.guest().entries.projected_growth(typed),
        }
    }
    pub(in crate::runtime::suspension) fn visit(&self, visit: impl FnMut(EntryRef<'_>)) {
        match self {
            Self::Guest(q) => q.entries.visit(visit),
            Self::Profiled(p) => p.guest().entries.visit(visit),
        }
    }
    #[inline]
    pub(in crate::runtime) fn append(&mut self, suffix: &mut Self) {
        match (&mut *self, &mut *suffix) {
            (Self::Guest(a), Self::Guest(b)) => a.entries.append(&mut b.entries),
            _ => self.append_profiled(suffix),
        }
    }
    #[cold]
    #[inline(never)]
    fn append_profiled(&mut self, suffix: &mut Self) {
        if matches!(self, Self::Profiled(p) if !p.epoch().is_valid()) {
            self.deprofile();
        }
        if matches!(suffix, Self::Profiled(p) if !p.epoch().is_valid()) {
            suffix.deprofile();
        }
        let epoch = match (&*self, &*suffix) {
            (Self::Profiled(p), _) | (_, Self::Profiled(p)) => Arc::clone(p.epoch()),
            (Self::Guest(_), Self::Guest(_)) => {
                self.append(suffix);
                return;
            }
        };
        self.enable_profile(Arc::clone(&epoch));
        suffix.enable_profile(epoch);
        if let (Self::Profiled(a), Self::Profiled(b)) = (&mut *self, &mut *suffix) {
            if a.append(b) {
                return;
            }
        }
        self.deprofile();
        suffix.deprofile();
        self.append(suffix);
    }
    #[cfg(test)]
    pub(in crate::runtime) fn frame_strong_counts(&self) -> Vec<usize> {
        let mut counts = Vec::new();
        self.visit(|entry| match entry {
            EntryRef::Legacy(frame) => counts.push(Arc::strong_count(frame)),
            EntryRef::Typed(frame) => counts.push(Arc::strong_count(frame)),
            EntryRef::EvaluationReturn => {}
        });
        counts
    }
    #[cfg(test)]
    pub(in crate::runtime) fn entry_sizes() -> (usize, usize) {
        (size_of::<Entry>(), size_of::<EntryRef<'_>>())
    }
}
