use std::collections::VecDeque;
use std::mem::{size_of, size_of_val};
use std::sync::Arc;
use std::time::Instant;

use super::trace::{FrameRootTrace, FrameRootVisitor};
use super::*;

type DynSuspensionFrame = dyn SuspensionFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct SuspensionGeneration(u64);

pub(in crate::runtime) struct SuspendedEvaluation {
    pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
    generation: SuspensionGeneration,
    frames: VecDeque<Arc<DynSuspensionFrame>>,
}

/// Opaque owner slot for the one pending continuation.
///
/// Runtime modules may inspect the owner for rooting and cold-path guards, but
/// only this module can install, take, or replace it.
pub(in crate::runtime) struct SuspensionSlot {
    owner: Option<SuspendedEvaluation>,
    next_generation: u64,
}

/// Why a live continuation owner is being abandoned.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum AbandonReason {
    CancelledRequest,
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
            frames: VecDeque::new(),
        }
    }

    fn push_back(&mut self, frame: Arc<DynSuspensionFrame>) {
        self.frames.push_back(frame);
    }

    fn push_front(&mut self, frame: Arc<DynSuspensionFrame>) {
        self.frames.push_front(frame);
    }

    fn front(&self) -> Option<Arc<DynSuspensionFrame>> {
        self.frames.front().cloned()
    }

    fn pop_front(&mut self) -> Arc<DynSuspensionFrame> {
        self.frames
            .pop_front()
            .expect("the resumed suspension frame should remain queued")
    }

    #[allow(dead_code)]
    fn take_front(&mut self) -> Option<Arc<DynSuspensionFrame>> {
        self.frames.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    fn append(&mut self, suffix: &mut Self) {
        self.frames.append(&mut suffix.frames);
    }

    pub(in crate::runtime) fn trace_frame_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for frame in &self.frames {
            frame.trace_roots(visitor);
        }
    }

    pub(in crate::runtime) fn frame_storage_memory_proxy_bytes(&self) -> usize {
        let queue_storage = self
            .frames
            .capacity()
            .saturating_mul(size_of::<Arc<DynSuspensionFrame>>());
        let allocations = self.frames.iter().fold(0usize, |total, frame| {
            total
                .saturating_add(2usize.saturating_mul(size_of::<usize>()))
                .saturating_add(size_of_val(frame.as_ref()))
        });
        queue_storage.saturating_add(allocations)
    }
}

struct SuspensionAccess<'a> {
    pending_request: &'a mut Option<RuntimeInputRequest>,
    pending_owner: &'a mut SuspensionSlot,
}

impl SuspensionSlot {
    pub(in crate::runtime) const fn empty() -> Self {
        Self {
            owner: None,
            next_generation: 0,
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
    }

    pub(in crate::runtime) fn pop_reap_scope(&mut self) {
        self.session
            .reap_scopes
            .pop()
            .expect("abandoned Reap frame should own a scope");
    }

    pub(in crate::runtime) fn pop_monitor_scope(&mut self) {
        self.session
            .monitor_scopes
            .pop()
            .expect("abandoned Monitor frame should own a scope");
    }

    pub(in crate::runtime) fn restore_time_deadline(&mut self, prior: Option<Instant>) {
        self.session.current_time_deadline = prior;
    }

    pub(in crate::runtime) fn restore_memory_limit(&mut self, prior: Option<usize>) {
        self.session.current_memory_limit = prior;
    }

    pub(in crate::runtime) fn restore_trace_capture(
        &mut self,
        trace_checkpoint: usize,
        prior_enabled: bool,
    ) {
        self.session
            .abandon_trace_capture(trace_checkpoint, prior_enabled);
    }

    pub(in crate::runtime) fn restore_evaluation_data_capture(
        &mut self,
        message_checkpoint: usize,
        print_checkpoint: usize,
        trace_checkpoint: usize,
        handled_error_checkpoint: usize,
        prior_tracing: bool,
    ) {
        self.session.tracing_enabled = prior_tracing;
        self.session.messages.truncate(message_checkpoint);
        self.session.prints.truncate(print_checkpoint);
        if !prior_tracing {
            self.session.trace_events.truncate(trace_checkpoint);
            self.session.trace_event_depths.truncate(trace_checkpoint);
        }
        self.session
            .handled_non_loader_eval_errors
            .truncate(handled_error_checkpoint);
    }

    pub(in crate::runtime) fn restore_message_capture(
        &mut self,
        message_checkpoint: usize,
        handled_error_checkpoint: usize,
    ) {
        self.session.messages.truncate(message_checkpoint);
        self.session
            .handled_non_loader_eval_errors
            .truncate(handled_error_checkpoint);
    }

    pub(in crate::runtime) fn restore_stream_capture(
        &mut self,
        message_checkpoint: usize,
        print_checkpoint: usize,
        handled_error_checkpoint: usize,
    ) {
        self.session.messages.truncate(message_checkpoint);
        self.session.prints.truncate(print_checkpoint);
        self.session
            .handled_non_loader_eval_errors
            .truncate(handled_error_checkpoint);
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

    pub(in crate::runtime) fn begin_input_suspension(&mut self, request: RuntimeInputRequest) {
        // The current InputString call and its immediate caller are both
        // resumed from saved suspension frames rather than by restarting
        // evaluate_once. Keeping either expression on the active stack causes
        // false-positive cycle detection when matcher backtracking or other
        // helper-level continuations legitimately re-enter that caller after
        // input arrives.
        let retained_depth = self.evaluation_stack.len().saturating_sub(2);
        let active_stack = self
            .evaluation_stack
            .iter()
            .take(retained_depth)
            .cloned()
            .collect::<Vec<_>>();

        self.suspension_access().install(request, active_stack);
    }

    /// Inserts a named, traceable frame into the fresh pending suspension.
    #[doc(hidden)]
    #[cold]
    #[inline(never)]
    pub fn insert_suspension_frame<F>(&mut self, frame: F)
    where
        F: SuspensionFrame,
    {
        let suspension = self
            .pending_suspension
            .as_mut()
            .expect("input suspension should be initialized before frames are pushed");
        suspension.push_back(Arc::new(frame));
    }

    fn finish_completed_resumption(&mut self) {
        debug_assert!(self.pending_suspension.is_none());
        let completed = self
            .resuming_suspension
            .take()
            .expect("completed resumption should retain its owner until the queue is empty");
        assert!(
            completed.is_empty(),
            "completed resumption must not discard queued frames"
        );
        self.pending_input_request = None;
    }

    pub(in crate::runtime) fn resume_suspended_evaluation(
        &mut self,
        input: impl Into<String>,
    ) -> Result<Expr, EvalError> {
        let stack_marker = 0_u8;
        self.native_stack_base = (&stack_marker as *const u8) as usize;
        debug_assert!(
            self.resuming_suspension.is_none(),
            "a Session resumes only one suspended evaluation at a time"
        );
        self.resuming_suspension = Some(
            self.suspension_access()
                .take_for_resume()
                .expect("resume requires a pending suspended evaluation"),
        );
        let active_stack = std::mem::take(
            &mut self
                .resuming_suspension
                .as_mut()
                .expect("resuming suspension was just installed")
                .active_stack,
        );
        let saved_stack = std::mem::replace(&mut self.evaluation_stack, active_stack);
        let saved_depth =
            std::mem::replace(&mut self.evaluation_depth, self.evaluation_stack.len());
        self.native_stack_next_check_depth = self
            .evaluation_depth
            .saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
        self.pending_input_request = None;
        let outcome = Ok(Expr::string(input.into()));

        let result = self.drive_resuming_frames(outcome);

        self.evaluation_stack = saved_stack;
        self.evaluation_depth = saved_depth;

        match result {
            Ok(value) => {
                self.finish_completed_resumption();
                Ok(value)
            }
            Err(EvalError::InputRequired { request }) => {
                debug_assert!(self.resuming_suspension.is_none());
                Err(EvalError::InputRequired { request })
            }
            Err(error) => {
                self.finish_completed_resumption();
                Err(error)
            }
        }
    }

    pub(in crate::runtime) fn abandon_pending_suspension(
        &mut self,
        reason: AbandonReason,
    ) -> AbandonReason {
        let pending = self.suspension_access().take_for_abandon();
        self.pending_eval_request = None;
        let Some(pending) = pending else {
            return reason;
        };

        self.abandonment_worklist.push(pending);
        if self.abandonment_active {
            return reason;
        }

        self.abandonment_active = true;
        let saved_stack = std::mem::take(&mut self.evaluation_stack);
        let saved_depth = std::mem::replace(&mut self.evaluation_depth, 0);

        while let Some(mut suspension) = self.abandonment_worklist.pop() {
            debug_assert!(self.evaluation_stack.is_empty());
            self.evaluation_stack = std::mem::take(&mut suspension.active_stack);
            self.evaluation_depth = self.evaluation_stack.len();

            while let Some(frame) = suspension.take_front() {
                let action = frame.abandon(&mut AbandonmentContext {
                    session: self,
                    reason,
                });

                match action {
                    FrameAbandonAction::Discard => {}
                    FrameAbandonAction::EvaluateCleanup(cleanup) => {
                        if !suspension.is_empty() {
                            suspension.active_stack = self.evaluation_stack.clone();
                            self.abandonment_worklist.push(suspension);
                        }

                        let cleanup_outcome = self.evaluate(cleanup.expr);
                        if matches!(cleanup_outcome, Err(EvalError::InputRequired { .. })) {
                            if let Some(nested) = self.suspension_access().take_for_abandon() {
                                self.abandonment_worklist.push(nested);
                            }
                            self.pending_eval_request = None;
                        }

                        self.evaluation_stack.clear();
                        self.evaluation_depth = 0;
                        break;
                    }
                }
            }

            self.evaluation_stack.clear();
            self.evaluation_depth = 0;
        }

        self.evaluation_stack = saved_stack;
        self.evaluation_depth = saved_depth;
        self.abandonment_active = false;
        reason
    }

    pub(in crate::runtime) fn pending_suspension_generation(&self) -> Option<SuspensionGeneration> {
        self.pending_suspension.generation()
    }

    fn drive_resuming_frames(
        &mut self,
        mut outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(dispatch_frame) = self
                .resuming_suspension
                .as_ref()
                .and_then(SuspendedEvaluation::front)
            else {
                return outcome;
            };

            let action = dispatch_frame.resume(self, outcome);
            let completed_frame = self
                .resuming_suspension
                .as_mut()
                .expect("the active resumption remains Session-rooted")
                .pop_front();
            drop(completed_frame);

            outcome = match action {
                FrameAction::Complete(outcome) => outcome,
                FrameAction::Evaluate { expr, next_frame } => {
                    self.resuming_suspension
                        .as_mut()
                        .expect("the active resumption remains Session-rooted")
                        .push_front(next_frame);
                    self.evaluate(expr)
                }
                FrameAction::Finalize(finalizer) => finalizer.finish(),
            };

            if let Err(EvalError::InputRequired { request }) = &outcome {
                let mut nested = self
                    .pending_suspension
                    .take()
                    .expect("nested input request should establish a fresh suspension");
                let mut suffix = self
                    .resuming_suspension
                    .take()
                    .expect("the still-rooted suffix should remain in Session");
                nested.append(&mut suffix);
                self.pending_input_request = Some(request.clone());
                self.pending_suspension.restore_nested(nested);
                return Err(EvalError::InputRequired {
                    request: request.clone(),
                });
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
