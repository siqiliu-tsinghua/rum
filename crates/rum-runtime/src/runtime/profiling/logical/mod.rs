//! LP02 infrastructure and LP03 evaluator, dispatch and argument phase records.
use crate::runtime::{EvalError, Expr, suspension::Slot};
use std::sync::Arc;

mod budget;
mod context;
pub(in crate::runtime) mod epoch;
pub(in crate::runtime) mod index;
mod queue_api;
mod record;
pub(in crate::runtime) mod segments;
pub(in crate::runtime) mod source_stack;
mod totals;

pub(in crate::runtime) use budget::Storage;
pub(in crate::runtime) use context::{Context, ContextGuard};
pub(in crate::runtime) use epoch::{Epoch, drop_session, summary};
pub(in crate::runtime) use record::*;

pub(in crate::runtime) struct ProfileBoundary {
    pub(in crate::runtime) epoch: Arc<Epoch>,
    pub(in crate::runtime) record: RecordId,
    pub(in crate::runtime) _storage: Storage,
}

impl ProfileBoundary {
    pub(in crate::runtime) fn belongs_to(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
    ) -> bool {
        self.epoch
            .inspect(self.record)
            .is_some_and(|(owner, _, _)| {
                Arc::ptr_eq(&owner.control, control)
                    && owner.generation == control.logical_generation()
            })
    }
    pub(in crate::runtime) fn complete(&self, outcome: Outcome) {
        segments::finish_active(&self.epoch, self.record);
        self.epoch.close(self.record, Ok(outcome));
    }
    pub(in crate::runtime) fn abandon(&self, reason: Retire) {
        segments::finish_active(&self.epoch, self.record);
        self.epoch.close(
            self.record,
            match reason {
                Retire::Structural(crate::runtime::suspension::AbandonReason::CancelledRequest) => {
                    Ok(Outcome::Cancelled)
                }
                _ => Err(reason),
            },
        );
    }
    pub(in crate::runtime) fn policy(&self) -> Option<Policy> {
        self.epoch.inspect(self.record).map(|(_, _, p)| p)
    }
}
impl Drop for ProfileBoundary {
    fn drop(&mut self) {
        self.abandon(Retire::LastDrop);
    }
}

pub(in crate::runtime) struct Call {
    inner: Box<CallInner>,
}

struct CallInner {
    epoch: Arc<Epoch>,
    record: RecordId,
    segment: Option<segments::Segment>,
    finished: bool,
    _storage: Storage,
}

#[derive(Clone)]
pub(in crate::runtime) struct Frontier {
    pub(in crate::runtime) generation: crate::runtime::suspension::SuspensionGeneration,
    pub(in crate::runtime) after: Option<crate::runtime::suspension::CellId>,
    pub(in crate::runtime) control: Arc<crate::runtime::RequestInterruptState>,
    pub(in crate::runtime) request_generation: u64,
}

pub(in crate::runtime) fn error_outcome(error: &crate::runtime::EvalError) -> Outcome {
    use crate::runtime::EvalError;
    match error {
        EvalError::Return { .. } => Outcome::Return,
        EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. } => Outcome::Throw,
        EvalError::Cancelled => Outcome::Cancelled,
        _ => Outcome::Error,
    }
}

impl crate::runtime::Session {
    #[inline]
    pub(in crate::runtime) fn logical_phase_start(
        &self,
        phase: &'static str,
        policy: Policy,
    ) -> Option<Call> {
        if !crate::runtime::profiling::lexical_profile_enabled() {
            return None;
        }
        Self::logical_phase_start_slow(self, phase, policy)
    }

    #[cold]
    #[inline(never)]
    fn logical_phase_start_slow(&self, phase: &'static str, policy: Policy) -> Option<Call> {
        Call::start(
            self,
            AggregateKey::Runtime {
                phase,
                source: crate::runtime::evaluation_profile_current_source_symbol(),
            },
            policy,
        )
    }

    #[inline]
    pub(in crate::runtime) fn logical_definition_rule_start(
        &self,
        kind: &'static str,
        symbol: crate::runtime::Symbol,
        index: usize,
    ) -> Option<Call> {
        if !crate::runtime::profiling::lexical_profile_enabled() {
            return None;
        }
        Self::logical_definition_rule_start_slow(self, kind, symbol, index)
    }

    #[cold]
    #[inline(never)]
    fn logical_definition_rule_start_slow(
        &self,
        kind: &'static str,
        symbol: crate::runtime::Symbol,
        index: usize,
    ) -> Option<Call> {
        let index = u32::try_from(index).ok()?;
        Call::start(
            self,
            AggregateKey::Definition {
                kind,
                symbol,
                index,
            },
            Policy::OptionalExpr,
        )
    }
}

pub(in crate::runtime) fn finish_logical_expr(
    call: Option<Call>,
    result: &Result<Expr, EvalError>,
) {
    finish_logical_total(call, result);
}

/// `rhs-total` in both overloads: a successful application total is `Ok`,
/// never `Some`, whatever the overload's carrier is.
pub(in crate::runtime) fn finish_logical_total<T>(
    call: Option<Call>,
    result: &Result<T, crate::runtime::EvalError>,
) {
    if let Some(call) = call {
        call.finish(match result {
            Ok(_) => Outcome::Ok,
            Err(error) => error_outcome(error),
        });
    }
}

pub(in crate::runtime) fn finish_logical_optional(
    call: Option<Call>,
    result: &Result<Option<Expr>, EvalError>,
) {
    if let Some(call) = call {
        call.finish(match result {
            Ok(Some(_)) => Outcome::Some,
            Ok(None) => Outcome::None,
            Err(error) => error_outcome(error),
        });
    }
}

pub(in crate::runtime) fn finish_logical_optional_none(call: Option<Call>) {
    if let Some(call) = call {
        call.finish(Outcome::None);
    }
}

/// Completes the rule record with the application overload's actual result.
/// `Err(InputRequired)` marks the pause and closes the record locally.
pub(in crate::runtime) fn complete_definition_rule(
    call: Option<Call>,
    result: &Result<Option<Expr>, crate::runtime::EvalError>,
) {
    let Some(call) = call else {
        return;
    };
    match result {
        Err(crate::runtime::EvalError::InputRequired { .. }) => call.finish_input_required(),
        other => finish_logical_optional(Some(call), other),
    }
}

pub(in crate::runtime) fn finish_logical_expr_ok(call: Option<Call>) {
    if let Some(call) = call {
        call.finish(Outcome::Ok);
    }
}

pub(in crate::runtime) fn finish_logical_error(call: Option<Call>, error: &EvalError) {
    if let Some(call) = call {
        call.finish(error_outcome(error));
    }
}

pub(in crate::runtime) fn finish_logical_prepared(
    call: Option<Call>,
    result: &Result<crate::runtime::eval_call_prepare::PreparedCallArguments, EvalError>,
) {
    if let Some(call) = call {
        call.finish(match result {
            Ok(_) => Outcome::Ok,
            Err(error) => error_outcome(error),
        });
    }
}

fn phase_timing_outcome<T>(result: &Result<T, EvalError>) -> super::RuntimeProfileOutcome {
    match result {
        Ok(_) => super::RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => super::RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. }) => {
            super::RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => super::RuntimeProfileOutcome::InputRequired,
        Err(_) => super::RuntimeProfileOutcome::Error,
    }
}

pub(in crate::runtime) trait PhaseCompletion {
    fn timing_outcome(result: &Result<Self, EvalError>) -> super::RuntimeProfileOutcome
    where
        Self: Sized;

    fn finish_logical(call: Option<Call>, result: &Result<Self, EvalError>)
    where
        Self: Sized;
}

impl PhaseCompletion for Expr {
    fn timing_outcome(result: &Result<Self, EvalError>) -> super::RuntimeProfileOutcome {
        phase_timing_outcome(result)
    }

    fn finish_logical(call: Option<Call>, result: &Result<Self, EvalError>) {
        finish_logical_expr(call, result);
    }
}

impl PhaseCompletion for Option<Expr> {
    fn timing_outcome(result: &Result<Self, EvalError>) -> super::RuntimeProfileOutcome {
        match result {
            Ok(Some(_)) => super::RuntimeProfileOutcome::Some,
            Ok(None) => super::RuntimeProfileOutcome::None,
            Err(_) => phase_timing_outcome(result),
        }
    }

    fn finish_logical(call: Option<Call>, result: &Result<Self, EvalError>) {
        finish_logical_optional(call, result);
    }
}

impl PhaseCompletion for crate::runtime::eval_call_prepare::PreparedCallArguments {
    fn timing_outcome(result: &Result<Self, EvalError>) -> super::RuntimeProfileOutcome {
        phase_timing_outcome(result)
    }

    fn finish_logical(call: Option<Call>, result: &Result<Self, EvalError>) {
        finish_logical_prepared(call, result);
    }
}

/// The matcher routes' carrier: a match and a miss both complete the call.
impl PhaseCompletion for crate::runtime::pattern_matching::CompiledMatchOutcome {
    fn timing_outcome(
        result: &Result<Self, crate::runtime::EvalError>,
    ) -> super::RuntimeProfileOutcome {
        phase_timing_outcome(result)
    }

    fn finish_logical(call: Option<Call>, result: &Result<Self, crate::runtime::EvalError>) {
        if let Some(call) = call {
            call.finish(match result {
                Ok(_) => Outcome::Ok,
                Err(error) => error_outcome(error),
            });
        }
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
pub struct LogicalProfileTestLease {
    inner: super::lease::TestLease,
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl crate::runtime::Session {
    #[doc(hidden)]
    pub fn logical_profile_test_acquire() -> LogicalProfileTestLease {
        LogicalProfileTestLease {
            inner: super::lease::lease(),
        }
    }
    /// Isolated native test span; no production phase uses the LP02 engine.
    #[doc(hidden)]
    pub fn logical_profile_test_span(
        &mut self,
        phase: &'static str,
        expr: crate::runtime::Expr,
    ) -> Result<crate::runtime::Expr, crate::runtime::EvalError> {
        if !super::lexical_profile_enabled() {
            return self.evaluate(expr);
        }
        let call = Call::start(
            self,
            AggregateKey::Runtime {
                phase,
                source: crate::runtime::evaluation_profile_current_source_symbol(),
            },
            Policy::Expr,
        );
        let outcome = self.evaluate(expr);
        if let Some(call) = call {
            match &outcome {
                Err(crate::runtime::EvalError::InputRequired { .. }) => {
                    if let Some(boundary) = call.pause() {
                        self.logical_insert_boundary(boundary, None, false);
                    }
                }
                Ok(_) => call.finish(Outcome::Ok),
                Err(error) => call.finish(error_outcome(error)),
            }
        }
        outcome
    }

    #[doc(hidden)]
    pub fn logical_profile_test_source_scope(
        &mut self,
        source: crate::runtime::Symbol,
        expr: crate::runtime::Expr,
    ) -> Result<crate::runtime::Expr, crate::runtime::EvalError> {
        let _guard = crate::runtime::evaluation_profile_definition_source(source);
        self.evaluate(expr)
    }

    #[doc(hidden)]
    pub fn logical_profile_test_rows() -> Vec<(String, [u64; 10])> {
        epoch::current()
            .map(|e| {
                e.rows()
                    .into_iter()
                    .map(|(key, c)| {
                        (
                            format!("{key:?}"),
                            [
                                c.calls,
                                c.ok,
                                c.err,
                                c.some,
                                c.none,
                                c.return_err,
                                c.throw_err,
                                c.cancelled,
                                c.abandoned,
                                c.open,
                            ],
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn logical_profile_test_leases() -> usize {
        epoch::current()
            .map(|e| e.contexts.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0)
    }

    #[doc(hidden)]
    pub fn logical_profile_test_assert_quiescent() {
        if let Some(epoch) = epoch::current() {
            epoch.assert_quiescent();
        }
    }

    #[doc(hidden)]
    pub fn logical_profile_test_reset(lease: &LogicalProfileTestLease) {
        epoch::reset_for_lease(&lease.inner);
    }

    #[doc(hidden)]
    pub fn logical_profile_test_reasons() -> [u64; 11] {
        let mut reasons = [0; 11];
        if let Some(epoch) = epoch::current() {
            for (_, counts) in epoch.rows() {
                for (target, value) in reasons.iter_mut().zip(counts.abandonment_reasons) {
                    *target += value;
                }
            }
        }
        reasons
    }

    #[doc(hidden)]
    pub fn logical_profile_test_pauses() -> u64 {
        epoch::current()
            .map(|e| {
                e.rows()
                    .iter()
                    .map(|(_, c)| c.pauses + c.input_required)
                    .sum()
            })
            .unwrap_or(0)
    }

    #[doc(hidden)]
    pub fn logical_profile_test_pauses_for(phase: &'static str) -> u64 {
        epoch::current()
            .map(|e| {
                e.rows()
                    .iter()
                    .filter(|(key, _)| {
                        matches!(key, AggregateKey::Runtime { phase: actual, .. } if *actual == phase)
                    })
                    .map(|(_, c)| c.pauses)
                    .sum()
            })
            .unwrap_or(0)
    }
}

impl Call {
    /// The caller invokes this only inside its existing enabled slow path.
    pub(in crate::runtime) fn start(
        session: &crate::runtime::Session,
        key: AggregateKey,
        policy: Policy,
    ) -> Option<Self> {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        if crate::runtime::profiling::lease::blocks_unregistered_thread() {
            return None;
        }
        let control = session.request_interrupt_control.borrow();
        let (epoch, owner, parent) = match segments::owner(&control) {
            Some((epoch, owner, parent)) => (epoch, owner, Some(parent)),
            None => (
                epoch::current()?,
                Owner {
                    control: Arc::clone(&control),
                    generation: control.logical_generation(),
                    root: epoch::next_id(),
                },
                None,
            ),
        };
        let record = epoch.open(owner, parent, key, policy)?;
        let segment = segments::enter(&epoch, record)?;
        let storage = epoch.storage(std::mem::size_of::<CallInner>() + 16)?;
        Some(Self {
            inner: Box::new(CallInner {
                epoch,
                record,
                segment: Some(segment),
                finished: false,
                _storage: storage,
            }),
        })
    }
    pub(in crate::runtime) fn finish(mut self, outcome: Outcome) {
        self.inner.segment.take();
        self.inner.epoch.close(self.inner.record, Ok(outcome));
        self.inner.finished = true;
    }
    fn finish_input_required(self) {
        self.inner.epoch.input_required(self.inner.record);
        self.finish(Outcome::Error);
    }
    pub(in crate::runtime) fn pause(mut self) -> Option<Arc<ProfileBoundary>> {
        if let Some(segment) = &self.inner.segment {
            segment.pause();
        }
        self.inner.segment.take();
        let storage = self
            .inner
            .epoch
            .storage(std::mem::size_of::<ProfileBoundary>() + 16)?;
        self.inner.finished = true;
        Some(Arc::new(ProfileBoundary {
            epoch: Arc::clone(&self.inner.epoch),
            record: self.inner.record,
            _storage: storage,
        }))
    }
}

#[derive(Clone, Copy)]
pub(in crate::runtime) struct AdoptedSite {
    pub policy: Policy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum PhaseKey {
    EvaluateTotal,
    EvaluateOnceUnstable,
    EvaluateInnerUnstableTotal,
    EvaluateInnerUnstableSymbol,
    EvaluateInnerUnstableCall,
    EvaluateInnerUnstableAtom,
    EvaluateAtomFastPathTotal,
    EvaluateAtomFastPathStableSymbol,
    EvaluateAtomFastPathUnstableSymbol,
    EvaluateAtomFastPathAtom,
    EvaluateAtomFastPathCall,
    EvaluateCallTotal,
    EvaluateCallHeadEval,
    EvaluateCallResumeAfterHead,
    EvaluateCallDispatch,
    CallStageSpecialForm,
    CallStagePrepareArguments,
    CallStageListable,
    CallStageUpValues,
    CallStageDownValues,
    CallStageSubValues,
    CallStageOrdinaryBuiltin,
    CallStageFunction,
    PrepareArgumentsTotal,
    PrepareArgumentsEvaluateArg,
    DefinitionRhsTotal,
    DefinitionRhsEvaluate,
    DownValuesDirectDefaultLookup,
    DownValuesCompiledMatcher,
    SubValuesCompiledMatcher,
}

impl PhaseKey {
    pub(in crate::runtime) const ALL: [PhaseKey; 30] = [
        PhaseKey::EvaluateTotal,
        PhaseKey::EvaluateOnceUnstable,
        PhaseKey::EvaluateInnerUnstableTotal,
        PhaseKey::EvaluateInnerUnstableSymbol,
        PhaseKey::EvaluateInnerUnstableCall,
        PhaseKey::EvaluateInnerUnstableAtom,
        PhaseKey::EvaluateAtomFastPathTotal,
        PhaseKey::EvaluateAtomFastPathStableSymbol,
        PhaseKey::EvaluateAtomFastPathUnstableSymbol,
        PhaseKey::EvaluateAtomFastPathAtom,
        PhaseKey::EvaluateAtomFastPathCall,
        PhaseKey::EvaluateCallTotal,
        PhaseKey::EvaluateCallHeadEval,
        PhaseKey::EvaluateCallResumeAfterHead,
        PhaseKey::EvaluateCallDispatch,
        PhaseKey::CallStageSpecialForm,
        PhaseKey::CallStagePrepareArguments,
        PhaseKey::CallStageListable,
        PhaseKey::CallStageUpValues,
        PhaseKey::CallStageDownValues,
        PhaseKey::CallStageSubValues,
        PhaseKey::CallStageOrdinaryBuiltin,
        PhaseKey::CallStageFunction,
        PhaseKey::PrepareArgumentsTotal,
        PhaseKey::PrepareArgumentsEvaluateArg,
        PhaseKey::DefinitionRhsTotal,
        PhaseKey::DefinitionRhsEvaluate,
        PhaseKey::DownValuesDirectDefaultLookup,
        PhaseKey::DownValuesCompiledMatcher,
        PhaseKey::SubValuesCompiledMatcher,
    ];

    pub(in crate::runtime) const fn name(self) -> &'static str {
        match self {
            PhaseKey::EvaluateTotal => "evaluate.total",
            PhaseKey::EvaluateOnceUnstable => "evaluate.once_unstable",
            PhaseKey::EvaluateInnerUnstableTotal => "evaluate_inner_unstable.total",
            PhaseKey::EvaluateInnerUnstableSymbol => "evaluate_inner_unstable.symbol",
            PhaseKey::EvaluateInnerUnstableCall => "evaluate_inner_unstable.call",
            PhaseKey::EvaluateInnerUnstableAtom => "evaluate_inner_unstable.atom",
            PhaseKey::EvaluateAtomFastPathTotal => "evaluate_atom_fast_path.total",
            PhaseKey::EvaluateAtomFastPathStableSymbol => "evaluate_atom_fast_path.stable_symbol",
            PhaseKey::EvaluateAtomFastPathUnstableSymbol => {
                "evaluate_atom_fast_path.unstable_symbol"
            }
            PhaseKey::EvaluateAtomFastPathAtom => "evaluate_atom_fast_path.atom",
            PhaseKey::EvaluateAtomFastPathCall => "evaluate_atom_fast_path.call",
            PhaseKey::EvaluateCallTotal => "evaluate_call.total",
            PhaseKey::EvaluateCallHeadEval => "evaluate_call.head_eval",
            PhaseKey::EvaluateCallResumeAfterHead => "evaluate_call.resume_after_head",
            PhaseKey::EvaluateCallDispatch => "evaluate_call.dispatch",
            PhaseKey::CallStageSpecialForm => "call.stage.special_form",
            PhaseKey::CallStagePrepareArguments => "call.stage.prepare_arguments",
            PhaseKey::CallStageListable => "call.stage.listable",
            PhaseKey::CallStageUpValues => "call.stage.up_values",
            PhaseKey::CallStageDownValues => "call.stage.down_values",
            PhaseKey::CallStageSubValues => "call.stage.sub_values",
            PhaseKey::CallStageOrdinaryBuiltin => "call.stage.ordinary_builtin",
            PhaseKey::CallStageFunction => "call.stage.function",
            PhaseKey::PrepareArgumentsTotal => "prepare_arguments.total",
            PhaseKey::PrepareArgumentsEvaluateArg => "prepare_arguments.evaluate_arg",
            PhaseKey::DefinitionRhsTotal => "definition.rhs.total",
            PhaseKey::DefinitionRhsEvaluate => "definition.rhs.evaluate",
            PhaseKey::DownValuesDirectDefaultLookup => "down_values.direct_default_lookup",
            PhaseKey::DownValuesCompiledMatcher => "down_values.compiled_matcher",
            PhaseKey::SubValuesCompiledMatcher => "sub_values.compiled_matcher",
        }
    }

    pub(in crate::runtime) fn from_name(phase: &'static str) -> Option<Self> {
        let index = match phase {
            "evaluate.total" => 0,
            "evaluate.once_unstable" => 1,
            "evaluate_inner_unstable.total" => 2,
            "evaluate_inner_unstable.symbol" => 3,
            "evaluate_inner_unstable.call" => 4,
            "evaluate_inner_unstable.atom" => 5,
            "evaluate_atom_fast_path.total" => 6,
            "evaluate_atom_fast_path.stable_symbol" => 7,
            "evaluate_atom_fast_path.unstable_symbol" => 8,
            "evaluate_atom_fast_path.atom" => 9,
            "evaluate_atom_fast_path.call" => 10,
            "evaluate_call.total" => 11,
            "evaluate_call.head_eval" => 12,
            "evaluate_call.resume_after_head" => 13,
            "evaluate_call.dispatch" => 14,
            "call.stage.special_form" => 15,
            "call.stage.prepare_arguments" => 16,
            "call.stage.listable" => 17,
            "call.stage.up_values" => 18,
            "call.stage.down_values" => 19,
            "call.stage.sub_values" => 20,
            "call.stage.ordinary_builtin" => 21,
            "call.stage.function" => 22,
            "prepare_arguments.total" => 23,
            "prepare_arguments.evaluate_arg" => 24,
            "definition.rhs.total" => 25,
            "definition.rhs.evaluate" => 26,
            "down_values.direct_default_lookup" => 27,
            "down_values.compiled_matcher" => 28,
            "sub_values.compiled_matcher" => 29,
            _ => Self::ALL.len(),
        };
        Self::ALL.get(index).copied()
    }

    pub(in crate::runtime) const fn site(self) -> AdoptedSite {
        let policy = match self {
            PhaseKey::CallStageSpecialForm
            | PhaseKey::CallStagePrepareArguments
            | PhaseKey::CallStageListable
            | PhaseKey::CallStageUpValues
            | PhaseKey::CallStageDownValues
            | PhaseKey::CallStageSubValues
            | PhaseKey::CallStageOrdinaryBuiltin
            | PhaseKey::CallStageFunction
            | PhaseKey::DownValuesDirectDefaultLookup => Policy::OptionalExpr,
            PhaseKey::PrepareArgumentsTotal => Policy::Prepared,
            PhaseKey::DownValuesCompiledMatcher | PhaseKey::SubValuesCompiledMatcher => {
                Policy::Match
            }
            _ => Policy::Expr,
        };
        AdoptedSite { policy }
    }
}

struct PhaseEntry {
    key: PhaseKey,
    call: Option<Call>,
    _storage: Option<Storage>,
}

thread_local! {
    static PHASE_LIFO: std::cell::RefCell<Vec<PhaseEntry>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

#[inline(always)]
pub(in crate::runtime) fn open_phase(
    session: &crate::runtime::Session,
    key: PhaseKey,
    site: AdoptedSite,
) {
    let Some(call) = session.logical_phase_start(key.name(), site.policy) else {
        return;
    };
    PHASE_LIFO.with_borrow_mut(|stack| {
        let storage = match executing_epoch() {
            Some(epoch) => match epoch.storage(std::mem::size_of::<PhaseEntry>() + 16) {
                Some(storage) => Some(storage),
                None => {
                    epoch.retire(Retire::DiagnosticCap);
                    None
                }
            },
            None => None,
        };
        stack.push(PhaseEntry {
            key,
            call: call.into(),
            _storage: storage,
        });
    });
}

fn take_phase(key: PhaseKey) -> Option<PhaseEntry> {
    PHASE_LIFO.with_borrow_mut(|stack| {
        let entry = stack.last()?;
        if entry.key != key {
            if let Some(call) = &entry.call {
                call.inner.epoch.fault();
            }
            return None;
        }
        stack.pop()
    })
}

#[cold]
#[inline(never)]
pub(in crate::runtime) fn complete_phase(
    _session: &mut crate::runtime::Session,
    key: PhaseKey,
    outcome: super::RuntimeProfileOutcome,
    finish: impl FnOnce(Option<Call>),
) {
    let Some(entry) = take_phase(key) else {
        return;
    };
    let Some(call) = entry.call else {
        return;
    };
    if matches!(outcome, super::RuntimeProfileOutcome::InputRequired) {
        call.finish_input_required();
    } else {
        finish(call.into());
    }
}

#[inline]
pub(in crate::runtime) fn open_logical_phase(
    session: &mut crate::runtime::Session,
    phase: &'static str,
) {
    if !crate::runtime::profiling::lexical_profile_enabled() {
        return;
    }
    if let Some(key) = PhaseKey::from_name(phase) {
        open_phase(session, key, key.site());
    }
}

#[inline]
pub(in crate::runtime) fn complete_logical_expr_defer_pause(
    session: &mut crate::runtime::Session,
    phase: &'static str,
    result: &Result<Expr, EvalError>,
) {
    if !crate::runtime::profiling::lexical_profile_enabled() {
        return;
    }
    complete_logical_expr_defer_pause_slow(session, phase, result)
}

#[cold]
#[inline(never)]
fn complete_logical_expr_defer_pause_slow(
    session: &mut crate::runtime::Session,
    phase: &'static str,
    result: &Result<Expr, EvalError>,
) {
    let Some(key) = PhaseKey::from_name(phase) else {
        return;
    };
    let outcome = <Expr as PhaseCompletion>::timing_outcome(result);
    if !matches!(outcome, super::RuntimeProfileOutcome::InputRequired) {
        complete_phase(session, key, outcome, |call| {
            finish_logical_expr(call, result)
        });
    }
}

#[inline]
pub(in crate::runtime) fn pause_phase_at_finish(
    session: &mut crate::runtime::Session,
    phase: &'static str,
) {
    if !crate::runtime::profiling::lexical_profile_enabled() {
        return;
    }
    pause_phase_at_finish_slow(session, phase)
}

#[cold]
#[inline(never)]
fn pause_phase_at_finish_slow(session: &mut crate::runtime::Session, phase: &'static str) {
    if let Some(key) = PhaseKey::from_name(phase) {
        complete_phase(
            session,
            key,
            super::RuntimeProfileOutcome::InputRequired,
            |_| {},
        );
    }
}

#[inline]
pub(in crate::runtime) fn complete_phase_error_at_finish(
    session: &mut crate::runtime::Session,
    token: &Option<crate::runtime::profiling::RuntimePathTimingToken>,
    error: &EvalError,
) {
    let Some(token) = token.as_ref() else {
        return;
    };
    complete_phase_error_at_finish_slow(session, token.key.phase, error)
}

#[cold]
#[inline(never)]
fn complete_phase_error_at_finish_slow(
    session: &mut crate::runtime::Session,
    phase: &'static str,
    error: &EvalError,
) {
    if let Some(key) = PhaseKey::from_name(phase) {
        complete_phase(session, key, super::RuntimeProfileOutcome::Error, |call| {
            finish_logical_error(call, error)
        });
    }
}

impl Drop for Call {
    fn drop(&mut self) {
        self.inner.segment.take();
        if !self.inner.finished {
            self.inner
                .epoch
                .close(self.inner.record, Err(Retire::LastDrop));
        }
    }
}

pub(in crate::runtime) fn executing_epoch() -> Option<Arc<Epoch>> {
    segments::executing_epoch()
}

impl ProfileBoundary {
    /// A raw read only. This path never obtains a codec key or a resume token.
    pub(in crate::runtime) fn observe(&self, result: &Result<Slot, EvalError>) {
        use crate::runtime::suspension::ContinuationState;
        let Some(policy) = self.policy() else {
            return;
        };
        let outcome = match result {
            Err(EvalError::InputRequired { .. }) => return,
            Err(EvalError::Return { .. }) => Outcome::Return,
            Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. }) => {
                Outcome::Throw
            }
            Err(EvalError::Cancelled) => Outcome::Cancelled,
            Err(_) => Outcome::Error,
            Ok(slot) => match (policy, slot) {
                (Policy::Incoming(outcome), Slot::Value(_)) => outcome,
                (Policy::Expr, Slot::Value(_)) => Outcome::Ok,
                (Policy::ApplicableExpr, Slot::Value(_)) => Outcome::Some,
                (Policy::OptionalExpr, Slot::State(ContinuationState::OptionalExpr(value))) => {
                    if value.is_some() {
                        Outcome::Some
                    } else {
                        Outcome::None
                    }
                }
                (Policy::Bool, Slot::State(ContinuationState::Bool(_)))
                | (Policy::Prepared, Slot::State(ContinuationState::PreparedArguments(_)))
                | (Policy::Match, Slot::State(ContinuationState::MatchOutcome(_))) => Outcome::Ok,
                _ => {
                    self.epoch.fault();
                    return;
                }
            },
        };
        self.complete(if let Policy::Incoming(incoming) = policy {
            incoming
        } else {
            outcome
        });
    }
}

#[cfg(test)]
pub(in crate::runtime) use super::tests::engine_support as test_support;

#[cfg(test)]
#[path = "../tests/evaluator.rs"]
mod evaluator;

#[cfg(test)]
#[path = "../tests/definitions.rs"]
mod definitions;
