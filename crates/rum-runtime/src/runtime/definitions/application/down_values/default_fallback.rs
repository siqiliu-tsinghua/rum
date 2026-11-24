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
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_2bb(self, session, outcome, _token);
        }

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
impl<const PROFILED: bool> DirectDefaultFallbackFrame<PROFILED> {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_2bb<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Option<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        // The enabled queue's before-consumer marker has observed this exact
        // Option before this frame can replace None with a fallback result.
        let result = match outcome {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => {
                if crate::cm740_probe::driven() {
                    let _class_guard = crate::cm740_probe::enter("P0248");
                    let class_produced = (|| {
                        session.try_apply_down_values_fallback_cm740_driven(
                            &self.head,
                            &self.args,
                            self.attributes,
                            &self.symbol,
                            self.down_values_cannot_match,
                        )
                    })();
                    if crate::cm740_probe::class_lease::wants_input_return("P0248") {
                        if let Err(crate::EvalError::InputRequired { .. }) = &class_produced {
                            _class_guard.input_return(&class_produced);
                            crate::cm740_probe::class_lease::note_input_return("P0248", 0);
                        }
                    }
                    let class_selected = if true {
                        crate::cm740_probe::frame("P0248")
                    } else {
                        crate::cm740_probe::ordinary("P0248")
                    };
                    if class_selected {
                        crate::cm740_probe::hit(
                            "P0248",
                            "ActualResultBeforeDeferredDefinitionPublication",
                        );
                    }
                    class_produced
                } else {
                    (|| {
                        session.try_apply_down_values_fallback_cm740_driven(
                            &self.head,
                            &self.args,
                            self.attributes,
                            &self.symbol,
                            self.down_values_cannot_match,
                        )
                    })()
                }
            }
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
        return (if crate::cm740_probe::driven() {
            suspend_profiled_cm740_driven(
                session,
                identity,
                symbol,
                down_values_cannot_match,
                witness,
                request,
            )
        } else {
            suspend_profiled(
                session,
                identity,
                symbol,
                down_values_cannot_match,
                witness,
                request,
            )
        });
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

#[cold]
#[inline(never)]
#[allow(dead_code)]
fn suspend_profiled_cm740_driven(
    session: &mut Session,
    identity: &PreparedDefinitionLookup<'_>,
    symbol: &Symbol,
    down_values_cannot_match: bool,
    witness: SlotWitness<Option<Expr>>,
    request: RuntimeInputRequest,
) -> Result<Option<Expr>, EvalError> {
    let result = {
        let generation = session.pending_suspension_generation();
        let original_request = request.clone();
        let actual = {
            let _cm740_producer = crate::cm740_probe::enter("C5.default.profiled.finish");
            let class_produced = suspend_frame::<true>(
                session,
                identity,
                symbol,
                down_values_cannot_match,
                witness,
                request,
            );
            if crate::cm740_probe::class_lease::wants_input_return("C5.default.profiled.finish") {
                if let Err(crate::EvalError::InputRequired { .. }) = &class_produced {
                    _cm740_producer.input_return(&class_produced);
                    crate::cm740_probe::class_lease::note_input_return(
                        "C5.default.profiled.finish",
                        0,
                    );
                }
            }
            class_produced
        };
        if let Err(EvalError::InputRequired { request: returned }) = &actual {
            assert!(generation.is_some());
            assert_eq!(generation, session.pending_suspension_generation());
            assert_eq!(&original_request, returned);
            assert_eq!(session.pending_input_request(), Some(returned));
            crate::cm740_probe::returned("C5.default.profiled.finish", &returned.prompt);
        }
        if crate::cm740_probe::frame("C5.default.profiled.finish") {
            crate::cm740_probe::hit(
                "C5.default.profiled.finish",
                "ActualReturnedResultBeforeTail",
            );
            return actual;
        }
        actual
    };
    crate::runtime::profiling::logical::pause_call_stage_at_finish(
        session,
        "down_values.direct_default_lookup",
    );
    result
}
