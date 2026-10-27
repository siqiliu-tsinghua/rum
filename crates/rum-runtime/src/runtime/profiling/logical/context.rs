//! Queue-cell leases contain diagnostic identities and source scopes, no values.
use super::source_stack::SourceStack;
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub(in crate::runtime) struct Context {
    epoch: Arc<Epoch>,
    records: Option<Arc<RecordPath>>,
    evaluation: SourceStack<crate::runtime::Symbol>,
    rebuild: SourceStack<crate::runtime::Symbol>,
    lexical: SourceStack<&'static str>,
    sites: SourceStack<super::super::PresenceArgSite>,
    executing: AtomicBool,
    _storage: Storage,
}

pub(in crate::runtime) struct ContextGuard {
    context: Arc<Context>,
    segment: Option<segments::Segment>,
    evaluation: SourceStack<crate::runtime::Symbol>,
    rebuild: SourceStack<crate::runtime::Symbol>,
    lexical: SourceStack<&'static str>,
    sites: SourceStack<super::super::PresenceArgSite>,
}

impl Context {
    pub(in crate::runtime) fn capture(epoch: &Arc<Epoch>) -> Option<Arc<Self>> {
        let records = segments::ancestry(epoch);
        use super::super::state::*;
        // The immutable ancestry path is shared; capture never scans records.
        let storage =
            epoch.storage(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())?;
        epoch.contexts.fetch_add(1, Ordering::Relaxed);
        Some(Arc::new(Self {
            epoch: Arc::clone(epoch),
            records,
            evaluation: crate::runtime::Session::logical_capture_sources(),
            rebuild: EXPRESSION_REBUILD_SOURCE_STACK.with_borrow(Clone::clone),
            lexical: LEXICAL_CONTEXT_STACK.with_borrow(Clone::clone),
            sites: ARG_PREFILTER_SITE_STACK.with_borrow(Clone::clone),
            executing: AtomicBool::new(false),
            _storage: storage,
        }))
    }

    pub(in crate::runtime) fn enter(self: &Arc<Self>) -> Option<ContextGuard> {
        if !self.epoch.valid.load(Ordering::Relaxed) {
            return None;
        }
        if self.executing.swap(true, Ordering::Relaxed) {
            self.epoch.fault();
            return None;
        }
        let result = self.enter_active();
        if result.is_none() {
            self.executing.store(false, Ordering::Relaxed);
        }
        result
    }

    fn enter_active(self: &Arc<Self>) -> Option<ContextGuard> {
        use super::super::state::*;
        let segment = self
            .records
            .as_ref()
            .and_then(|path| segments::restore(&self.epoch, path));
        if !self.epoch.is_valid() {
            return None;
        }
        let evaluation = crate::runtime::Session::logical_swap_sources(self.evaluation.clone());
        let rebuild = EXPRESSION_REBUILD_SOURCE_STACK
            .with_borrow_mut(|s| std::mem::replace(s, self.rebuild.clone()));
        let lexical =
            LEXICAL_CONTEXT_STACK.with_borrow_mut(|s| std::mem::replace(s, self.lexical.clone()));
        let sites =
            ARG_PREFILTER_SITE_STACK.with_borrow_mut(|s| std::mem::replace(s, self.sites.clone()));
        EXPRESSION_REBUILD_SOURCE_DEPTH.with(|d| d.set(self.rebuild.len()));
        LEXICAL_CONTEXT_DEPTH.with(|d| d.set(self.lexical.len()));
        ARG_PREFILTER_SITE_DEPTH.with(|d| d.set(self.sites.len()));
        Some(ContextGuard {
            context: Arc::clone(self),
            segment,
            evaluation,
            rebuild,
            lexical,
            sites,
        })
    }
}

impl ContextGuard {
    pub(in crate::runtime) fn pause(&self) {
        if let Some(segment) = &self.segment {
            segment.pause();
        }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        use super::super::state::*;
        self.segment.take();
        crate::runtime::Session::logical_swap_sources(std::mem::take(&mut self.evaluation));
        EXPRESSION_REBUILD_SOURCE_DEPTH.with(|d| d.set(self.rebuild.len()));
        LEXICAL_CONTEXT_DEPTH.with(|d| d.set(self.lexical.len()));
        ARG_PREFILTER_SITE_DEPTH.with(|d| d.set(self.sites.len()));
        EXPRESSION_REBUILD_SOURCE_STACK.with_borrow_mut(|s| *s = std::mem::take(&mut self.rebuild));
        LEXICAL_CONTEXT_STACK.with_borrow_mut(|s| *s = std::mem::take(&mut self.lexical));
        ARG_PREFILTER_SITE_STACK.with_borrow_mut(|s| *s = std::mem::take(&mut self.sites));
        self.context.executing.store(false, Ordering::Relaxed);
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        let _ = self
            .epoch
            .contexts
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                Some(n.saturating_sub(1))
            });
    }
}
