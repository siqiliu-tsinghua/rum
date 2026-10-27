//! One driver algorithm, specialized over empty ordinary hooks.
use super::*;

impl Session {
    #[inline(always)]
    pub(super) fn drive_resuming_frames<O: SlotValue>(
        &mut self,
        outcome: Result<Slot, EvalError>,
    ) -> Result<Result<O, EvalError>, DriverFault> {
        if self
            .resuming_suspension
            .as_ref()
            .is_some_and(|s| s.frames.is_profiled())
        {
            self.drive_frames::<O, profile_mode::Profiled>(outcome)
        } else {
            self.drive_frames::<O, profile_mode::Ordinary>(outcome)
        }
    }
    fn drive_frames<O: SlotValue, M: profile_mode::Mode>(
        &mut self,
        mut outcome: Result<Slot, EvalError>,
    ) -> Result<Result<O, EvalError>, DriverFault> {
        loop {
            let profile_context = M::resume(self, &outcome);
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
                M::pause(&profile_context);
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
