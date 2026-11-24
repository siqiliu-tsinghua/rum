//! Pure `rum` kernel.
//!
//! This crate owns lowering, evaluation sessions, definitions, scoping,
//! messages, matching, tracing, resource limits, capability policy enforcement,
//! and the host-facing request APIs used by embedding layers.

mod runtime;

pub use runtime::*;

#[cfg(any(test, feature = "test-continuation-slots"))]
#[doc(hidden)]
pub fn test_logical_profile_snapshot() -> LogicalProfileSnapshot {
    runtime::logical_profile_snapshot()
}
#[cfg(any(test, feature = "test-continuation-slots"))]
#[doc(hidden)]
pub fn test_logical_profile_reset(lease: &LogicalProfileTestLease) {
    runtime::logical_profile_reset(lease);
}

pub mod cm740_probe {
    // Generated mutation attribution. This module is absent from live builds.
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    pub const ENABLED: bool = !cfg!(disabled) && !cfg!(live);

    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct Episode {
        row: String,
        program: String,
        pair: u64,
    }
    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct Activation {
        episode: Episode,
        // Source-scoped runtime entry path, never a renderer-supplied occurrence.
        path: Vec<(&'static str, u64)>,
    }
    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct Token {
        owner: Activation,
        origin: Activation,
        occurrence: u64,
        frame_identity: Option<(usize, &'static str)>,
    }
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum SourceKind {
        ProducerPause,
        OwnedReturn,
        Control,
        ParentReturn,
    }
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ReturnKind {
        Evaluation,
        Request,
    }
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    enum InitialReturnEdge {
        ActualInputRequired,
    }
    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct InitialReturnKey {
        source: Token,
        edge: InitialReturnEdge,
    }
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    enum RequestReturnEdge {
        ActualInputRequired,
    }
    #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
    struct RequestReturnKey {
        source: Token,
        edge: RequestReturnEdge,
    }
    #[derive(Clone, Default)]
    struct Pair {
        frame_permits: BTreeMap<Token, u64>,
        source_records: BTreeMap<Token, SourceKind>,
        return_records: Vec<(Activation, ReturnKind, Token)>,
        request_records: Vec<(Activation, Token)>,
        episode: Option<Episode>,
        next_occurrence: u64,
        tokens: BTreeMap<Activation, Vec<Token>>,
        // Q21: these are actual request-return proofs, never frame identities.
        request_returns: BTreeMap<Activation, Token>,
        current_request_returns: BTreeMap<RequestReturnKey, bool>,
        current_initial_returns: BTreeMap<InitialReturnKey, bool>,
        descendant_edges: Vec<DescendantEdge>,
        descendant_permits: BTreeMap<(Activation, u64), DescendantEdge>,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum PairStage {
        Open,
        Ready,
        Consumed,
        Retired,
    }
    #[derive(Debug)]
    struct RunAuthority {
        episode: Option<Episode>,
        queued: bool,
        current: std::cell::Cell<bool>,
    }
    fn run_current(run: &Run) -> bool {
        run.source_authority.current.get()
            && run.source_authority.episode == run.pair.episode
            && run.source_authority.queued == run.queued
    }
    fn retire_actual_run(l: &mut Ledger) {
        if let Some(authority) = l.run_authority.take() {
            authority.current.set(false);
            if let Some(episode) = &authority.episode {
                if l.pair_stages.get(episode) == Some(&PairStage::Open) {
                    l.pair_stages.insert(episode.clone(), PairStage::Retired);
                }
            }
        }
    }
    fn retire_pair_mailbox(l: &mut Ledger, key: &(String, String)) {
        for (episode, status) in &mut l.pair_stages {
            if (&episode.row, &episode.program) == (&key.0, &key.1)
                && matches!(status, PairStage::Open | PairStage::Ready)
            {
                *status = PairStage::Retired;
            }
        }
    }
    fn take_pair_once(l: &mut Ledger, key: &(String, String)) -> Pair {
        let pair = l.pending.remove(key).unwrap_or_default();
        let Some(episode) = &pair.episode else {
            return Pair::default();
        };
        if (&episode.row, &episode.program) != (&key.0, &key.1)
            || l.pair_stages.get(episode) != Some(&PairStage::Ready)
        {
            return Pair::default();
        }
        l.pair_stages.insert(episode.clone(), PairStage::Consumed);
        pair
    }

    struct Run {
        source_authority: std::rc::Rc<RunAuthority>,
        pair: Pair,
        queued: bool,
        next_child: BTreeMap<Vec<(&'static str, u64)>, u64>,
        hits: BTreeMap<(Activation, u64), u64>,
        pending: BTreeMap<Activation, Token>,
        current_pending_sources: BTreeMap<Token, bool>,
        returning: BTreeMap<Activation, Token>,
        request_returning: BTreeMap<Activation, Token>,
    }
    #[derive(Default)]
    struct Ledger {
        queued_frame_issues: BTreeMap<(Activation, u64), Token>,
        next_descendant_issue: u64,
        queued_descendant_issues: BTreeMap<(Activation, u64), DescendantEdge>,
        run_authority: Option<std::rc::Rc<RunAuthority>>,
        pair_stages: BTreeMap<Episode, PairStage>,
        scope_authority: ScopeAuthority,
        profile: ProfileState,
        deferred: BTreeMap<(usize, &'static str), Token>,
        deferred_proofs: BTreeMap<(usize, &'static str), Token>,
        next_pair: u64,
        pending: BTreeMap<(String, String), Pair>,
        current: Option<Run>,
        stack: Vec<Activation>,
        // Only an actual producer entry may arm. A resumed/deferred owner is a
        // boundary, while a producer entered inside it has its own live scope.
        arming_scopes: Vec<Option<Activation>>,
        bound_slots: BTreeMap<(usize, &'static str), Token>,
        current_frame_records: BTreeMap<Token, bool>,
        constructing: Vec<ConstructionIdentity>,
        evaluation_provenance: Vec<EvaluationProvenance>,
        resuming: Vec<Option<Token>>,
    }
    thread_local! {
        static LEDGER: RefCell<Ledger> = RefCell::new(Ledger::default());
        static SELECTED: String = std::env::var("RUM_CM740_OMIT").unwrap_or_default();
    }

    // Q18 equivalence: one unqueued begin opens a pair; exactly the next queued
    // begin for the same opaque identity consumes it. A new unqueued begin replaces
    // an unpaired predecessor. The attempt boundary is this thread's fresh ledger.
    pub fn begin(queued: bool, program: &str) {
        LEDGER.with(|cell| {
            let l = cell.borrow();
            assert!(
                l.stack.is_empty()
                    && l.arming_scopes.is_empty()
                    && l.constructing.is_empty()
                    && l.resuming.is_empty()
                    && l.evaluation_provenance.is_empty()
                    && l.scope_authority == ScopeAuthority::default(),
                "PortBlocked::LiveScopeAtBegin"
            );
        });
        control::retire();
        reset_operation_episode();
        identity_proofs::retire_namespace(|_| true);
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            assert!(l.stack.is_empty(), "unclosed producer activation");
            retire_actual_run(&mut l);
            l.current = None;
            l.profile.reset();
            l.bound_slots.clear();
            DIRECT_CLAIMS.with(|c| c.borrow_mut().clear());
            l.deferred.clear();
            l.deferred_proofs.clear();
            let row = SELECTED.with(Clone::clone);
            let key = (row.clone(), program.to_owned());
            let pair = if queued {
                take_pair_once(&mut l, &key)
            } else {
                retire_pair_mailbox(&mut l, &key);
                l.queued_frame_issues.retain(|(owner, _), _| {
                    (&owner.episode.row, &owner.episode.program) != (&key.0, &key.1)
                });
                l.queued_descendant_issues.retain(|(owner, _), _| {
                    (&owner.episode.row, &owner.episode.program) != (&key.0, &key.1)
                });
                l.next_pair = l.next_pair.checked_add(1).expect("pair overflow");
                l.pending.remove(&key);
                Pair {
                    episode: Some(Episode {
                        row,
                        program: program.to_owned(),
                        pair: l.next_pair,
                    }),
                    ..Pair::default()
                }
            };
            assert!(
                l.constructing.is_empty()
                    && l.resuming.is_empty()
                    && l.evaluation_provenance.is_empty()
            );
            if !queued {
                assert!(
                    l.pair_stages
                        .insert(pair.episode.clone().unwrap(), PairStage::Open)
                        .is_none()
                );
            }
            let source_authority = std::rc::Rc::new(RunAuthority {
                episode: pair.episode.clone(),
                queued,
                current: std::cell::Cell::new(true),
            });
            assert!(l.run_authority.replace(source_authority.clone()).is_none());
            l.current = Some(Run {
                source_authority,
                pair,
                queued,
                next_child: BTreeMap::new(),
                hits: BTreeMap::new(),
                pending: BTreeMap::new(),
                current_pending_sources: BTreeMap::new(),
                returning: BTreeMap::new(),
                request_returning: BTreeMap::new(),
            });
        });
        q25::on_begin(queued, program);
    }
    pub fn end() {
        control::retire();
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            assert!(l.stack.is_empty(), "unclosed producer activation");
            l.profile.reset();
            if let Some(run) = l.current.take() {
                if run.queued {
                    l.queued_frame_issues
                        .retain(|(owner, _), _| Some(&owner.episode) != run.pair.episode.as_ref());
                    l.queued_descendant_issues
                        .retain(|(owner, _), _| Some(&owner.episode) != run.pair.episode.as_ref());
                }
                if run_current(&run) && !run.queued {
                    if let Some(e) = &run.pair.episode {
                        assert_eq!(
                            l.pair_stages.get(e),
                            Some(&PairStage::Open),
                            "PortBlocked::UnissuedPairEnd"
                        );
                        l.pair_stages.insert(e.clone(), PairStage::Ready);
                        l.pending
                            .insert((e.row.clone(), e.program.clone()), run.pair);
                    }
                }
            }
            retire_actual_run(&mut l);
            l.bound_slots.clear();
            DIRECT_CLAIMS.with(|c| c.borrow_mut().clear());
            l.deferred.clear();
            l.deferred_proofs.clear();
        });
        q25::on_end();
    }
    // Source-written current scopes are independent of all transported stacks.
    #[derive(Default, Clone, Debug, PartialEq, Eq)]
    struct ScopeAuthority {
        stack: Vec<Activation>,
        arming: Vec<Option<Activation>>,
        constructing: Vec<ConstructionIdentity>,
        provenance: Vec<EvaluationProvenance>,
        resuming: Vec<Option<Token>>,
        resume_depths: Vec<(usize, usize)>,
        entries: BTreeMap<Activation, Option<u64>>,
        guard_sources: BTreeMap<Activation, std::rc::Rc<GuardSource>>,
    }
    fn lexical_current(l: &Ledger) -> bool {
        l.stack == l.scope_authority.stack
            && l.arming_scopes == l.scope_authority.arming
            && l.constructing == l.scope_authority.constructing
            && l.evaluation_provenance == l.scope_authority.provenance
            && l.resuming == l.scope_authority.resuming
    }
    fn guard_current(l: &Ledger, activation: &Activation, step: Option<u64>) -> bool {
        l.current.as_ref().is_some_and(run_current)
            && lexical_current(l)
            && l.scope_authority.stack.last() == Some(activation)
            && l.scope_authority.entries.get(activation) == Some(&step)
    }

    // The actual Guard owns this independently minted source identity. Moving the
    // Guard preserves it; changing the received activation/profile tuple does not.
    // It contains only probe data and holds no production object or frame alive.
    #[derive(Debug, PartialEq, Eq)]
    struct GuardSource {
        owner: Activation,
        profile_step: Option<u64>,
    }
    fn mint_guard_source(
        l: &mut Ledger,
        owner: &Activation,
        profile_step: Option<u64>,
    ) -> std::rc::Rc<GuardSource> {
        assert!(
            !l.scope_authority.guard_sources.contains_key(owner),
            "PortBlocked::DuplicateActualGuardSource"
        );
        let source = std::rc::Rc::new(GuardSource {
            owner: owner.clone(),
            profile_step,
        });
        assert!(
            l.scope_authority
                .guard_sources
                .insert(owner.clone(), source.clone())
                .is_none()
        );
        source
    }
    fn guard_source_matches(l: &Ledger, guard: &Guard) -> bool {
        match guard.0.as_ref() {
            None => true,
            Some(active) => {
                active.source.owner == active.activation
                    && active.source.profile_step == active.step
                    && l.scope_authority
                        .guard_sources
                        .get(&active.activation)
                        .is_some_and(|actual| std::rc::Rc::ptr_eq(actual, &active.source))
            }
        }
    }
    fn retire_guard_source(l: &mut Ledger, guard: &Guard) {
        let Some(active) = guard.0.as_ref() else {
            panic!("PortBlocked::MissingActualGuardSource");
        };
        assert!(
            l.scope_authority
                .guard_sources
                .get(&active.activation)
                .is_some_and(|actual| std::rc::Rc::ptr_eq(actual, &active.source)),
            "PortBlocked::WrongActualGuardRetirement"
        );
        l.scope_authority.guard_sources.remove(&active.activation);
    }

    #[inline(never)]
    fn run_is_driven() -> bool {
        if !ENABLED {
            return false;
        }
        LEDGER.with(|cell| {
            let l = cell.borrow();
            if !lexical_current(&l) {
                return false;
            }
            let Some(run) = l.current.as_ref() else {
                return false;
            };
            run_current(run) && run.pair.episode.is_some()
        })
    }
    #[inline(never)]
    #[allow(dead_code)]
    pub fn driven() -> bool {
        run_is_driven()
    }

    // Inactive guards are pointer-sized. The activation, step and source live in
    // the box only while a driven run has entered, so an ordinary recursive
    // evaluation does not carry the active guard frame on its native stack.
    struct GuardActive {
        activation: Activation,
        step: Option<u64>,
        source: std::rc::Rc<GuardSource>,
    }
    pub struct Guard(Option<Box<GuardActive>>);
    impl Guard {
        #[inline(always)]
        #[allow(dead_code)]
        pub fn inactive() -> Self {
            Self(None)
        }
    }

    #[inline(never)]
    pub fn enter(row: &'static str) -> Guard {
        if !run_is_driven() {
            return Guard(None);
        }
        enter_slow(row)
    }
    #[inline(never)]
    fn enter_slow(row: &'static str) -> Guard {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            if !lexical_current(&l) {
                return Guard(None);
            }
            let parent = l.stack.last().map(|a| a.path.clone()).unwrap_or_default();
            let profile_step = l.profile.active.last().copied();
            let Some(run) = &mut l.current else {
                return Guard(None);
            };
            if !run_current(run) {
                return Guard(None);
            }
            let Some(episode) = run.pair.episode.clone() else {
                return Guard(None);
            };
            let index = run.next_child.entry(parent.clone()).or_default();
            let mut path = parent;
            path.push((row, *index));
            *index += 1;
            let activation = Activation { episode, path };
            let queued = run.queued;
            let source = mint_guard_source(&mut l, &activation, profile_step);
            identity_proofs::source_entry(&source, queued);
            assert!(
                l.scope_authority
                    .entries
                    .insert(activation.clone(), profile_step)
                    .is_none()
            );
            l.scope_authority.stack.push(activation.clone());
            l.scope_authority.arming.push(Some(activation.clone()));
            l.stack.push(activation.clone());
            l.arming_scopes.push(Some(activation.clone()));
            Guard(Some(Box::new(GuardActive {
                activation,
                step: profile_step,
                source,
            })))
        })
    }

    /// Private proof carried from a selected suspension to its actual typed adapter.
    pub(crate) mod control {
        use super::*;
        use std::rc::Rc;
        use std::sync::Arc;

        struct Installing {
            row: &'static str,
            kind: &'static str,
            authority: Rc<RunAuthority>,
            guard: Option<Guard>,
            bound: std::cell::Cell<bool>,
            deferred: bool,
        }

        struct Bound {
            row: &'static str,
            kind: &'static str,
            authority: Rc<RunAuthority>,
            address: usize,
            generation: u64,
            live: Box<dyn Fn() -> bool>,
            deferred: bool,
            emitted: std::cell::Cell<bool>,
        }

        thread_local! {
            static INSTALLING: RefCell<Option<Rc<Installing>>> = const { RefCell::new(None) };
            static FRAMES: RefCell<BTreeMap<usize, Rc<Bound>>> = const { RefCell::new(BTreeMap::new()) };
            static RESUMED: RefCell<Option<Rc<Bound>>> = const { RefCell::new(None) };
        }

        fn authority() -> Option<Rc<RunAuthority>> {
            LEDGER.with(|c| {
                let l = c.borrow();
                let r = l.current.as_ref()?;
                let episode = r.pair.episode.as_ref()?;
                (!r.queued
                    && run_current(r)
                    && lexical_current(&l)
                    && episode.program == "CM740-SEMANTIC"
                    && l.pair_stages.get(episode) == Some(&PairStage::Open))
                .then(|| Rc::clone(&r.source_authority))
            })
        }

        pub(crate) struct SourceScope {
            previous: Option<Rc<Installing>>,
        }
        impl Drop for SourceScope {
            fn drop(&mut self) {
                INSTALLING.with(|slot| slot.replace(self.previous.take()));
            }
        }

        pub(crate) fn source<F: 'static>(
            allowed: &[&'static str],
            deferred: bool,
        ) -> Option<SourceScope> {
            let authority = authority()?;
            let selection = std::env::var("RUM_CM740_SEMANTIC").ok()?;
            let row = *allowed.iter().find(|row| **row == selection)?;
            if ENABLED {
                if !selected(row) || !driven() {
                    return None;
                }
            } else if row != "Some" {
                return None;
            }
            let guard = ENABLED.then(|| enter(row));
            if deferred && ENABLED {
                // The source is issued at this real suspension. Its existing phase
                // marker is emitted only when this very frame resumes successfully.
                arm_source(row, "control-frame-source", "", false);
            }
            let source = Rc::new(Installing {
                row,
                kind: std::any::type_name::<F>(),
                authority,
                guard,
                deferred,
                bound: std::cell::Cell::new(false),
            });
            Some(SourceScope {
                previous: INSTALLING.with(|slot| slot.replace(Some(source))),
            })
        }

        pub(super) fn allocated<T: ?Sized + 'static>(
            actual: &Arc<T>,
            kind: &'static str,
            generation: u64,
        ) {
            let Some(s) = INSTALLING.with(|slot| slot.borrow().clone()) else {
                return;
            };
            if s.kind != kind
                || generation == 0
                || s.bound.get()
                || !authority().is_some_and(|a| Rc::ptr_eq(&a, &s.authority))
            {
                return;
            }
            let address = Arc::as_ptr(actual) as *const () as usize;
            if let Some(guard) = &s.guard {
                let _construction = guard.suspending();
                bind_frame(address, kind);
                if !LEDGER.with(|l| {
                    let l = l.borrow();
                    l.bound_slots.get(&(address, kind)).is_some_and(|t| {
                        authenticated_parent(&l, t)
                            && t.owner.path.last().is_some_and(|(r, _)| *r == s.row)
                    })
                }) {
                    return;
                }
            }
            s.bound.set(true);
            let weak = Arc::downgrade(actual);
            let frame = Rc::new(Bound {
                row: s.row,
                kind,
                authority: Rc::clone(&s.authority),
                address,
                generation,
                live: Box::new(move || weak.strong_count() > 0),
                deferred: s.deferred,
                emitted: std::cell::Cell::new(false),
            });
            FRAMES.with(|frames| assert!(frames.borrow_mut().insert(address, frame).is_none()));
        }

        pub(crate) struct ResumeScope {
            previous: Option<Rc<Bound>>,
            frame: Option<Rc<Bound>>,
            generic: Option<ResumeGuard>,
        }
        impl Drop for ResumeScope {
            fn drop(&mut self) {
                RESUMED.with(|slot| slot.replace(self.previous.take()));
                drop(self.generic.take());
                if let Some(frame) = self.frame.take() {
                    if ENABLED {
                        forget_frame(frame.address, frame.kind);
                    }
                }
            }
        }

        pub(crate) fn resume<F: 'static, A>(adapter: &A) -> Option<ResumeScope> {
            let address = std::ptr::from_ref(adapter) as usize;
            let frame = FRAMES.with(|frames| frames.borrow_mut().remove(&address));
            let previous = RESUMED.with(|slot| slot.take());
            if previous.is_none() && frame.is_none() {
                return None;
            }
            let frame = frame.filter(|f| {
                f.kind == std::any::type_name::<F>()
                    && f.generation != 0
                    && (f.live)()
                    && authority().is_some_and(|a| Rc::ptr_eq(&a, &f.authority))
            });
            let generic = frame
                .as_ref()
                .and_then(|f| ENABLED.then(|| resume_frame(address, f.kind)));
            RESUMED.with(|slot| slot.replace(frame.clone()));
            Some(ResumeScope {
                previous,
                frame,
                generic,
            })
        }

        pub(crate) fn active(row: &str) -> bool {
            RESUMED.with(|slot| {
                slot.borrow().as_ref().is_some_and(|f| {
                    f.row == row
                        && (f.live)()
                        && authority().is_some_and(|a| Rc::ptr_eq(&a, &f.authority))
                })
            })
        }

        pub(crate) fn emit_resumed(row: &'static str, phase: &str) -> bool {
            if !active(row) || !frame(row) {
                return false;
            }
            RESUMED.with(|slot| {
                let slot = slot.borrow();
                let f = slot.as_ref().unwrap();
                if !f.deferred || f.emitted.replace(true) {
                    return false;
                }
                emit_armed(row, phase, "");
                true
            })
        }

        pub(super) fn retire() {
            INSTALLING.with(|s| assert!(s.borrow().is_none()));
            RESUMED.with(|s| assert!(s.borrow().is_none()));
            FRAMES.with(|s| s.borrow_mut().clear());
        }
    }
    // Fast path is the body alone. The guard and the ordinary/frame match live in
    // the never-inlined arm, so an undriven recursive evaluation does not carry
    // them on the native stack. A driven success still records the hit; the
    // caller's tail is not skipped.
    #[inline(always)]
    pub fn around<R>(row: &'static str, body: impl FnOnce() -> R) -> R {
        if run_is_driven() {
            around_armed(row, body)
        } else {
            body()
        }
    }
    #[inline(never)]
    fn around_armed<R>(row: &'static str, body: impl FnOnce() -> R) -> R {
        let _guard = enter_slow(row);
        body()
    }
    #[inline(always)]
    pub fn around_ok<T, E>(
        row: &'static str,
        phase: &'static str,
        framed: bool,
        body: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        if run_is_driven() {
            around_ok_armed(row, phase, framed, body)
        } else {
            body()
        }
    }
    #[inline(never)]
    fn around_ok_armed<T, E>(
        row: &'static str,
        phase: &'static str,
        framed: bool,
        body: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let _guard = enter_slow(row);
        let result = body();
        let selected = if framed { frame(row) } else { ordinary(row) };
        if result.is_ok() && selected {
            hit(row, phase);
        }
        result
    }
    #[inline(always)]
    pub fn around_some<T, E>(
        row: &'static str,
        phase: &'static str,
        framed: bool,
        body: impl FnOnce() -> Result<Option<T>, E>,
    ) -> Result<Option<T>, E> {
        if run_is_driven() {
            around_some_armed(row, phase, framed, body)
        } else {
            body()
        }
    }
    #[inline(never)]
    fn around_some_armed<T, E>(
        row: &'static str,
        phase: &'static str,
        framed: bool,
        body: impl FnOnce() -> Result<Option<T>, E>,
    ) -> Result<Option<T>, E> {
        let _guard = enter_slow(row);
        let result = body();
        let selected = if framed { frame(row) } else { ordinary(row) };
        if matches!(result, Ok(Some(_))) && selected {
            hit(row, phase);
        }
        result
    }
    #[inline(always)]
    pub fn around_hit<R>(
        row: &'static str,
        phase: &'static str,
        framed: bool,
        body: impl FnOnce() -> R,
    ) -> R {
        if run_is_driven() {
            around_hit_armed(row, phase, framed, body)
        } else {
            body()
        }
    }
    #[inline(never)]
    fn around_hit_armed<R>(
        row: &'static str,
        phase: &'static str,
        framed: bool,
        body: impl FnOnce() -> R,
    ) -> R {
        let _guard = enter_slow(row);
        let result = body();
        let selected = if framed { frame(row) } else { ordinary(row) };
        if selected {
            hit(row, phase);
        }
        result
    }

    // --- Route A: Non-weakening descendant-frame queued-success authentication ---
    //
    // A profile-return wrapper activation W does not construct its own suspension
    // frame before returning; a strictly deeper callee D inside W constructs and
    // installs a real suspension frame while W's Guard and pending producer pause
    // are both live during the unqueued half. This primitive records a pair-scoped
    // DescendantEdge into both Pair (consumed once at queued begin) and Ledger's
    // independent queued_descendant_issues table (retired at queued end / next
    // unqueued begin), without touching Pair::tokens, ordinary(), frame(),
    // eligible(), issued_queued_frame(), or hit().
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct DescendantEdge {
        ancestor: Token,
        descendant: Token,
        generation: u64,
        issue_id: u64,
    }

    pub enum OkAction<T, E> {
        Continue(Result<T, E>),
        ReturnOk(T),
    }

    fn path_proper_prefix(
        ancestor: &[(&'static str, u64)],
        descendant: &[(&'static str, u64)],
    ) -> bool {
        ancestor.len() < descendant.len() && descendant.starts_with(ancestor)
    }

    /// Crate-private recorder callable only inside `rum-runtime` (invoked solely
    /// from the two suspension-frame installation sinks in `runtime::suspension::core`).
    /// Binds a live `&Arc<T>` frame allocation and non-zero `SuspensionGeneration`
    /// to the active armed ancestor wrapper W on the unqueued half.
    pub(crate) fn record_descendant_frame<T: ?Sized + 'static>(
        actual: &std::sync::Arc<T>,
        kind: &'static str,
        generation: u64,
    ) {
        control::allocated(actual, kind, generation);
        if !ENABLED || generation == 0 {
            return;
        }
        let physical = (std::sync::Arc::as_ptr(actual) as *const () as usize, kind);
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            if !lexical_current(&l) {
                return;
            }
            let (episode, wrapper, descendant_act, ancestor_token) = {
                let Some(run) = l.current.as_ref() else {
                    return;
                };
                if run.queued || !run_current(run) {
                    return;
                }
                let Some(episode) = run.pair.episode.clone() else {
                    return;
                };
                if l.pair_stages.get(&episode) != Some(&PairStage::Open) {
                    return;
                }
                let Some(descendant_act) = l.stack.last().cloned() else {
                    return;
                };
                if descendant_act.episode != episode {
                    return;
                }
                let Some(wrapper) = producer_owner(&l, &episode.row) else {
                    return;
                };
                if !path_proper_prefix(&wrapper.path, &descendant_act.path) {
                    return;
                }
                if !l.scope_authority.guard_sources.contains_key(&wrapper)
                    || !l
                        .scope_authority
                        .guard_sources
                        .contains_key(&descendant_act)
                {
                    return;
                }
                let Some(ancestor_token) = run.pending.get(&wrapper).cloned() else {
                    return;
                };
                if ancestor_token.owner != wrapper
                    || ancestor_token.frame_identity.is_some()
                    || !current_pending_source(run, &ancestor_token)
                {
                    return;
                }
                if !matches!(
                    run.pair.source_records.get(&ancestor_token),
                    Some(SourceKind::ProducerPause | SourceKind::OwnedReturn)
                ) {
                    return;
                }
                (episode, wrapper, descendant_act, ancestor_token)
            };
            debug_assert_eq!(wrapper.episode, episode);
            l.next_descendant_issue = l
                .next_descendant_issue
                .checked_add(1)
                .expect("descendant issue overflow");
            let issue_id = l.next_descendant_issue;
            let descendant = Token {
                owner: descendant_act,
                origin: ancestor_token.origin.clone(),
                occurrence: ancestor_token.occurrence,
                frame_identity: Some(physical),
            };
            let edge = DescendantEdge {
                ancestor: ancestor_token,
                descendant,
                generation,
                issue_id,
            };
            let run = l.current.as_mut().unwrap();
            run.pair.descendant_edges.push(edge.clone());
            assert!(
                run.pair
                    .descendant_permits
                    .insert((wrapper.clone(), issue_id), edge.clone())
                    .is_none(),
                "PortBlocked::DuplicateDescendantPermit"
            );
            assert!(
                l.queued_descendant_issues
                    .insert((wrapper, issue_id), edge)
                    .is_none(),
                "PortBlocked::DuplicateLedgerDescendantIssue"
            );
        });
    }

    fn descendant_gate_clause(
        l: &Ledger,
        row: &str,
        guard: &Guard,
    ) -> Result<DescendantEdge, &'static str> {
        if !ENABLED {
            return Err("disabled");
        }
        if !lexical_current(l) {
            return Err("lexical_not_current");
        }
        let Some(active) = guard.0.as_ref() else {
            return Err("guard_inactive");
        };
        if !guard_source_matches(l, guard) || !guard_current(l, &active.activation, active.step) {
            return Err("guard_source_mismatch");
        }
        let Some(run) = l.current.as_ref() else {
            return Err("no_active_run");
        };
        if !run.queued {
            return Err("run_not_queued");
        }
        if !run_current(run) {
            return Err("run_not_current");
        }
        let Some(episode) = run.pair.episode.as_ref() else {
            return Err("no_episode");
        };
        if l.pair_stages.get(episode) != Some(&PairStage::Consumed) {
            return Err("pair_not_consumed");
        }
        let Some(wrapper) = owner(l, row) else {
            return Err("owner_missing");
        };
        if wrapper != active.activation || l.stack.last() != Some(&wrapper) {
            return Err("activation_not_top");
        }
        let Some(((_, issue_id), edge)) = run
            .pair
            .descendant_permits
            .range((wrapper.clone(), u64::MIN)..=(wrapper.clone(), u64::MAX))
            .next_back()
        else {
            return Err("descendant_permit_missing");
        };
        if edge.issue_id != *issue_id || edge.generation == 0 {
            return Err("invalid_edge_metadata");
        }
        if edge.ancestor.owner != wrapper
            || edge.ancestor.owner.episode != *episode
            || edge.ancestor.origin.episode != *episode
            || edge.ancestor.frame_identity.is_some()
        {
            return Err("ancestor_mismatch");
        }
        if edge.descendant.owner.episode != *episode
            || edge.descendant.origin != edge.ancestor.origin
            || edge.descendant.occurrence != edge.ancestor.occurrence
            || edge.descendant.frame_identity.is_none()
            || !path_proper_prefix(&wrapper.path, &edge.descendant.owner.path)
        {
            return Err("descendant_mismatch");
        }
        if !matches!(
            run.pair.source_records.get(&edge.ancestor),
            Some(SourceKind::ProducerPause | SourceKind::OwnedReturn)
        ) {
            return Err("source_kind_disallowed");
        }
        if !run
            .pair
            .descendant_edges
            .iter()
            .any(|recorded| recorded == edge)
        {
            return Err("edge_not_recorded");
        }
        if l.queued_descendant_issues.get(&(wrapper, *issue_id)) != Some(edge) {
            return Err("ledger_issue_mismatch");
        }
        Ok(edge.clone())
    }

    fn hit_descendant(row: &str, phase: &str, guard: &Guard) -> bool {
        if !ENABLED || phase.is_empty() {
            return false;
        }
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let Ok(edge) = descendant_gate_clause(&l, row, guard) else {
                return false;
            };
            let token = edge.ancestor;
            let run = l.current.as_mut().unwrap();
            let n = run
                .hits
                .entry((token.owner.clone(), token.occurrence))
                .or_default();
            eprintln!("CM740_SOURCE_HIT row={row} phase={phase} selector={row}");
            eprintln!(
                "CM742_HIT_ID program={:?} pair={} activation={:?} producer_occurrence={} hit_occurrence={}",
                token.owner.episode.program,
                token.owner.episode.pair,
                token.owner.path,
                token.occurrence,
                n
            );
            *n += 1;
            true
        })
    }

    #[inline(always)]
    pub fn around_ok_profile_return<T, E>(
        row: &'static str,
        phase: &'static str,
        body: impl FnOnce() -> Result<T, E>,
    ) -> OkAction<T, E> {
        if run_is_driven() {
            around_ok_profile_return_armed(row, phase, body)
        } else {
            OkAction::Continue(body())
        }
    }

    #[inline(never)]
    fn around_ok_profile_return_armed<T, E>(
        row: &'static str,
        phase: &'static str,
        body: impl FnOnce() -> Result<T, E>,
    ) -> OkAction<T, E> {
        let guard = enter_slow(row);
        let result = body();
        match result {
            Ok(value) => {
                if hit_descendant(row, phase, &guard) {
                    OkAction::ReturnOk(value)
                } else {
                    OkAction::Continue(Ok(value))
                }
            }
            Err(error) => OkAction::Continue(Err(error)),
        }
    }

    #[cfg(test)]
    #[path = "tests/cm743_route_a_falsifiers.rs"]
    mod cm743_route_a_falsifiers;

    impl Drop for Guard {
        #[inline(never)]
        fn drop(&mut self) {
            let (a, step) = {
                let Some(active) = self.0.as_ref() else {
                    return;
                };
                (active.activation.clone(), active.step)
            };
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                assert!(
                    guard_source_matches(&l, self),
                    "PortBlocked::ActualGuardSource"
                );
                assert_eq!(
                    l.scope_authority.stack.last(),
                    Some(&a),
                    "PortBlocked::ProducerScopeOwner"
                );
                assert_eq!(
                    l.scope_authority.entries.get(&a),
                    Some(&step),
                    "PortBlocked::ProducerScopeEntry"
                );
                assert_eq!(l.scope_authority.stack.pop().as_ref(), Some(&a));
                let actual = l
                    .scope_authority
                    .arming
                    .pop()
                    .expect("actual producer scope");
                assert!(actual.as_ref().is_none_or(|owner| owner == &a));
                assert_eq!(l.scope_authority.entries.remove(&a), Some(step));
                retire_guard_source(&mut l, self);
                assert_eq!(l.stack.pop().as_ref(), Some(&a));
                let scope = l.arming_scopes.pop().expect("producer or deferred scope");
                assert!(scope.as_ref().is_none_or(|owner| owner == &a));
                // Return evidence is transferred only after this activation's
                // exact producer result was checked, or its own actual frame
                // was constructed. The parent still needs its own edge check.
                let parent = owner(&l, &a.episode.row);
                if let Some(run) = &mut l.current {
                    let returned = run
                        .returning
                        .remove(&a)
                        .filter(|token| return_authorized(run, &a, token, ReturnKind::Evaluation))
                        .or_else(|| {
                            run.request_returning.remove(&a).filter(|token| {
                                return_authorized(run, &a, token, ReturnKind::Request)
                            })
                        });
                    take_pending_source(run, &a);
                    if !run.queued {
                        let proof = returned
                            .or_else(|| run.pair.tokens.get(&a).and_then(|ts| ts.last()).cloned());
                        if let (Some(parent), Some(mut token)) = (parent, proof) {
                            token.owner = parent.clone();
                            token.frame_identity = None;
                            if !run.pair.source_records.contains_key(&token) {
                                run.pair
                                    .source_records
                                    .insert(token.clone(), SourceKind::ParentReturn);
                            }
                            issue_pending_source(run, parent, token);
                        }
                    }
                }
            });
        }
    }
    fn owner(l: &Ledger, row: &str) -> Option<Activation> {
        if !lexical_current(l) {
            return None;
        }
        let run = l.current.as_ref()?;
        if !run_current(run) {
            return None;
        }
        let episode = run.pair.episode.as_ref()?;
        if episode.row != row {
            return None;
        }
        l.stack
            .iter()
            .rev()
            .find(|a| a.path.last().is_some_and(|(r, _)| *r == row))
            .filter(|a| &a.episode == episode)
            .cloned()
    }
    // The operation table supplies the issue id. The actual Arc and the recorded
    // pause/parent return supply the source identity independently of Pair::tokens.
    fn issue_queued_frame<F: 'static>(actual: &std::sync::Arc<F>) {
        let key = (
            std::sync::Arc::as_ptr(actual) as usize,
            std::any::type_name::<F>(),
        );
        let operation = OPERATIONS.with_borrow(|ops| {
            let id = *ops.direct.get(&key)?;
            let record = ops.records.get(&id)?;
            (record.kind == OperationKind::Direct(key) && !ops.retired.contains(&id))
                .then(|| (id, record.token.clone()))
        });
        let Some((id, recorded)) = operation else {
            return;
        };
        LEDGER.with_borrow_mut(|ledger| {
            let Some(run) = ledger.current.as_mut() else {
                return;
            };
            if run.queued || !run_current(run) || recorded.frame_identity != Some(key) {
                return;
            }
            let mut lookup = recorded;
            lookup.frame_identity = None;
            let Some((source, kind)) = run.pair.source_records.get_key_value(&lookup) else {
                return;
            };
            if matches!(kind, SourceKind::Control) {
                return;
            }
            let mut receipt = source.clone();
            receipt.frame_identity = Some(key);
            let owner = receipt.owner.clone();
            run.pair.frame_permits.insert(receipt.clone(), id);
            ledger.queued_frame_issues.insert((owner, id), receipt);
        });
    }

    fn issued_queued_frame(ledger: &Ledger, actual: &Activation, proposed: &Token) -> bool {
        let Some(run) = ledger.current.as_ref() else {
            return false;
        };
        let Some(id) = run.pair.frame_permits.get(proposed) else {
            return false;
        };
        run.queued
            && run_current(run)
            && run.pair.episode.as_ref() == Some(&actual.episode)
            && proposed.owner == *actual
            && ledger.queued_frame_issues.get(&(actual.clone(), *id)) == Some(proposed)
    }

    fn eligible(l: &Ledger, row: &str, queued: bool) -> Option<Token> {
        if !ENABLED {
            return None;
        }
        let run = l.current.as_ref()?;
        if run.queued != queued {
            return None;
        }
        let activation = owner(l, row)?;
        let token = if queued {
            run.pair.tokens.get(&activation)?.last()?
        } else {
            l.resuming.last()?.as_ref()?
        };
        (token.owner == activation
            && token.frame_identity.is_some()
            && (if queued {
                issued_queued_frame(l, &activation, token)
            } else {
                authenticated_parent(l, token)
            }))
        .then(|| token.clone())
    }
    pub fn selected(row: &str) -> bool {
        LEDGER.with(|cell| {
            cell.borrow()
                .current
                .as_ref()
                .and_then(|r| r.pair.episode.as_ref())
                .is_some_and(|e| e.row == row)
        })
    }
    pub fn ordinary(row: &str) -> bool {
        if !ENABLED {
            return false;
        }
        LEDGER.with(|c| eligible(&c.borrow(), row, true).is_some())
    }
    pub fn frame(row: &str) -> bool {
        if !ENABLED {
            return false;
        }
        LEDGER.with(|c| eligible(&c.borrow(), row, false).is_some())
    }
    // Q76: a consumer that reselects an ambient permission authenticates it against
    // its independent local record at the omission boundary; the ambient copy is
    // read-only and supplies no authority here.
    fn local_source_token(l: &Ledger, row: &str) -> Option<Token> {
        if !ENABLED {
            return None;
        }
        let run = l.current.as_ref()?;
        if !run.queued || !run_current(run) {
            return None;
        }
        let activation = owner(l, row)?;
        let episode = run.pair.episode.as_ref()?;
        let ((_, id), receipt) = l
            .queued_frame_issues
            .range((activation.clone(), u64::MIN)..=(activation.clone(), u64::MAX))
            .next_back()?;
        let mut source = receipt.clone();
        source.frame_identity = None;
        (receipt.owner == activation
            && receipt.frame_identity.is_some()
            && &receipt.owner.episode == episode
            && run.pair.frame_permits.get(receipt) == Some(id)
            && run.pair.source_records.contains_key(&source))
        .then(|| receipt.clone())
    }
    pub fn hit(row: &str, phase: &str) {
        if !class_lease::authorize_existing_hit(row) {
            return;
        }
        LEDGER.with(|cell| {
        let mut l = cell.borrow_mut();
        let queued = l.current.as_ref().expect("active run").queued;
        let token = initial_token(&l, row).or_else(|| local_source_token(&l, row))
            .or_else(|| eligible(&l, row, queued))
            .or_else(|| request_token(&l, row)).expect("hit without own activation token");
        let run = l.current.as_mut().unwrap();
        let n = run.hits.entry((token.owner.clone(), token.occurrence)).or_default();
        eprintln!("CM740_SOURCE_HIT row={row} phase={phase} selector={row}");
        eprintln!("CM742_HIT_ID program={:?} pair={} activation={:?} producer_occurrence={} hit_occurrence={}",
            token.owner.episode.program, token.owner.episode.pair, token.owner.path, token.occurrence, n);
        *n += 1;
    });
    }
    fn current_initial_return(run: &Run, actual: &Activation, token: &Token) -> bool {
        run_current(run)
            && token.owner == *actual
            && token.origin.episode == actual.episode
            && run.pair.episode.as_ref() == Some(&actual.episode)
            && run.pair.current_initial_returns.get(&InitialReturnKey {
                source: token.clone(),
                edge: InitialReturnEdge::ActualInputRequired,
            }) == Some(&true)
    }
    fn current_request_return(run: &Run, actual: &Activation, token: &Token) -> bool {
        run_current(run)
            && token.owner == *actual
            && token.origin.episode == actual.episode
            && run.pair.episode.as_ref() == Some(&actual.episode)
            && run.pair.current_request_returns.get(&RequestReturnKey {
                source: token.clone(),
                edge: RequestReturnEdge::ActualInputRequired,
            }) == Some(&true)
    }
    fn source_authorized(run: &Run, actual: &Activation, token: &Token) -> bool {
        run_current(run)
            && !run.queued
            && &token.owner == actual
            && token.frame_identity.is_none()
            && run.pair.episode.as_ref() == Some(&actual.episode)
            && token.origin.episode == actual.episode
            && run.pair.source_records.contains_key(token)
    }
    fn return_authorized(run: &Run, actual: &Activation, token: &Token, kind: ReturnKind) -> bool {
        source_authorized(run, actual, token)
            && (kind != ReturnKind::Request || current_request_return(run, actual, token))
            && (kind != ReturnKind::Evaluation || current_initial_return(run, actual, token))
            && run
                .pair
                .return_records
                .iter()
                .any(|(owner, recorded_kind, proof)| {
                    owner == actual && *recorded_kind == kind && proof == token
                })
    }
    fn current_pending_source(run: &Run, token: &Token) -> bool {
        source_authorized(run, &token.owner, token)
            && run.current_pending_sources.get(token) == Some(&true)
    }
    fn retire_pending_source(run: &mut Run, actual: &Activation) {
        for (token, current) in &mut run.current_pending_sources {
            if token.owner == *actual {
                *current = false;
            }
        }
    }
    fn take_pending_source(run: &mut Run, actual: &Activation) -> Option<Token> {
        let token = run
            .pending
            .remove(actual)
            .filter(|token| token.owner == *actual && current_pending_source(run, token));
        retire_pending_source(run, actual);
        token
    }
    fn issue_pending_source(run: &mut Run, actual: Activation, token: Token) {
        assert_eq!(token.owner, actual);
        assert!(source_authorized(run, &actual, &token));
        retire_pending_source(run, &actual);
        assert!(
            !run.current_pending_sources.contains_key(&token),
            "PortBlocked::RepeatedPendingSourceKey"
        );
        run.current_pending_sources.insert(token.clone(), true);
        run.pending.insert(actual, token);
    }
    fn producer_owner(l: &Ledger, row: &str) -> Option<Activation> {
        if !lexical_current(l) {
            return None;
        }
        if !l.current.as_ref().is_some_and(run_current) {
            return None;
        }
        let episode = l.current.as_ref()?.pair.episode.as_ref()?;
        if episode.row != row {
            return None;
        }
        l.arming_scopes
            .iter()
            .rev()
            .take_while(|scope| scope.is_some())
            .filter_map(Option::as_ref)
            .find(|a| a.path.last().is_some_and(|(r, _)| *r == row) && &a.episode == episode)
            .cloned()
    }
    fn arm(row: &str, phase: &str, prompt: &str) {
        arm_source(row, phase, prompt, true);
    }

    fn arm_source(row: &str, phase: &str, prompt: &str, emit: bool) {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let Some(activation) = producer_owner(&l, row) else {
                return;
            };
            let run = l.current.as_mut().unwrap();
            if run.queued {
                return;
            }
            let occurrence = run.pair.next_occurrence;
            run.pair.next_occurrence = occurrence.checked_add(1).expect("occurrence overflow");
            let token = Token {
                owner: activation.clone(),
                origin: activation.clone(),
                occurrence,
                frame_identity: None,
            };
            if phase == "producer-InputRequired" {
                identity_proofs::source_pause(&activation, occurrence);
            }
            let source_kind = match phase {
                "producer-InputRequired" => SourceKind::ProducerPause,
                "owned-returned-InputRequired" => SourceKind::OwnedReturn,
                _ => SourceKind::Control,
            };
            assert!(
                !run.pair.source_records.contains_key(&token),
                "duplicate source identity"
            );
            run.pair.source_records.insert(token.clone(), source_kind);
            issue_pending_source(run, activation, token);
            if emit {
                emit_armed(row, phase, prompt);
            }
            q25::note_successful_arm(row, phase, prompt);
        });
    }

    fn emit_armed(row: &str, phase: &str, prompt: &str) {
        eprintln!("CM740_SELECTOR_ARMED row={row} phase={phase} prompt={prompt:?}");
    }
    pub fn pause(prompt: &str) {
        let suppress = q25::ordinary_suppresses_arm();
        if q25::observing() {
            q25::note_physical_pause(prompt);
        }
        if suppress {
            return;
        }
        SELECTED.with(|row| arm(row, "producer-InputRequired", prompt));
    }
    pub fn returned(row: &'static str, prompt: &str) {
        arm(row, "owned-returned-InputRequired", prompt);
    }
    pub fn arm_control(row: &'static str, phase: &str) {
        arm(row, phase, "");
    }
    pub fn observe(row: &str) -> bool {
        selected(row) && LEDGER.with(|c| c.borrow().current.as_ref().is_some_and(|r| !r.queued))
    }

    // Identity is attached to the actual Arc<F>, without changing its layout or
    // allocation accounting. Drop removes the address before it can be reused.
    // Independent source objects for copied instrumentation scopes. Received
    // indices select records; only the actual object's Rc authorizes a write.
    pub fn source_pause(prompt: &str) {
        if !run_is_driven() {
            return;
        }
        pause(prompt);
    }

    pub fn source_enter_selected(row: &'static str) -> Guard {
        if !run_is_driven() {
            return Guard(None);
        }
        enter_selected(row)
    }

    mod scope_object {
        use std::{
            cell::{Cell, RefCell},
            collections::BTreeMap,
            rc::Rc,
        };
        #[derive(Debug)]
        pub struct Identity {
            kind: &'static str,
            slot: u64,
            current: Cell<bool>,
        }
        struct Entry {
            identity: Rc<Identity>,
            binding: String,
        }
        thread_local! {
            static SOURCES: RefCell<BTreeMap<(&'static str, u64), Entry>> = RefCell::new(BTreeMap::new());
        }
        pub struct Object(pub Rc<Identity>);
        #[cfg(test)]
        pub(super) fn current(identity: &Rc<Identity>) -> bool {
            identity.current.get()
        }
        #[cfg(test)]
        pub(super) fn snapshot() -> String {
            SOURCES.with(|cell| {
                format!(
                    "{:?}",
                    cell.borrow()
                        .iter()
                        .map(|(key, e)| (
                            key,
                            Rc::as_ptr(&e.identity) as usize,
                            e.identity.kind,
                            e.identity.slot,
                            e.identity.current.get(),
                            &e.binding
                        ))
                        .collect::<Vec<_>>()
                )
            })
        }

        fn same_object(object: &Object, entry: &Entry) -> bool {
            Rc::ptr_eq(&object.0, &entry.identity)
        }
        pub fn mint(kind: &'static str, slot: u64, binding: String) -> Object {
            let identity = Rc::new(Identity {
                kind,
                slot,
                current: Cell::new(true),
            });
            SOURCES.with(|cell| {
                assert!(
                    !cell.borrow().contains_key(&(kind, slot)),
                    "PortBlocked::DuplicateActualScope"
                );
                cell.borrow_mut().insert(
                    (kind, slot),
                    Entry {
                        identity: identity.clone(),
                        binding,
                    },
                );
            });
            Object(identity)
        }
        impl Object {
            pub fn matches(&self, kind: &'static str, slot: u64, binding: &str) -> bool {
                SOURCES.with(|cell| {
                    let sources = cell.borrow();
                    let Some(entry) = sources.get(&(kind, slot)) else {
                        return false;
                    };
                    self.0.current.get()
                        && entry.identity.current.get()
                        && entry.identity.kind == kind
                        && entry.identity.slot == slot
                        && entry.binding == binding
                        && same_object(self, entry)
                })
            }
            pub fn check(&self, kind: &'static str, slot: u64, binding: &str) {
                assert!(
                    self.matches(kind, slot, binding),
                    "PortBlocked::ActualScopeObject"
                );
            }
            pub fn update(&self, kind: &'static str, slot: u64, before: &str, after: String) {
                self.check(kind, slot, before);
                SOURCES
                    .with(|cell| cell.borrow_mut().get_mut(&(kind, slot)).unwrap().binding = after);
            }
            pub fn retire(&self, kind: &'static str, slot: u64, binding: &str) {
                self.check(kind, slot, binding);
                SOURCES.with(|cell| {
                    let entry = cell.borrow_mut().remove(&(kind, slot)).unwrap();
                    entry.identity.current.set(false);
                });
            }
        }
    }
    fn construction_scope_binding(l: &Ledger, slot: usize) -> String {
        format!(
            "{:?}",
            (
                slot,
                l.scope_authority.constructing.len(),
                l.scope_authority.constructing.last(),
                l.current.as_ref().and_then(|run| run.pair.episode.as_ref())
            )
        )
    }
    fn new_construction_guard(l: &Ledger) -> ConstructionGuard {
        let slot = l.scope_authority.constructing.len();
        ConstructionGuard(
            slot,
            scope_object::mint(
                "construction",
                slot as u64,
                construction_scope_binding(l, slot),
            ),
        )
    }
    fn deferred_scope_binding(l: &Ledger, slot: usize) -> String {
        format!(
            "{:?}",
            (
                slot,
                l.scope_authority.provenance.len(),
                l.scope_authority.provenance.last(),
                l.current.as_ref().and_then(|run| run.pair.episode.as_ref())
            )
        )
    }
    fn new_deferred_guard(l: &Ledger) -> DeferredEvaluationGuard {
        let slot = l.scope_authority.provenance.len();
        DeferredEvaluationGuard(
            slot,
            scope_object::mint("deferred", slot as u64, deferred_scope_binding(l, slot)),
        )
    }
    fn resume_scope_binding(l: &Ledger, depth: usize) -> String {
        format!(
            "{:?}",
            (
                depth,
                l.scope_authority.resume_depths.last(),
                l.scope_authority.resuming.last(),
                l.scope_authority.provenance.last(),
                l.current.as_ref().and_then(|run| run.pair.episode.as_ref())
            )
        )
    }
    fn profile_construction_binding(l: &Ledger, id: u64) -> String {
        let step = l.profile.steps.get(&id);
        format!(
            "{:?}",
            (
                id,
                l.profile.constructing.last(),
                step.map(|s| (&s.episode, s.resume, s.constructing))
            )
        )
    }
    fn profile_finish_binding(l: &Ledger, id: u64, key: ProfileKey) -> String {
        let step = l.profile.steps.get(&id);
        format!(
            "{:?}",
            (
                id,
                key,
                l.profile.finishing.last(),
                step.map(|s| (&s.episode, s.owner, s.finished_duplicate))
            )
        )
    }
    fn operation_construction_binding(o: &Operations, slot: usize) -> String {
        format!(
            "{:?}",
            (
                slot,
                o.current_construction_scopes.len(),
                o.current_construction_scopes.last(),
                o.constructing.last().map(|(op, target, _)| (op, target))
            )
        )
    }
    fn operation_call_binding(o: &Operations, ticket: u64) -> String {
        format!(
            "{:?}",
            (
                ticket,
                o.call_scopes.last(),
                o.call_records
                    .get(&ticket)
                    .map(|r| (&r.key, r.operation, &r.token))
            )
        )
    }
    pub struct ConstructionGuard(usize, scope_object::Object);
    impl Guard {
        // Generated only in this producer's actual InputRequired branch, while
        // this guard is still alive, around its unchanged witness/frame call.
        pub fn suspending(&self) -> ConstructionGuard {
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                assert!(
                    guard_source_matches(&l, self),
                    "PortBlocked::ActualGuardSource"
                );
                let token = self
                    .0
                    .as_ref()
                    .map(|active| &active.activation)
                    .and_then(|a| {
                        assert_eq!(l.stack.last(), Some(a), "wrong construction owner");
                        l.current.as_ref()?.pending.get(a).cloned()
                    });
                l.scope_authority.constructing.push(
                    token
                        .clone()
                        .map(ConstructionIdentity::Pending)
                        .unwrap_or(ConstructionIdentity::Absent),
                );
                l.constructing.push(
                    token
                        .map(ConstructionIdentity::Pending)
                        .unwrap_or(ConstructionIdentity::Absent),
                );
            });
            LEDGER.with(|cell| new_construction_guard(&cell.borrow()))
        }
    }
    impl Drop for ConstructionGuard {
        fn drop(&mut self) {
            LEDGER.with(|c| {
                let mut l = c.borrow_mut();
                let binding = construction_scope_binding(&l, self.0);
                self.1.check("construction", self.0 as u64, &binding);
                l.scope_authority
                    .constructing
                    .pop()
                    .expect("actual construction scope");
                l.constructing.pop().expect("construction scope");
                self.1.retire("construction", self.0 as u64, &binding);
            });
        }
    }
    // Q33: absence can inherit; invalid present evidence cannot.
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum ConstructionIdentity {
        Absent,
        Pending(Token),
        Deferred(Token),
        InvalidPresent,
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum EvaluationProvenance {
        Resume(Option<Token>),
        Deferred(Option<Token>),
        InvalidPresent,
    }
    enum AuthenticatedIdentity {
        Absent,
        Valid(Token),
        InvalidPresent,
    }
    fn construction_episode(l: &Ledger, token: &Token) -> bool {
        l.current.as_ref().is_some_and(|run| {
            run_current(run)
                && !run.queued
                && run.pair.episode.as_ref() == Some(&token.owner.episode)
                && token.origin.episode == token.owner.episode
        })
    }
    fn pending_construction(l: &Ledger, token: &Token) -> bool {
        construction_episode(l, token)
            && token.frame_identity.is_none()
            && l.current.as_ref().is_some_and(|run| {
                run.pending.get(&token.owner) == Some(token) && current_pending_source(run, token)
            })
    }
    fn current_frame_construction(l: &Ledger, token: &Token) -> bool {
        l.current_frame_records.get(token) == Some(&true)
    }
    fn authenticated_parent(l: &Ledger, token: &Token) -> bool {
        construction_episode(l, token)
            && current_frame_construction(l, token)
            && token
                .frame_identity
                .is_some_and(|key| l.bound_slots.get(&key) == Some(token))
            && l.current
                .as_ref()
                .and_then(|run| run.pair.tokens.get(&token.owner))
                .is_some_and(|proofs| proofs.iter().any(|proof| proof == token))
    }
    fn enclosing_construction(l: &Ledger) -> AuthenticatedIdentity {
        if !lexical_current(l) {
            return AuthenticatedIdentity::InvalidPresent;
        }
        for provenance in l.evaluation_provenance.iter().rev() {
            let valid = match provenance {
                // An unbound resume is a boundary, even with an older armed owner.
                EvaluationProvenance::Resume(None) => return AuthenticatedIdentity::Absent,
                EvaluationProvenance::Resume(Some(token)) => {
                    (token, authenticated_parent(l, token))
                }
                // A missing deferred token is absence; an invalid one is a veto.
                EvaluationProvenance::Deferred(None) => continue,
                EvaluationProvenance::Deferred(Some(token)) => {
                    (token, pending_construction(l, token))
                }
                EvaluationProvenance::InvalidPresent => {
                    return AuthenticatedIdentity::InvalidPresent;
                }
            };
            return if valid.1 {
                AuthenticatedIdentity::Valid(valid.0.clone())
            } else {
                AuthenticatedIdentity::InvalidPresent
            };
        }
        AuthenticatedIdentity::Absent
    }
    fn construction_identity(l: &Ledger) -> AuthenticatedIdentity {
        if !lexical_current(l) {
            return AuthenticatedIdentity::InvalidPresent;
        }
        match l.constructing.last() {
            Some(ConstructionIdentity::Pending(token) | ConstructionIdentity::Deferred(token)) => {
                if pending_construction(l, token) {
                    AuthenticatedIdentity::Valid(token.clone())
                } else {
                    AuthenticatedIdentity::InvalidPresent
                }
            }
            Some(ConstructionIdentity::InvalidPresent) => AuthenticatedIdentity::InvalidPresent,
            Some(ConstructionIdentity::Absent) | None => enclosing_construction(l),
        }
    }
    // The adapter supplies its live allocation separately from the proposed address/type.
    // An inactive construction has no records to issue, including on a rejected transport.
    pub fn bind_allocated_frame<F: 'static>(
        actual: &std::sync::Arc<F>,
        proposed: (usize, &'static str),
    ) {
        let physical = (
            std::sync::Arc::as_ptr(actual) as usize,
            std::any::type_name::<F>(),
        );
        if proposed != physical {
            return;
        }
        bind_frame(physical.0, physical.1);
        issue_queued_frame(actual);
    }

    pub fn bind_frame(key: usize, kind: &'static str) {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let AuthenticatedIdentity::Valid(mut token) = construction_identity(&l) else {
                return;
            };
            // No child-owner rebase is admitted without its actual source route.
            assert_eq!(
                l.stack.last(),
                Some(&token.owner),
                "PortBlocked::UnprovedChildConstructionOwner"
            );
            token.frame_identity = Some((key, kind));
            assert!(
                !l.bound_slots.contains_key(&(key, kind)),
                "PortBlocked::DuplicateLiveFrameConstruction"
            );
            let run = l
                .current
                .as_mut()
                .expect("frame construction in active run");
            if run.queued {
                return;
            }
            operation_constructed(key, kind, &token);
            run.pair
                .tokens
                .entry(token.owner.clone())
                .or_default()
                .push(token.clone());
            assert_ne!(
                l.current_frame_records.get(&token),
                Some(&true),
                "PortBlocked::DuplicateCurrentFrame"
            );
            l.current_frame_records.insert(token.clone(), true);
            assert!(l.bound_slots.insert((key, kind), token).is_none());
        });
    }

    pub struct ResumeGuard(usize, usize, scope_object::Object);
    pub fn resume_frame(key: usize, kind: &'static str) -> ResumeGuard {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let old = l.stack.len();
            let actual_depth = l.scope_authority.stack.len();
            l.scope_authority.resume_depths.push((old, actual_depth));
            let received = l.bound_slots.get(&(key, kind)).cloned();
            let saved = received
                .as_ref()
                .filter(|token| {
                    lexical_current(&l)
                        && token.frame_identity == Some((key, kind))
                        && authenticated_parent(&l, token)
                })
                .cloned();
            let invalid_present = received.is_some() && saved.is_none();
            if let Some(token) = &saved {
                assert_eq!(
                    Some(&token.owner.episode),
                    l.current.as_ref().and_then(|r| r.pair.episode.as_ref()),
                    "stale frame episode"
                );
                l.scope_authority.stack.push(token.owner.clone());
                l.stack.push(token.owner.clone());
            }
            l.scope_authority.provenance.push(if invalid_present {
                EvaluationProvenance::InvalidPresent
            } else {
                EvaluationProvenance::Resume(saved.clone())
            });
            l.scope_authority.resuming.push(saved.clone());
            l.scope_authority.arming.push(None);
            l.evaluation_provenance.push(if invalid_present {
                EvaluationProvenance::InvalidPresent
            } else {
                EvaluationProvenance::Resume(saved.clone())
            });
            l.resuming.push(saved);
            l.arming_scopes.push(None);
            let slot = l.scope_authority.resume_depths.len();
            ResumeGuard(
                old,
                slot,
                scope_object::mint("resume", slot as u64, resume_scope_binding(&l, old)),
            )
        })
    }
    impl Drop for ResumeGuard {
        fn drop(&mut self) {
            LEDGER.with(|c| {
                let mut l = c.borrow_mut();
                let binding = resume_scope_binding(&l, self.0);
                self.2.check("resume", self.1 as u64, &binding);
                assert_eq!(
                    self.1,
                    l.scope_authority.resume_depths.len(),
                    "PortBlocked::ActualResumeSlot"
                );
                assert_eq!(
                    l.scope_authority.resume_depths.last().map(|entry| entry.0),
                    Some(self.0),
                    "PortBlocked::ResumeScopeDepth"
                );
                let (transport_depth, actual_depth) = l
                    .scope_authority
                    .resume_depths
                    .pop()
                    .expect("actual resume scope");
                assert_eq!(transport_depth, self.0, "PortBlocked::ResumeScopeDepth");
                l.scope_authority.stack.truncate(actual_depth);
                l.scope_authority
                    .resuming
                    .pop()
                    .expect("actual resume token");
                l.scope_authority
                    .provenance
                    .pop()
                    .expect("actual resume provenance");
                assert!(
                    l.scope_authority
                        .arming
                        .pop()
                        .expect("actual resume arming")
                        .is_none()
                );
                l.stack.truncate(self.0);
                l.resuming.pop().expect("resume scope");
                assert!(matches!(
                    l.evaluation_provenance.pop(),
                    Some(EvaluationProvenance::Resume(_) | EvaluationProvenance::InvalidPresent)
                ));
                assert!(
                    l.arming_scopes
                        .pop()
                        .expect("resume arming boundary")
                        .is_none()
                );
                self.2.retire("resume", self.1 as u64, &binding);
            });
        }
    }
    // The adapter supplies its own Arc. The proposed address only selects a
    // construction record; it never identifies the object being abandoned.
    pub fn retirement_for_adapter<F: 'static>(
        actual: &std::sync::Arc<F>,
        proposed: (usize, &'static str),
    ) -> Option<(usize, &'static str)> {
        let actual_key = (
            std::sync::Arc::as_ptr(actual) as usize,
            std::any::type_name::<F>(),
        );
        OPERATIONS.with(|cell| {
            let operations = cell.borrow();
            let id = operations.direct.get(&proposed)?;
            let record = operations.records.get(id)?;
            if proposed != actual_key
                || record.kind != OperationKind::Direct(actual_key)
                || record.token.frame_identity != Some(actual_key)
            {
                return None;
            }
            LEDGER.with(|cell| {
                let ledger = cell.borrow();
                (ledger.current_frame_records.get(&record.token) == Some(&true))
                    .then_some(actual_key)
            })
        })
    }

    // Each sibling's proposal is checked against the live adapter before the
    // first effect. The ordinary frame retirement invalidates the construction
    // record needed by the final direct-claim cleanup.
    pub fn live_teardown_for_adapter<F: 'static>(
        actual: &std::sync::Arc<F>,
        proposed: (usize, &'static str),
    ) -> Option<(usize, &'static str)> {
        let key = retirement_for_adapter(actual, proposed)?;
        OPERATIONS.with(|cell| {
            let operations = cell.borrow();
            let id = operations.direct.get(&key)?;
            (!operations.retired.contains(id)).then_some(key)
        })
    }

    pub fn forget_frame(key: usize, kind: &'static str) {
        LEDGER.with(|c| {
            let mut l = c.borrow_mut();
            for (record, current) in &mut l.current_frame_records {
                if record.frame_identity == Some((key, kind)) {
                    *current = false;
                }
            }
            l.bound_slots.remove(&(key, kind));
        });
        retire_operation_frame(key, kind);
    }

    // The source audit admits this only for the existing Box<AwaitCursor>.
    // Ownership leaves the active call stack while the same box moves to the
    // mandatory constructor. It is consumed there, before the box is destroyed.
    impl Guard {
        pub fn defer<T>(mut self, cursor: &T) {
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                let (a, step) = {
                    let Some(active) = self.0.as_ref() else {
                        return;
                    };
                    (active.activation.clone(), active.step)
                };
                assert!(
                    guard_source_matches(&l, &self),
                    "PortBlocked::ActualGuardSource"
                );
                assert_eq!(l.stack.last(), Some(&a), "deferred owner is not current");
                let token = l.current.as_ref().and_then(|r| r.pending.get(&a)).cloned();
                let Some(token) = token else {
                    return;
                };
                if !pending_construction(&l, &token) {
                    return;
                }
                let key = (cursor as *const T as usize, std::any::type_name::<T>());
                assert!(
                    !l.deferred_proofs.contains_key(&key) && !l.deferred.contains_key(&key),
                    "PortBlocked::DuplicateLiveDeferredConstruction"
                );
                assert!(
                    l.deferred_proofs.insert(key, token.clone()).is_none(),
                    "unconsumed cursor proof"
                );
                assert!(
                    l.deferred.insert(key, token).is_none(),
                    "unconsumed cursor binding"
                );
                assert_eq!(l.scope_authority.arming.pop().flatten().as_ref(), Some(&a));
                assert_eq!(l.scope_authority.stack.pop().as_ref(), Some(&a));
                assert_eq!(l.scope_authority.entries.remove(&a), Some(step));
                retire_guard_source(&mut l, &self);
                assert_eq!(l.arming_scopes.pop().flatten().as_ref(), Some(&a));
                assert_eq!(l.stack.pop(), self.0.take().map(|active| active.activation));
            });
        }
    }
    pub struct DeferredEvaluationGuard(usize, scope_object::Object);
    impl Drop for DeferredEvaluationGuard {
        fn drop(&mut self) {
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                let binding = deferred_scope_binding(&l, self.0);
                self.1.check("deferred", self.0 as u64, &binding);
                l.scope_authority
                    .provenance
                    .pop()
                    .expect("actual deferred provenance");
                assert!(matches!(
                    l.evaluation_provenance.pop(),
                    Some(EvaluationProvenance::Deferred(_) | EvaluationProvenance::InvalidPresent)
                ));
                self.1.retire("deferred", self.0 as u64, &binding);
            });
        }
    }
    pub struct DeferredConstruction {
        // Retire the construction and provenance before its source owner.
        _scope: ConstructionGuard,
        _evaluation: DeferredEvaluationGuard,
        _owner: Guard,
    }
    pub fn construct_deferred<T>(cursor: &T) -> DeferredConstruction {
        let owner = LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let key = (cursor as *const T as usize, std::any::type_name::<T>());
            let token = l.deferred.remove(&key);
            let proof = l.deferred_proofs.remove(&key);
            let valid = token.as_ref().is_some_and(|token| {
                proof.as_ref() == Some(token) && pending_construction(&l, token)
            });
            let activation = if valid {
                token.as_ref().map(|t| t.owner.clone())
            } else {
                None
            };
            if let Some(a) = &activation {
                assert!(l.scope_authority.entries.insert(a.clone(), None).is_none());
                l.scope_authority.stack.push(a.clone());
                l.scope_authority.arming.push(None);
                l.stack.push(a.clone());
                l.arming_scopes.push(None);
            }
            let (construction, provenance) = match token {
                Some(token) if valid => (
                    ConstructionIdentity::Deferred(token.clone()),
                    EvaluationProvenance::Deferred(Some(token)),
                ),
                Some(_) => (
                    ConstructionIdentity::InvalidPresent,
                    EvaluationProvenance::InvalidPresent,
                ),
                None => (
                    ConstructionIdentity::Absent,
                    EvaluationProvenance::Deferred(None),
                ),
            };
            l.scope_authority.constructing.push(construction.clone());
            l.scope_authority.provenance.push(provenance.clone());
            l.constructing.push(construction);
            l.evaluation_provenance.push(provenance);
            let source = activation
                .as_ref()
                .map(|a| mint_guard_source(&mut l, a, None));
            match (activation, source) {
                (Some(activation), Some(source)) => Guard(Some(Box::new(GuardActive {
                    activation,
                    step: None,
                    source,
                }))),
                (None, None) => Guard(None),
                _ => panic!("PortBlocked::MissingActualGuardSource"),
            }
        });
        let (scope, evaluation) = LEDGER.with(|cell| {
            let l = cell.borrow();
            (new_construction_guard(&l), new_deferred_guard(&l))
        });
        DeferredConstruction {
            _scope: scope,
            _evaluation: evaluation,
            _owner: owner,
        }
    }

    // Q20 initial-return transport. This borrows the actual producer Result;
    // emission/audit require that binding, never a boolean or a fabricated error.
    impl Guard {
        pub fn input_return<T>(&self, result: &Result<T, crate::EvalError>) {
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                let Some(active) = self.0.as_ref() else {
                    return;
                };
                let a = &active.activation;
                let step = active.step;
                assert_eq!(l.stack.last(), Some(a), "wrong return-edge owner");
                if !guard_source_matches(&l, self) || !guard_current(&l, a, step) {
                    return;
                }
                let run = l.current.as_mut().expect("return edge in active run");
                assert_eq!(
                    run.pair.episode.as_ref(),
                    Some(&a.episode),
                    "stale return edge"
                );
                let token = take_pending_source(run, a);
                run.returning.remove(a);
                for (key, current) in &mut run.pair.current_initial_returns {
                    if key.source.owner == *a {
                        *current = false;
                    }
                }
                if !run.queued && matches!(result, Err(crate::EvalError::InputRequired { .. })) {
                    if let Some(token) = token {
                        assert_eq!(&token.owner, a, "misattributed return edge");
                        if !source_authorized(run, a, &token) {
                            return;
                        }
                        let key = InitialReturnKey {
                            source: token.clone(),
                            edge: InitialReturnEdge::ActualInputRequired,
                        };
                        assert!(
                            !run.pair.current_initial_returns.contains_key(&key),
                            "PortBlocked::RepeatedInitialReturnKey"
                        );
                        run.pair.current_initial_returns.insert(key, true);
                        run.pair.return_records.push((
                            a.clone(),
                            ReturnKind::Evaluation,
                            token.clone(),
                        ));
                        run.returning.insert(a.clone(), token);
                    }
                }
            });
        }
    }
    fn initial_token(l: &Ledger, row: &str) -> Option<Token> {
        if !ENABLED {
            return None;
        }
        let run = l.current.as_ref()?;
        if run.queued {
            return None;
        }
        let activation = owner(l, row)?;
        if l.stack.last() != Some(&activation) {
            return None;
        }
        let token = run.returning.get(&activation)?;
        (return_authorized(run, &activation, token, ReturnKind::Evaluation)).then(|| token.clone())
    }
    pub fn initial(row: &str) -> bool {
        if !ENABLED {
            return false;
        }
        LEDGER.with(|c| initial_token(&c.borrow(), row).is_some())
    }

    // Q21 pair transport. Only this typed, source-bound request boundary records
    // this proof kind. Q20 eval returns remain unqueued-only; Q19 frames retain
    // their own separate construction map and resume permission.
    impl Guard {
        pub fn request_return<T>(&self, result: &Result<T, crate::SessionRequestFailure>) {
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                let Some(active) = self.0.as_ref() else {
                    return;
                };
                let a = &active.activation;
                let step = active.step;
                assert_eq!(l.stack.last(), Some(a), "wrong request-return owner");
                if !guard_source_matches(&l, self) || !guard_current(&l, a, step) {
                    return;
                }
                let run = l.current.as_mut().expect("request return in active run");
                assert_eq!(
                    run.pair.episode.as_ref(),
                    Some(&a.episode),
                    "stale request return"
                );
                // A queued execution consumes the predecessor's proof at begin;
                // its successful result must not overwrite that predecessor.
                if run.queued {
                    return;
                }
                let token = take_pending_source(run, a);
                run.request_returning.remove(a);
                run.pair.request_returns.remove(a);
                for (key, current) in &mut run.pair.current_request_returns {
                    if key.source.owner == *a {
                        *current = false;
                    }
                }
                if matches!(
                    result,
                    Err(crate::SessionRequestFailure {
                        error: crate::SessionRequestError::InputRequired { .. },
                        ..
                    })
                ) {
                    if let Some(mut token) = token {
                        assert_eq!(&token.owner, a, "misattributed request return");
                        if !source_authorized(run, a, &token) {
                            return;
                        }
                        token.frame_identity = None;
                        let key = RequestReturnKey {
                            source: token.clone(),
                            edge: RequestReturnEdge::ActualInputRequired,
                        };
                        assert!(
                            !run.pair.current_request_returns.contains_key(&key),
                            "PortBlocked::RepeatedRequestReturnKey"
                        );
                        run.pair.current_request_returns.insert(key, true);
                        run.pair.return_records.push((
                            a.clone(),
                            ReturnKind::Request,
                            token.clone(),
                        ));
                        run.pair.request_records.push((a.clone(), token.clone()));
                        run.pair.request_returns.insert(a.clone(), token.clone());
                        run.request_returning.insert(a.clone(), token);
                    }
                }
            });
        }
    }
    fn request_token(l: &Ledger, row: &str) -> Option<Token> {
        if !ENABLED {
            return None;
        }
        let run = l.current.as_ref()?;
        if !run.queued {
            return None;
        }
        let activation = owner(l, row)?;
        if l.stack.last() != Some(&activation) {
            return None;
        }
        let token = run.pair.request_returns.get(&activation)?;
        (token.owner == activation
            && token.frame_identity.is_none()
            && run.pair.episode.as_ref() == Some(&token.owner.episode)
            && current_request_return(run, &activation, token)
            && run.pair.source_records.contains_key(token)
            && run.pair.return_records.iter().any(|(owner, kind, proof)| {
                owner == &activation && *kind == ReturnKind::Request && proof == token
            })
            && run
                .pair
                .request_records
                .iter()
                .any(|(owner, proof)| owner == &activation && proof == token))
        .then(|| token.clone())
    }
    // The plan fixes this proof kind at the declared queued request omission.
    // ordinary/frame keep requiring an actual constructed-frame proof.
    pub fn ordinary_request(row: &str) -> bool {
        if !ENABLED {
            return false;
        }
        LEDGER.with(|c| request_token(&c.borrow(), row).is_some())
    }

    // Generated-only Segment owners and immutable source/return records.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ProfileKey {
        epoch: u64,
        index: usize,
        segment: u64,
        completion: usize,
        thread: std::thread::ThreadId,
    }
    fn profile_key(epoch: u64, index: usize, segment: u64, completion: usize) -> ProfileKey {
        ProfileKey {
            epoch,
            index,
            segment,
            completion,
            thread: std::thread::current().id(),
        }
    }
    #[derive(Clone)]
    struct SegmentRecord {
        id: u64,
        episode: Episode,
        resume: bool,
        key: ProfileKey,
    }
    #[derive(Clone)]
    struct ProfileReturnRecord {
        id: u64,
        episode: Episode,
        key: ProfileKey,
        token: Token,
    }
    #[derive(Default)]
    struct ProfileState {
        next_step: u64,
        highest_segment: u64,
        // Read-only diagnostic snapshot retained for the fixed Q22 control.
        // Actual scope selection and effect authority use the records below.
        current: Option<ProfileStep>,
        steps: BTreeMap<u64, ProfileStep>,
        segments: BTreeMap<u64, SegmentRecord>,
        returns: BTreeMap<u64, ProfileReturnRecord>,
        active: Vec<u64>,
        constructing: Vec<(u64, Option<u64>)>,
        finishing: Vec<(u64, ProfileKey)>,
    }
    #[derive(Clone)]
    struct ProfileStep {
        id: u64,
        episode: Episode,
        resume: bool,
        constructing: bool,
        owner: Option<ProfileKey>,
        proof: Option<Token>,
        finished_duplicate: Option<ProfileKey>,
        retired: bool,
        consumed: bool,
    }
    impl ProfileState {
        fn sync_current(&mut self) {
            self.current = self
                .active
                .last()
                .and_then(|id| self.steps.get(id))
                .cloned();
        }
        fn retire_step(&mut self, id: u64) {
            if let Some(step) = self.steps.get_mut(&id) {
                step.retired = true;
                step.proof = None;
            }
            self.active.retain(|active| *active != id);
            self.sync_current();
        }
        fn reset(&mut self) {
            assert!(
                self.constructing.is_empty() && self.finishing.is_empty(),
                "live profile scope at run boundary"
            );
            let ids = self.active.clone();
            for id in ids {
                self.retire_step(id);
            }
            self.current = None;
        }
        fn actual_segment(&self, key: ProfileKey) -> Option<u64> {
            let mut found = self.segments.iter().filter(|(id, record)| {
                record.key == key && self.steps.get(*id).is_some_and(|step| !step.retired)
            });
            let id = found.next().map(|(id, _)| *id)?;
            assert!(
                found.next().is_none(),
                "PortBlocked::AmbiguousProfileSegment"
            );
            Some(id)
        }
    }
    pub struct ProfileConstruction(Option<u64>, Option<scope_object::Object>);
    pub fn profile_construct(resume: bool) -> ProfileConstruction {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let Some(episode) = l.current.as_ref().and_then(|run| run.pair.episode.clone()) else {
                l.profile.reset();
                return ProfileConstruction(None, None);
            };
            let previous = l.profile.active.last().copied();
            l.profile.next_step = l
                .profile
                .next_step
                .checked_add(1)
                .expect("profile step overflow");
            let id = l.profile.next_step;
            assert!(!l.profile.steps.contains_key(&id), "duplicate profile step");
            l.profile.steps.insert(
                id,
                ProfileStep {
                    id,
                    episode,
                    resume,
                    constructing: true,
                    owner: None,
                    proof: None,
                    finished_duplicate: None,
                    retired: false,
                    consumed: false,
                },
            );
            l.profile.constructing.push((id, previous));
            l.profile.active.push(id);
            l.profile.sync_current();
            ProfileConstruction(
                Some(id),
                Some(scope_object::mint(
                    "profile-construction",
                    id,
                    profile_construction_binding(&l, id),
                )),
            )
        })
    }
    impl Drop for ProfileConstruction {
        fn drop(&mut self) {
            let Some(id) = self.0 else {
                return;
            };
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                let binding = profile_construction_binding(&l, id);
                let object = self.1.as_ref().expect("actual profile construction object");
                object.check("profile-construction", id, &binding);
                assert_eq!(
                    l.profile.constructing.last().map(|scope| scope.0),
                    Some(id),
                    "profile construction scope order"
                );
                let (_, previous) = l.profile.constructing.pop().unwrap();
                let step = l
                    .profile
                    .steps
                    .get_mut(&id)
                    .expect("actual profile construction");
                step.constructing = false;
                let unbound = step.owner.is_none();
                if unbound {
                    l.profile.retire_step(id);
                    if let Some(previous) = previous {
                        l.profile.retire_step(previous);
                    }
                }
                l.profile.sync_current();
                object.retire("profile-construction", id, &binding);
            });
        }
    }
    pub fn profile_bind(epoch: u64, index: usize, segment: u64, completion: usize) {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let Some((id, _)) = l.profile.constructing.last().copied() else {
                return;
            };
            let step = l.profile.steps.get(&id).unwrap();
            assert!(step.owner.is_none(), "ambiguous guard Segment construction");
            if segment <= l.profile.highest_segment {
                l.profile.retire_step(id);
                return;
            }
            let key = profile_key(epoch, index, segment, completion);
            let episode = step.episode.clone();
            let resume = step.resume;
            let previous: Vec<u64> = l
                .profile
                .segments
                .iter()
                .filter_map(|(other, record)| {
                    (record.key.epoch == key.epoch
                        && record.key.index == key.index
                        && record.key.completion == key.completion
                        && record.key.thread == key.thread)
                        .then_some(*other)
                })
                .collect();
            for previous in previous {
                l.profile.retire_step(previous);
            }
            assert!(
                !l.profile.segments.contains_key(&id),
                "duplicate Segment construction record"
            );
            l.profile.segments.insert(
                id,
                SegmentRecord {
                    id,
                    episode,
                    resume,
                    key,
                },
            );
            l.profile.steps.get_mut(&id).unwrap().owner = Some(key);
            l.profile.highest_segment = segment;
            l.profile.sync_current();
        });
    }
    impl Guard {
        pub fn profile_return<T>(&self, result: &Result<T, crate::EvalError>) {
            LEDGER.with(|cell| {
            let mut l=cell.borrow_mut();
            let Some(active)=self.0.as_ref() else{return;};
            let a=&active.activation;
            let guard_step=active.step;
            assert_eq!(l.stack.last(),Some(a),"wrong profiled producer owner");
            if !guard_source_matches(&l,self) || !guard_current(&l,a,guard_step){return;}
            let Some(run)=&l.current else{return;};
            if !ENABLED || run.queued || a.episode.row!="C10.prepared.profiled_consumer"
                || !matches!(result,Err(crate::EvalError::InputRequired {..})) {return;}
            let Some(token)=run.pending.get(a).filter(|token|source_authorized(run,a,token)).cloned() else{return;};
            let Some(id)=guard_step else{return;};
            let Some(record)=l.profile.segments.get(&id).cloned() else{return;};
            let Some(step)=l.profile.steps.get(&id) else{return;};
            if step.id!=record.id || step.episode!=a.episode || record.episode!=a.episode
                || !step.resume || !record.resume || step.constructing || step.retired || step.consumed
                || step.owner!=Some(record.key) {return;}
            assert!(step.proof.is_none() && !l.profile.returns.contains_key(&id),"duplicate profiled producer return");
            l.profile.returns.insert(id,ProfileReturnRecord {id,episode:a.episode.clone(),key:record.key,token:token.clone()});
            l.profile.steps.get_mut(&id).unwrap().proof=Some(token);l.profile.sync_current();
            eprintln!("CM740_SELECTOR_ARMED row=C10.prepared.profiled_consumer phase=profiled-consumer-InputRequired");
        });
        }
    }
    pub fn profile_pause(epoch: u64, index: usize, segment: u64, completion: usize) -> bool {
        LEDGER.with(|cell| {
        let mut l=cell.borrow_mut();let Some(run)=&l.current else{return false;};
        if !ENABLED || run.queued {return false;}
        let key=profile_key(epoch,index,segment,completion);
        let Some(id)=l.profile.actual_segment(key) else{return false;};
        let record=&l.profile.segments[&id];let step=&l.profile.steps[&id];
        if step.id!=id || record.id!=id || Some(&step.episode)!=run.pair.episode.as_ref()
            || step.episode!=record.episode || !step.resume || !record.resume || step.constructing
            || step.retired || step.consumed || step.owner!=Some(key) {return false;}
        let Some(proof)=&step.proof else{return false;};
        let Some(returned)=l.profile.returns.get(&id) else{return false;};
        if returned.id!=id || returned.episode!=record.episode || returned.key!=key || &returned.token!=proof
            || !source_authorized(run,&proof.owner,proof) {return false;}
        l.profile.steps.get_mut(&id).unwrap().consumed=true;l.profile.sync_current();
        eprintln!("CM740_SOURCE_HIT row=C10.prepared.profiled_consumer phase=completed-generation-inert-pause selector=C10.prepared.profiled_consumer");
        true
    })
    }
    pub struct ProfileFinish(Option<(u64, ProfileKey)>, Option<scope_object::Object>);
    pub fn profile_finish(
        epoch: u64,
        index: usize,
        segment: u64,
        completion: usize,
    ) -> ProfileFinish {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let key = profile_key(epoch, index, segment, completion);
            let Some(id) = l.profile.actual_segment(key) else {
                return ProfileFinish(None, None);
            };
            let step = l.profile.steps.get_mut(&id).unwrap();
            assert!(step.finished_duplicate.is_none(), "nested finish duplicate");
            step.finished_duplicate = Some(key);
            l.profile.finishing.push((id, key));
            l.profile.sync_current();
            ProfileFinish(
                Some((id, key)),
                Some(scope_object::mint(
                    "profile-finish",
                    id,
                    profile_finish_binding(&l, id, key),
                )),
            )
        })
    }
    impl Drop for ProfileFinish {
        fn drop(&mut self) {
            let Some((id, key)) = self.0 else {
                return;
            };
            LEDGER.with(|cell| {
                let mut l = cell.borrow_mut();
                let binding = profile_finish_binding(&l, id, key);
                let object = self.1.as_ref().expect("actual profile finish object");
                object.check("profile-finish", id, &binding);
                assert_eq!(
                    l.profile.finishing.last(),
                    Some(&(id, key)),
                    "profile finish scope order"
                );
                l.profile.finishing.pop();
                l.profile.steps.get_mut(&id).unwrap().finished_duplicate = None;
                l.profile.sync_current();
                object.retire("profile-finish", id, &binding);
            });
        }
    }
    pub fn profile_retire(epoch: u64, index: usize, segment: u64, completion: usize) {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let key = profile_key(epoch, index, segment, completion);
            let Some(id) = l.profile.actual_segment(key) else {
                return;
            };
            if !l.profile.finishing.contains(&(id, key)) {
                l.profile.retire_step(id);
            }
        });
    }

    // Legacy JoinState removed; queue_transport owns all queue carriers.

    // Q29: a capability belongs to one operation, then moves across exact calls.
    // The frame's existing SharedArgs allocation survives its clone on resume.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct OperationKey {
        operator: &'static str,
        target: rum_core::Symbol,
        args: usize,
        len: usize,
    }
    impl OperationKey {
        pub fn new(
            operator: &'static str,
            target: rum_core::Symbol,
            args: usize,
            len: usize,
        ) -> Self {
            Self {
                operator,
                target,
                args,
                len,
            }
        }
    }
    type CallKey = (usize, &'static str, rum_core::Symbol, bool);
    type Projection = Box<dyn Fn(&dyn std::any::Any) -> Option<OperationKey>>;
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum OperationKind {
        Projected(OperationKey),
        Direct((usize, &'static str)),
    }
    struct OperationRecord {
        kind: OperationKind,
        token: Token,
    }
    struct OperationCallRecord {
        key: CallKey,
        operation: Option<u64>,
        token: Option<Token>,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum PermitStage {
        Ready,
        AwaitingHit,
        Transferred,
        Consumed,
    }
    struct LocalPermitRecord {
        operation: u64,
        ticket: Option<u64>,
        token: Token,
    }
    #[derive(Default)]
    struct Operations {
        next_record: u64,
        records: BTreeMap<u64, OperationRecord>,
        projected: BTreeMap<(usize, &'static str), u64>,
        direct: BTreeMap<(usize, &'static str), u64>,
        claimed: std::collections::BTreeSet<u64>,
        retired: std::collections::BTreeSet<u64>,
        call_records: BTreeMap<u64, OperationCallRecord>,
        next_permit: u64,
        permit_records: BTreeMap<u64, LocalPermitRecord>,
        permit_stages: BTreeMap<u64, PermitStage>,
        call_claimed: std::collections::BTreeSet<u64>,
        call_scopes: Vec<u64>,
        constructing: Vec<(&'static str, rum_core::Symbol, Projection)>,
        current_construction_scopes: Vec<(&'static str, rum_core::Symbol, &'static str)>,
        bound_slots: BTreeMap<(usize, &'static str), (OperationKey, Token)>,
        calls: Vec<Option<(CallKey, OperationPermit)>>,
    }
    thread_local! { static OPERATIONS: RefCell<Operations> = RefCell::new(Operations::default()); }
    impl Operations {
        fn issue_permit(
            &mut self,
            operation: Option<u64>,
            token: Option<Token>,
            ticket: Option<u64>,
        ) -> OperationPermit {
            let permit = match (&token, operation) {
                (Some(token), Some(operation)) if self.authorized(operation, token) => {
                    self.next_permit = self.next_permit.checked_add(1).expect("permit id overflow");
                    let id = self.next_permit;
                    assert!(
                        self.permit_records
                            .insert(
                                id,
                                LocalPermitRecord {
                                    operation,
                                    ticket,
                                    token: token.clone()
                                }
                            )
                            .is_none()
                    );
                    assert!(self.permit_stages.insert(id, PermitStage::Ready).is_none());
                    Some(id)
                }
                _ => None,
            };
            OperationPermit {
                token: token.filter(|_| permit.is_some()),
                hit: None,
                operation,
                ticket,
                permit,
            }
        }
        fn permit_matches(
            &self,
            id: u64,
            operation: Option<u64>,
            ticket: Option<u64>,
            token: &Token,
        ) -> bool {
            self.permit_records.get(&id).is_some_and(|record| {
                Some(record.operation) == operation
                    && record.ticket == ticket
                    && record.token == *token
            })
        }
        fn next_id(&mut self) -> u64 {
            self.next_record = self
                .next_record
                .checked_add(1)
                .expect("operation record overflow");
            self.next_record
        }
        fn record(&mut self, kind: OperationKind, token: Token) -> u64 {
            let id = self.next_id();
            assert!(
                self.records
                    .insert(id, OperationRecord { kind, token })
                    .is_none()
            );
            id
        }
        fn authorized(&self, id: u64, token: &Token) -> bool {
            !self.retired.contains(&id)
                && self.claimed.contains(&id)
                && self
                    .records
                    .get(&id)
                    .is_some_and(|record| record.token == *token)
        }
    }
    fn reset_operation_episode() {
        OPERATIONS.with(|c| {
            let mut o = c.borrow_mut();
            assert!(
                o.constructing.is_empty()
                    && o.current_construction_scopes.is_empty()
                    && o.calls.is_empty()
                    && o.call_scopes.is_empty(),
                "PortBlocked::LiveOperationScopeAtBegin"
            );
            let ids: Vec<_> = o.records.keys().copied().collect();
            o.retired.extend(ids);
            o.bound_slots.clear();
            o.projected.clear();
            o.direct.clear();
        });
    }
    fn operation_constructed(key: usize, kind: &'static str, token: &Token) {
        OPERATIONS.with(|c| {
            let mut o = c.borrow_mut();
            let frame = (key, kind);
            assert!(
                !o.direct
                    .get(&frame)
                    .is_some_and(|id| !o.retired.contains(id)),
                "PortBlocked::DuplicateLiveDirectRecord"
            );
            let id = o.record(OperationKind::Direct(frame), token.clone());
            o.direct.insert(frame, id);
        });
    }
    fn retire_operation_frame(key: usize, kind: &'static str) {
        OPERATIONS.with(|c| {
            let mut o = c.borrow_mut();
            let frame = (key, kind);
            for id in [
                o.direct.get(&frame).copied(),
                o.projected.get(&frame).copied(),
            ]
            .into_iter()
            .flatten()
            {
                o.retired.insert(id);
            }
            o.bound_slots.remove(&frame);
        });
    }
    pub struct OperationConstruction {
        _construction: Option<std::mem::ManuallyDrop<ConstructionGuard>>,
        slot: usize,
        object: scope_object::Object,
    }
    impl Guard {
        pub fn suspending_operation<F: 'static>(
            &self,
            operator: &'static str,
            target: rum_core::Symbol,
            project: fn(&F) -> OperationKey,
        ) -> OperationConstruction {
            let construction = self.suspending();
            OPERATIONS.with(|c| {
                c.borrow_mut().current_construction_scopes.push((
                    operator,
                    target,
                    std::any::type_name::<F>(),
                ))
            });
            OPERATIONS.with(|c| {
                c.borrow_mut().constructing.push((
                    operator,
                    target,
                    Box::new(move |value| value.downcast_ref::<F>().map(project)),
                ))
            });
            let (slot, object) = OPERATIONS.with(|cell| {
                let o = cell.borrow();
                let slot = o.current_construction_scopes.len();
                (
                    slot,
                    scope_object::mint(
                        "operation-construction",
                        slot as u64,
                        operation_construction_binding(&o, slot),
                    ),
                )
            });
            OperationConstruction {
                _construction: Some(std::mem::ManuallyDrop::new(construction)),
                slot,
                object,
            }
        }
    }
    impl Drop for OperationConstruction {
        fn drop(&mut self) {
            OPERATIONS.with(|c| {
                let mut o = c.borrow_mut();
                let binding = operation_construction_binding(&o, self.slot);
                self.object
                    .check("operation-construction", self.slot as u64, &binding);
                o.current_construction_scopes
                    .pop()
                    .expect("actual operation constructor");
                o.constructing.pop().expect("operation constructor");
                self.object
                    .retire("operation-construction", self.slot as u64, &binding);
            });
            drop(
                self._construction
                    .take()
                    .map(std::mem::ManuallyDrop::into_inner),
            );
        }
    }
    pub fn bind_operation<F: 'static>(frame: &std::sync::Arc<F>) {
        let frame_key = (
            std::sync::Arc::as_ptr(frame) as usize,
            std::any::type_name::<F>(),
        );
        let token = LEDGER.with(|c| {
            let l = c.borrow();
            l.bound_slots
                .get(&frame_key)
                .filter(|token| {
                    token.frame_identity == Some(frame_key) && authenticated_parent(&l, token)
                })
                .cloned()
        });
        OPERATIONS.with(|c| {
            let mut o = c.borrow_mut();
            let Some((operator, target, project)) = o.constructing.last() else {
                return;
            };
            if o.current_construction_scopes.last()
                != Some(&(*operator, *target, std::any::type_name::<F>()))
                || !LEDGER.with(|cell| {
                    let l = cell.borrow();
                    lexical_current(&l) && !l.scope_authority.constructing.is_empty()
                })
            {
                return;
            }
            let Some(key) = project(frame.as_ref()) else {
                return;
            };
            assert_eq!(
                (key.operator, key.target),
                (*operator, *target),
                "constructed operation changed"
            );
            if let Some(token) = token {
                assert!(
                    !o.bound_slots.contains_key(&frame_key),
                    "PortBlocked::DuplicateLiveOperation"
                );
                assert!(
                    !o.projected
                        .get(&frame_key)
                        .is_some_and(|id| !o.retired.contains(id)),
                    "PortBlocked::DuplicateLiveOperationRecord"
                );
                let id = o.record(OperationKind::Projected(key.clone()), token.clone());
                o.projected.insert(frame_key, id);
                assert!(o.bound_slots.insert(frame_key, (key, token)).is_none());
            }
        });
    }
    pub fn forget_operation(key: usize, kind: &'static str) {
        retire_operation_frame(key, kind);
    }
    #[derive(Default)]
    pub struct OperationPermit {
        token: Option<Token>,
        hit: Option<Token>,
        operation: Option<u64>,
        ticket: Option<u64>,
        permit: Option<u64>,
    }
    fn current_token(token: &Token) -> bool {
        if !LEDGER.with(|cell| lexical_current(&cell.borrow())) {
            return false;
        }
        LEDGER.with(|c| {
            let l = c.borrow();
            let Some(run) = &l.current else {
                return false;
            };
            authenticated_parent(&l, token)
                && !run.queued
                && run.pair.episode.as_ref() == Some(&token.owner.episode)
                && l.resuming.last().and_then(Option::as_ref) == Some(token)
                && l.stack.last() == Some(&token.owner)
                && token
                    .frame_identity
                    .is_some_and(|frame| l.bound_slots.get(&frame) == Some(token))
        })
    }
    pub fn claim_typed_operation(key: OperationKey) -> OperationPermit {
        let resumed = LEDGER.with(|c| c.borrow().resuming.last().cloned().flatten());
        let Some(token) = resumed else {
            return OperationPermit::default();
        };
        if !current_token(&token) {
            return OperationPermit::default();
        }
        let Some(frame) = token.frame_identity else {
            return OperationPermit::default();
        };
        let owned = OPERATIONS.with(|c| {
            let mut o = c.borrow_mut();
            let (expected, actual) = o.bound_slots.get(&frame)?;
            if expected != &key || actual != &token {
                return None;
            }
            let id = *o.projected.get(&frame)?;
            let record = o.records.get(&id)?;
            if record.kind != OperationKind::Projected(key.clone())
                || record.token != token
                || o.retired.contains(&id)
                || o.claimed.contains(&id)
            {
                return None;
            }
            assert!(o.claimed.insert(id));
            let (_, token) = o.bound_slots.remove(&frame).unwrap();
            Some((id, token))
        });
        let (operation, token) = owned
            .map(|(id, token)| (Some(id), Some(token)))
            .unwrap_or_default();
        OPERATIONS.with(|cell| cell.borrow_mut().issue_permit(operation, token, None))
    }
    pub fn claim_operation_call(key: CallKey) -> OperationPermit {
        OPERATIONS.with(|c| {
            let mut o = c.borrow_mut();
            let Some(Some((expected, _))) = o.calls.last() else {
                return OperationPermit::default();
            };
            if expected != &key {
                return OperationPermit::default();
            }
            let permit = &o.calls.last().unwrap().as_ref().unwrap().1;
            let Some(id) = permit.ticket else {
                return OperationPermit::default();
            };
            let Some(operation) = permit.operation else {
                return OperationPermit::default();
            };
            let Some(token) = permit.token.as_ref() else {
                return OperationPermit::default();
            };
            if !permit.permit.is_some_and(|local| {
                o.permit_matches(local, permit.operation, permit.ticket, token)
                    && o.permit_stages.get(&local) == Some(&PermitStage::Ready)
            }) {
                return OperationPermit::default();
            }
            if !current_token(token)
                || !o.authorized(operation, token)
                || o.call_scopes.last() != Some(&id)
                || o.call_claimed.contains(&id)
                || !o.call_records.get(&id).is_some_and(|record| {
                    record.key == key
                        && record.operation == Some(operation)
                        && record.token.as_ref() == Some(token)
                })
            {
                return OperationPermit::default();
            }
            assert!(o.call_claimed.insert(id));
            o.calls.last_mut().unwrap().take().unwrap().1
        })
    }
    struct OperationCallScope(u64, scope_object::Object);
    impl Drop for OperationCallScope {
        fn drop(&mut self) {
            OPERATIONS.with(|c| {
                let mut o = c.borrow_mut();
                let binding = operation_call_binding(&o, self.0);
                self.1.check("operation-call", self.0, &binding);
                assert_eq!(o.call_scopes.pop(), Some(self.0));
                o.calls.pop().expect("operation call");
                self.1.retire("operation-call", self.0, &binding);
            });
        }
    }
    impl OperationPermit {
        fn authorized(&self, token: &Token) -> bool {
            self.lineage_authorized(token)
                && self.permit.is_some_and(|id| {
                    OPERATIONS.with(|cell| {
                        let o = cell.borrow();
                        o.permit_matches(id, self.operation, self.ticket, token)
                            && matches!(
                                o.permit_stages.get(&id),
                                Some(PermitStage::Ready | PermitStage::AwaitingHit)
                            )
                    })
                })
        }
        fn in_stage(&self, stage: PermitStage) -> bool {
            self.permit.is_some_and(|id| {
                OPERATIONS.with(|cell| cell.borrow().permit_stages.get(&id) == Some(&stage))
            })
        }
        fn advance(&self, token: &Token, from: PermitStage, to: PermitStage) -> bool {
            let Some(id) = self.permit else {
                return false;
            };
            OPERATIONS.with(|cell| {
                let mut o = cell.borrow_mut();
                if !o.permit_matches(id, self.operation, self.ticket, token)
                    || o.permit_stages.get(&id) != Some(&from)
                {
                    return false;
                }
                o.permit_stages.insert(id, to);
                true
            })
        }
        fn lineage_authorized(&self, token: &Token) -> bool {
            if !current_token(token) {
                return false;
            }
            let Some(operation) = self.operation else {
                return false;
            };
            OPERATIONS.with(|c| {
                let o = c.borrow();
                o.authorized(operation, token)
                    && self.ticket.is_none_or(|id| {
                        o.call_scopes.last() == Some(&id)
                            && o.call_claimed.contains(&id)
                            && o.call_records.get(&id).is_some_and(|record| {
                                record.operation == Some(operation)
                                    && record.token.as_ref() == Some(token)
                            })
                    })
            })
        }
        fn call_scope(&mut self, key: CallKey) -> OperationCallScope {
            let valid = self
                .token
                .as_ref()
                .is_some_and(|token| self.authorized(token) && self.in_stage(PermitStage::Ready));
            let token = self.token.take().filter(|token| {
                valid && self.advance(token, PermitStage::Ready, PermitStage::Transferred)
            });
            let operation = self.operation.filter(|_| valid);
            let ticket = OPERATIONS.with(|c| {
                let mut o = c.borrow_mut();
                let id = o.next_id();
                assert!(
                    o.call_records
                        .insert(
                            id,
                            OperationCallRecord {
                                key,
                                operation,
                                token: token.clone()
                            }
                        )
                        .is_none()
                );
                o.call_scopes.push(id);
                let permit = o.issue_permit(operation, token, Some(id));
                o.calls.push(Some((key, permit)));
                id
            });
            let object = OPERATIONS.with(|cell| {
                scope_object::mint(
                    "operation-call",
                    ticket,
                    operation_call_binding(&cell.borrow(), ticket),
                )
            });
            OperationCallScope(ticket, object)
        }
        pub fn call<T>(&mut self, key: CallKey, f: impl FnOnce() -> T) -> T {
            let _scope = self.call_scope(key);
            f()
        }
        pub fn omit(&mut self, row: &str) -> bool {
            assert!(self.hit.is_none(), "PortBlocked::UnreportedOperationHit");
            if self
                .token
                .as_ref()
                .is_some_and(|token| token.owner.episode.row != row)
            {
                return false;
            }
            let token = self.token.take();
            self.hit = token.filter(|token| {
                if !self.authorized(token) || !self.in_stage(PermitStage::Ready) {
                    return false;
                }
                let enabled = ENABLED && token.owner.episode.row == row;
                self.advance(
                    token,
                    PermitStage::Ready,
                    if enabled {
                        PermitStage::AwaitingHit
                    } else {
                        PermitStage::Consumed
                    },
                ) && enabled
            });
            self.hit.is_some()
        }
        pub fn hit(&mut self, row: &str, phase: &str) {
            assert!(
                self.hit.as_ref().is_some_and(
                    |token| self.authorized(token) && self.in_stage(PermitStage::AwaitingHit)
                ),
                "PortBlocked::OperationHitIdentity"
            );
            let token = self
                .hit
                .take()
                .expect("operation hit without one-use permission");
            assert!(
                self.advance(&token, PermitStage::AwaitingHit, PermitStage::Consumed),
                "PortBlocked::ConsumedOperationHit"
            );
            assert_eq!(token.owner.episode.row, row);
            LEDGER.with(|c| {
            let mut l = c.borrow_mut();
            let run = l.current.as_mut().expect("operation run");
            let n = run.hits.entry((token.owner.clone(), token.occurrence)).or_default();
            eprintln!("CM740_SOURCE_HIT row={row} phase={phase} selector={row}");
            eprintln!("CM742_HIT_ID program={:?} pair={} activation={:?} producer_occurrence={} hit_occurrence={}",
                token.owner.episode.program, token.owner.episode.pair, token.owner.path, token.occurrence, n);
            *n += 1;
        });
        }
    }

    // Multiple lexical producers share one constructor. Only the selected
    // producer enters this operation scope; the actual error branch binds it.
    pub fn enter_selected(row: &'static str) -> Guard {
        if selected(row) {
            enter(row)
        } else {
            Guard(None)
        }
    }
    pub fn suspending_current_operation<F: 'static>(
        operator: &'static str,
        target: rum_core::Symbol,
        project: fn(&F) -> OperationKey,
    ) -> OperationConstruction {
        LEDGER.with(|cell| {
            let mut l = cell.borrow_mut();
            let token = l
                .stack
                .last()
                .and_then(|owner| l.current.as_ref()?.pending.get(owner).cloned());
            l.scope_authority.constructing.push(
                token
                    .clone()
                    .map(ConstructionIdentity::Pending)
                    .unwrap_or(ConstructionIdentity::Absent),
            );
            l.constructing.push(
                token
                    .map(ConstructionIdentity::Pending)
                    .unwrap_or(ConstructionIdentity::Absent),
            );
        });
        OPERATIONS.with(|c| {
            c.borrow_mut().current_construction_scopes.push((
                operator,
                target,
                std::any::type_name::<F>(),
            ))
        });
        OPERATIONS.with(|c| {
            c.borrow_mut().constructing.push((
                operator,
                target,
                Box::new(move |value| value.downcast_ref::<F>().map(project)),
            ))
        });
        let construction = LEDGER.with(|cell| new_construction_guard(&cell.borrow()));
        let (slot, object) = OPERATIONS.with(|cell| {
            let o = cell.borrow();
            let slot = o.current_construction_scopes.len();
            (
                slot,
                scope_object::mint(
                    "operation-construction",
                    slot as u64,
                    operation_construction_binding(&o, slot),
                ),
            )
        });
        OperationConstruction {
            _construction: Some(std::mem::ManuallyDrop::new(construction)),
            slot,
            object,
        }
    }

    // Direct frame claims carry the actual allocation and concrete type. The
    // claim ledger contains no strong owner and is retired with the adapter.
    thread_local! {
        static DIRECT_CLAIMS: RefCell<BTreeMap<(usize, &'static str), Token>> = RefCell::new(BTreeMap::new());
    }
    pub fn claim_typed_frame<F: 'static>(frame: &std::sync::Arc<F>) -> OperationPermit {
        let key = (
            std::sync::Arc::as_ptr(frame) as usize,
            std::any::type_name::<F>(),
        );
        // The transported token is not its own evidence. bind_frame recorded
        // this complete identity independently at the actual Arc construction.
        // Resume reads frames; it cannot replace the construction record.
        let token = LEDGER.with(|c| {
            let ledger = c.borrow();
            let token = ledger.bound_slots.get(&key)?;
            if token.frame_identity != Some(key) {
                return None;
            }
            let proofs = ledger.current.as_ref()?.pair.tokens.get(&token.owner)?;
            proofs
                .iter()
                .any(|proof| proof == token)
                .then(|| token.clone())
        });
        let Some(token) = token.filter(current_token) else {
            return OperationPermit::default();
        };
        let proof = OPERATIONS.with(|c| {
            let o = c.borrow();
            let id = *o.direct.get(&key)?;
            let record = o.records.get(&id)?;
            (record.kind == OperationKind::Direct(key)
                && record.token == token
                && !o.retired.contains(&id)
                && !o.claimed.contains(&id))
            .then_some(id)
        });
        let Some(proof) = proof else {
            return OperationPermit::default();
        };
        let claimed = DIRECT_CLAIMS.with(|c| {
            let mut claims = c.borrow_mut();
            if claims.contains_key(&key) {
                return false;
            }
            claims.insert(key, token.clone());
            true
        });
        if claimed {
            OPERATIONS.with(|c| {
                assert!(c.borrow_mut().claimed.insert(proof));
            });
        }
        OPERATIONS.with(|cell| {
            cell.borrow_mut()
                .issue_permit(claimed.then_some(proof), claimed.then_some(token), None)
        })
    }
    pub fn forget_direct_claim(key: usize, kind: &'static str) {
        retire_operation_frame(key, kind);
        DIRECT_CLAIMS.with(|c| {
            c.borrow_mut().remove(&(key, kind));
        });
    }

    // Generated-only proof authority. Records are appended at source events;
    // consumers can authenticate, claim or retire them, never replace identity.
    mod identity_proofs {
        use super::{Activation, Episode, Token};
        use std::cell::RefCell;
        use std::collections::{BTreeMap, BTreeSet};

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub(super) struct ProofId(pub(super) u64);
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub(super) struct Identity {
            episode: Episode,
            origin: Activation,
            owner: Activation,
            occurrence: u64,
            frame: Option<(usize, &'static str)>,
        }
        impl From<&Token> for Identity {
            fn from(token: &Token) -> Self {
                Self {
                    episode: token.owner.episode.clone(),
                    origin: token.origin.clone(),
                    owner: token.owner.clone(),
                    occurrence: token.occurrence,
                    frame: token.frame_identity,
                }
            }
        }
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub(super) enum SourceEdge {
            ProducerPause,
            QueuePause,
            EvaluationReturn,
        }
        // One immutable source ticket per actual occurrence, including its proof kind.
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub(super) struct SourceKey {
            episode: Episode,
            queued: bool,
            origin: Activation,
            owner: Activation,
            occurrence: u64,
            edge: SourceEdge,
        }
        impl SourceKey {
            pub(super) fn pause(token: &Token) -> Self {
                Self {
                    episode: token.owner.episode.clone(),
                    queued: false,
                    origin: token.origin.clone(),
                    owner: token.owner.clone(),
                    occurrence: token.occurrence,
                    edge: SourceEdge::ProducerPause,
                }
            }
        }
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub(super) struct Transported {
            pub(super) proof: ProofId,
            identity: Identity,
        }
        #[derive(Clone, Debug, PartialEq, Eq)]
        enum ObjectIdentity {
            Producer(Activation),
            Queue(ProofId),
        }
        struct ProofRecord {
            identity: Identity,
            edge: SourceEdge,
            parent: Option<ProofId>,
            object: ObjectIdentity,
        }
        struct EntryRecord {
            owner: Activation,
            queued: bool,
            producer: std::rc::Weak<super::GuardSource>,
        }
        #[derive(Clone)]
        pub(super) struct CapturedEntry {
            entry: ProofId,
            producer: std::rc::Rc<super::GuardSource>,
        }

        impl CapturedEntry {
            pub(super) fn entry(&self) -> ProofId {
                self.entry
            }

            pub(super) fn owner(&self) -> &Activation {
                &self.producer.owner
            }

            fn recorded(&self, records: &Records) -> bool {
                let owner = self.owner();
                records.entry_at.get(&(owner.clone(), false)) == Some(&self.entry)
                    && records.entries.get(&self.entry).is_some_and(|record| {
                        !record.queued
                            && &record.owner == owner
                            && record
                                .producer
                                .ptr_eq(&std::rc::Rc::downgrade(&self.producer))
                    })
            }

            pub(super) fn authenticates(
                &self,
                received: &Self,
                entry: ProofId,
                owner: &Activation,
            ) -> bool {
                self.entry == entry
                    && received.entry == self.entry
                    && self.owner() == owner
                    && std::rc::Rc::ptr_eq(&self.producer, &received.producer)
                    && super::LEDGER.with(|cell| {
                        cell.borrow().current.as_ref().is_some_and(|run| {
                            super::run_current(run)
                                && !run.queued
                                && run.pair.episode.as_ref() == Some(&owner.episode)
                        })
                    })
                    && RECORDS.with(|cell| self.recorded(&cell.borrow()))
            }

            pub(super) fn owns_queue(&self, installed: ProofId, source: &SourceKey) -> bool {
                RECORDS.with(|cell| {
                    let records = cell.borrow();
                    self.recorded(&records)
                        && source.owner == *self.owner()
                        && source.origin == *self.owner()
                        && records
                            .installed
                            .get(&installed)
                            .is_some_and(|record| record.entry == self.entry)
                        && records.queue_at.contains_key(&(
                            installed,
                            source.clone(),
                            SourceEdge::QueuePause,
                        ))
                })
            }
        }

        pub(super) fn capture_producer(
            producer: &std::rc::Rc<super::GuardSource>,
        ) -> Option<CapturedEntry> {
            RECORDS.with(|cell| {
                let records = cell.borrow();
                let entry = *records.entry_at.get(&(producer.owner.clone(), false))?;
                let capture = CapturedEntry {
                    entry,
                    producer: producer.clone(),
                };
                capture.recorded(&records).then_some(capture)
            })
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(super) enum AppendReceiver {
            Queue { address: usize, kind: &'static str },
            Logical((usize, usize, usize)),
        }

        pub(super) struct ReceiverRegistration {
            pub(super) namespace: (usize, usize, usize),
            pub(super) receiver: Option<AppendReceiver>,
        }

        impl From<(usize, usize, usize)> for ReceiverRegistration {
            fn from(namespace: (usize, usize, usize)) -> Self {
                Self {
                    namespace,
                    receiver: Some(AppendReceiver::Logical(namespace)),
                }
            }
        }

        struct InstalledRecord {
            owner: Activation,
            generation: u64,
            namespace: (usize, usize, usize),
            entry: ProofId,
        }
        // The constructor is private to this module. A caller cannot manufacture a
        // successful authentication from its transported copy or from a ProofId alone.
        pub(super) struct Authenticated {
            proof: ProofId,
        }
        #[derive(Default)]
        struct Records {
            next: u64,
            entries: BTreeMap<ProofId, EntryRecord>,
            entry_at: BTreeMap<(Activation, bool), ProofId>,
            installed: BTreeMap<ProofId, InstalledRecord>,
            receivers: BTreeMap<ProofId, AppendReceiver>,
            records: BTreeMap<ProofId, ProofRecord>,
            pause_at: BTreeMap<SourceKey, ProofId>,
            queue_at: BTreeMap<(ProofId, SourceKey, SourceEdge), ProofId>,
            consumed: BTreeSet<ProofId>,
            retired: BTreeSet<ProofId>,
            queue_return_confirmed: BTreeSet<ProofId>,
            queue_return_revoked: BTreeSet<ProofId>,
        }
        impl Records {
            fn id(&mut self) -> ProofId {
                self.next = self.next.checked_add(1).expect("proof id overflow");
                ProofId(self.next)
            }
            fn append(&mut self, record: ProofRecord) -> ProofId {
                let id = self.id();
                assert!(self.records.insert(id, record).is_none());
                id
            }
            fn installation(
                &self,
                id: ProofId,
                owner: &Activation,
                generation: u64,
                namespace: (usize, usize, usize),
            ) -> bool {
                self.installed.get(&id).is_some_and(|record| {
                    !self.retired.contains(&id)
                        && record.owner == *owner
                        && record.generation == generation
                        && record.namespace == namespace
                        && self
                            .entries
                            .get(&record.entry)
                            .is_some_and(|entry| !entry.queued && entry.owner == *owner)
                })
            }
            fn pause(&self, token: &Token, id: ProofId) -> bool {
                self.records.get(&id).is_some_and(|record| {
                    record.edge == SourceEdge::ProducerPause
                        && record.identity == Identity::from(token)
                        && record.object == ObjectIdentity::Producer(token.owner.clone())
                        && token.frame_identity.is_none()
                        && token.origin == token.owner
                        && self.pause_at.get(&SourceKey::pause(token)) == Some(&id)
                        && record
                            .parent
                            .and_then(|p| self.entries.get(&p))
                            .is_some_and(|entry| !entry.queued && entry.owner == token.owner)
                        && !self.retired.contains(&id)
                })
            }
        }
        thread_local! { static RECORDS: RefCell<Records> = RefCell::new(Records::default()); }

        // Only the actual lexical producer entry invokes this writer.
        pub(super) fn source_entry(producer: &std::rc::Rc<super::GuardSource>, queued: bool) {
            let owner = &producer.owner;
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                assert!(
                    !records.entry_at.contains_key(&(owner.clone(), queued)),
                    "PortBlocked::DuplicateSourceEntry"
                );
                let id = records.id();
                assert!(
                    records
                        .entries
                        .insert(
                            id,
                            EntryRecord {
                                owner: owner.clone(),
                                queued,
                                producer: std::rc::Rc::downgrade(producer)
                            }
                        )
                        .is_none()
                );
                records.entry_at.insert((owner.clone(), queued), id);
            });
        }
        pub(super) fn entry_key(owner: &Activation) -> Option<ProofId> {
            RECORDS.with(|cell| {
                let records = cell.borrow();
                let id = *records.entry_at.get(&(owner.clone(), false))?;
                let entry = records.entries.get(&id)?;
                (!entry.queued && entry.owner == *owner).then_some(id)
            })
        }
        // Called before the mutable pending transport is published, with the actual
        // producer activation and freshly counted occurrence, not a received token.
        pub(super) fn source_pause(owner: &Activation, occurrence: u64) {
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                let Some(entry) = records.entry_at.get(&(owner.clone(), false)).copied() else {
                    return;
                };
                let token = Token {
                    owner: owner.clone(),
                    origin: owner.clone(),
                    occurrence,
                    frame_identity: None,
                };
                let key = SourceKey::pause(&token);
                assert!(
                    !records.pause_at.contains_key(&key),
                    "PortBlocked::DuplicateSourcePause"
                );
                let identity = Identity {
                    episode: owner.episode.clone(),
                    origin: owner.clone(),
                    owner: owner.clone(),
                    occurrence,
                    frame: None,
                };
                let id = records.append(ProofRecord {
                    identity,
                    edge: SourceEdge::ProducerPause,
                    parent: Some(entry),
                    object: ObjectIdentity::Producer(owner.clone()),
                });
                records.pause_at.insert(key, id);
            });
        }
        // Only the actual install/registration scope invokes this writer.
        pub(super) fn source_install(
            owner: &Activation,
            generation: u64,
            registration: impl Into<ReceiverRegistration>,
        ) -> Option<ProofId> {
            let ReceiverRegistration {
                namespace,
                receiver,
            } = registration.into();
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                let entry = *records.entry_at.get(&(owner.clone(), false))?;
                let id = records.id();
                assert!(
                    records
                        .installed
                        .insert(
                            id,
                            InstalledRecord {
                                owner: owner.clone(),
                                generation,
                                namespace,
                                entry
                            }
                        )
                        .is_none()
                );
                if let Some(receiver) = receiver {
                    records.receivers.insert(id, receiver);
                }
                Some(id)
            })
        }
        pub(super) fn receiver_installation(
            receiver: AppendReceiver,
            generation: u64,
            namespace: (usize, usize, usize),
        ) -> Option<ProofId> {
            RECORDS.with(|cell| {
                let records = cell.borrow();
                let mut registrations = records.installed.iter().filter(|(id, _)| {
                    records.receivers.get(id) == Some(&receiver) && !records.retired.contains(id)
                });
                let (&id, record) = registrations.next()?;
                (registrations.next().is_none()
                    && records.installation(id, &record.owner, generation, namespace))
                .then_some(id)
            })
        }

        pub(super) fn registered_at_pause(token: &Token) -> Option<ProofId> {
            use std::ops::Bound::Excluded;
            RECORDS.with(|cell| {
                let records = cell.borrow();
                let pause = *records.pause_at.get(&SourceKey::pause(token))?;
                if !records.pause(token, pause) {
                    return None;
                }
                let entry = records.records.get(&pause)?.parent?;
                let previous = records
                    .records
                    .range(..pause)
                    .rev()
                    .find(|(_, record)| {
                        record.parent == Some(entry) && record.edge == SourceEdge::ProducerPause
                    })
                    .map_or(entry, |(&id, _)| id);
                let mut registrations = records
                    .installed
                    .range((Excluded(previous), Excluded(pause)))
                    .filter(|(_, registration)| {
                        registration.entry == entry && registration.owner == token.owner
                    });
                let (&id, _) = registrations.next()?;
                (registrations.next().is_none() && !records.retired.contains(&id)).then_some(id)
            })
        }

        pub(super) fn source_queue(
            token: &Token,
            install: ProofId,
            generation: u64,
            namespace: (usize, usize, usize),
            edge: SourceEdge,
        ) -> Option<ProofId> {
            if !matches!(edge, SourceEdge::QueuePause | SourceEdge::EvaluationReturn) {
                return None;
            }
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                let pause = *records.pause_at.get(&SourceKey::pause(token))?;
                if !records.pause(token, pause)
                    || !records.installation(install, &token.owner, generation, namespace)
                    || records
                        .queue_at
                        .contains_key(&(install, SourceKey::pause(token), edge))
                {
                    return None;
                }
                let id = records.append(ProofRecord {
                    identity: Identity::from(token),
                    edge,
                    parent: Some(pause),
                    object: ObjectIdentity::Queue(install),
                });
                records
                    .queue_at
                    .insert((install, SourceKey::pause(token), edge), id);
                Some(id)
            })
        }
        pub(super) fn transported(proof: ProofId, token: &Token) -> Transported {
            Transported {
                proof,
                identity: Identity::from(token),
            }
        }
        pub(super) fn authenticate(
            received: &Transported,
            token: &Token,
            install: ProofId,
            generation: u64,
            namespace: (usize, usize, usize),
            edge: SourceEdge,
            episode: &Episode,
            queued: bool,
        ) -> Option<Authenticated> {
            if queued
                || &received.identity.episode != episode
                || received.identity != Identity::from(token)
            {
                return None;
            }
            RECORDS.with(|cell| {
                let records = cell.borrow();
                let record = records.records.get(&received.proof)?;
                (record.identity == received.identity
                    && record.edge == edge
                    && record.object == ObjectIdentity::Queue(install)
                    && records
                        .queue_at
                        .get(&(install, SourceKey::pause(token), edge))
                        == Some(&received.proof)
                    && records.installation(install, &token.owner, generation, namespace)
                    && record
                        .parent
                        .is_some_and(|pause| records.pause(token, pause))
                    && !records.consumed.contains(&received.proof)
                    && !records.retired.contains(&received.proof)
                    && !records.queue_return_revoked.contains(&received.proof))
                .then_some(Authenticated {
                    proof: received.proof,
                })
            })
        }
        pub(super) fn confirm_queue_return(
            capture: &CapturedEntry,
            ticket: &super::queue_transport::SourceScope,
            install: ProofId,
            source: &SourceKey,
            authenticated: Authenticated,
        ) -> bool {
            if !ticket.authentic(capture) || !capture.owns_queue(install, source) {
                return false;
            }
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                if records
                    .queue_at
                    .get(&(install, source.clone(), SourceEdge::QueuePause))
                    != Some(&authenticated.proof)
                {
                    return false;
                }
                assert!(
                    !records.queue_return_revoked.contains(&authenticated.proof),
                    "PortBlocked::RevokedQueueReturn"
                );
                assert!(
                    records
                        .records
                        .get(&authenticated.proof)
                        .is_some_and(|record| record.edge == SourceEdge::QueuePause)
                );
                assert!(
                    records.queue_return_confirmed.insert(authenticated.proof),
                    "PortBlocked::RepeatedQueueReturn"
                );
                true
            })
        }
        pub(super) fn revoke_unconfirmed_queue_return(
            capture: &CapturedEntry,
            ticket: &super::queue_transport::SourceScope,
        ) {
            if !ticket.authentic(capture) {
                return;
            }
            let actual = capture.owner();
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                let ids: Vec<_> = records
                    .records
                    .iter()
                    .filter_map(|(id, record)| {
                        (record.edge == SourceEdge::QueuePause
                            && record.identity.owner == *actual
                            && matches!(&record.object, ObjectIdentity::Queue(install)
                    if records.installed.get(install).is_some_and(|i| i.entry == capture.entry()))
                            && !records.queue_return_confirmed.contains(id))
                        .then_some(*id)
                    })
                    .collect();
                records.queue_return_revoked.extend(ids);
            });
        }
        pub(super) fn confirmed_append_source(
            install: ProofId,
            source: &SourceKey,
            authenticated: &Authenticated,
        ) -> bool {
            RECORDS.with(|cell| {
                let records = cell.borrow();
                records
                    .queue_at
                    .get(&(install, source.clone(), SourceEdge::QueuePause))
                    == Some(&authenticated.proof)
                    && records
                        .queue_return_confirmed
                        .contains(&authenticated.proof)
            })
        }
        pub(super) fn claim(authenticated: Authenticated) -> bool {
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                !records.retired.contains(&authenticated.proof)
                    && records.consumed.insert(authenticated.proof)
            })
        }
        // Revocation keys come from the actual registry or append object, never the
        // transported copy. Immutable identities survive claim and retirement.
        pub(super) struct Retirement<'a> {
            pub(super) install: ProofId,
            pub(super) owner: &'a Activation,
            pub(super) generation: u64,
            pub(super) namespace: (usize, usize, usize),
            pub(super) entry: ProofId,
            pub(super) sources: Vec<(&'a SourceKey, SourceEdge, ProofId)>,
        }

        pub(super) fn retire_registered(
            matches_object: impl Fn((usize, usize, usize)) -> bool,
            mut cleanup: impl FnMut(Retirement<'_>),
        ) {
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                let ids: Vec<_> = records
                    .installed
                    .iter()
                    .filter_map(|(&id, registration)| {
                        (matches_object(registration.namespace) && !records.retired.contains(&id))
                            .then_some(id)
                    })
                    .collect();
                for &install in &ids {
                    let registration = &records.installed[&install];
                    let sources = records
                        .queue_at
                        .iter()
                        .filter_map(|((id, source, edge), proof)| {
                            (*id == install).then_some((source, *edge, *proof))
                        })
                        .collect();
                    cleanup(Retirement {
                        install,
                        owner: &registration.owner,
                        generation: registration.generation,
                        namespace: registration.namespace,
                        entry: registration.entry,
                        sources,
                    });
                }
                records.retired.extend(ids);
            });
        }

        pub(super) fn retire_namespace(predicate: impl Fn((usize, usize, usize)) -> bool) {
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                let ids: Vec<_> = records
                    .installed
                    .iter()
                    .filter_map(|(id, record)| predicate(record.namespace).then_some(*id))
                    .collect();
                records.retired.extend(ids);
            });
        }
        pub(super) fn retire_append(
            actual: ProofId,
            generation: u64,
            namespace: (usize, usize, usize),
        ) {
            RECORDS.with(|cell| {
                let mut records = cell.borrow_mut();
                let ids: Vec<_> = records
                    .queue_at
                    .iter()
                    .filter_map(|((install, _source, edge), id)| {
                        (*install == actual
                            && *edge == SourceEdge::QueuePause
                            && records.installed.get(install).is_some_and(|i| {
                                i.generation == generation && i.namespace == namespace
                            }))
                        .then_some(*id)
                    })
                    .collect();
                records.retired.extend(ids);
            });
        }
    }

    // Source-scoped queue carriers. All lookup tickets precede transported copies.
    pub mod queue_transport {
        use super::identity_proofs::{self, Authenticated, ProofId, SourceEdge, SourceKey};
        use super::{
            Activation, ENABLED, Guard, LEDGER, Token, pending_construction, producer_owner,
            run_is_driven,
        };
        use std::cell::RefCell;
        use std::collections::BTreeMap;

        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Namespace {
            root: usize,
            epoch: usize,
            control: usize,
        }
        impl Namespace {
            pub fn new(root: usize, epoch: usize, control: usize) -> Self {
                Self {
                    root,
                    epoch,
                    control,
                }
            }
            fn key(self) -> (usize, usize, usize) {
                (self.root, self.epoch, self.control)
            }
        }
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct InstallId(ProofId);
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct ScopeId(u64);
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct QueueKey {
            source: SourceKey,
            install: InstallId,
        }
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct Installed {
            owner: Activation,
            generation: u64,
            namespace: Namespace,
            proof: ProofId,
        }
        struct Installing {
            scope: ScopeId,
            owner: Option<Activation>,
            generation: Option<u64>,
            namespace: Option<Namespace>,
            identity: std::rc::Rc<InstallationIdentity>,
        }
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct PassProof {
            installed: Installed,
            token: Token,
            proof: ProofId,
        }
        #[derive(Clone)]
        pub struct SourceScope {
            entry: ProofId,
            owner: Activation,
            capture: identity_proofs::CapturedEntry,
        }
        impl SourceScope {
            pub(super) fn authentic(&self, actual: &identity_proofs::CapturedEntry) -> bool {
                actual.authenticates(&self.capture, self.entry, &self.owner)
            }
        }
        struct AppendScope {
            id: ScopeId,
            generation: u64,
            namespace: Option<Option<Namespace>>,
            install: Option<InstallId>,
        }
        #[derive(Default)]
        struct State {
            next_scope: u64,
            installing: Vec<Installing>,
            installed: BTreeMap<InstallId, Installed>,
            // This index exists only between the actual install and its adjacent pause.
            // A second unpaused install for one producer fails before any replacement.
            unpaused: BTreeMap<Activation, InstallId>,
            reverse: BTreeMap<(u64, Namespace), InstallId>,
            sources: BTreeMap<SourceKey, InstallId>,
            scopes: Vec<ScopeId>,
            evaluation: BTreeMap<ScopeId, PassProof>,
            current_evaluation_scopes: BTreeMap<ScopeId, bool>,
            evaluation_sources: BTreeMap<ScopeId, Option<PassProof>>,
            paused: BTreeMap<QueueKey, PassProof>,
            paused_at: BTreeMap<ProofId, QueueKey>,
            confirmed: BTreeMap<QueueKey, PassProof>,
            confirmed_at: BTreeMap<InstallId, QueueKey>,
            join_owner: Vec<(ScopeId, Option<SourceScope>)>,
            appending: Vec<AppendScope>,
            hits: usize,
            join_hits: usize,
        }
        fn insert<K: Ord, V>(map: &mut BTreeMap<K, V>, key: K, value: V) {
            match map.entry(key) {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(value);
                }
                std::collections::btree_map::Entry::Occupied(_) => {
                    panic!("PortBlocked::DuplicateLiveCarrierKey")
                }
            }
        }
        impl State {
            fn scope(&mut self) -> ScopeId {
                self.next_scope = self.next_scope.checked_add(1).expect("scope overflow");
                ScopeId(self.next_scope)
            }
            fn retire(&mut self, registration: identity_proofs::Retirement<'_>) {
                let id = InstallId(registration.install);
                let (root, epoch, control) = registration.namespace;
                let namespace = Namespace::new(root, epoch, control);
                let expected = Installed {
                    owner: registration.owner.clone(),
                    generation: registration.generation,
                    namespace,
                    proof: registration.install,
                };
                if self.installed.get(&id) != Some(&expected) {
                    return;
                }

                let reverse = (registration.generation, namespace);
                if self.reverse.get(&reverse) == Some(&id) {
                    self.reverse.remove(&reverse);
                }
                if self.unpaused.get(registration.owner) == Some(&id) {
                    self.unpaused.remove(registration.owner);
                }
                for (source, edge, proof_id) in registration.sources {
                    if self.sources.get(source) == Some(&id) {
                        self.sources.remove(source);
                    }
                    if edge != SourceEdge::QueuePause {
                        continue;
                    }
                    let key = QueueKey {
                        source: source.clone(),
                        install: id,
                    };
                    let owns_payload = |proof: &PassProof| {
                        proof.proof == proof_id
                            && proof.installed == expected
                            && proof.token.frame_identity.is_none()
                            && SourceKey::pause(&proof.token) == *source
                    };
                    if self.paused_at.get(&registration.entry) == Some(&key)
                        && self.paused.get(&key).is_some_and(owns_payload)
                    {
                        self.paused_at.remove(&registration.entry);
                        self.paused.remove(&key);
                    }
                    if self.confirmed_at.get(&id) == Some(&key)
                        && self.confirmed.get(&key).is_some_and(owns_payload)
                    {
                        self.confirmed_at.remove(&id);
                        self.confirmed.remove(&key);
                    }
                }
                self.installed.remove(&id);
            }
        }
        thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
        pub fn source_installing() -> Option<InstallGuard> {
            if !run_is_driven() {
                return None;
            }
            Some(installing())
        }

        pub fn source_evaluation_owner(row: &str) -> Option<PassGuard> {
            if !run_is_driven() {
                return None;
            }
            Some(evaluation_owner(row))
        }

        pub fn source_pause_installed() {
            if !run_is_driven() {
                return;
            }
            pause_installed();
        }

        pub fn source_observe_registration<T>(actual_queue: &T, namespace: Namespace) {
            if !run_is_driven() {
                return;
            }
            observe_registration(actual_queue, namespace);
        }

        pub fn source_observe_join_result<T>(actual: &Result<T, crate::EvalError>) {
            if !run_is_driven() {
                return;
            }
            observe_join_result(actual);
        }

        pub fn source_join_owner(source: Option<std::convert::Infallible>) -> Option<JoinOwner> {
            if !run_is_driven() {
                return None;
            }
            Some(join_owner_at(
                CompletionIssuer::default(),
                source.map(|never| match never {}),
            ))
        }

        pub fn source_join_owner_at(
            issuer: CompletionIssuer,
            source: Option<SourceScope>,
        ) -> Option<JoinOwner> {
            if !run_is_driven() {
                return None;
            }
            Some(join_owner_at(issuer, source))
        }

        pub fn source_capture_scope(guard: &Guard) -> Option<IssuedSource> {
            if !run_is_driven() {
                return None;
            }
            source_scope(guard)
        }
        impl InstallGuard {
            pub fn source_generation(&self, generation: u64) {
                if !run_is_driven() {
                    return;
                }
                self.generation(generation);
            }
        }

        impl InstallGuard {
            pub fn source_expect_registration<T>(&self, actual_queue: &T, control: usize) {
                if !run_is_driven() {
                    return;
                }
                self.expect_registration(actual_queue, control);
            }
        }

        impl InstallGuard {
            pub fn source_finish_registration<T>(&self, actual_queue: &T) {
                if !run_is_driven() {
                    return;
                }
                self.finish_registration(actual_queue);
            }
        }

        impl JoinOwner {
            pub fn source_expect_result(&self) {
                if !run_is_driven() {
                    return;
                }
                self.expect_result();
            }
        }

        impl JoinOwner {
            pub fn source_finish_result(&self) {
                if !run_is_driven() {
                    return;
                }
                self.finish_result();
            }
        }

        pub fn reset() {
            identity_proofs::retire_namespace(|_| true);
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                assert!(
                    state.installing.is_empty()
                        && state.scopes.is_empty()
                        && state.join_owner.is_empty()
                        && state.appending.is_empty()
                );
                registration_observation::reset();
                result_observation::reset();
                append_observation::reset();
                *state = State::default();
            });
        }
        #[derive(Debug)]
        struct InstallationIdentity {
            scope: ScopeId,
            owner: Option<Activation>,
            current: std::cell::Cell<bool>,
            generation: std::cell::Cell<Option<u64>>,
            namespace: std::cell::Cell<Option<Namespace>>,
        }
        fn installation_object_matches(guard: &InstallGuard, install: &Installing) -> bool {
            std::rc::Rc::ptr_eq(&guard.1, &install.identity)
        }
        fn installation_matches(state: &State, guard: &InstallGuard) -> bool {
            let Some(install) = state.installing.last() else {
                return false;
            };
            guard.1.current.get()
                && install.identity.current.get()
                && guard.0 == install.scope
                && install.identity.scope == install.scope
                && install.identity.owner == install.owner
                && install.identity.generation.get() == install.generation
                && install.identity.namespace.get() == install.namespace
                && installation_object_matches(guard, install)
        }

        pub struct InstallGuard(ScopeId, std::rc::Rc<InstallationIdentity>);
        // Source observations do not write installation authority. The actual
        // InstallGuard joins them to its independently captured call arguments.
        mod registration_observation {
            use super::*;
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            struct QueueObject {
                address: usize,
                kind: &'static str,
            }
            impl QueueObject {
                fn of<T>(queue: &T) -> Self {
                    Self {
                        address: queue as *const T as usize,
                        kind: std::any::type_name::<T>(),
                    }
                }
            }
            struct Capture {
                source: std::rc::Rc<InstallationIdentity>,
                scope: ScopeId,
                queue: QueueObject,
                control: usize,
                after: u64,
                current: bool,
            }
            struct Observation {
                queue: QueueObject,
                namespace: Namespace,
            }
            #[derive(Default)]
            struct Sources {
                next: u64,
                captures: BTreeMap<ScopeId, Capture>,
                observations: BTreeMap<u64, Observation>,
            }
            thread_local! { static SOURCES: RefCell<Sources> = RefCell::new(Sources::default()); }
            pub(super) fn reset() {
                SOURCES.with(|cell| *cell.borrow_mut() = Sources::default());
            }
            pub(super) fn observe<T>(queue: &T, namespace: Namespace) {
                SOURCES.with(|cell| {
                    let mut source = cell.borrow_mut();
                    source.next = source
                        .next
                        .checked_add(1)
                        .expect("registration observation id");
                    let id = source.next;
                    assert!(
                        source
                            .observations
                            .insert(
                                id,
                                Observation {
                                    queue: QueueObject::of(queue),
                                    namespace,
                                }
                            )
                            .is_none(),
                        "PortBlocked::DuplicateRegistrationObservation"
                    );
                });
            }
            pub(super) fn capture<T>(
                scope: ScopeId,
                identity: std::rc::Rc<InstallationIdentity>,
                queue: &T,
                control: usize,
            ) {
                SOURCES.with(|cell| {
                    let mut source = cell.borrow_mut();
                    assert!(
                        !source.captures.contains_key(&scope),
                        "PortBlocked::RepeatedRegistrationCall"
                    );
                    let after = source.next;
                    source.captures.insert(
                        scope,
                        Capture {
                            source: identity,
                            scope,
                            queue: QueueObject::of(queue),
                            control,
                            after,
                            current: true,
                        },
                    );
                });
            }
            pub(super) fn observed<T>(
                scope: ScopeId,
                identity: &std::rc::Rc<InstallationIdentity>,
                queue: &T,
            ) -> Option<Namespace> {
                SOURCES.with(|cell| {
                    let source = cell.borrow();
                    let capture = source
                        .captures
                        .get(&scope)
                        .expect("actual registration capture");
                    assert!(
                        capture.scope == scope
                            && capture.current
                            && identity.current.get()
                            && std::rc::Rc::ptr_eq(&capture.source, identity)
                            && capture.queue == QueueObject::of(queue),
                        "PortBlocked::RegistrationCallIdentity"
                    );
                    let mut actual = source
                        .observations
                        .range((
                            std::ops::Bound::Excluded(capture.after),
                            std::ops::Bound::Unbounded,
                        ))
                        .filter(|(_, observation)| {
                            observation.queue == capture.queue
                                && observation.namespace.control == capture.control
                        })
                        .map(|(_, observation)| observation.namespace);
                    let result = actual.next();
                    assert!(
                        actual.next().is_none(),
                        "PortBlocked::AmbiguousRegistrationObservation"
                    );
                    result
                })
            }
            pub(super) fn registered_receiver(
                scope: ScopeId,
                identity: &std::rc::Rc<InstallationIdentity>,
                namespace: Namespace,
            ) -> Option<identity_proofs::AppendReceiver> {
                SOURCES.with(|cell| {
                    let source = cell.borrow();
                    let Some(capture) = source.captures.get(&scope) else {
                        return Some(identity_proofs::AppendReceiver::Logical(namespace.key()));
                    };
                    if capture.scope != scope
                        || capture.current
                        || !identity.current.get()
                        || !std::rc::Rc::ptr_eq(&capture.source, identity)
                    {
                        return None;
                    }
                    let mut observed = source
                        .observations
                        .range((
                            std::ops::Bound::Excluded(capture.after),
                            std::ops::Bound::Unbounded,
                        ))
                        .filter(|(_, event)| {
                            event.queue == capture.queue
                                && event.namespace.control == capture.control
                        });
                    let (_, event) = observed.next()?;
                    (event.namespace == namespace && observed.next().is_none()).then_some(
                        identity_proofs::AppendReceiver::Queue {
                            address: capture.queue.address,
                            kind: capture.queue.kind,
                        },
                    )
                })
            }
            pub(super) fn consume(scope: ScopeId, identity: &std::rc::Rc<InstallationIdentity>) {
                SOURCES.with(|cell| {
                    let mut source = cell.borrow_mut();
                    let capture = source
                        .captures
                        .get_mut(&scope)
                        .expect("actual registration capture");
                    assert!(
                        capture.scope == scope
                            && capture.current
                            && identity.current.get()
                            && std::rc::Rc::ptr_eq(&capture.source, identity),
                        "PortBlocked::RegistrationCaptureConsumption"
                    );
                    capture.current = false;
                });
            }
            pub(super) fn retire(scope: ScopeId, identity: &std::rc::Rc<InstallationIdentity>) {
                SOURCES.with(|cell| {
                    let mut source = cell.borrow_mut();
                    if let Some(capture) = source.captures.get_mut(&scope) {
                        assert!(
                            capture.scope == scope
                                && identity.current.get()
                                && std::rc::Rc::ptr_eq(&capture.source, identity),
                            "PortBlocked::RegistrationCaptureRetirement"
                        );
                        capture.current = false;
                    }
                });
            }
        }
        pub fn observe_registration<T>(actual_queue: &T, namespace: Namespace) {
            registration_observation::observe(actual_queue, namespace);
        }
        impl InstallGuard {
            pub fn expect_registration<T>(&self, actual_queue: &T, control: usize) {
                let identity = STATE.with(|cell| {
                    let state = cell.borrow();
                    assert!(
                        installation_matches(&state, self),
                        "PortBlocked::ActualInstallationSource"
                    );
                    let install = state.installing.last().expect("actual installation");
                    assert!(
                        install.generation.is_some() && install.namespace.is_none(),
                        "PortBlocked::RegistrationCallOrder"
                    );
                    install.identity.clone()
                });
                registration_observation::capture(self.0, identity, actual_queue, control);
            }
            pub fn finish_registration<T>(&self, actual_queue: &T) {
                let identity = STATE.with(|cell| {
                    let state = cell.borrow();
                    assert!(
                        installation_matches(&state, self),
                        "PortBlocked::ActualInstallationSource"
                    );
                    let install = state.installing.last().expect("actual installation");
                    assert!(
                        install.generation.is_some() && install.namespace.is_none(),
                        "PortBlocked::RegistrationCallOrder"
                    );
                    install.identity.clone()
                });
                if let Some(namespace) =
                    registration_observation::observed(self.0, &identity, actual_queue)
                {
                    self.registered(namespace);
                }
                registration_observation::consume(self.0, &identity);
            }
        }

        pub fn installing() -> InstallGuard {
            let owner = LEDGER.with(|cell| {
                let ledger = cell.borrow();
                let run = ledger.current.as_ref()?;
                if run.queued {
                    return None;
                }
                producer_owner(&ledger, &run.pair.episode.as_ref()?.row)
            });
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                let scope = state.scope();
                let identity = std::rc::Rc::new(InstallationIdentity {
                    scope,
                    owner: owner.clone(),
                    current: std::cell::Cell::new(true),
                    generation: std::cell::Cell::new(None),
                    namespace: std::cell::Cell::new(None),
                });
                state.installing.push(Installing {
                    scope,
                    owner,
                    generation: None,
                    namespace: None,
                    identity: identity.clone(),
                });
                InstallGuard(scope, identity)
            })
        }
        impl InstallGuard {
            pub fn generation(&self, generation: u64) {
                STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    assert!(
                        installation_matches(&state, self),
                        "PortBlocked::ActualInstallationSource"
                    );
                    let install = state.installing.last_mut().expect("actual install scope");
                    assert_eq!(install.scope, self.0);
                    assert!(install.generation.is_none(), "one generation per install");
                    install.generation = Some(generation);
                    install.identity.generation.set(Some(generation));
                });
            }
        }
        impl InstallGuard {
            pub fn registered(&self, namespace: Namespace) {
                STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    assert!(
                        installation_matches(&state, self),
                        "PortBlocked::ActualInstallationSource"
                    );
                    let install = state.installing.last_mut().expect("actual install scope");
                    if install.generation.is_some() {
                        assert!(
                            install.namespace.is_none(),
                            "one actual queue registration per install"
                        );
                        install.namespace = Some(namespace);
                        install.identity.namespace.set(Some(namespace));
                    }
                });
            }
        }
        impl Drop for InstallGuard {
            fn drop(&mut self) {
                STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    assert!(
                        installation_matches(&state, self),
                        "PortBlocked::ActualInstallationSource"
                    );
                    let receiver = self.1.namespace.get().and_then(|namespace| {
                        registration_observation::registered_receiver(self.0, &self.1, namespace)
                    });
                    registration_observation::retire(self.0, &self.1);
                    let install = state.installing.pop().expect("actual install scope");
                    assert_eq!(install.scope, self.0);
                    install.identity.current.set(false);
                    if let (Some(owner), Some(generation), Some(namespace)) =
                        (install.owner, install.generation, install.namespace)
                    {
                        assert!(
                            !state.unpaused.contains_key(&owner),
                            "PortBlocked::DuplicateUnpausedSourceInstall"
                        );
                        assert!(
                            !state.reverse.contains_key(&(generation, namespace)),
                            "PortBlocked::AmbiguousInstalledQueue"
                        );
                        if let Some(proof) = identity_proofs::source_install(
                            &owner,
                            generation,
                            identity_proofs::ReceiverRegistration {
                                namespace: namespace.key(),
                                receiver,
                            },
                        ) {
                            let id = InstallId(proof);
                            insert(&mut state.unpaused, owner.clone(), id);
                            insert(&mut state.reverse, (generation, namespace), id);
                            insert(
                                &mut state.installed,
                                id,
                                Installed {
                                    owner,
                                    generation,
                                    namespace,
                                    proof,
                                },
                            );
                        }
                    }
                });
            }
        }
        fn authenticate(proof: &PassProof, edge: SourceEdge) -> Option<Authenticated> {
            if proof.installed.owner != proof.token.owner {
                return None;
            }
            let received = identity_proofs::transported(proof.proof, &proof.token);
            LEDGER.with(|cell| {
                let ledger = cell.borrow();
                let run = ledger.current.as_ref()?;
                identity_proofs::authenticate(
                    &received,
                    &proof.token,
                    proof.installed.proof,
                    proof.installed.generation,
                    proof.installed.namespace.key(),
                    edge,
                    run.pair.episode.as_ref()?,
                    run.queued,
                )
            })
        }
        fn source_proof(
            state: &State,
            token: Token,
            id: InstallId,
            edge: SourceEdge,
        ) -> Option<PassProof> {
            let installed = state.installed.get(&id)?;
            if installed.proof != id.0
                || installed.owner != token.owner
                || identity_proofs::registered_at_pause(&token) != Some(id.0)
                || state
                    .reverse
                    .get(&(installed.generation, installed.namespace))
                    != Some(&id)
            {
                return None;
            }
            let proof = identity_proofs::source_queue(
                &token,
                id.0,
                installed.generation,
                installed.namespace.key(),
                edge,
            )?;
            Some(PassProof {
                installed: installed.clone(),
                token,
                proof,
            })
        }
        fn current_pending(row: &str) -> Option<Token> {
            LEDGER.with(|cell| {
                let ledger = cell.borrow();
                let owner = producer_owner(&ledger, row)?;
                let token = ledger.current.as_ref()?.pending.get(&owner)?;
                (token.owner == owner && pending_construction(&ledger, token))
                    .then(|| token.clone())
            })
        }
        use super::scope_object;
        fn evaluation_binding(state: &State, scope: ScopeId) -> String {
            format!(
                "{:?}",
                (
                    scope,
                    state.scopes.last(),
                    state.current_evaluation_scopes.get(&scope),
                    state.evaluation_sources.get(&scope)
                )
            )
        }
        fn join_owner_binding(state: &State, scope: ScopeId) -> String {
            format!(
                "{:?}",
                (
                    scope,
                    state
                        .join_owner
                        .last()
                        .map(|(s, source)| (s, source.as_ref().map(|p| (&p.entry, &p.owner))))
                )
            )
        }
        fn join_append_binding(state: &State, scope: ScopeId) -> String {
            format!(
                "{:?}",
                (
                    scope,
                    state
                        .appending
                        .last()
                        .map(|a| (a.id, a.generation, a.namespace, a.install))
                )
            )
        }
        pub struct PassGuard(ScopeId, scope_object::Object);
        pub fn evaluation_owner(row: &str) -> PassGuard {
            let token = current_pending(row);
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                let scope = state.scope();
                let proof = token.and_then(|token| {
                    let source = SourceKey::pause(&token);
                    if let Some(&id) = state.sources.get(&source) {
                        return source_proof(&state, token, id, SourceEdge::EvaluationReturn);
                    }
                    let id = *state.unpaused.get(&token.owner)?;
                    let proof = source_proof(&state, token, id, SourceEdge::EvaluationReturn)?;
                    state.unpaused.remove(&proof.token.owner);
                    insert(&mut state.sources, source, id);
                    Some(proof)
                });
                insert(&mut state.evaluation_sources, scope, proof.clone());
                if let Some(proof) = proof {
                    insert(&mut state.evaluation, scope, proof);
                }
                insert(&mut state.current_evaluation_scopes, scope, true);
                state.scopes.push(scope);
                let object =
                    scope_object::mint("evaluation", scope.0, evaluation_binding(&state, scope));
                PassGuard(scope, object)
            })
        }
        impl Drop for PassGuard {
            fn drop(&mut self) {
                STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    let binding = evaluation_binding(&state, self.0);
                    self.1.check("evaluation", self.0.0, &binding);
                    assert_eq!(state.scopes.pop(), Some(self.0));
                    *state
                        .current_evaluation_scopes
                        .get_mut(&self.0)
                        .expect("actual evaluation scope") = false;
                    state.evaluation.remove(&self.0);
                    state.evaluation_sources.remove(&self.0);
                    self.1.retire("evaluation", self.0.0, &binding);
                });
            }
        }
        pub struct PublicationReceiver {
            queue: Option<identity_proofs::AppendReceiver>,
            registered: Option<Namespace>,
            proposed: Option<Namespace>,
        }

        pub fn publication_receiver<T>(
            queue: &T,
            registered: Option<Namespace>,
            proposed: Option<Namespace>,
        ) -> PublicationReceiver {
            PublicationReceiver {
                queue: Some(identity_proofs::AppendReceiver::Queue {
                    address: queue as *const T as usize,
                    kind: std::any::type_name::<T>(),
                }),
                registered,
                proposed,
            }
        }

        impl From<Option<Namespace>> for PublicationReceiver {
            fn from(namespace: Option<Namespace>) -> Self {
                Self {
                    queue: namespace.map(|n| identity_proofs::AppendReceiver::Logical(n.key())),
                    registered: namespace,
                    proposed: namespace,
                }
            }
        }

        pub fn omit_evaluation_pass(publication: impl Into<PublicationReceiver>) -> bool {
            if !ENABLED {
                return false;
            }
            let PublicationReceiver {
                queue,
                registered,
                proposed: namespace,
            } = publication.into();
            let owned = STATE.with(|cell| {
                let state = cell.borrow();
                let scope = *state.scopes.last()?;
                if state.current_evaluation_scopes.get(&scope) != Some(&true) {
                    return None;
                }
                let proof = state.evaluation.get(&scope)?;
                let actual = registered?;
                if Some(actual) != namespace
                    || identity_proofs::receiver_installation(
                        queue?,
                        proof.installed.generation,
                        actual.key(),
                    ) != Some(proof.installed.proof)
                {
                    return None;
                }
                (Some(proof.installed.namespace) == namespace).then(|| (scope, proof.clone()))
            });
            let Some((scope, proof)) = owned else {
                return false;
            };
            let valid = LEDGER.with(|cell| {
                let ledger = cell.borrow();
                pending_construction(&ledger, &proof.token)
                    && producer_owner(&ledger, &proof.token.owner.episode.row).as_ref()
                        == Some(&proof.token.owner)
            });
            if !valid {
                return false;
            }
            let Some(authenticated) = authenticate(&proof, SourceEdge::EvaluationReturn) else {
                return false;
            };
            if !identity_proofs::claim(authenticated) {
                return false;
            }
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                state.evaluation.remove(&scope);
                state.hits += 1;
            });
            let row = &proof.token.owner.episode.row;
            eprintln!(
                "CM740_SOURCE_HIT row={row} phase=OwnFixedPointPassPublicationAfterEvaluationReturn selector={row}"
            );
            eprintln!(
                "CM742_QUEUE_HIT_ID program={:?} pair={} activation={:?} producer_occurrence={} generation={} namespace={:?}",
                proof.token.owner.episode.program,
                proof.token.owner.episode.pair,
                proof.token.owner.path,
                proof.token.occurrence,
                proof.installed.generation,
                proof.installed.namespace
            );
            true
        }
        pub fn pause_installed() {
            let row = LEDGER.with(|cell| {
                cell.borrow()
                    .current
                    .as_ref()?
                    .pair
                    .episode
                    .as_ref()
                    .map(|e| e.row.clone())
            });
            let Some(token) = row.and_then(|row| current_pending(&row)) else {
                return;
            };
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                let Some(&id) = state.unpaused.get(&token.owner) else {
                    return;
                };
                let Some(entry) = identity_proofs::entry_key(&token.owner) else {
                    return;
                };
                let source = SourceKey::pause(&token);
                assert!(
                    !state.sources.contains_key(&source),
                    "PortBlocked::DuplicateSourcePause"
                );
                assert!(
                    !state.paused_at.contains_key(&entry),
                    "PortBlocked::AmbiguousProducerReturnInstall"
                );
                if let Some(proof) = source_proof(&state, token, id, SourceEdge::QueuePause) {
                    state.unpaused.remove(&proof.token.owner);
                    let key = QueueKey {
                        source: source.clone(),
                        install: id,
                    };
                    insert(&mut state.sources, source, id);
                    insert(&mut state.paused_at, entry, key.clone());
                    insert(&mut state.paused, key, proof);
                }
            });
        }
        // Captured while the actual producer Guard is alive, before its call. The
        // completion receives this ticket after that Guard retires; it never uses a row
        // string or a transported pause to discover whose result it is completing.
        pub fn source_scope(guard: &Guard) -> Option<IssuedSource> {
            if !LEDGER.with(|cell| super::guard_source_matches(&cell.borrow(), guard)) {
                return None;
            }
            if !LEDGER.with(|cell| {
                cell.borrow()
                    .current
                    .as_ref()
                    .is_some_and(|run| !run.queued)
            }) {
                return None;
            }
            let capture =
                identity_proofs::capture_producer(guard.0.as_ref().map(|active| &active.source)?)?;
            let ticket = SourceScope {
                entry: capture.entry(),
                owner: capture.owner().clone(),
                capture: capture.clone(),
            };
            Some(IssuedSource { capture, ticket })
        }
        // The actual producer and the value transported to complete_work have separate
        // owners. Only the lexical enter can construct both parts of this issuance.
        #[derive(Clone)]
        pub struct IssuedSource {
            capture: identity_proofs::CapturedEntry,
            ticket: SourceScope,
        }

        #[derive(Default)]
        pub struct CompletionIssuer(Option<identity_proofs::CapturedEntry>);

        impl CompletionIssuer {
            pub fn retain(&mut self, issued: Option<IssuedSource>) -> Option<SourceScope> {
                let IssuedSource { capture, ticket } = issued?;
                assert!(self.0.is_none(), "PortBlocked::RepeatedCompletionIssuer");
                self.0 = Some(capture);
                Some(ticket)
            }
        }

        // Fixture code can inspect and corrupt its separately transported value.
        // The actual issuer stays in the caller's retained capture.
        #[cfg(test)]
        impl std::ops::Deref for IssuedSource {
            type Target = SourceScope;
            fn deref(&self) -> &SourceScope {
                &self.ticket
            }
        }
        #[cfg(test)]
        impl std::ops::DerefMut for IssuedSource {
            fn deref_mut(&mut self) -> &mut SourceScope {
                &mut self.ticket
            }
        }

        pub struct JoinOwner(
            ScopeId,
            scope_object::Object,
            Option<identity_proofs::CapturedEntry>,
        );
        // The lower complete_work hook records the actual result kind. Only the
        // actual caller's JoinOwner may apply that observation to queue authority.
        mod result_observation {
            use super::*;
            struct Capture {
                source: std::rc::Rc<scope_object::Identity>,
                scope: ScopeId,
                after: u64,
                current: bool,
            }
            #[derive(Default)]
            struct Sources {
                next: u64,
                captures: BTreeMap<ScopeId, Capture>,
                results: BTreeMap<u64, bool>,
            }
            thread_local! { static SOURCES: RefCell<Sources> = RefCell::new(Sources::default()); }
            pub(super) fn reset() {
                SOURCES.with(|cell| *cell.borrow_mut() = Sources::default());
            }
            pub(super) fn observe(input_required: bool) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    s.next = s
                        .next
                        .checked_add(1)
                        .expect("actual completion observation id");
                    let id = s.next;
                    assert!(s.results.insert(id, input_required).is_none());
                });
            }
            pub(super) fn capture(actual: &JoinOwner) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    assert!(
                        !s.captures.contains_key(&actual.0),
                        "PortBlocked::RepeatedActualCompletionCall"
                    );
                    let after = s.next;
                    s.captures.insert(
                        actual.0,
                        Capture {
                            source: actual.1.0.clone(),
                            scope: actual.0,
                            after,
                            current: true,
                        },
                    );
                });
            }
            pub(super) fn result(actual: &JoinOwner) -> Option<bool> {
                SOURCES.with(|cell| {
                    let s = cell.borrow();
                    let capture = s
                        .captures
                        .get(&actual.0)
                        .expect("actual completion capture");
                    assert!(
                        capture.current
                            && capture.scope == actual.0
                            && std::rc::Rc::ptr_eq(&capture.source, &actual.1.0),
                        "PortBlocked::ActualCompletionCallIdentity"
                    );
                    let mut values = s.results.range((
                        std::ops::Bound::Excluded(capture.after),
                        std::ops::Bound::Unbounded,
                    ));
                    let Some((_, result)) = values.next() else {
                        return None;
                    };
                    assert!(
                        values.next().is_none(),
                        "PortBlocked::AmbiguousActualCompletionObservation"
                    );
                    Some(*result)
                })
            }
            pub(super) fn retire(actual: &JoinOwner) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    if let Some(capture) = s.captures.get_mut(&actual.0) {
                        assert!(
                            capture.scope == actual.0
                                && std::rc::Rc::ptr_eq(&capture.source, &actual.1.0),
                            "PortBlocked::ActualCompletionCaptureRetirement"
                        );
                        capture.current = false;
                    }
                });
            }
        }
        pub fn observe_join_result<T>(actual: &Result<T, crate::EvalError>) {
            result_observation::observe(matches!(
                actual,
                Err(crate::EvalError::InputRequired { .. })
            ));
        }
        impl JoinOwner {
            pub fn expect_result(&self) {
                STATE.with(|cell| {
                    let state = cell.borrow();
                    self.1
                        .check("join-owner", self.0.0, &join_owner_binding(&state, self.0));
                });
                result_observation::capture(self);
            }
            pub fn finish_result(&self) {
                STATE.with(|cell| {
                    let state = cell.borrow();
                    self.1
                        .check("join-owner", self.0.0, &join_owner_binding(&state, self.0));
                });
                if let Some(input_required) = result_observation::result(self) {
                    join_result_kind(self, input_required);
                }
                result_observation::retire(self);
            }
        }

        pub fn join_owner(issued: Option<IssuedSource>) -> JoinOwner {
            let mut issuer = CompletionIssuer::default();
            let source = issuer.retain(issued);
            join_owner_at(issuer, source)
        }

        pub fn join_owner_at(issuer: CompletionIssuer, source: Option<SourceScope>) -> JoinOwner {
            let capture = issuer.0;
            match (&capture, &source) {
                (Some(actual), Some(ticket)) => assert!(
                    ticket.authentic(actual),
                    "PortBlocked::ActualSourceTicketMint"
                ),
                (None, None) => {}
                _ => panic!("PortBlocked::ActualSourceTicketMint"),
            }
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                let scope = state.scope();
                state.join_owner.push((scope, source));
                JoinOwner(
                    scope,
                    scope_object::mint("join-owner", scope.0, join_owner_binding(&state, scope)),
                    capture,
                )
            })
        }
        impl Drop for JoinOwner {
            fn drop(&mut self) {
                STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    let binding = join_owner_binding(&state, self.0);
                    self.1.check("join-owner", self.0.0, &binding);
                    result_observation::retire(self);
                    let (scope, source) = state.join_owner.pop().expect("actual completion scope");
                    assert_eq!(scope, self.0);
                    if let (Some(source), Some(capture)) = (&source, &self.2) {
                        if source.authentic(capture) {
                            identity_proofs::revoke_unconfirmed_queue_return(capture, source);
                            if let Some(key) = state.paused_at.get(&capture.entry()).cloned() {
                                if capture.owns_queue(key.install.0, &key.source) {
                                    state.paused_at.remove(&capture.entry());
                                    state.paused.remove(&key);
                                }
                            }
                        }
                    }
                    self.1.retire("join-owner", self.0.0, &binding);
                });
            }
        }
        pub fn join_result<T>(actual: &JoinOwner, result: &Result<T, crate::EvalError>) {
            join_result_kind(
                actual,
                matches!(result, Err(crate::EvalError::InputRequired { .. })),
            )
        }
        fn join_result_kind(actual: &JoinOwner, input_required: bool) {
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                actual.1.check(
                    "join-owner",
                    actual.0.0,
                    &join_owner_binding(&state, actual.0),
                );
                let Some(source) = state
                    .join_owner
                    .last()
                    .and_then(|(_, source)| source.clone())
                else {
                    return;
                };
                let Some(capture) = &actual.2 else {
                    return;
                };
                if !source.authentic(capture) {
                    return;
                }
                let Some(key) = state.paused_at.get(&capture.entry()).cloned() else {
                    identity_proofs::revoke_unconfirmed_queue_return(capture, &source);
                    return;
                };
                if !capture.owns_queue(key.install.0, &key.source) {
                    return;
                }
                state.paused_at.remove(&capture.entry());
                let Some(proof) = state.paused.remove(&key) else {
                    identity_proofs::revoke_unconfirmed_queue_return(capture, &source);
                    return;
                };
                if input_required && authenticate(&proof, SourceEdge::QueuePause).is_some() {
                    assert!(
                        !state.confirmed_at.contains_key(&key.install),
                        "PortBlocked::DuplicateQueueConfirmation"
                    );
                    assert!(
                        !state.confirmed.contains_key(&key),
                        "PortBlocked::DuplicateQueueConfirmation"
                    );
                    if !identity_proofs::confirm_queue_return(
                        capture,
                        &source,
                        key.install.0,
                        &key.source,
                        authenticate(&proof, SourceEdge::QueuePause)
                            .expect("same authenticated actual return"),
                    ) {
                        return;
                    }
                    insert(&mut state.confirmed_at, key.install, key.clone());
                    insert(&mut state.confirmed, key, proof);
                } else {
                    identity_proofs::revoke_unconfirmed_queue_return(capture, &source);
                }
            });
        }
        // The lookup index proposes a key. The actual append supplies the namespace,
        // generation and installation. The confirmed return was issued independently.
        fn confirmed_append(
            state: &State,
            scope: &AppendScope,
            namespace: Namespace,
        ) -> Option<(QueueKey, PassProof, Authenticated)> {
            let install = scope.install?;
            if scope.namespace != Some(Some(namespace))
                || state.reverse.get(&(scope.generation, namespace)) != Some(&install)
            {
                return None;
            }
            let installed = state.installed.get(&install)?;
            if installed.proof != install.0
                || installed.generation != scope.generation
                || installed.namespace != namespace
            {
                return None;
            }
            let key = state.confirmed_at.get(&install)?;
            if key.install != install {
                return None;
            }
            let proof = state.confirmed.get(key)?;
            if &proof.installed != installed || key.source != SourceKey::pause(&proof.token) {
                return None;
            }
            let authenticated = authenticate(proof, SourceEdge::QueuePause)?;
            identity_proofs::confirmed_append_source(install.0, &key.source, &authenticated)
                .then(|| (key.clone(), proof.clone(), authenticated))
        }

        fn retire_append_confirmation(state: &mut State, scope: &AppendScope) {
            let Some(Some(namespace)) = scope.namespace else {
                return;
            };
            let Some(id) = scope.install else {
                return;
            };
            if state.confirmed_at.contains_key(&id) {
                let Some((key, _, _)) = confirmed_append(state, scope, namespace) else {
                    return;
                };
                state.confirmed_at.remove(&id);
                state.confirmed.remove(&key);
            }
            identity_proofs::retire_append(id.0, scope.generation, namespace.key());
        }

        pub struct JoinAppend(ScopeId, scope_object::Object);
        // The source caller owns one real Rc<JoinAppend>. The queue hook receives
        // that same object through a queue-bound Weak; it never synthesizes a guard.
        mod append_observation {
            use super::*;
            #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
            struct QueueObject {
                address: usize,
                kind: &'static str,
            }
            impl QueueObject {
                fn of<T>(queue: &T) -> Self {
                    Self {
                        address: queue as *const T as usize,
                        kind: std::any::type_name::<T>(),
                    }
                }
            }
            struct Call {
                actual: std::rc::Weak<JoinAppend>,
                source: std::rc::Rc<scope_object::Identity>,
                scope: ScopeId,
                queue: Option<QueueObject>,
                after: u64,
                current: bool,
            }
            #[derive(Clone, Copy)]
            struct Observation {
                queue: QueueObject,
                namespace: Option<Namespace>,
                receiver: Option<ScopeId>,
            }
            #[derive(Default)]
            struct Sources {
                next: u64,
                calls: BTreeMap<ScopeId, Call>,
                queues: BTreeMap<QueueObject, ScopeId>,
                observations: BTreeMap<u64, Observation>,
                consumed: BTreeMap<u64, std::rc::Rc<scope_object::Identity>>,
            }
            thread_local! { static SOURCES:RefCell<Sources>=RefCell::new(Sources::default()); }
            pub(super) fn reset() {
                SOURCES.with(|cell| *cell.borrow_mut() = Sources::default());
            }
            fn same(call: &Call, actual: &JoinAppend) -> bool {
                call.scope == actual.0
                    && call.actual.as_ptr() == actual as *const JoinAppend
                    && std::rc::Rc::ptr_eq(&call.source, &actual.1.0)
            }
            pub(super) fn begin(actual: &std::rc::Rc<JoinAppend>) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    assert!(
                        !s.calls.contains_key(&actual.0),
                        "PortBlocked::RepeatedActualAppendCall"
                    );
                    let after = s.next;
                    s.calls.insert(
                        actual.0,
                        Call {
                            actual: std::rc::Rc::downgrade(actual),
                            source: actual.1.0.clone(),
                            scope: actual.0,
                            queue: None,
                            after,
                            current: true,
                        },
                    );
                });
            }
            pub(super) fn bind<T>(actual: &std::rc::Rc<JoinAppend>, queue: &T) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    let key = QueueObject::of(queue);
                    let call = s.calls.get(&actual.0).expect("actual append call");
                    assert!(
                        same(call, actual) && call.current && call.queue.is_none(),
                        "PortBlocked::ActualAppendQueueBinding"
                    );
                    assert!(
                        !s.queues.contains_key(&key),
                        "PortBlocked::AmbiguousActualAppendQueue"
                    );
                    s.calls.get_mut(&actual.0).unwrap().queue = Some(key);
                    s.queues.insert(key, actual.0);
                });
            }
            pub(super) fn observe<T>(queue: &T, namespace: Option<Namespace>) {
                let actual = SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    let key = QueueObject::of(queue);
                    let receiver = s.queues.get(&key).copied();
                    let actual = receiver.map(|scope| {
                        let call = s.calls.get(&scope).expect("queue-bound actual append call");
                        let actual = call.actual.upgrade().expect("live actual append object");
                        assert!(
                            same(call, &actual) && call.current && call.queue == Some(key),
                            "PortBlocked::ActualAppendQueueSource"
                        );
                        actual
                    });
                    assert!(
                        namespace.is_none() || actual.is_some(),
                        "PortBlocked::MissingActualAppendQueueSource"
                    );
                    s.next = s.next.checked_add(1).expect("actual append observation id");
                    let id = s.next;
                    assert!(
                        s.observations
                            .insert(
                                id,
                                Observation {
                                    queue: key,
                                    namespace,
                                    receiver
                                }
                            )
                            .is_none()
                    );
                    actual
                });
                if let Some(actual) = actual {
                    let object = QueueObject::of(queue);
                    let receiver = identity_proofs::AppendReceiver::Queue {
                        address: object.address,
                        kind: object.kind,
                    };
                    join_queue(
                        &actual,
                        AppendInput {
                            receiver: Some(receiver),
                            namespace,
                        },
                    );
                }
            }
            pub(super) fn observations(actual: &JoinAppend) -> Vec<(u64, Option<Namespace>, bool)> {
                SOURCES.with(|cell| {
                    let s = cell.borrow();
                    let call = s.calls.get(&actual.0).expect("actual append capture");
                    assert!(
                        same(call, actual) && call.current,
                        "PortBlocked::ActualAppendCompletionSource"
                    );
                    let events: Vec<_> = s
                        .observations
                        .range((
                            std::ops::Bound::Excluded(call.after),
                            std::ops::Bound::Unbounded,
                        ))
                        .filter(|(id, event)| {
                            !s.consumed.contains_key(*id)
                                && (event.receiver == Some(actual.0) || event.receiver.is_none())
                        })
                        .map(|(id, event)| {
                            if event.receiver.is_some() {
                                assert_eq!(call.queue, Some(event.queue));
                            } else {
                                assert!(
                                    event.namespace.is_none(),
                                    "PortBlocked::UnboundAppendAuthority"
                                );
                            }
                            (*id, event.namespace, event.receiver.is_some())
                        })
                        .collect();
                    assert!(events.len() <= 1, "one actual queue per append");
                    events
                })
            }
            pub(super) fn consume(actual: &JoinAppend, events: &[(u64, Option<Namespace>, bool)]) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    let call = s.calls.get(&actual.0).expect("actual append capture");
                    assert!(
                        same(call, actual) && call.current,
                        "PortBlocked::ActualAppendConsumptionSource"
                    );
                    for (id, _, _) in events {
                        assert!(
                            !s.consumed.contains_key(id),
                            "PortBlocked::RepeatedAppendObservation"
                        );
                    }
                    for (id, _, _) in events {
                        s.consumed.insert(*id, actual.1.0.clone());
                    }
                    s.calls.get_mut(&actual.0).unwrap().current = false;
                });
            }
            pub(super) fn retire(actual: &JoinAppend) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    if let Some(call) = s.calls.get(&actual.0) {
                        assert!(
                            same(call, actual),
                            "PortBlocked::ActualAppendCaptureRetirement"
                        );
                        let queue = call.queue;
                        if let Some(queue) = queue {
                            assert_eq!(s.queues.get(&queue), Some(&actual.0));
                            s.queues.remove(&queue);
                        }
                        s.calls.get_mut(&actual.0).unwrap().current = false;
                    }
                });
            }
            #[cfg(test)]
            pub(super) fn fixture_queue_scope<T>(queue: &T, scope: ScopeId) -> Option<ScopeId> {
                SOURCES.with(|cell| {
                    cell.borrow_mut()
                        .queues
                        .insert(QueueObject::of(queue), scope)
                })
            }
            #[cfg(test)]
            pub(super) fn fixture_restore_queue<T>(queue: &T, old: Option<ScopeId>) {
                SOURCES.with(|cell| {
                    let mut s = cell.borrow_mut();
                    let key = QueueObject::of(queue);
                    if let Some(old) = old {
                        s.queues.insert(key, old);
                    } else {
                        s.queues.remove(&key);
                    }
                });
            }
            #[cfg(test)]
            pub(super) fn fixture_sources() -> String {
                SOURCES.with(|cell| {
                    format!(
                        "{:?}",
                        cell.borrow()
                            .calls
                            .iter()
                            .map(|(key, call)| (
                                key,
                                call.actual.as_ptr() as usize,
                                std::rc::Rc::as_ptr(&call.source) as usize,
                                call.scope,
                                call.queue,
                                call.after,
                                call.current
                            ))
                            .collect::<Vec<_>>()
                    )
                })
            }
        }
        fn check_actual_append(actual: &JoinAppend) {
            STATE.with(|cell| {
                let state = cell.borrow();
                actual.1.check(
                    "join-append",
                    actual.0.0,
                    &join_append_binding(&state, actual.0),
                );
            });
        }
        fn begin_append_call(actual: &std::rc::Rc<JoinAppend>) {
            check_actual_append(actual);
            append_observation::begin(actual);
        }
        fn bind_append_queue<T>(actual: &std::rc::Rc<JoinAppend>, queue: &T) {
            check_actual_append(actual);
            append_observation::bind(actual, queue);
        }
        fn observe_append_queue<T>(queue: &T, namespace: Option<Namespace>) {
            append_observation::observe(queue, namespace);
        }
        fn finish_append_call(actual: &JoinAppend) {
            check_actual_append(actual);
            let events = append_observation::observations(actual);
            for (_, namespace, already_applied) in &events {
                if *already_applied {
                    STATE.with(|cell| {
                        assert_eq!(
                            cell.borrow().appending.last().unwrap().namespace,
                            Some(*namespace)
                        )
                    });
                } else {
                    join_queue(
                        actual,
                        AppendInput {
                            receiver: None,
                            namespace: *namespace,
                        },
                    );
                }
            }
            append_observation::consume(actual, &events);
        }

        pub fn source_append(generation: u64) -> Option<std::rc::Rc<JoinAppend>> {
            if !run_is_driven() {
                return None;
            }
            let actual = std::rc::Rc::new(join_append(generation));
            begin_append_call(&actual);
            Some(actual)
        }
        pub fn source_bind_append_queue<T>(actual: &std::rc::Rc<JoinAppend>, queue: &T) {
            if !run_is_driven() {
                return;
            }
            bind_append_queue(actual, queue);
        }
        pub fn source_observe_append_queue<T>(queue: &T, namespace: Option<Namespace>) {
            if !run_is_driven() {
                return;
            }
            observe_append_queue(queue, namespace);
        }
        pub fn source_finish_append_call(actual: &JoinAppend) {
            if !run_is_driven() {
                return;
            }
            finish_append_call(actual);
        }

        fn join_append(generation: u64) -> JoinAppend {
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                let id = state.scope();
                state.appending.push(AppendScope {
                    id,
                    generation,
                    namespace: None,
                    install: None,
                });
                JoinAppend(
                    id,
                    scope_object::mint("join-append", id.0, join_append_binding(&state, id)),
                )
            })
        }
        struct AppendInput {
            receiver: Option<identity_proofs::AppendReceiver>,
            namespace: Option<Namespace>,
        }

        impl From<Option<Namespace>> for AppendInput {
            fn from(namespace: Option<Namespace>) -> Self {
                Self {
                    receiver: namespace.map(|n| identity_proofs::AppendReceiver::Logical(n.key())),
                    namespace,
                }
            }
        }

        fn join_queue(actual: &JoinAppend, received: impl Into<AppendInput>) {
            let AppendInput {
                receiver,
                namespace,
            } = received.into();
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                let binding = join_append_binding(&state, actual.0);
                actual.1.check("join-append", actual.0.0, &binding);
                let Some(scope) = state.appending.last() else {
                    return;
                };
                let install = namespace.and_then(|namespace| {
                    let id = InstallId(identity_proofs::receiver_installation(
                        receiver?,
                        scope.generation,
                        namespace.key(),
                    )?);
                    (state.reverse.get(&(scope.generation, namespace)) == Some(&id)).then_some(id)
                });
                let scope = state.appending.last_mut().unwrap();
                assert!(scope.namespace.is_none(), "one actual queue per append");
                scope.namespace = Some(namespace);
                scope.install = install;
                actual.1.update(
                    "join-append",
                    actual.0.0,
                    &binding,
                    join_append_binding(&state, actual.0),
                );
            });
        }
        impl Drop for JoinAppend {
            fn drop(&mut self) {
                STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    let binding = join_append_binding(&state, self.0);
                    self.1.check("join-append", self.0.0, &binding);
                    append_observation::retire(self);
                    let scope = state.appending.pop().expect("actual append scope");
                    assert_eq!(scope.id, self.0);
                    retire_append_confirmation(&mut state, &scope);
                    self.1.retire("join-append", self.0.0, &binding);
                });
            }
        }
        pub fn omit_join(namespace: Option<Namespace>) -> bool {
            if !ENABLED {
                return false;
            }
            let owned = STATE.with(|cell| {
                let state = cell.borrow();
                let scope = state.appending.last()?;
                confirmed_append(&state, scope, namespace?)
            });
            let Some((key, proof, authenticated)) = owned else {
                return false;
            };
            if !identity_proofs::claim(authenticated) {
                return false;
            }
            STATE.with(|cell| {
                let mut state = cell.borrow_mut();
                state.confirmed.remove(&key);
                state.confirmed_at.remove(&key.install);
                state.join_hits += 1;
            });
            let row = &proof.token.owner.episode.row;
            eprintln!(
                "CM740_SOURCE_HIT row={row} phase=OwnDeferredDefinitionPublicationAtActualProfileQueueJoin selector={row}"
            );
            eprintln!(
                "CM742_JOIN_HIT_ID program={:?} pair={} activation={:?} producer_occurrence={} generation={} namespace={:?}",
                proof.token.owner.episode.program,
                proof.token.owner.episode.pair,
                proof.token.owner.path,
                proof.token.occurrence,
                proof.installed.generation,
                proof.installed.namespace
            );
            true
        }
        pub fn retire_root(root: usize) {
            if !run_is_driven() {
                return;
            }
            identity_proofs::retire_registered(
                |n| n.0 == root,
                |registration| {
                    STATE.with(|cell| cell.borrow_mut().retire(registration));
                },
            );
        }
        pub fn retire_control(control: usize) {
            if !run_is_driven() {
                return;
            }
            identity_proofs::retire_registered(
                |n| n.2 == control,
                |registration| {
                    STATE.with(|cell| cell.borrow_mut().retire(registration));
                },
            );
        }
        pub fn clear_registry() {
            if !run_is_driven() {
                return;
            }
            identity_proofs::retire_registered(
                |_| true,
                |registration| {
                    STATE.with(|cell| cell.borrow_mut().retire(registration));
                },
            );
        }
        pub fn hits() -> usize {
            STATE.with(|c| c.borrow().hits)
        }
        pub fn join_hits() -> usize {
            STATE.with(|c| c.borrow().join_hits)
        }
    }

    pub fn construction_counts() -> (usize, usize) {
        LEDGER.with(|c| {
            let l = c.borrow();
            (
                l.bound_slots.len(),
                l.current
                    .as_ref()
                    .map(|r| r.pair.tokens.values().map(Vec::len).sum())
                    .unwrap_or(0),
            )
        })
    }
    pub fn count_hits() -> usize {
        LEDGER.with(|c| {
            c.borrow()
                .current
                .as_ref()
                .map(|r| r.hits.values().sum::<u64>() as usize)
                .unwrap_or(0)
        })
    }
    pub fn count_arms() -> usize {
        LEDGER.with(|c| {
            c.borrow()
                .current
                .as_ref()
                .map(|r| r.pair.next_occurrence as usize)
                .unwrap_or(0)
        })
    }
    pub fn corrupt(kind: u8) {
        LEDGER.with(|c| {
            let mut l = c.borrow_mut();
            let token = l.bound_slots.values_mut().next().unwrap();
            if kind == 1 {
                token.occurrence += 1;
            } else {
                token.owner.path.push(("wrong-owner", 99));
            }
        });
    }

    // Observation attaches a serial to the selected lexical invocation. The candidate
    // still chooses its token and construction record independently, from live state.
    pub mod retirement_observation {
        use super::*;
        use std::cell::{Cell, RefCell};

        type Key = (usize, &'static str);
        thread_local! {
            static SCENE: Cell<&'static str> = const { Cell::new("") };
            static SERIAL: Cell<u64> = const { Cell::new(0) };
            static CALLS: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
            static MAKER: Cell<u64> = const { Cell::new(0) };
            static PHYSICAL: RefCell<Vec<Key>> = const { RefCell::new(Vec::new()) };
            static SURVIVORS: RefCell<Vec<Key>> = const { RefCell::new(Vec::new()) };
        }
        pub fn scene(name: &'static str) {
            SCENE.set(name);
        }
        pub fn scene_name() -> &'static str {
            SCENE.get()
        }

        fn path(a: &Activation) -> String {
            a.path
                .iter()
                .map(|(row, index)| format!("{row}@{index}"))
                .collect::<Vec<_>>()
                .join("/")
        }

        // Observe the token at its writer, in the parked pair, and at the
        // actual queued invocation. Keep the complete identity in the log.
        fn token(where_: &str, id: u64, token: &Token) {
            let (key, kind) = token.frame_identity.unwrap_or((0, "-"));
            eprintln!(
                "CM742C75 TOKEN scene={} at={where_} id={id} pair={} program={} row={} owner={} origin_pair={} origin_program={} origin_row={} origin={} occurrence={} key={key} kind={kind}",
                SCENE.get(),
                token.owner.episode.pair,
                token.owner.episode.program,
                token.owner.episode.row,
                path(&token.owner),
                token.origin.episode.pair,
                token.origin.episode.program,
                token.origin.episode.row,
                path(&token.origin),
                token.occurrence
            );
        }

        pub fn window(at: &str) {
            LEDGER.with_borrow(|l| {
            let run = l.current.as_ref().expect("opened source window");
            let e = run.pair.episode.as_ref().expect("actual program pair");
            eprintln!("CM742C75 WINDOW scene={} at={at} pair={} row={} program={} queued={} current={} sources={} frames={}",
                SCENE.get(), e.pair, e.row, e.program, run.queued, run_current(run),
                run.pair.source_records.values().filter(|k| **k == SourceKind::ProducerPause).count(),
                run.pair.tokens.values().map(Vec::len).sum::<usize>());
            for (source, kind) in &run.pair.source_records {
                if *kind == SourceKind::ProducerPause { token(&format!("{at}_source"), 0, source); }
            }
            for t in run.pair.tokens.values().flatten() { token(at, 0, t); }
        });
        }

        fn queued_call(id: u64, guard: &Guard) {
            LEDGER.with_borrow(|l| {
                let Some(run) = l.current.as_ref().filter(|r| r.queued) else {
                    return;
                };
                let Some(actual) = guard.0.as_ref().map(|active| &active.activation) else {
                    return;
                };
                eprintln!(
                    "CM742C75 VISIT scene={} id={id} pair={} row={} program={} path={} top={}",
                    SCENE.get(),
                    actual.episode.pair,
                    actual.episode.row,
                    actual.episode.program,
                    path(actual),
                    l.stack.last() == Some(actual)
                );
                if let Some(proposal) = run.pair.tokens.get(actual).and_then(|v| v.last()) {
                    token("queued_candidate", id, proposal);
                }
            });
        }
        pub struct Invocation {
            row: &'static str,
            id: u64,
            guard: Option<Guard>,
        }
        impl Invocation {
            pub fn enter(row: &'static str) -> Self {
                if !selected(row) || SCENE.get().is_empty() {
                    return Self {
                        row,
                        id: 0,
                        guard: None,
                    };
                }
                let id = SERIAL.get() + 1;
                SERIAL.set(id);
                CALLS.with_borrow_mut(|stack| stack.push(id));
                eprintln!(
                    "CM742C75 CALL scene={} row={row} id={id} event=enter",
                    SCENE.get()
                );
                let guard = enter_selected(row);
                if let Some(a) = guard.0.as_ref().map(|active| &active.activation) {
                    eprintln!(
                        "CM742C76 ACTIVATION scene={} id={id} pair={} program={} row={} path={}",
                        SCENE.get(),
                        a.episode.pair,
                        a.episode.program,
                        a.episode.row,
                        path(a)
                    );
                }
                Self {
                    row,
                    id,
                    guard: Some(guard),
                }
            }
            pub fn result<T>(&mut self, value: &Result<T, crate::EvalError>) {
                if self.id == 0 {
                    return;
                }
                assert_eq!(CALLS.with_borrow_mut(Vec::pop), Some(self.id));
                let event = match value {
                    Ok(_) => "ok",
                    Err(crate::EvalError::InputRequired { .. }) => "input",
                    Err(_) => "error",
                };
                eprintln!(
                    "CM742C75 CALL scene={} row={} id={} event={event} ordinary={}",
                    SCENE.get(),
                    self.row,
                    self.id,
                    ordinary(self.row)
                );
                if let Some(g) = &self.guard {
                    queued_call(self.id, g);
                }
                if event != "input" {
                    self.guard.take();
                }
            }
            pub fn projected_maker(&self) -> Option<Maker> {
                if self.id != 0 {
                    Some(maker(self.id))
                } else {
                    CALLS.with_borrow(|ids| ids.last().copied().map(maker))
                }
            }
            pub fn construction(&self) -> Option<(Maker, ConstructionGuard)> {
                self.guard
                    .as_ref()
                    .map(|g| (maker(self.id), g.suspending()))
            }
        }
        impl Drop for Invocation {
            fn drop(&mut self) {
                if self.id != 0 {
                    eprintln!(
                        "CM742C75 CALL scene={} row={} id={} event=exit",
                        SCENE.get(),
                        self.row,
                        self.id
                    );
                }
            }
        }
        pub struct Maker(u64);
        fn maker(id: u64) -> Maker {
            eprintln!("CM742C75 FACTORY scene={} id={id}", SCENE.get());
            Maker(MAKER.replace(id))
        }
        pub fn actual_input(prompt: &str) {
            if !SCENE.get().is_empty() {
                let id = CALLS.with_borrow(|ids| ids.last().copied().unwrap_or(0));
                eprintln!(
                    "CM742C75 INPUT scene={} id={id} prompt={prompt:?}",
                    SCENE.get()
                );
            }
        }
        impl Drop for Maker {
            fn drop(&mut self) {
                MAKER.set(self.0);
            }
        }
        pub fn frame(event: &str, key: usize, kind: &'static str) {
            let scene = SCENE.get();
            if scene.is_empty() {
                return;
            }
            let direct = OPERATIONS.with(|cell| {
                let o = cell.borrow();
                let id = *o.direct.get(&(key, kind))?;
                let record = o.records.get(&id)?;
                let projected = o.projected.get(&(key, kind)).copied();
                Some((
                    id,
                    record.token.clone(),
                    o.retired.contains(&id),
                    projected,
                    projected.is_some_and(|x| o.retired.contains(&x)),
                ))
            });
            if event == "allocate" && MAKER.get() != 0 {
                if let Some((_, issued, _, _, _)) = &direct {
                    token("issued", MAKER.get(), issued);
                    LEDGER.with_borrow(|l| {
                    let mut source = issued.clone();
                    source.frame_identity = None;
                    if let Some((actual, edge)) = l.current.as_ref().and_then(|r| r.pair.source_records.get_key_value(&source)) {
                        match edge {
                            SourceKind::ProducerPause => token("native_pause", MAKER.get(), actual),
                            SourceKind::ParentReturn => token("parent_return", MAKER.get(), actual),
                            _ => token("other_source", MAKER.get(), actual)
                        }
                    }
                    if let Some(a) = l.stack.last() {
                        eprintln!("CM742C76 CONSTRUCTING scene={scene} id={} pair={} program={} row={} path={} scope_live={}",
                            MAKER.get(), a.episode.pair, a.episode.program, a.episode.row, path(a),
                            l.scope_authority.stack.last() == Some(a) && l.scope_authority.entries.contains_key(a));
                    }
                });
                }
            }
            let (record, occ, live, bound, paired, retired, projected, projected_retired, owner) =
                direct
                    .map(|(id, t, r, p, pr)| {
                        LEDGER.with(|cell| {
                            let l = cell.borrow();
                            (
                                id,
                                t.occurrence.to_string(),
                                l.current_frame_records.get(&t) == Some(&true),
                                l.bound_slots.get(&(key, kind)) == Some(&t),
                                l.current
                                    .as_ref()
                                    .and_then(|v| v.pair.tokens.get(&t.owner))
                                    .is_some_and(|v| v.contains(&t)),
                                r,
                                p.unwrap_or(0),
                                pr,
                                format!("{:?}", t.owner.path),
                            )
                        })
                    })
                    .unwrap_or((
                        0,
                        String::from("-"),
                        false,
                        false,
                        false,
                        false,
                        0,
                        false,
                        String::from("[]"),
                    ));
            if event == "allocate"
                && record != 0
                && MAKER.get() != 0
                && kind.rsplit("::").next() == Some(std::env::var("CM742_FRAME").unwrap().as_str())
            {
                if scene == "survivor" {
                    SURVIVORS.with_borrow_mut(|s| s.push((key, kind)));
                }
                if scene == "physical" {
                    PHYSICAL.with_borrow_mut(|s| s.push((key, kind)));
                }
            }
            eprintln!(
                "CM742C75 FRAME scene={scene} event={event} key={key} kind={kind} maker={} record={record} occurrence={occ} live={live} bound={bound} paired={paired} retired={retired} projected={projected} projected_retired={projected_retired} owner={owner}",
                MAKER.get()
            );
        }
        pub fn proposal(actual: Key, boundary: &str) -> Key {
            if SCENE.get() != "retire" {
                return actual;
            }
            let fault = std::env::var("CM742_FAULT").unwrap();
            let used = std::env::var("CM742_ACTION").unwrap();
            if fault == "valid" || boundary != used {
                return actual;
            }
            let eligible = OPERATIONS.with(|c| {
                let o = c.borrow();
                o.direct
                    .get(&actual)
                    .is_some_and(|id| !o.retired.contains(id))
            });
            if !eligible || !PHYSICAL.with_borrow(|v| v.contains(&actual)) {
                return actual;
            }
            let proposed = match fault.as_str() {
                "foreign" => SURVIVORS
                    .with_borrow(|keys| {
                        keys.iter().copied().find(|k| {
                            k.1 == actual.1
                                && k.0 != actual.0
                                && OPERATIONS.with(|c| {
                                    let o = c.borrow();
                                    o.direct.get(k).is_some_and(|id| !o.retired.contains(id))
                                })
                        })
                    })
                    .expect("independently live same-type receiver"),
                "missing" => (0, actual.1),
                "kind" => (actual.0, "cm742::WrongConcreteType"),
                _ => panic!("unregistered proposal"),
            };
            eprintln!(
                "CM742C75 PROPOSAL boundary={boundary} fault={fault} actual={} actual_kind={} proposed={} proposed_kind={}",
                actual.0, actual.1, proposed.0, proposed.1
            );
            proposed
        }
    }

    #[path = "class_lease.rs"]
    pub mod class_lease;
    #[path = "q25_ownership.rs"]
    pub mod q25;
}
