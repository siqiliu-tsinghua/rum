use std::collections::VecDeque;
use std::mem::size_of;
#[cfg(test)]
use std::mem::size_of_val;
use std::sync::Arc;
use std::time::Instant;

use super::trace::{FrameRootTrace, FrameRootVisitor};
use super::typed::{
    Action, DecodeFailure, Entry, EntryRef, ErasedFrame, Slot, SlotValue, TypedFrame,
    decode_completion, erase,
};
use super::*;

mod abandon;
mod evaluation_context;
pub(super) mod fault;
mod queue;
use crate::runtime::session::guest_loading::{
    GuestFileTransaction, GuestFormEnvironment, GuestFormScope,
};
use evaluation_context::{Queued, ResumeContext};
use fault::DriverFault;
use queue::FrameQueue;

type DynSuspensionFrame = dyn SuspensionFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct SuspensionGeneration(u64);

pub(in crate::runtime) struct SuspendedEvaluation {
    pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
    generation: SuspensionGeneration,
    guest_form_environment: Option<Box<GuestFormEnvironment>>,
    pub(super) frames: FrameQueue,
}

/// Opaque owner slot for the one pending continuation.
///
/// Runtime modules may inspect the owner for rooting and cold-path guards, but
/// only this module can install, take, or replace it.
pub(in crate::runtime) struct SuspensionSlot {
    owner: Option<SuspendedEvaluation>,
    next_generation: u64,
    fault: Option<DriverFault>,
    #[cfg(any(test, feature = "test-continuation-slots"))]
    fault_probe: fault::FaultProbe,
}

/// Why a live continuation owner is being abandoned.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum AbandonReason {
    ContinuationSlotMismatch,
    CancelledRequest,
    FailedSnapshotRestore,
    HelperBestEffortDiscard,
    CheckpointRollbackDiscard,
    SupersededOrQueuedInputDiscard,
}

/// The restricted structural-restoration surface available to frames while a
/// continuation is being abandoned.
#[doc(hidden)]
pub struct AbandonmentContext<'a> {
    session: &'a mut Session,
    #[allow(dead_code)]
    reason: AbandonReason,
}

/// Capability token for the sole abandonment transition allowed to evaluate.
#[doc(hidden)]
pub struct AbandonCleanup {
    #[allow(dead_code)]
    expr: Expr,
}

/// The closed transition protocol used only by the abandonment driver.
#[doc(hidden)]
pub enum FrameAbandonAction {
    Discard,
    EvaluateCleanup(AbandonCleanup),
}

impl FrameAbandonAction {
    pub(in crate::runtime) fn evaluate_cleanup(expr: Expr) -> Self {
        Self::EvaluateCleanup(AbandonCleanup { expr })
    }
}

/// Mandatory abandonment classification for every suspension frame.
#[doc(hidden)]
pub trait FrameAbandon {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction;
}

/// A traceable, immutable continuation frame installed at an input boundary.
#[doc(hidden)]
pub trait SuspensionFrame: FrameRootTrace + FrameAbandon + Send + Sync + 'static {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction;

    /// Buffered output owned until this frame's capture settles. Metadata only.
    fn output_boundary(&self) -> OutputBoundary {
        OutputBoundary::default()
    }
}

/// A Session-free completion step run after its originating frame is dropped.
#[doc(hidden)]
pub trait PureFrameFinalizer: Send + 'static {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError>;
}

/// The closed transition protocol between a named frame and the driver.
#[doc(hidden)]
pub enum FrameAction {
    Complete(Result<Expr, EvalError>),
    Evaluate {
        expr: Expr,
        next_frame: Arc<DynSuspensionFrame>,
    },
    Finalize(Box<dyn PureFrameFinalizer>),
}

impl FrameAction {
    #[inline]
    pub fn complete(outcome: Result<Expr, EvalError>) -> Self {
        Self::Complete(outcome)
    }

    #[inline]
    pub fn evaluate<F>(expr: Expr, next_frame: F) -> Self
    where
        F: SuspensionFrame,
    {
        Self::Evaluate {
            expr,
            next_frame: Arc::new(next_frame),
        }
    }

    #[inline]
    pub fn finalize<F>(finalizer: F) -> Self
    where
        F: PureFrameFinalizer,
    {
        Self::Finalize(Box::new(finalizer))
    }
}

pub trait ContinuationValue: Sized {
    fn into_continuation_expr(self) -> Expr;

    fn from_continuation_expr(expr: Expr) -> Self;
}

impl SuspendedEvaluation {
    fn new(active_stack: Vec<EvaluationStackEntry>, generation: SuspensionGeneration) -> Self {
        Self {
            active_stack,
            generation,
            guest_form_environment: None,
            frames: FrameQueue::new(),
        }
    }

    #[inline]
    fn push_back(&mut self, context: ResumeContext, frame: Arc<DynSuspensionFrame>) {
        self.frames.push_back(context, frame);
    }

    #[inline]
    fn push_front(&mut self, context: ResumeContext, frame: Arc<DynSuspensionFrame>) {
        self.frames.push_front(context, frame);
    }

    #[inline]
    fn front(&self) -> Option<Queued<EntryRef<'_>>> {
        self.frames.front()
    }

    #[inline]
    fn pop_front(&mut self) -> Queued<Entry> {
        self.frames
            .pop_front()
            .expect("the resumed suspension frame should remain queued")
    }

    #[inline]
    fn pop_front_legacy(&mut self) -> Arc<DynSuspensionFrame> {
        self.frames.pop_front_legacy()
    }

    #[allow(dead_code)]
    fn take_front(&mut self) -> Option<Queued<Entry>> {
        self.frames.pop_front()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    #[inline]
    fn append(&mut self, suffix: &mut Self) {
        self.frames.append(&mut suffix.frames);
    }

    pub(in crate::runtime) fn trace_frame_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.guest_form_environment.trace_roots(visitor);
        self.frames.visit(|frame| frame.trace_roots(visitor));
    }

    pub(in crate::runtime) fn frame_storage_memory_proxy_bytes(&self) -> usize {
        let mut allocations = 0usize;
        self.frames.visit(|frame| {
            allocations = allocations.saturating_add(frame.allocation_bytes());
        });
        self.frames.storage_bytes().saturating_add(allocations)
    }
}

struct SuspensionAccess<'a> {
    pending_request: &'a mut Option<RuntimeInputRequest>,
    pending_owner: &'a mut SuspensionSlot,
}

impl SuspensionSlot {
    pub(in crate::runtime) fn empty() -> Self {
        Self {
            owner: None,
            next_generation: 0,
            fault: None,
            #[cfg(any(test, feature = "test-continuation-slots"))]
            fault_probe: fault::FaultProbe::default(),
        }
    }

    pub(in crate::runtime) fn as_ref(&self) -> Option<&SuspendedEvaluation> {
        self.owner.as_ref()
    }

    fn as_mut(&mut self) -> Option<&mut SuspendedEvaluation> {
        self.owner.as_mut()
    }

    pub(in crate::runtime) fn is_none(&self) -> bool {
        self.owner.is_none()
    }

    #[cfg(test)]
    pub(in crate::runtime) fn is_some(&self) -> bool {
        self.owner.is_some()
    }

    fn generation(&self) -> Option<SuspensionGeneration> {
        self.owner.as_ref().map(|owner| owner.generation)
    }

    fn install(&mut self, active_stack: Vec<EvaluationStackEntry>) -> SuspensionGeneration {
        assert!(
            self.owner.is_none(),
            "a new suspension must not replace a live continuation owner"
        );
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .expect("suspension generation should not wrap");
        let generation = SuspensionGeneration(self.next_generation);
        self.owner = Some(SuspendedEvaluation::new(active_stack, generation));
        generation
    }

    fn take(&mut self) -> Option<SuspendedEvaluation> {
        self.owner.take()
    }

    fn restore_nested(&mut self, suspension: SuspendedEvaluation) {
        assert!(
            self.owner.is_none(),
            "a nested suspension must not replace a live continuation owner"
        );
        self.owner = Some(suspension);
    }
}

impl SuspensionAccess<'_> {
    fn install(
        &mut self,
        request: RuntimeInputRequest,
        active_stack: Vec<EvaluationStackEntry>,
    ) -> SuspensionGeneration {
        let generation = self.pending_owner.install(active_stack);
        *self.pending_request = Some(request);
        generation
    }

    fn take_for_resume(&mut self) -> Option<SuspendedEvaluation> {
        *self.pending_request = None;
        self.pending_owner.take()
    }

    fn take_for_abandon(&mut self) -> Option<SuspendedEvaluation> {
        *self.pending_request = None;
        self.pending_owner.take()
    }
}

impl AbandonmentContext<'_> {
    pub(in crate::runtime) fn restore_guest_form_scope(&mut self, scope: &GuestFormScope) {
        scope.restore(self.session, true);
    }

    pub(in crate::runtime) fn rollback_guest_file(&mut self, transaction: &GuestFileTransaction) {
        self.session.close_guest_file(transaction, true);
    }

    pub(in crate::runtime) fn rollback_public_file(
        &mut self,
        path: &Path,
        depth: usize,
        snapshot: &LoaderSnapshot,
    ) {
        self.session
            .close_public_file(path, depth, Some(snapshot.clone()));
    }

    pub(in crate::runtime) fn rollback_begin_package_dependency(
        &mut self,
        rollback: &crate::runtime::builtins_context_loader::BeginPackageRollback,
    ) {
        self.session.restore_begin_package_dependency(rollback);
    }

    #[allow(dead_code)]
    pub(in crate::runtime) fn reason(&self) -> AbandonReason {
        self.reason
    }

    pub(in crate::runtime) fn restore_block_scope(
        &mut self,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
    ) {
        self.session.abandon_block_scope(saved_state, dynamic_depth);
    }

    pub(in crate::runtime) fn restore_single_block_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
    ) {
        self.session
            .abandon_single_block_scope(symbol, saved_state, dynamic_depth);
    }

    pub(in crate::runtime) fn finish_module_activation(&mut self, activation_depth: usize) {
        self.session.abandon_module_scope(activation_depth);
    }

    pub(in crate::runtime) fn restore_loop_iterator_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
    ) {
        self.session
            .abandon_loop_iterator_scope(symbol, saved_state);
    }

    pub(in crate::runtime) fn restore_loop_iterator_dynamic_scope(
        &mut self,
        symbol: Symbol,
        dynamic_depth: usize,
    ) {
        self.session
            .abandon_loop_iterator_dynamic_scope(symbol, dynamic_depth);
    }

    pub(in crate::runtime) fn pop_option_value_scope(&mut self) {
        self.session
            .option_value_scopes
            .pop()
            .expect("abandoned OptionValue frame should own a scope");
        self.session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::OptionValueScopes,
        );
    }

    pub(in crate::runtime) fn pop_reap_scope(&mut self) {
        self.session
            .pop_accounted_reap_scope()
            .expect("abandoned Reap frame should own a scope");
    }

    pub(in crate::runtime) fn pop_monitor_scope(&mut self) {
        self.session
            .pop_accounted_monitor_scope()
            .expect("abandoned Monitor frame should own a scope");
    }

    pub(in crate::runtime) fn restore_time_deadline(&mut self, prior: Option<Instant>) {
        self.session.current_time_deadline = prior;
    }

    pub(in crate::runtime) fn restore_memory_limit(&mut self, prior: Option<usize>) {
        self.session.current_memory_limit = prior;
    }

    pub(in crate::runtime) fn restore_trace_capture(&mut self, prior_enabled: bool) {
        self.session.abandon_trace_capture(prior_enabled);
    }

    pub(in crate::runtime) fn restore_evaluation_data_capture(
        &mut self,
        handled_error_checkpoint: usize,
        prior_tracing: bool,
    ) {
        self.session.tracing_enabled = prior_tracing;
        self.restore_message_capture(handled_error_checkpoint);
    }

    pub(in crate::runtime) fn restore_message_capture(&mut self, handled_error_checkpoint: usize) {
        self.session
            .handled_non_loader_eval_errors
            .truncate(handled_error_checkpoint);
        self.session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
        );
    }

    pub(in crate::runtime) fn restore_stream_capture(&mut self, handled_error_checkpoint: usize) {
        self.restore_message_capture(handled_error_checkpoint);
    }

    pub(in crate::runtime) fn restore_abort_checkpoint(&mut self, pending_checkpoint: usize) {
        self.session.abort_protect_pending = pending_checkpoint;
    }

    pub(in crate::runtime) fn restore_abort_protect_scope(&mut self, pending_checkpoint: usize) {
        self.session.abandon_abort_protect(pending_checkpoint);
    }

    pub(in crate::runtime) fn restore_iteration_product_budget(&mut self, budget: (usize, u8)) {
        self.session.iteration_product_budget.set(budget);
    }

    pub(in crate::runtime) fn restore_prng_state(&mut self, state: PrngState) {
        self.session.prng_state = state;
    }

    pub(in crate::runtime) fn rollback_package_install(
        &mut self,
        plan: &crate::runtime::scoping::packages::PackageInstallPlan,
    ) {
        self.session.rollback_package_install(plan);
    }
}

impl Session {
    fn suspension_access(&mut self) -> SuspensionAccess<'_> {
        SuspensionAccess {
            pending_request: &mut self.pending_input_request,
            pending_owner: &mut self.pending_suspension,
        }
    }

    fn take_accounted_pending_suspension_for_resume(&mut self) -> Option<SuspendedEvaluation> {
        let pending = self.suspension_access().take_for_resume();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
        pending
    }

    fn take_accounted_pending_suspension_for_abandon(&mut self) -> Option<SuspendedEvaluation> {
        let pending = self.suspension_access().take_for_abandon();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
        pending
    }

    pub(in crate::runtime) fn begin_input_suspension(&mut self, request: RuntimeInputRequest) {
        #[cfg(test)]
        self.record_context_for_test("pause");
        let active_stack = self.evaluation_stack.clone();

        let guest_form_environment = self.capture_guest_form_environment();
        self.suspension_access().install(request, active_stack);
        self.pending_suspension
            .as_mut()
            .expect("just installed input owner")
            .guest_form_environment = guest_form_environment;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
    }

    /// Inserts a named, traceable frame into the fresh pending suspension.
    ///
    /// Retention scopes precheck their frame through pending_frame_allocation_bytes
    /// before reaching this common installation boundary.
    #[doc(hidden)]
    #[cold]
    #[inline(never)]
    pub fn insert_suspension_frame<F>(&mut self, frame: F)
    where
        F: SuspensionFrame,
    {
        let context = self.resume_context();
        let suspension = self
            .pending_suspension
            .as_mut()
            .expect("input suspension should be initialized before frames are pushed");
        suspension.push_back(context, Arc::new(frame));
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
    }

    pub(in crate::runtime) fn pending_frame_allocation_bytes<F: SuspensionFrame>(
        &self,
    ) -> Option<usize> {
        size_of::<F>()
            .checked_add(2 * size_of::<usize>())?
            .checked_add(
                self.pending_suspension
                    .as_ref()?
                    .frames
                    .projected_growth(false)?,
            )
    }

    pub(in crate::runtime::suspension) fn insert_typed_suspension_frame<F: TypedFrame>(
        &mut self,
        frame: F,
    ) {
        self.insert_typed_erased_frame(erase(frame));
    }

    pub(in crate::runtime::suspension) fn insert_typed_erased_frame(
        &mut self,
        frame: Arc<dyn ErasedFrame>,
    ) {
        let context = self.resume_context();
        self.pending_suspension
            .as_mut()
            .expect("input suspension should be initialized before frames are pushed")
            .frames
            .push_back_typed(context, frame);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
    }

    // S18 uses this layout projection when retention's frame becomes typed.
    #[allow(dead_code)]
    pub(in crate::runtime) fn pending_typed_frame_allocation_bytes<F: TypedFrame>(
        &self,
    ) -> Option<usize> {
        super::typed::typed_allocation_bytes::<F>().checked_add(
            self.pending_suspension
                .as_ref()?
                .frames
                .projected_growth(true)?,
        )
    }

    fn finish_completed_resumption(&mut self) {
        debug_assert!(self.pending_suspension.is_none());
        let completed = self
            .resuming_suspension
            .take()
            .expect("completed resumption should retain its owner until the queue is empty");
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
        );
        assert!(
            completed.is_empty(),
            "completed resumption must not discard queued frames"
        );
        self.pending_input_request = None;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
    }

    pub(in crate::runtime) fn resume_suspended_evaluation(
        &mut self,
        input: impl Into<String>,
    ) -> Result<Expr, EvalError> {
        self.resume_suspended_slot::<Expr>(input)
    }

    pub(in crate::runtime::suspension) fn resume_suspended_slot<O: SlotValue>(
        &mut self,
        input: impl Into<String>,
    ) -> Result<O, EvalError> {
        let saved_native_base = self.native_stack_base;
        let saved_native_threshold = self.native_stack_next_check_depth;
        let stack_marker = 0_u8;
        self.native_stack_base = (&stack_marker as *const u8) as usize;
        debug_assert!(
            self.resuming_suspension.is_none(),
            "a Session resumes only one suspended evaluation at a time"
        );
        self.resuming_suspension = Some(
            self.take_accounted_pending_suspension_for_resume()
                .expect("resume requires a pending suspended evaluation"),
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
        );
        let active_stack = std::mem::take(
            &mut self
                .resuming_suspension
                .as_mut()
                .expect("resuming suspension was just installed")
                .active_stack,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
        );
        let saved_stack = self.replace_accounted_evaluation_stack(active_stack);
        // The first queue cell restores its independently captured logical depth.
        let saved_depth = self.evaluation_depth;
        self.pending_input_request = None;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
        let previous_guest_form = self.guest_form;
        let displaced_guest_environment = if let Some(environment) = self
            .resuming_suspension
            .as_mut()
            .expect("resuming owner is installed")
            .guest_form_environment
            .take()
        {
            Some(environment.install_for_resume(self))
        } else {
            None
        };
        let outcome = Ok(Slot::Value(Expr::string(input.into())));

        let result = self.drive_resuming_frames::<O>(outcome);
        if result.is_err() {
            self.unwind_continuation_fault();
        }
        self.guest_form = previous_guest_form;

        self.replace_accounted_evaluation_stack(saved_stack);
        self.evaluation_depth = saved_depth;
        self.native_stack_base = saved_native_base;
        self.native_stack_next_check_depth = saved_native_threshold;

        match result {
            Err(fault) => {
                if let Some(displaced) = displaced_guest_environment {
                    displaced.restore(self);
                }
                Err(fault.into_error())
            }
            Ok(Ok(value)) => {
                self.finish_completed_resumption();
                Ok(value)
            }
            Ok(Err(EvalError::InputRequired { request })) => {
                debug_assert!(self.resuming_suspension.is_none());
                if let Some(displaced) = displaced_guest_environment {
                    displaced.restore(self);
                }
                Err(EvalError::InputRequired { request })
            }
            Ok(Err(error)) => {
                self.finish_completed_resumption();
                Err(error)
            }
        }
    }

    pub(in crate::runtime) fn pending_suspension_generation(&self) -> Option<SuspensionGeneration> {
        self.pending_suspension.generation()
    }

    fn drive_resuming_frames<O: SlotValue>(
        &mut self,
        mut outcome: Result<Slot, EvalError>,
    ) -> Result<Result<O, EvalError>, DriverFault> {
        loop {
            let Some(context) = self
                .resuming_suspension
                .as_ref()
                .and_then(SuspendedEvaluation::front)
                .map(|cell| cell.context)
            else {
                return decode_completion::<O>(outcome)
                    .map_err(|failure| self.record_slot_fault(failure));
            };

            self.restore_resume_context(context);
            let dispatch_frame = self
                .resuming_suspension
                .as_ref()
                .and_then(SuspendedEvaluation::front)
                .expect("restoring context preserves the queue front")
                .payload;

            // Borrow the entry, then clone only its Arc. Exact decoding happens
            // while the consumer remains rooted at the front of the queue.
            match dispatch_frame {
                EntryRef::EvaluationReturn => {
                    #[cfg(test)]
                    self.record_context_for_test("return");
                    outcome = self.complete_evaluation_return(outcome)?;
                    self.resuming_suspension
                        .as_mut()
                        .expect("the return event remains Session-rooted")
                        .pop_front();
                    self.refresh_memory_owner(
                        crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
                    );
                }
                EntryRef::Legacy(frame) => {
                    let frame = Arc::clone(frame);
                    #[cfg(test)]
                    self.record_context_for_test("frame");
                    let native_check = self.poll_resumed_native_stack();
                    outcome = outcome.and_then(|value| native_check.map(|()| value));
                    let input = match outcome {
                        Ok(Slot::Value(value)) => Ok(value),
                        Ok(actual) => {
                            return Err(self.record_slot_fault(DecodeFailure {
                                expected: std::any::type_name::<Expr>(),
                                actual,
                            }));
                        }
                        Err(error) => Err(error),
                    };
                    let action = frame.resume(self, input);
                    let completed_frame = self
                        .resuming_suspension
                        .as_mut()
                        .expect("the active resumption remains Session-rooted")
                        .pop_front_legacy();
                    self.refresh_memory_owner(
                        crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
                    );
                    drop(completed_frame);

                    outcome = match action {
                        FrameAction::Complete(outcome) => outcome.map(Slot::Value),
                        FrameAction::Evaluate { expr, next_frame } => {
                            let context = self.resume_context();
                            self.resuming_suspension
                                .as_mut()
                                .expect("the active resumption remains Session-rooted")
                                .push_front(context, next_frame);
                            self.refresh_memory_owner(
                                crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
                            );
                            self.poll_resumed_native_stack()
                                .and_then(|()| self.evaluate(expr))
                                .map(Slot::Value)
                        }
                        FrameAction::Finalize(finalizer) => finalizer.finish().map(Slot::Value),
                    };
                }
                EntryRef::Typed(frame) => {
                    let frame = Arc::clone(frame);
                    #[cfg(test)]
                    self.record_context_for_test("frame");
                    let native_check = self.poll_resumed_native_stack();
                    outcome = outcome.and_then(|value| native_check.map(|()| value));
                    let action = frame.resume(self, &mut outcome);
                    drop(frame);
                    if let Action::DecodeFailure(failure) = action {
                        return Err(self.record_slot_fault(failure));
                    }
                    let completed_frame = self
                        .resuming_suspension
                        .as_mut()
                        .expect("the active resumption remains Session-rooted")
                        .pop_front();
                    self.refresh_memory_owner(
                        crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
                    );
                    drop(completed_frame);

                    // The sole erased adapter takes the input and leaves Abort,
                    // which owns no data. End that empty placeholder's lifetime
                    // before extracting the next outcome from the action.
                    debug_assert!(matches!(outcome, Err(EvalError::Abort)));
                    std::mem::forget(outcome);
                    match action {
                        Action::DecodeFailure(_) => {
                            unreachable!("decode failure returned before pop")
                        }
                        Action::Complete(next_outcome) => outcome = next_outcome,
                        Action::Evaluate { expr, next } => {
                            let context = self.resume_context();
                            self.resuming_suspension
                                .as_mut()
                                .expect("the active resumption remains Session-rooted")
                                .frames
                                .push_front_typed(context, next);
                            self.refresh_memory_owner(
                                crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
                            );
                            outcome = self
                                .poll_resumed_native_stack()
                                .and_then(|()| self.evaluate(expr))
                                .map(Slot::Value);
                        }
                        Action::Finalize(finalizer) => outcome = finalizer.finish(),
                    }
                }
            };

            if let Err(EvalError::InputRequired { request }) = &outcome {
                let mut nested = self
                    .pending_suspension
                    .take()
                    .expect("nested input request should establish a fresh suspension");
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
                );
                let mut suffix = self
                    .resuming_suspension
                    .take()
                    .expect("the still-rooted suffix should remain in Session");
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
                );
                nested.append(&mut suffix);
                self.pending_input_request = Some(request.clone());
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
                );
                self.pending_suspension.restore_nested(nested);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
                );
                return Ok(Err(EvalError::InputRequired {
                    request: request.clone(),
                }));
            }
        }
    }
}

/// Matches a potentially suspended result without constructing frame state on
/// the ordinary path. The macro accepts data expressions only, never closures.
#[doc(hidden)]
#[macro_export]
macro_rules! resume_or_suspend {
    (
        $session:expr,
        $result:expr,
        ok($value:pat) => $continue:expr,
        suspend => $frame:expr $(,)?
    ) => {{
        match $result {
            Ok($value) => $continue,
            Err($crate::kernel_api::EvalError::InputRequired { request }) => {
                $session.insert_suspension_frame($frame);
                Err($crate::kernel_api::EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }};
    (insert $session:expr, $frame:expr $(,)?) => {{ $session.insert_suspension_frame($frame) }};
}

// Test-only oracle access; keep production inventory source locations unchanged.
#[cfg(test)]
mod occurrence_memory {
    crate::runtime::debug_tools::resources::occurrence::suspension::define_oracle!();
}
