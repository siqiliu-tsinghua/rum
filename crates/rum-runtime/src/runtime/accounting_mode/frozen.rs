use super::lease::{Mode, RetentionLease};
use super::retained::{RetainedValues, RetentionContext, values_bytes};
use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::{Arc, Mutex};

#[cfg(test)]
#[path = "tests/frozen.rs"]
mod tests;

#[cfg(test)]
#[path = "../options/tests/resume.rs"]
mod option_tests;

/// Pure continuation data for a retained accumulator. The kernel-owned frame
/// always owns and traces the frozen values; a continuation only borrows them.
/// Abandonment drops this data without running guest code.
pub trait RetentionContinuation: FrameRootTrace + Send + Sync + 'static {
    fn resume(
        &self,
        session: &mut Session,
        prefix: &FrozenValues,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction;
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct RetentionFrame<C: RetentionContinuation> {
    prefix: FrozenValues,
    continuation: C,
}

impl<C: RetentionContinuation> SuspensionFrame for RetentionFrame<C> {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        self.continuation.resume(session, &self.prefix, outcome)
    }
}

/// Frame-owned expression roots plus the R complement of S's traversal:
/// spare slots and the frozen receipt allocation. Construction is private.
/// Sharing a frame shares this immutable owner; resumption imports a separate
/// occurrence instead of assuming Arc uniqueness.
#[derive(rum_derive::FrameRootTrace)]
pub struct FrozenValues {
    values: Vec<Expr>,
    complement: FrozenComplement,
}

impl FrozenValues {
    /// Only the private rule continuation uses this view. The frame remains an
    /// immutable S owner; finish_rule charges the cloned working occurrence.
    pub(super) fn rule_name(&self) -> &Expr {
        assert_eq!(self.values.len(), 1, "one frozen option name");
        &self.values[0]
    }
}

struct FrozenComplement {
    spare_slots: usize,
    receipt: Option<Arc<Mutex<RetentionLease>>>,
}

impl FrameRootTrace for FrozenComplement {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        if let Some(receipt) = &self.receipt {
            receipt.trace_roots(visitor);
        } else {
            visitor.visit_retained_capacity(self.spare_slots);
        }
    }
}

#[allow(private_bounds)]
impl<'id, M: Mode> RetentionContext<'_, 'id, M> {
    /// Transfer exactly once, only on a real pending input suspension. Frame
    /// insertion first establishes S roots; only then does R retire those roots.
    /// The receipt remains in the frame and releases on actual destruction.
    pub fn suspend<C: RetentionContinuation>(
        mut self,
        prefix: RetainedValues<'id, M>,
        error: EvalError,
        continuation: impl FnOnce() -> C,
    ) -> RetainedResult<'id, M> {
        let error = self.suspend_error(prefix, error, continuation);
        RetainedResult::failure(error)
    }

    fn suspend_error<C: RetentionContinuation>(
        &mut self,
        prefix: RetainedValues<'id, M>,
        error: EvalError,
        continuation: impl FnOnce() -> C,
    ) -> EvalError {
        if !matches!(error, EvalError::InputRequired { .. })
            || self.session.pending_suspension.is_none()
        {
            return error;
        }
        let RetainedValues {
            values, receipt, ..
        } = prefix;
        let mut lease = M::freeze(receipt, self.session);
        let control_bytes = size_of::<Mutex<RetentionLease>>() + 2 * size_of::<usize>();
        if let Some(lease) = &mut lease {
            lease.resize(values_bytes(&values).and_then(|n| n.checked_add(control_bytes)));
        }
        if let Some(limit) = self.session.current_memory_limit {
            let admission = self
                .session
                .pending_frame_allocation_bytes::<RetentionFrame<C>>()
                .ok_or(EvalError::MemoryConstraintExceeded { limit })
                .and_then(|bytes| {
                    self.session
                        .enforce_active_memory_limit_for_projected_bytes(bytes)
                });
            if let Err(error) = admission {
                self.session.abandon_pending_suspension(
                    crate::runtime::suspension::AbandonReason::HelperBestEffortDiscard,
                );
                return error;
            }
        }
        let spare_slots = values.capacity() - values.len();
        let complement = spare_slots
            .checked_mul(size_of::<Expr>())
            .and_then(|n| n.checked_add(control_bytes));
        let receipt = lease.map(|lease| Arc::new(Mutex::new(lease)));
        let transfer = receipt.as_ref().map(Arc::downgrade);
        crate::resume_or_suspend!(insert self.session, RetentionFrame {
            prefix: FrozenValues {
                values,
                complement: FrozenComplement { spare_slots, receipt },
            },
            continuation: continuation(),
        });
        if let Some(receipt) = transfer.and_then(|receipt| receipt.upgrade()) {
            receipt
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .transfer(complement);
        }
        // The factory may add traced roots beyond the known frame shell.
        // Check the installed S owner plus R before publishing the suspension.
        if self.session.current_memory_limit.is_some() {
            if let Err(error) = self
                .session
                .enforce_active_memory_limit_for_projected_bytes(0)
            {
                self.session.abandon_pending_suspension(
                    crate::runtime::suspension::AbandonReason::HelperBestEffortDiscard,
                );
                return error;
            }
        }
        error
    }

    /// The frozen frame is still an S root during resume. Charge the copied
    /// working occurrence independently; its lease never cancels the frame's.
    pub fn resume_values(
        &mut self,
        mut empty: RetainedValues<'id, M>,
        frozen: &FrozenValues,
    ) -> Result<RetainedValues<'id, M>, EvalError> {
        assert!(
            empty.values.is_empty(),
            "resume requires the scope's empty accumulator"
        );
        if M::ARMED {
            let bytes =
                values_bytes(&frozen.values).ok_or(EvalError::MemoryConstraintExceeded {
                    limit: self.session.current_memory_limit.unwrap_or(usize::MAX),
                })?;
            self.session
                .enforce_active_memory_limit_for_projected_bytes(bytes)?;
        }
        empty.values = frozen.values.clone();
        self.charge(&mut empty)?;
        Ok(empty)
    }
}
