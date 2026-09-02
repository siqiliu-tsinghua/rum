use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct RenderDelayedMessageTextFrame {
    pub(super) key: MessageNameKey,
    pub(super) args: Arc<[Expr]>,
    pub(super) named_args: Arc<[(String, Expr)]>,
}

impl crate::runtime::suspension::SuspensionFrame for RenderDelayedMessageTextFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        crate::runtime::suspension::FrameAction::complete(outcome.map(|rendered| {
            Expr::string(session.finish_render_message_text(
                rendered,
                &self.key,
                self.args.as_ref(),
                self.named_args.as_ref(),
            ))
        }))
    }
}
