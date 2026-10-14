use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct BuiltinEvaluateSuspensionFrame;

impl crate::runtime::suspension::TypedFrame for BuiltinEvaluateSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: crate::runtime::suspension::ContinueToken<'resume, Self>,
    ) -> crate::runtime::suspension::TypedAction<Expr> {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::TypedAction::complete(resumed)
    }
}
