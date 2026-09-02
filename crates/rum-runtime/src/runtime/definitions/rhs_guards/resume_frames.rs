use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledDefinitionConditionOwnedSuspensionFrame {
    pub(super) guards: rum_core::SharedArgs,
    pub(super) option_scope: std::option::Option<crate::runtime::options::OptionValueScope>,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchCompiledDefinitionConditionOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => CompiledMatchOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        if !outcome.matched {
            return crate::runtime::suspension::FrameAction::complete(Ok(
                outcome.into_continuation_expr()
            ));
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

impl crate::runtime::suspension::SuspensionFrame for EvaluateOwnedGuardsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        resume_guard_frame(session, self.state.clone(), outcome)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateReadonlySuspendedGuardsSuspensionFrame {
    pub(super) state: crate::runtime::definitions::rhs_guards::SuspendedReadonlyGuardEvaluation,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateReadonlySuspendedGuardsSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(result) if is_system_symbol_name(&result, heads::structure::TRUE) => {
                state.index += 1;
                session
                    .evaluate_readonly_suspended_guards(state)
                    .map(|matched| matched.into_continuation_expr())
            }
            Ok(_) => Ok(false.into_continuation_expr()),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

fn resume_guard_frame(
    session: &mut Session,
    state: SuspendedGuardEvaluation,
    outcome: Result<Expr, EvalError>,
) -> crate::runtime::suspension::FrameAction {
    let result = match outcome {
        Ok(result) => result,
        Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
    };
    complete_guard_frame(session.resume_suspended_guard_deferred(state, result))
}

fn continue_guard_frame(
    session: &mut Session,
    state: SuspendedGuardEvaluation,
) -> crate::runtime::suspension::FrameAction {
    complete_guard_frame(session.evaluate_suspended_guards_deferred(state))
}

fn complete_guard_frame(
    completion: Result<SuspendedGuardCompletion, EvalError>,
) -> crate::runtime::suspension::FrameAction {
    match completion {
        Ok(completion) => {
            crate::runtime::suspension::FrameAction::finalize(GuardOutcomeFinalizer {
                matched: completion.matched,
                bindings: completion.bindings,
            })
        }
        Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
    }
}

struct GuardOutcomeFinalizer {
    matched: bool,
    bindings: FrozenMatchBindings,
}

impl crate::runtime::suspension::PureFrameFinalizer for GuardOutcomeFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(guard_outcome(self.matched, self.bindings.into_owned()).into_continuation_expr())
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateDefinitionRhsGuardsForMatchSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateDefinitionRhsGuardsForMatchSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<bool, EvalError> { Ok(outcome.matched) })().map(|value| {
                    <bool as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(
                        value,
                    )
                })
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
