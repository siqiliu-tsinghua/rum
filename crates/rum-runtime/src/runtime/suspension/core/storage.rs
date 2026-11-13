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
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    pub(in crate::runtime) fn lease_generation(&self) -> Option<u64> {
        match self {
            Self::Guest(_) => None,
            Self::Profiled(queue) => queue.lease_generation(),
        }
    }

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
        self.enable_execution_context(Some(epoch));
    }
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn enable_execution_context(&mut self, epoch: Option<Arc<Epoch>>) {
        // Execution selection survives retirement. A fresh valid epoch may
        // attach new diagnostics to the existing guest continuation.
        if matches!(self, Self::Profiled(profile)
            if !epoch.as_ref().is_some_and(|epoch| epoch.is_valid())
                || profile.epoch().is_some_and(|epoch| epoch.is_valid()))
        {
            return;
        }
        let old = std::mem::replace(self, Self::new());
        let guest = match old {
            Self::Guest(guest) => guest,
            Self::Profiled(profile) => profile.into_guest(),
        };
        *self = Self::Profiled(ProfileQueue::with_control(guest, epoch));
    }
    #[cold]
    #[inline(never)]
    fn deprofile(&mut self) {
        if let Self::Profiled(profile) = self {
            profile.retire_diagnostics();
        }
    }
    pub(in crate::runtime) fn register_profile_owner(
        &self,
        control: Arc<crate::runtime::RequestInterruptState>,
    ) {
        if let Self::Profiled(queue) = self {
            profile_queue::register_owner(control, queue);
        }
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
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn profile_boundary_before_consumer(
        &mut self,
        boundary: Arc<ProfileBoundary>,
    ) {
        if let Self::Profiled(profile) = self {
            if !profile.insert_before_consumer(boundary) {
                self.deprofile();
            }
        } else {
            boundary.epoch.fault();
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
                } else {
                    p.publish_stashed_after_consumer();
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
    pub(super) fn back_context(&self) -> Option<ResumeContext> {
        match self {
            Self::Guest(queue) => queue.entries.back_context(),
            Self::Profiled(queue) => queue.guest().entries.back_context(),
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
        if matches!(self, Self::Profiled(p) if p.epoch().is_some_and(|e| !e.is_valid())) {
            self.deprofile();
        }
        if matches!(suffix, Self::Profiled(p) if p.epoch().is_some_and(|e| !e.is_valid())) {
            suffix.deprofile();
        }
        let epoch = match (&*self, &*suffix) {
            (Self::Profiled(a), Self::Profiled(b)) => {
                let first = a.epoch();
                let second = b.epoch();
                first
                    .filter(|epoch| epoch.is_valid())
                    .or_else(|| second.filter(|epoch| epoch.is_valid()))
                    .or(first)
                    .or(second)
                    .cloned()
            }
            (Self::Profiled(p), _) | (_, Self::Profiled(p)) => p.epoch().cloned(),
            (Self::Guest(_), Self::Guest(_)) => {
                self.append(suffix);
                return;
            }
        };
        self.enable_execution_context(epoch.clone());
        suffix.enable_execution_context(epoch);
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

impl FrameStorage {
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn bind_prepared_boundary(&mut self, boundary: &Arc<ProfileBoundary>) {
        if let Self::Profiled(profile) = self {
            if !profile.bind_prepared_boundary(boundary) {
                self.deprofile();
            }
        }
    }
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn complete_prepared_boundary(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
    ) {
        if let Self::Profiled(profile) = self {
            profile.complete_prepared_boundary(control);
        }
    }
}
