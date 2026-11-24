use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledDefinitionConditionOwnedSuspensionFrame {
    pub(super) guards: rum_core::SharedArgs,
    pub(super) option_scope: std::option::Option<crate::runtime::options::OptionValueScope>,
}

impl TypedFrame for MatchCompiledDefinitionConditionOwnedSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = CompiledMatchOutcome;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<CompiledMatchOutcome> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_20e(self, session, outcome, _token);
        }

        let outcome = match outcome {
            Ok(value) => value,
            Err(error) => return TypedAction::complete(Err(error)),
        };
        if !outcome.matched {
            return TypedAction::complete(Ok(outcome));
        }
        let false_bindings = outcome.bindings.clone();
        continue_guard_frame(
            session,
            SuspendedGuardEvaluation {
                guards: SuspendedGuards::Many(self.guards.clone()),
                bindings: FrozenMatchBindings::freeze(outcome.bindings),
                false_bindings: FrozenMatchBindings::freeze(false_bindings),
                option_scope: self.option_scope.clone(),
                index: 0,
            },
        )
    }
}
impl MatchCompiledDefinitionConditionOwnedSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_20e<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<CompiledMatchOutcome> {
        if crate::cm740_probe::frame("P0442") {
            crate::cm740_probe::hit("P0442", "ForwardActualInnerMatchBeforeGuards");
            return TypedAction::complete(outcome);
        }
        let outcome = match outcome {
            Ok(value) => value,
            Err(error) => return TypedAction::complete(Err(error)),
        };
        if !outcome.matched {
            return TypedAction::complete(Ok(outcome));
        }
        let false_bindings = outcome.bindings.clone();
        continue_guard_frame(
            session,
            SuspendedGuardEvaluation {
                guards: SuspendedGuards::Many(self.guards.clone()),
                bindings: FrozenMatchBindings::freeze(outcome.bindings),
                false_bindings: FrozenMatchBindings::freeze(false_bindings),
                option_scope: self.option_scope.clone(),
                index: 0,
            },
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateOwnedGuardsSuspensionFrame {
    pub(super) state: crate::runtime::definitions::rhs_guards::SuspendedGuardEvaluation,
}

impl TypedFrame for EvaluateOwnedGuardsSuspensionFrame {
    type Input = Expr;
    type Output = CompiledMatchOutcome;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<CompiledMatchOutcome> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_7c3(self, session, outcome, token);
        }

        resume_guard_frame(session, self.state.clone(), outcome, token)
    }
}
impl EvaluateOwnedGuardsSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_7c3<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<CompiledMatchOutcome> {
        if crate::cm740_probe::frame("P0450") {
            crate::cm740_probe::hit("P0450", "OmitGuardSuffixPreserveRealFinalizer");
            return match outcome {
                Ok(value) => {
                    let matched = is_system_symbol_name(&value, heads::structure::TRUE);
                    let bindings = if matched {
                        self.state.bindings.clone()
                    } else {
                        self.state.false_bindings.clone()
                    };
                    TypedAction::finalize(GuardOutcomeFinalizer { matched, bindings })
                }
                Err(error) => TypedAction::complete(Err(error)),
            };
        }
        if crate::cm740_probe::frame("P0455") {
            crate::cm740_probe::hit("P0455", "OmitGuardSuffixPreserveRealFinalizer");
            return match outcome {
                Ok(value) => {
                    let matched = is_system_symbol_name(&value, heads::structure::TRUE);
                    let bindings = if matched {
                        self.state.bindings.clone()
                    } else {
                        self.state.false_bindings.clone()
                    };
                    TypedAction::finalize(GuardOutcomeFinalizer { matched, bindings })
                }
                Err(error) => TypedAction::complete(Err(error)),
            };
        }
        resume_guard_frame(session, self.state.clone(), outcome, token)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateReadonlySuspendedGuardsSuspensionFrame {
    pub(super) state: crate::runtime::definitions::rhs_guards::SuspendedReadonlyGuardEvaluation,
}

impl TypedFrame for EvaluateReadonlySuspendedGuardsSuspensionFrame {
    type Input = Expr;
    type Output = bool;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<bool> {
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_f0f(self, session, outcome, token);
        }

        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(result) if is_system_symbol_name(&result, heads::structure::TRUE) => {
                state.index += 1;
                session.evaluate_readonly_suspended_guards(state, token)
            }
            Ok(_) => Ok(false),
            Err(error) => Err(ResumeStop::failure(error)),
        };
        TypedAction::complete_work(resumed)
    }
}
impl EvaluateReadonlySuspendedGuardsSuspensionFrame {
    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_f0f<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<bool> {
        if crate::cm740_probe::frame("P0459") && self.state.index == 1 {
            crate::cm740_probe::hit("P0459", "OmitActualReadonlySuffixAtGuardOne");
            return TypedAction::complete(
                outcome.map(|result| is_system_symbol_name(&result, heads::structure::TRUE)),
            );
        }
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(result) if is_system_symbol_name(&result, heads::structure::TRUE) => {
                state.index += 1;
                session.evaluate_readonly_suspended_guards_cm740_driven(state, token)
            }
            Ok(_) => Ok(false),
            Err(error) => Err(ResumeStop::failure(error)),
        };
        TypedAction::complete_work_cm740_driven(resumed)
    }
}

fn resume_guard_frame<'resume>(
    session: &mut Session,
    state: SuspendedGuardEvaluation,
    outcome: Result<Expr, EvalError>,
    token: ContinueToken<'resume, EvaluateOwnedGuardsSuspensionFrame>,
) -> TypedAction<CompiledMatchOutcome> {
    let result = match outcome {
        Ok(result) => result,
        Err(error) => return TypedAction::complete(Err(error)),
    };
    complete_guard_frame(if crate::cm740_probe::driven() {
        session.resume_suspended_guard_deferred_cm740_driven(state, result, token)
    } else {
        session.resume_suspended_guard_deferred(state, result, token)
    })
}

fn continue_guard_frame(
    session: &mut Session,
    state: SuspendedGuardEvaluation,
) -> TypedAction<CompiledMatchOutcome> {
    // F0266 inserts a different frame, F0267, through the ordinary witnessed
    // route. F0267 owns the remaining guards; there is no chained F0266 entry.
    complete_guard_frame(
        if crate::cm740_probe::driven() {
            session.evaluate_suspended_guards_deferred_cm740_driven(state, ())
        } else {
            session.evaluate_suspended_guards_deferred(state, ())
        }
        .map_err(ResumeStop::failure),
    )
}

fn complete_guard_frame(
    completion: Result<SuspendedGuardCompletion, ResumeStop<CompiledMatchOutcome>>,
) -> TypedAction<CompiledMatchOutcome> {
    TypedAction::finish_work(completion, |completion| GuardOutcomeFinalizer {
        matched: completion.matched,
        bindings: completion.bindings,
    })
}

struct GuardOutcomeFinalizer {
    matched: bool,
    bindings: FrozenMatchBindings,
}

impl SlotFinalizer for GuardOutcomeFinalizer {
    type Output = CompiledMatchOutcome;

    fn finish(self: Box<Self>) -> Result<CompiledMatchOutcome, EvalError> {
        Ok(guard_outcome(self.matched, self.bindings.into_owned()))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateDefinitionRhsGuardsForMatchSuspensionFrame;

impl TypedFrame for EvaluateDefinitionRhsGuardsForMatchSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = bool;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<bool> {
        let resumed = match outcome {
            Ok(outcome) => (|| -> Result<bool, EvalError> { Ok(outcome.matched) })(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}
