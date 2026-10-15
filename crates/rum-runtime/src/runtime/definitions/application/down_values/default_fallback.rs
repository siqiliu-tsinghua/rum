use super::*;
use crate::runtime::kernel_api::SlotWitness;
use crate::runtime::kernel_api::{ContinueToken, TypedAction, TypedFrame};
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

impl TypedFrame for DirectDefaultFallbackFrame {
    type Input = Option<Expr>;
    type Output = Option<Expr>;

    fn resume<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Option<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        let result = match outcome {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => session.try_apply_down_values_fallback(
                &self.head,
                &self.args,
                self.attributes,
                &self.symbol,
                self.down_values_cannot_match,
            ),
            Err(error) => Err(error),
        };
        TypedAction::complete(result)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend(
    session: &mut Session,
    identity: &PreparedDefinitionLookup<'_>,
    symbol: &Symbol,
    down_values_cannot_match: bool,
    witness: SlotWitness<Option<Expr>>,
    request: RuntimeInputRequest,
) -> Result<Option<Expr>, EvalError> {
    witness.suspend(
        session,
        request,
        DirectDefaultFallbackFrame {
            head: identity.head().clone(),
            args: identity.args().clone(),
            attributes: identity.attributes(),
            symbol: symbol.clone(),
            down_values_cannot_match,
        },
    )
}
