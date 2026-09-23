use super::*;
use crate::runtime::suspension::{FrameAction, SuspensionFrame};
use rum_core::SharedArgs;
use std::sync::Arc;

/// Rebuild the immediate option rule after its delayed value resumes.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct RuntimeOptionSuspensionFrame {
    pub(super) name: Expr,
}

impl SuspensionFrame for RuntimeOptionSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(match outcome {
            Ok(value) => Ok(Expr::call(
                system_expr("Rule"),
                vec![self.name.clone(), value],
            )),
            Err(error) => Err(error),
        })
    }
}

/// Frozen loop inputs and the already evaluated prefix are all roots. Defaults
/// are captured once, so guest changes to Options do not restart the loop.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MergedOptionsSuspensionFrame<R: OptionReentry> {
    pub(super) reentry: R,
    pub(super) head: Expr,
    pub(super) args: SharedArgs,
    pub(super) target: Symbol,
    pub(super) defaults: SharedArgs,
    pub(super) explicit: SharedArgs,
    pub(super) next_index: usize,
    pub(super) merged: SharedArgs,
}

impl<R: OptionReentry> SuspensionFrame for MergedOptionsSuspensionFrame<R> {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        if !session.check_option_reentry::<R>() {
            return FrameAction::complete(Ok(system_expr("$Failed")));
        }
        let resumed = match outcome {
            Ok(value) => session.resume_merged_options_after_value(&self, value),
            Err(error) => Err(error),
        };
        FrameAction::complete(resumed)
    }
}

#[cfg(test)]
#[path = "tests/resume.rs"]
mod tests;
