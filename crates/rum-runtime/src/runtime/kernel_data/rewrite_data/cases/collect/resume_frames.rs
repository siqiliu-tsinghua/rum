use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalCasesSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCasesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(target) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareCasesFinalizer {
                    target,
                    context_target: None,
                    args: Some(self.args.clone()),
                    call_args: None,
                    kind: CasesContinuationKind::Target,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeCasesAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeCasesAfterTargetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(resolved_spec) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareCasesFinalizer {
                    target: resolved_spec,
                    context_target: Some(self.target.clone()),
                    args: None,
                    call_args: Some(self.call_args.clone()),
                    kind: CasesContinuationKind::Spec,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(Clone, Copy)]
enum CasesContinuationKind {
    Target,
    Spec,
}

struct PrepareCasesFinalizer {
    target: Expr,
    context_target: Option<Expr>,
    args: Option<SharedArgs>,
    call_args: Option<SharedArgs>,
    kind: CasesContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareCasesFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        match self.kind {
            CasesContinuationKind::Target => Ok(Expr::call(
                system_expr("InternalContinuationCasesTarget"),
                vec![
                    self.target,
                    self.args
                        .expect("Cases target continuation stores call arguments")
                        .into_vec()
                        .into_continuation_expr(),
                ],
            )),
            CasesContinuationKind::Spec => Ok(Expr::call(
                system_expr("InternalContinuationCasesSpec"),
                vec![
                    self.context_target
                        .expect("Cases spec continuation stores its target"),
                    self.target,
                    self.call_args
                        .expect("Cases spec continuation stores call arguments")
                        .into_vec()
                        .into_continuation_expr(),
                ],
            )),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCasesFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCasesFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected Cases continuation payload")
            };
            let mut args = args.into_iter();
            if is_system_symbol_name(&head, "InternalContinuationCasesTarget") {
                let target = args
                    .next()
                    .expect("Cases target continuation stores target");
                let call_args = Vec::<Expr>::from_continuation_expr(
                    args.next()
                        .expect("Cases target continuation stores call arguments"),
                );
                debug_assert!(args.next().is_none());
                session.resume_cases_after_target_suspended(target, SharedArgs::new(call_args))
            } else if is_system_symbol_name(&head, "InternalContinuationCasesSpec") {
                let target = args.next().expect("Cases spec continuation stores target");
                let resolved_spec = args
                    .next()
                    .expect("Cases spec continuation stores resolved spec");
                let call_args = Vec::<Expr>::from_continuation_expr(
                    args.next()
                        .expect("Cases spec continuation stores call arguments"),
                );
                debug_assert!(args.next().is_none());
                session.resume_cases_after_resolved_spec_suspended(
                    target,
                    resolved_spec,
                    SharedArgs::new(call_args),
                )
            } else {
                panic!("unexpected Cases continuation tag: {head:?}")
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeCasesAfterResolvedSpecSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) spec: crate::runtime::kernel_data::rewrite_data::suspended_state::FrozenRewriteValue<
        crate::runtime::CasesSpec,
    >,
    pub(super) call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeCasesAfterResolvedSpecSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        let spec = self.spec.clone();
        let call_args = self.call_args.clone();
        let value = outcome;
        let resumed = (|| {
            let traversal = CasesTraversalResolution::from_continuation_expr(value?);
            session
                .finish_cases_after_resolved_traversal_suspended(target, spec, traversal, call_args)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishCasesWithTraversalSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishCasesWithTraversalSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <crate::runtime::CasesCollectionOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(Expr::call(system_expr("List"), outcome.matches))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCasesTraversalSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesTraversal,
}

impl crate::runtime::suspension::SuspensionFrame for ContinueCasesTraversalSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let case = match outcome {
            Ok(value) => Option::<Expr>::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        let resumed = session
            .resume_cases_traversal(self.state.clone(), case)
            .map(|outcome| outcome.into_continuation_expr());
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct TryCollectCaseOwnedSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesRuleMatch,
}

impl crate::runtime::suspension::SuspensionFrame for TryCollectCaseOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .finish_suspended_cases_rule_match(state, outcome)
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct TryCollectCaseSuspendedSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesRuleMatch,
}

impl crate::runtime::suspension::SuspensionFrame for TryCollectCaseSuspendedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .finish_suspended_cases_rule_match(state, outcome)
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishCasesPatternMatchSuspensionFrame {
    pub(super) expr: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for FinishCasesPatternMatchSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            Ok(outcome.matched.then_some(expr).into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishCasesRuleMatchSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishCasesRuleMatchSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => (|| Ok(value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
