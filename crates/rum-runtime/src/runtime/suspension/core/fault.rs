//! First-fault state is structural control, never a frame completion value.
use super::*;
use crate::runtime::suspension::typed::DecodeFailure;

/// A structural continuation fault projected after the driver has unwound.
/// Hosts may inspect its labels, but only the suspension driver can create it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
pub struct SlotMismatch {
    expected: &'static str,
    got: &'static str,
}

impl SlotMismatch {
    pub(in crate::runtime::suspension) fn new(expected: &'static str, got: &'static str) -> Self {
        Self { expected, got }
    }

    pub fn expected(&self) -> &'static str {
        self.expected
    }

    pub fn got(&self) -> &'static str {
        self.got
    }

    #[cfg(test)]
    pub(in crate::runtime) fn for_test(expected: &'static str, got: &'static str) -> Self {
        Self::new(expected, got)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DriverFault {
    expected: &'static str,
    got: &'static str,
}

impl DriverFault {
    pub(super) fn into_error(self) -> EvalError {
        EvalError::ContinuationSlotMismatch(SlotMismatch::new(self.expected, self.got))
    }
}

/// A monotonic observer that outlives replacement or rollback of its Session.
#[cfg(any(test, feature = "test-continuation-slots"))]
#[derive(Clone, Default)]
pub struct FaultProbe {
    occurrences: Arc<std::sync::atomic::AtomicU64>,
}

#[cfg(any(test, feature = "test-continuation-slots"))]
impl FaultProbe {
    pub fn occurrences(&self) -> u64 {
        self.occurrences.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Session {
    #[cfg(any(test, feature = "test-continuation-slots"))]
    pub fn continuation_fault_probe(&self) -> FaultProbe {
        self.pending_suspension.fault_probe.clone()
    }

    pub(super) fn record_slot_fault(&mut self, failure: DecodeFailure) -> DriverFault {
        if let Some(first) = self.pending_suspension.fault {
            return first;
        }
        let fault = DriverFault {
            expected: failure.expected,
            got: failure.actual.kind_name(),
        };
        self.pending_suspension.fault = Some(fault);
        #[cfg(any(test, feature = "test-continuation-slots"))]
        self.pending_suspension
            .fault_probe
            .occurrences
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        fault
    }

    #[inline]
    pub(in crate::runtime) fn has_continuation_fault(&self) -> bool {
        self.pending_suspension.fault.is_some()
    }

    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn continuation_fault_outcome<T>(
        &mut self,
        _outcome: Result<T, EvalError>,
    ) -> Result<T, EvalError> {
        Err(self
            .pending_suspension
            .fault
            .take()
            .expect("the boundary checked the continuation fault latch")
            .into_error())
    }

    pub(super) fn unwind_continuation_fault(&mut self) {
        let mut owner = self
            .resuming_suspension
            .take()
            .expect("a slot fault keeps the active owner until unwind");
        owner.active_stack = self.evaluation_stack.clone();
        owner.guest_form_environment = self.capture_guest_form_environment();
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ResumingSuspension,
        );
        // The stack drains in reverse insertion order: nested work executes
        // before the unrun active suffix, exactly as normal queue splicing.
        self.abandonment_worklist.push(owner);
        if let Some(nested) = self.take_accounted_pending_suspension_for_abandon() {
            self.abandonment_worklist.push(nested);
        }
        self.pending_input_request = None;
        self.pending_eval_request = None;
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingInputRequest,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingEvalRequest,
        );
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::AbandonmentWorklist,
        );
        self.drain_abandonment_worklist(AbandonReason::ContinuationSlotMismatch);
    }
}
