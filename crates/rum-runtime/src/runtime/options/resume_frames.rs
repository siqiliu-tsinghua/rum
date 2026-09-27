use super::*;
use crate::runtime::accounting_mode::{
    AccountingShadow, FrozenValues, RetentionContinuation, Unarmed, options as retained_options,
    rule,
};
use crate::runtime::suspension::FrameAction;
use rum_core::SharedArgs;

/// The private RetentionFrame owns the charged, lazily captured name. This
/// descriptor only borrows it, including when another Arc keeps it alive.
#[derive(rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct RuntimeOptionSuspensionFrame;

impl RetentionContinuation for RuntimeOptionSuspensionFrame {
    fn resume(
        &self,
        session: &mut Session,
        prefix: &FrozenValues,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(rule::resume_rule(session, prefix, outcome))
    }
}

/// Frozen loop inputs and the already evaluated prefix are all roots. Defaults
/// are captured once, so guest changes to Options do not restart the loop.
#[derive(rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct MergedOptionsSuspensionFrame<R: OptionReentry> {
    pub(in crate::runtime) domain_generation: u64,
    pub(in crate::runtime) reentry: R,
    pub(in crate::runtime) head: Expr,
    pub(in crate::runtime) args: SharedArgs,
    pub(in crate::runtime) target: Symbol,
    pub(in crate::runtime) defaults: usize,
    pub(in crate::runtime) explicit: SharedArgs,
    pub(in crate::runtime) next_index: usize,
}

impl<R: OptionReentry> RetentionContinuation for MergedOptionsSuspensionFrame<R> {
    fn resume(
        &self,
        session: &mut Session,
        frozen: &FrozenValues,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let resumed = if session.current_memory_limit.is_some() {
            retained_options::resume::<AccountingShadow, R>(session, self, frozen, outcome)
        } else {
            retained_options::resume::<Unarmed, R>(session, self, frozen, outcome)
        };
        FrameAction::complete(resumed)
    }
}
