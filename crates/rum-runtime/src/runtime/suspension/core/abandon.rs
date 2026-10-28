//! Structural owner abandonment, shared by cancellation and slot faults.
use super::*;

impl Session {
    pub(in crate::runtime) fn abandon_pending_suspension(
        &mut self,
        reason: AbandonReason,
    ) -> AbandonReason {
        let pending = self.take_accounted_pending_suspension_for_abandon();
        self.pending_eval_request = None;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingEvalRequest,
        );
        let Some(pending) = pending else {
            return reason;
        };

        self.abandonment_worklist.push(pending);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist,
        );
        self.drain_abandonment_worklist(reason)
    }

    pub(super) fn drain_abandonment_worklist(&mut self, reason: AbandonReason) -> AbandonReason {
        if self.abandonment_active {
            return reason;
        }

        let _entry = NativeStackEntry::enter();
        self.abandonment_active = true;
        let previous_guest_form = self.guest_form;
        let saved_stack = self.replace_accounted_evaluation_stack(Vec::new());
        let saved_depth = self.evaluation_depth;
        self.evaluation_depth.restore(0);

        while let Some(mut suspension) = self.abandonment_worklist.pop() {
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist,
            );
            debug_assert!(self.evaluation_stack.is_empty());
            self.replace_accounted_evaluation_stack(std::mem::take(&mut suspension.active_stack));
            if let Some(environment) = suspension.guest_form_environment.take() {
                environment.install(self);
            }

            if suspension.frames.is_profiled() {
                self.abandon_frames_profiled(suspension, reason);
            } else {
                self.abandon_frames::<profile_mode::Ordinary>(suspension, reason);
            }

            self.clear_accounted_evaluation_stack();
            self.evaluation_depth.restore(0);
        }

        self.replace_accounted_evaluation_stack(saved_stack);
        self.evaluation_depth = saved_depth;
        self.abandonment_active = false;
        self.guest_form = previous_guest_form;
        reason
    }

    #[cold]
    #[inline(never)]
    fn abandon_frames_profiled(&mut self, suspension: SuspendedEvaluation, reason: AbandonReason) {
        self.abandon_frames::<profile_mode::Profiled>(suspension, reason);
    }

    #[inline(always)]
    fn abandon_frames<M: profile_mode::Mode>(
        &mut self,
        mut suspension: SuspendedEvaluation,
        reason: AbandonReason,
    ) {
        loop {
            let _profile_context = M::abandon(&mut suspension, reason);
            let Some(cell) = suspension.take_front() else {
                break;
            };
            self.restore_resume_context(cell.context);
            let frame = cell.payload;
            self.rollback_capture_output(frame.output_boundary());
            let action = frame.abandon(&mut AbandonmentContext {
                session: self,
                reason,
            });

            match action {
                FrameAbandonAction::Discard => {}
                FrameAbandonAction::EvaluateCleanup(cleanup) => {
                    if !suspension.is_empty() {
                        suspension.active_stack = self.evaluation_stack.clone();
                        suspension.guest_form_environment = self.capture_guest_form_environment();
                        self.abandonment_worklist.push(suspension);
                        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist);
                    }

                    let cleanup_outcome = self.evaluate(cleanup.expr);
                    if matches!(cleanup_outcome, Err(EvalError::InputRequired { .. })) {
                        if let Some(nested) = self.take_accounted_pending_suspension_for_abandon() {
                            self.abandonment_worklist.push(nested);
                            self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist);
                        }
                        self.pending_eval_request = None;
                        self.refresh_memory_owner(
                            crate::runtime::debug_tools::resources::MemoryOwner::PendingEvalRequest,
                        );
                    }

                    self.clear_accounted_evaluation_stack();
                    self.evaluation_depth.restore(0);
                    break;
                }
            }
        }
    }
}
