//! Statically selected hooks: the ordinary mode has no state or work.
use super::*;
use crate::runtime::profiling::logical::ContextGuard;

pub(super) struct Ordinary;
pub(super) struct Profiled;

pub(super) trait Mode {
    type Guard;
    fn resume(session: &mut Session, outcome: &Result<Slot, EvalError>) -> Self::Guard;
    fn abandon(suspension: &mut SuspendedEvaluation, reason: AbandonReason) -> Self::Guard;
    fn pause(guard: &Self::Guard);
}

impl Mode for Ordinary {
    type Guard = ();
    #[inline(always)]
    fn resume(_: &mut Session, _: &Result<Slot, EvalError>) {}
    #[inline(always)]
    fn abandon(_: &mut SuspendedEvaluation, _: AbandonReason) {}
    #[inline(always)]
    fn pause(_: &()) {}
}

impl Mode for Profiled {
    type Guard = Option<ContextGuard>;
    #[cold]
    #[inline(never)]
    fn resume(session: &mut Session, outcome: &Result<Slot, EvalError>) -> Self::Guard {
        if let Some(suspension) = &mut session.resuming_suspension {
            if let FrameStorage::Profiled(profile) = &mut suspension.frames {
                profile.observe_boundaries(outcome);
                return profile.enter_front();
            }
        }
        None
    }
    #[cold]
    #[inline(never)]
    fn abandon(suspension: &mut SuspendedEvaluation, reason: AbandonReason) -> Self::Guard {
        if let FrameStorage::Profiled(profile) = &mut suspension.frames {
            profile.abandon_boundaries(reason);
            return profile.enter_front();
        }
        None
    }
    #[inline]
    fn pause(guard: &Self::Guard) {
        if let Some(guard) = guard {
            guard.pause();
        }
    }
}
