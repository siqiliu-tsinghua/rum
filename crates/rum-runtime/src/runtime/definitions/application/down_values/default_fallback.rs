use super::*;
use crate::runtime::kernel_api::SlotWitness;
use crate::runtime::kernel_api::{ContinueToken, TypedAction, TypedFrame};
use std::sync::Arc;

/// The direct lookup owns its candidate snapshot; this caller owns only the
/// decision to enter the ordinary scan, which reads live definitions.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct DirectDefaultFallbackFrame<const PROFILED: bool> {
    head: Expr,
    args: SharedArgs,
    attributes: AttributeSet,
    symbol: Symbol,
    down_values_cannot_match: bool,
}

impl<const PROFILED: bool> TypedFrame for DirectDefaultFallbackFrame<PROFILED> {
    type Input = Option<Expr>;
    type Output = Option<Expr>;

    fn resume<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Option<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        // The enabled queue's before-consumer marker has observed this exact
        // Option before this frame can replace None with a fallback result.
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
        if PROFILED && matches!(result, Err(EvalError::InputRequired { .. })) {
            session.flush_definition_boundaries();
        }
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
    if crate::runtime::profiling::lexical_profile_enabled() {
        return suspend_enabled(
            session,
            identity,
            symbol,
            down_values_cannot_match,
            witness,
            request,
        );
    }
    suspend_frame::<false>(
        session,
        identity,
        symbol,
        down_values_cannot_match,
        witness,
        request,
    )
}

#[cold]
#[inline(never)]
fn suspend_enabled(
    session: &mut Session,
    identity: &PreparedDefinitionLookup<'_>,
    symbol: &Symbol,
    down_values_cannot_match: bool,
    witness: SlotWitness<Option<Expr>>,
    request: RuntimeInputRequest,
) -> Result<Option<Expr>, EvalError> {
    if crate::runtime::profiling::logical::direct_default_fallback_profiled() {
        return suspend_profiled(
            session,
            identity,
            symbol,
            down_values_cannot_match,
            witness,
            request,
        );
    }
    suspend_frame::<false>(
        session,
        identity,
        symbol,
        down_values_cannot_match,
        witness,
        request,
    )
}

#[cold]
#[inline(always)]
fn suspend_frame<const PROFILED: bool>(
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
        DirectDefaultFallbackFrame::<PROFILED> {
            head: identity.head().clone(),
            args: identity.args().clone(),
            attributes: identity.attributes(),
            symbol: symbol.clone(),
            down_values_cannot_match,
        },
    )
}

#[cold]
#[inline(never)]
fn suspend_profiled(
    session: &mut Session,
    identity: &PreparedDefinitionLookup<'_>,
    symbol: &Symbol,
    down_values_cannot_match: bool,
    witness: SlotWitness<Option<Expr>>,
    request: RuntimeInputRequest,
) -> Result<Option<Expr>, EvalError> {
    let result = suspend_frame::<true>(
        session,
        identity,
        symbol,
        down_values_cannot_match,
        witness,
        request,
    );
    crate::runtime::profiling::logical::pause_call_stage_at_finish(
        session,
        "down_values.direct_default_lookup",
    );
    result
}
