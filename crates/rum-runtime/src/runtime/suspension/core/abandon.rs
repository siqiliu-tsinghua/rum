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
        self.begin_ordered_abandonment(reason);
        let previous_guest_form = self.guest_form;
        let saved_stack = self.replace_accounted_evaluation_stack(Vec::new());
        let saved_depth = self.evaluation_depth;
        self.evaluation_depth.restore(0);
        let mut skip_cleanups = false;
        let mut skipped = 0usize;
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
                self.abandon_frames_profiled(suspension, reason, &mut skip_cleanups, &mut skipped);
            } else {
                self.abandon_frames::<profile_mode::Ordinary>(
                    suspension,
                    reason,
                    &mut skip_cleanups,
                    &mut skipped,
                );
            }

            self.clear_accounted_evaluation_stack();
            self.evaluation_depth.restore(0);
        }

        self.replace_accounted_evaluation_stack(saved_stack);
        self.evaluation_depth = saved_depth;
        self.finish_ordered_abandonment(skipped);
        self.guest_form = previous_guest_form;
        reason
    }

    #[inline(always)]
    fn abandon_frames<M: profile_mode::Mode>(
        &mut self,
        mut suspension: SuspendedEvaluation,
        reason: AbandonReason,
        skip_cleanups: &mut bool,
        skipped: &mut usize,
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
                    if *skip_cleanups || self.drain_host_interrupt_pending() {
                        *skipped += 1;
                        *skip_cleanups = true;
                        self.clear_accounted_evaluation_stack();
                        self.evaluation_depth.restore(0);
                        break;
                    }
                    let entry_depth = self.evaluation_depth.get();
                    let cleanup_outcome = self.evaluate(cleanup.expr);
                    if matches!(cleanup_outcome, Err(EvalError::InputRequired { .. })) {
                        if let Some(nested) = self.take_accounted_pending_suspension_for_abandon() {
                            debug_assert!(
                                nested_front_is_deeper(&nested, entry_depth),
                                "a nested suspension requeued by a cleanup is deeper than the cleanup"
                            );
                            self.abandonment_worklist.push(nested);
                            self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist);
                        }
                        self.pending_eval_request = None;
                        self.refresh_memory_owner(
                            crate::runtime::debug_tools::resources::MemoryOwner::PendingEvalRequest,
                        );
                    }
                    if self.drain_host_interrupt_pending() {
                        *skip_cleanups = true;
                    }
                    self.clear_accounted_evaluation_stack();
                    self.evaluation_depth.restore(0);
                    break;
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn abandon_frames_profiled(
        &mut self,
        suspension: SuspendedEvaluation,
        reason: AbandonReason,
        skip_cleanups: &mut bool,
        skipped: &mut usize,
    ) {
        self.abandon_frames::<profile_mode::Profiled>(suspension, reason, skip_cleanups, skipped);
    }

    /// The retirement ticket stays installed. Its first cancel is consumed
    /// here, before any cleanup, and the masked poll stays armed for a later
    /// cancel or the suspended request's own absolute deadline.
    fn begin_ordered_abandonment(&mut self, reason: AbandonReason) {
        self.abandonment_active = true;
        if reason == AbandonReason::CancelledRequest {
            if let Some(accepted) = self.active_accepted_request.as_ref() {
                accepted.consume_cancellation();
            }
            self.interrupt_armed =
                self.eval_deadline.is_some() || self.active_accepted_request.is_some();
        }
    }

    fn finish_ordered_abandonment(&mut self, skipped: usize) {
        if skipped > 0 {
            self.push_message(
                "WithCleanup",
                "skipped",
                format!(
                    "Pending cleanups skipped because a host interrupt stopped the retirement: {skipped}."
                ),
            );
        }
        self.abandonment_active = false;
    }

    /// Cold, once per cleanup frame. An unconsumed cancel, or the suspended
    /// request's absolute deadline. A guest time-constraint deadline is not
    /// this signal.
    #[cold]
    #[inline(never)]
    fn drain_host_interrupt_pending(&self) -> bool {
        if self.active_accepted_request_is_cancelled() {
            return true;
        }
        self.eval_deadline
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    }
}

fn nested_front_is_deeper(nested: &SuspendedEvaluation, entry_depth: usize) -> bool {
    nested
        .frames
        .front()
        .is_some_and(|cell| cell.context.depth > entry_depth)
}
