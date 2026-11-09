//! Queue-cell leases contain diagnostic identities and source scopes, no values.
use super::source_stack::SourceStack;
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub(in crate::runtime) struct Context {
    epoch: Option<Arc<Epoch>>,
    records: Option<Arc<RecordPath>>,
    evaluation: SourceStack<crate::runtime::Symbol>,
    rebuild: SourceStack<crate::runtime::Symbol>,
    lexical: SourceStack<&'static str>,
    sites: SourceStack<super::super::PresenceArgSite>,
    executing: AtomicBool,
    _storage: Option<Storage>,
}

pub(in crate::runtime) struct ContextGuard {
    context: Arc<Context>,
    segment: Option<segments::Segment>,
    evaluation: SourceStack<crate::runtime::Symbol>,
    rebuild: SourceStack<crate::runtime::Symbol>,
    lexical: SourceStack<&'static str>,
    sites: SourceStack<super::super::PresenceArgSite>,
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _registration: crate::runtime::profiling::lease::ThreadRegistration,
}

impl Context {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    fn lease_generation(&self) -> Option<u64> {
        self.epoch.as_ref().map(|epoch| epoch.lease_generation())
    }

    pub(in crate::runtime) fn capture(epoch: &Arc<Epoch>) -> Option<Arc<Self>> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(access) = crate::runtime::profiling::lease::admit() else {
            return None;
        };
        Self::capture_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            &access,
            epoch,
        )
    }

    pub(in crate::runtime) fn capture_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: &crate::runtime::profiling::lease::Admitted,
        epoch: &Arc<Epoch>,
    ) -> Option<Arc<Self>> {
        let records = segments::ancestry_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            epoch,
        );
        use super::super::state::*;
        // The immutable ancestry path is shared; capture never scans records.
        let storage = epoch.storage_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>(),
        )?;
        epoch.contexts.fetch_add(1, Ordering::Relaxed);
        Some(Arc::new(Self {
            epoch: Some(Arc::clone(epoch)),
            records,
            evaluation: crate::runtime::Session::logical_capture_sources(),
            rebuild: EXPRESSION_REBUILD_SOURCE_STACK.with_borrow(Clone::clone),
            lexical: LEXICAL_CONTEXT_STACK.with_borrow(Clone::clone),
            sites: ARG_PREFILTER_SITE_STACK.with_borrow(Clone::clone),
            executing: AtomicBool::new(false),
            _storage: Some(storage),
        }))
    }

    pub(in crate::runtime) fn enter(self: &Arc<Self>) -> Option<ContextGuard> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some((access, _registration)) =
            crate::runtime::profiling::lease::admit_generation(self.lease_generation())
        else {
            return None;
        };
        self.enter_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            access,
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _registration,
        )
    }

    pub(in crate::runtime) fn enter_admitted(
        self: &Arc<Self>,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: crate::runtime::profiling::lease::Admitted,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _registration: crate::runtime::profiling::lease::ThreadRegistration,
    ) -> Option<ContextGuard> {
        self.enter_with_control_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _registration,
            execution_control(),
        )
    }

    pub(in crate::runtime) fn enter_with_control(
        self: &Arc<Self>,
        control: Option<Arc<crate::runtime::RequestInterruptState>>,
    ) -> Option<ContextGuard> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some((access, _registration)) =
            crate::runtime::profiling::lease::admit_generation(self.lease_generation())
        else {
            return None;
        };
        self.enter_with_control_admitted(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            access,
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _registration,
            control,
        )
    }

    pub(in crate::runtime) fn enter_with_control_admitted(
        self: &Arc<Self>,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: crate::runtime::profiling::lease::Admitted,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _registration: crate::runtime::profiling::lease::ThreadRegistration,

        control: Option<Arc<crate::runtime::RequestInterruptState>>,
    ) -> Option<ContextGuard> {
        let epoch = self.epoch.as_ref()?;
        if !epoch.valid.load(Ordering::Relaxed) && control.is_none() {
            return None;
        }
        if self.executing.swap(true, Ordering::Relaxed) {
            epoch.fault_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                Some(&_access),
            );
            return None;
        }
        let result = self.enter_active(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _registration,
            control,
        );
        if result.is_none() {
            self.executing.store(false, Ordering::Relaxed);
        }
        result
    }

    fn enter_active(
        self: &Arc<Self>,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access: crate::runtime::profiling::lease::Admitted,
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _registration: crate::runtime::profiling::lease::ThreadRegistration,

        control: Option<Arc<crate::runtime::RequestInterruptState>>,
    ) -> Option<ContextGuard> {
        use super::super::state::*;
        let epoch = self.epoch.as_ref()?;
        #[cfg(not(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        )))]
        let segment = self
            .records
            .as_ref()
            .and_then(|path| segments::restore(epoch, path));
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let segment = self.records.as_ref().and_then(|path| {
            segments::restore_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                &_access,
                epoch,
                path,
            )
        });
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        drop(_access);
        if !epoch.is_valid() {
            control.as_ref()?;
            // A reset invalidates diagnostic ancestry, not the current frame's
            // execution identity. Preserve the sources exactly as the old
            // no-context path did, and use the existing guard for restoration.
            let evaluation = crate::runtime::Session::logical_capture_sources();
            let rebuild = EXPRESSION_REBUILD_SOURCE_STACK.with_borrow(Clone::clone);
            let lexical = LEXICAL_CONTEXT_STACK.with_borrow(Clone::clone);
            let sites = ARG_PREFILTER_SITE_STACK.with_borrow(Clone::clone);
            EXECUTION_CONTROLS.with_borrow_mut(|stack| stack.push(control));
            return Some(ContextGuard {
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _registration,

                context: Arc::clone(self),
                segment: None,
                evaluation,
                rebuild,
                lexical,
                sites,
            });
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
        EXECUTION_CONTROLS.with_borrow_mut(|stack| stack.push(control));
        Some(ContextGuard {
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _registration,

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
    pub(in crate::runtime) fn enter_control(
        control: Arc<crate::runtime::RequestInterruptState>,
    ) -> Self {
        use super::super::state::*;
        // A non-null Context keeps the existing Option<ContextGuard> niche.
        // This execution-only context holds no epoch or diagnostic lease.
        let context = Arc::new(Context {
            epoch: None,
            records: None,
            evaluation: SourceStack::default(),
            rebuild: SourceStack::default(),
            lexical: SourceStack::default(),
            sites: SourceStack::default(),
            executing: AtomicBool::new(true),
            _storage: None,
        });
        let evaluation = crate::runtime::Session::logical_capture_sources();
        let rebuild = EXPRESSION_REBUILD_SOURCE_STACK.with_borrow(Clone::clone);
        let lexical = LEXICAL_CONTEXT_STACK.with_borrow(Clone::clone);
        let sites = ARG_PREFILTER_SITE_STACK.with_borrow(Clone::clone);
        EXECUTION_CONTROLS.with_borrow_mut(|stack| stack.push(Some(control)));
        Self {
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _registration: crate::runtime::profiling::lease::ThreadRegistration::inactive(),
            context,
            segment: None,
            evaluation,
            rebuild,
            lexical,
            sites,
        }
    }

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
        EXECUTION_CONTROLS.with_borrow_mut(|stack| {
            stack.pop();
        });
        self.context.executing.store(false, Ordering::Relaxed);
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        if let Some(epoch) = &self.epoch {
            let _ = epoch
                .contexts
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                    Some(n.saturating_sub(1))
                });
        }
    }
}

thread_local! {
    static EXECUTION_CONTROLS: std::cell::RefCell<Vec<Option<Arc<crate::runtime::RequestInterruptState>>>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

/// Production is zero-sized; test builds remember ownership of the TLS push.
pub(in crate::runtime) struct ExecutionScope {
    _private: (),
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    pushed: bool,
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _registration: crate::runtime::profiling::lease::ThreadRegistration,
}

#[cfg(not(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
)))]
const _: () = assert!(std::mem::size_of::<ExecutionScope>() == 0);

impl ExecutionScope {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    #[inline]
    pub(in crate::runtime) fn enter(session: &crate::runtime::Session) -> Self {
        let Some((access, _registration)) = crate::runtime::profiling::lease::admit_generation(
            session.logical_profile_generation(),
        ) else {
            return Self {
                _private: (),
                pushed: false,
                _registration: crate::runtime::profiling::lease::ThreadRegistration::inactive(),
            };
        };
        let pushed = super::super::config::lexical_profile_configured_admitted(&access);
        if pushed {
            let control = session.request_interrupt_control.borrow();
            EXECUTION_CONTROLS.with_borrow_mut(|stack| stack.push(Some(Arc::clone(&control))));
        }
        Self {
            _private: (),
            pushed,
            _registration,
        }
    }

    /// The disabled path is the cold-load check itself: the configured body is
    /// outlined so no unconfigured entry pays a call.
    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    #[inline]
    pub(in crate::runtime) fn enter(session: &crate::runtime::Session) -> Self {
        if super::super::state::LEXICAL_PROFILE_ENABLED.load(Ordering::Relaxed) != 2 {
            Self::push_control(session);
        }
        Self { _private: () }
    }

    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    #[cold]
    #[inline(never)]
    fn push_control(session: &crate::runtime::Session) {
        if !super::super::lexical_profile_configured() {
            return;
        }
        let control = session.request_interrupt_control.borrow();
        EXECUTION_CONTROLS.with_borrow_mut(|stack| stack.push(Some(Arc::clone(&control))));
    }

    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    #[cold]
    #[inline(never)]
    fn pop_control() {
        if !super::super::lexical_profile_configured() {
            return;
        }
        let _ = EXECUTION_CONTROLS.try_with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

impl Drop for ExecutionScope {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    #[inline(always)]
    fn drop(&mut self) {
        if self.pushed {
            let _ = EXECUTION_CONTROLS.try_with(|stack| {
                stack.borrow_mut().pop();
            });
        }
    }

    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    #[inline(always)]
    fn drop(&mut self) {
        if super::super::state::LEXICAL_PROFILE_ENABLED.load(Ordering::Relaxed) != 2 {
            Self::pop_control();
        }
    }
}

pub(in crate::runtime) fn execution_control() -> Option<Arc<crate::runtime::RequestInterruptState>>
{
    EXECUTION_CONTROLS.with_borrow(|stack| stack.last().cloned().flatten())
}

/// The request-control rollover is a rare cold branch; every active scope of
/// the same session must observe the replacement control. Outlined so the
/// admission path that calls it keeps its exact emitted code.
#[cold]
#[inline(never)]
pub(in crate::runtime) fn refresh_execution_control(
    previous: &Arc<crate::runtime::RequestInterruptState>,
    replacement: &Arc<crate::runtime::RequestInterruptState>,
) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    refresh_execution_control_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        previous,
        replacement,
    )
}

fn refresh_execution_control_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,

    previous: &Arc<crate::runtime::RequestInterruptState>,
    replacement: &Arc<crate::runtime::RequestInterruptState>,
) {
    if !super::super::config::lexical_profile_configured_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }
    EXECUTION_CONTROLS.with_borrow_mut(|stack| {
        for entry in stack.iter_mut().flatten() {
            if Arc::ptr_eq(entry, previous) {
                *entry = Arc::clone(replacement);
            }
        }
    });
}

#[cfg(test)]
#[path = "../tests/brief713_execution_scope.rs"]
mod brief713_execution_scope;

#[cfg(test)]
#[path = "../tests/brief713_generation.rs"]
mod brief713_generation;

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl crate::runtime::Session {
    /// Read only immutable provenance in the three live suspension roots.
    /// A session with no profiled owner does not gain a registration.
    pub(in crate::runtime) fn logical_profile_generation(&self) -> Option<u64> {
        let mut generation = None;
        for candidate in self
            .pending_suspension
            .as_ref()
            .into_iter()
            .chain(self.resuming_suspension.as_ref())
            .chain(self.abandonment_worklist.iter())
            .filter_map(|suspension| suspension.frames.lease_generation())
        {
            if let Some(expected) = generation {
                assert_eq!(
                    candidate, expected,
                    "mixed profiling lease generations in session roots"
                );
            } else {
                generation = Some(candidate);
            }
        }
        generation
    }
}
