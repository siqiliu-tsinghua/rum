use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Representative observations only. These are never memory-limit decisions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AccountingSnapshot {
    pub pushes: usize,
    pub pops: usize,
    pub live: usize,
    pub peak: usize,
    pub callbacks: usize,
    pub walker_fallbacks: usize,
    pub suspends: usize,
    pub resumes: usize,
    pub host_mutations: usize,
    pub arms: usize,
}

#[derive(Default, rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct Counter {
    pushes: AtomicUsize,
    pops: AtomicUsize,
    live: AtomicUsize,
    peak: AtomicUsize,
    pub(super) callbacks: AtomicUsize,
    pub(super) walker_fallbacks: AtomicUsize,
    pub(super) suspends: AtomicUsize,
    pub(super) resumes: AtomicUsize,
    pub(super) host_mutations: AtomicUsize,
    pub(super) arms: AtomicUsize,
}

/// Session-owned handle, also carried explicitly by adapted callbacks/frames.
#[derive(Clone, Default, rum_derive::FrameRootTrace)]
pub struct AccountingShadow(pub(in crate::runtime) Arc<Counter>);

// Atomic integer leaves cannot retain expression or symbol roots. Keep the
// containing records derived so adding a root there requires traversal.
impl FrameRootTrace for AtomicUsize {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl AccountingShadow {
    pub fn snapshot(&self) -> AccountingSnapshot {
        let c = &self.0;
        AccountingSnapshot {
            pushes: c.pushes.load(Ordering::Relaxed),
            pops: c.pops.load(Ordering::Relaxed),
            live: c.live.load(Ordering::Relaxed),
            peak: c.peak.load(Ordering::Relaxed),
            callbacks: c.callbacks.load(Ordering::Relaxed),
            walker_fallbacks: c.walker_fallbacks.load(Ordering::Relaxed),
            suspends: c.suspends.load(Ordering::Relaxed),
            resumes: c.resumes.load(Ordering::Relaxed),
            host_mutations: c.host_mutations.load(Ordering::Relaxed),
            arms: c.arms.load(Ordering::Relaxed),
        }
    }
}

pub(in crate::runtime) trait AccountingMode {
    const ARMED: bool;
    type StackGuard;
    fn stack_push(&self, pushed: bool) -> Self::StackGuard;
}

pub struct Unarmed;

impl AccountingMode for Unarmed {
    const ARMED: bool = false;
    type StackGuard = ();

    #[inline(always)]
    fn stack_push(&self, _pushed: bool) {}
}

/// Owns only the counter. No borrow of Session is held across guest code.
pub(in crate::runtime) struct StackGuard(Option<AccountingShadow>);

impl AccountingMode for AccountingShadow {
    const ARMED: bool = true;
    type StackGuard = StackGuard;

    fn stack_push(&self, pushed: bool) -> StackGuard {
        if !pushed {
            return StackGuard(None);
        }
        self.0.pushes.fetch_add(1, Ordering::Relaxed);
        let live = self.0.live.fetch_add(1, Ordering::Relaxed) + 1;
        self.0.peak.fetch_max(live, Ordering::Relaxed);
        StackGuard(Some(self.clone()))
    }
}

impl Drop for StackGuard {
    fn drop(&mut self) {
        if let Some(shadow) = &self.0 {
            shadow.0.live.fetch_sub(1, Ordering::Relaxed);
            shadow.0.pops.fetch_add(1, Ordering::Relaxed);
        }
    }
}
