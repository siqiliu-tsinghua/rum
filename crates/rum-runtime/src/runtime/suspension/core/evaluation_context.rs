//! Logical continuation levels are queue metadata, independent of frame types.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime::suspension) struct ResumeContext {
    pub(in crate::runtime::suspension) stack_len: usize,
    pub(in crate::runtime::suspension) depth: usize,
}

pub(in crate::runtime::suspension) struct Queued<P> {
    pub(in crate::runtime::suspension) context: ResumeContext,
    pub(in crate::runtime::suspension) payload: P,
}

impl<P> Queued<P> {
    pub(super) fn map<Q>(self, map: impl FnOnce(P) -> Q) -> Queued<Q> {
        Queued {
            context: self.context,
            payload: map(self.payload),
        }
    }
}

impl Session {
    pub(super) fn resume_context(&self) -> ResumeContext {
        ResumeContext {
            stack_len: self.evaluation_stack.len(),
            depth: self.evaluation_depth,
        }
    }

    pub(super) fn restore_resume_context(&mut self, context: ResumeContext) {
        debug_assert!(
            context.stack_len <= self.evaluation_stack.len(),
            "continuation contexts unwind a saved prefix"
        );
        self.truncate_accounted_evaluation_stack(context.stack_len);
        self.evaluation_depth = context.depth;
        // Entry epochs remain those of their original evaluations. In particular,
        // do not rewind the global epoch after resumed work changes definitions.
    }

    /// Physical safety for the current native segment, not replay of its address.
    #[cold]
    #[inline(never)]
    pub(super) fn poll_resumed_native_stack(&mut self) -> Result<(), EvalError> {
        self.native_stack_next_check_depth = self
            .evaluation_depth
            .saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
        if self.native_stack_budget_exhausted() {
            Err(EvalError::StackLimitExceeded {
                limit: self.resource_limits.native_stack_limit,
            })
        } else {
            Ok(())
        }
    }

    /// Called after a suspended native level has unwound, before its consumer.
    pub(in crate::runtime::suspension) fn insert_evaluation_return(&mut self) {
        let context = self.resume_context();
        self.pending_suspension
            .as_mut()
            .expect("evaluation return belongs to a pending input owner")
            .frames
            .push_evaluation_return(context);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
    }

    pub(super) fn complete_evaluation_return(
        &mut self,
        outcome: Result<Slot, EvalError>,
    ) -> Result<Result<Slot, EvalError>, DriverFault> {
        let value = match outcome {
            Ok(Slot::Value(value)) => value,
            Ok(actual) => {
                return Err(self.record_slot_fault(DecodeFailure {
                    expected: std::any::type_name::<Expr>(),
                    actual,
                }));
            }
            Err(error) => return Ok(Err(error)),
        };
        Ok(self
            .enforce_active_resource_limits_for_expr(&value)
            .map(|()| Slot::Value(value)))
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(in crate::runtime) struct ContextObservation {
    pub(in crate::runtime) phase: &'static str,
    pub(in crate::runtime) entries: Vec<(String, u64)>,
    pub(in crate::runtime) depth: usize,
    pub(in crate::runtime) epoch: u64,
}
#[cfg(test)]
std::thread_local! {
    static CONTEXT_OBSERVATIONS: std::cell::RefCell<Option<Vec<ContextObservation>>> = const { std::cell::RefCell::new(None) };
}
#[cfg(test)]
impl Session {
    pub(in crate::runtime) fn record_context_for_test(&self, phase: &'static str) {
        CONTEXT_OBSERVATIONS.with_borrow_mut(|observations| {
            if let Some(observations) = observations {
                observations.push(ContextObservation {
                    phase,
                    entries: self
                        .evaluation_stack
                        .iter()
                        .map(|e| (e.expr.to_string(), e.epoch))
                        .collect(),
                    depth: self.evaluation_depth,
                    epoch: self.evaluation_epoch,
                });
            }
        });
    }
    pub(in crate::runtime) fn start_context_observations_for_test(&self) {
        CONTEXT_OBSERVATIONS.with_borrow_mut(|observations| *observations = Some(Vec::new()));
    }
    pub(in crate::runtime) fn take_context_observations_for_test(&self) -> Vec<ContextObservation> {
        CONTEXT_OBSERVATIONS.with_borrow_mut(|observations| observations.take().unwrap())
    }
}
