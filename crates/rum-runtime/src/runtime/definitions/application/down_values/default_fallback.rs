use super::*;
use crate::runtime::suspension::{FrameAction, SuspensionFrame};
use std::sync::Arc;

/// The direct lookup owns its candidate snapshot; this caller owns only the
/// decision to enter the ordinary scan, which reads live definitions.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct DirectDefaultFallbackFrame {
    head: Expr,
    args: SharedArgs,
    attributes: AttributeSet,
    symbol: Symbol,
    down_values_cannot_match: bool,
}

impl SuspensionFrame for DirectDefaultFallbackFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let result = match outcome {
            Ok(value) => match Option::<Expr>::from_continuation_expr(value) {
                Some(value) => Ok(Some(value)),
                None => session.try_apply_down_values_fallback(
                    &self.head,
                    &self.args,
                    self.attributes,
                    &self.symbol,
                    self.down_values_cannot_match,
                ),
            },
            Err(error) => Err(error),
        };
        FrameAction::complete(result.map(ContinuationValue::into_continuation_expr))
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend(
    session: &mut Session,
    identity: &PreparedDefinitionLookup<'_>,
    symbol: &Symbol,
    down_values_cannot_match: bool,
) {
    crate::resume_or_suspend!(insert session, DirectDefaultFallbackFrame {
        head: identity.head().clone(),
        args: identity.args().clone(),
        attributes: identity.attributes(),
        symbol: symbol.clone(),
        down_values_cannot_match,
    });
}
