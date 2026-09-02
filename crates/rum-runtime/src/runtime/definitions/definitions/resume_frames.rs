use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalSetSuspensionFrame {
    pub(super) lhs: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalSetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => (|| session.resume_set_with_rhs(lhs, rhs))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalRuleSuspensionFrame {
    pub(super) lhs: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalRuleSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => (|| session.resume_rule_with_rhs(lhs, rhs))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
