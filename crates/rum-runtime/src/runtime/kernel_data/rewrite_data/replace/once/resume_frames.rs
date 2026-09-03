use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalReplaceSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) levels_expr: std::option::Option<rum_core::Expr>,
    pub(super) heads_option_exprs: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalReplaceSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareReplaceSetupFinalizer {
                    result,
                    target: None,
                    args: Some(self.args.clone()),
                    levels_expr: self.levels_expr.clone(),
                    heads_option_exprs: Some(self.heads_option_exprs.clone()),
                    call_args: None,
                    include_heads: None,
                    kind: ReplaceSetupContinuationKind::Target,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeReplaceAfterRuleExprsSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) rule_plan: crate::runtime::RewriteRulePlan,
    pub(super) levels_expr: std::option::Option<rum_core::Expr>,
    pub(super) call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeReplaceAfterRuleExprsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareReplaceSetupFinalizer {
                    result,
                    target: Some(self.target.clone()),
                    args: None,
                    levels_expr: self.levels_expr.clone(),
                    heads_option_exprs: None,
                    call_args: Some(self.call_args.clone()),
                    include_heads: None,
                    kind: ReplaceSetupContinuationKind::Heads,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishReplaceHeadsResolutionSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) rule_plan: crate::runtime::RewriteRulePlan,
    pub(super) call_args: rum_core::SharedArgs,
    pub(super) include_heads: bool,
}

impl crate::runtime::suspension::SuspensionFrame for FinishReplaceHeadsResolutionSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareReplaceSetupFinalizer {
                    result,
                    target: Some(self.target.clone()),
                    args: None,
                    levels_expr: None,
                    heads_option_exprs: None,
                    call_args: Some(self.call_args.clone()),
                    include_heads: Some(self.include_heads),
                    kind: ReplaceSetupContinuationKind::Levels,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(Clone, Copy)]
enum ReplaceSetupContinuationKind {
    Target,
    Heads,
    Levels,
}

struct PrepareReplaceSetupFinalizer {
    result: Expr,
    target: Option<Expr>,
    args: Option<SharedArgs>,
    levels_expr: Option<Expr>,
    heads_option_exprs: Option<SharedArgs>,
    call_args: Option<SharedArgs>,
    include_heads: Option<bool>,
    kind: ReplaceSetupContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareReplaceSetupFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let tag = match self.kind {
            ReplaceSetupContinuationKind::Target => "InternalContinuationReplaceTarget",
            ReplaceSetupContinuationKind::Heads => "InternalContinuationReplaceHeads",
            ReplaceSetupContinuationKind::Levels => "InternalContinuationReplaceLevels",
        };
        Ok(Expr::call(
            system_expr(tag),
            vec![
                self.result,
                self.target.into_continuation_expr(),
                self.args.map(SharedArgs::into_vec).into_continuation_expr(),
                self.levels_expr.into_continuation_expr(),
                self.heads_option_exprs
                    .map(SharedArgs::into_vec)
                    .into_continuation_expr(),
                self.call_args
                    .map(SharedArgs::into_vec)
                    .into_continuation_expr(),
                self.include_heads
                    .map(bool::into_continuation_expr)
                    .into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueReplaceSetupFrame {
    pub(super) rule_plan: Option<RewriteRulePlan>,
}

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceSetupFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected Replace setup continuation payload")
            };
            let mut args = args.into_iter();
            let result = args
                .next()
                .expect("Replace setup continuation stores its result");
            let target = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("Replace setup continuation stores target"),
            );
            let original_args = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("Replace setup continuation stores original arguments"),
            );
            let levels_expr = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("Replace setup continuation stores level expression"),
            );
            let heads_option_exprs = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("Replace setup continuation stores Heads options"),
            );
            let call_args = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("Replace setup continuation stores call arguments"),
            );
            let include_heads = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("Replace setup continuation stores Heads flag"),
            )
            .map(bool::from_continuation_expr);
            debug_assert!(args.next().is_none());

            if is_system_symbol_name(&head, "InternalContinuationReplaceTarget") {
                let original_args =
                    original_args.expect("Replace target continuation stores arguments");
                session.resume_replace_after_target(
                    result,
                    original_args[1].clone(),
                    levels_expr,
                    heads_option_exprs.expect("Replace target continuation stores Heads options"),
                    original_args,
                )
            } else if is_system_symbol_name(&head, "InternalContinuationReplaceHeads") {
                session.finish_replace_heads_resolution(
                    target.expect("Replace Heads continuation stores target"),
                    self.rule_plan
                        .clone()
                        .expect("Replace Heads continuation stores rule plan"),
                    levels_expr,
                    call_args.expect("Replace Heads continuation stores call arguments"),
                    Option::<Expr>::from_continuation_expr(result),
                )
            } else if is_system_symbol_name(&head, "InternalContinuationReplaceLevels") {
                session.finish_replace_levels_resolution(
                    target.expect("Replace levels continuation stores target"),
                    self.rule_plan
                        .clone()
                        .expect("Replace levels continuation stores rule plan"),
                    call_args.expect("Replace levels continuation stores call arguments"),
                    include_heads.expect("Replace levels continuation stores Heads flag"),
                    Option::<CasesLevelSpec>::from_continuation_expr(result),
                )
            } else {
                panic!("unexpected Replace setup continuation tag: {head:?}")
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishReplaceLevelsResolutionSuspensionFrame1;

impl crate::runtime::suspension::SuspensionFrame for FinishReplaceLevelsResolutionSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <rewrite_data::ReplaceOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            let evaluated_result = session.evaluate(outcome.expr);
            crate::resume_or_suspend!(
    session,
    evaluated_result,
    ok(value) => Ok(value),
    suspend => FinishReplaceLevelsResolutionSuspensionFrame2,
)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishReplaceLevelsResolutionSuspensionFrame2;

impl crate::runtime::suspension::SuspensionFrame for FinishReplaceLevelsResolutionSuspensionFrame2 {
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

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResolveReplaceHeadsOptionsOwnedSuspensionFrame {
    pub(super) default: bool,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResolveReplaceHeadsOptionsOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut default = self.default.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let resolved_exprs = <Vec<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Option<rum_core::Expr>, EvalError> {
            let mut include_heads = default;
            for expr in resolved_exprs {
                let Some(resolved) = parse_heads_option_with_default(&expr, include_heads) else {
                    return Ok(None);
                };
                include_heads = resolved;
            }
            Ok(Some(bool_expr(include_heads)))
        })()
                    .map(|value| <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResolveReplaceLevelSpecOwnedSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for ResolveReplaceLevelSpecOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Option<crate::runtime::CasesLevelSpec>, EvalError> {
            Ok(parse_standard_level_spec(&resolved))
        })()
                    .map(|value| <Option<crate::runtime::CasesLevelSpec> as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ReplaceOwnedSuspensionFrame {
    pub(super) include_heads: bool,
    pub(super) expr: rum_core::Expr,
    pub(super) rule_plan: crate::runtime::RewriteRulePlan,
    pub(super) levels: crate::runtime::CasesLevelSpec,
    pub(super) level: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ReplaceOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut include_heads = self.include_heads.clone();
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        #[allow(unused_mut)]
        let mut rule_plan = self.rule_plan.clone();
        #[allow(unused_mut)]
        let mut levels = self.levels.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let replaced = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rewrite_data::ReplaceOutcome, EvalError> {
                if let Some(replaced) = replaced {
                    let depth = if include_heads {
                        replace_expr_depth_with_heads(&replaced)
                    } else {
                        replace_expr_depth(&replaced)
                    };
                    Ok(ReplaceOutcome {
                        depth,
                        expr: replaced,
                        changed: true,
                    })
                } else {
                    session.replace_children_owned(
                        expr,
                        rule_plan,
                        levels,
                        level + 1,
                        include_heads,
                    )
                }
            })()
                    .map(|value| <rewrite_data::ReplaceOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ReplaceChildrenOwnedSuspensionFrame1 {
    pub(super) args: crate::runtime::kernel_data::rewrite_data::suspended_state::FrozenRewriteQueue<
        rum_core::Expr,
    >,
    pub(super) rule_plan: crate::runtime::RewriteRulePlan,
    pub(super) levels: crate::runtime::CasesLevelSpec,
    pub(super) child_level: usize,
    pub(super) include_heads: bool,
}

impl crate::runtime::suspension::SuspensionFrame for ReplaceChildrenOwnedSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => ReplaceOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_action(session.finish_suspended_replace_association_head(
            self.args.clone(),
            self.rule_plan.clone(),
            self.levels,
            self.child_level,
            self.include_heads,
            outcome,
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ReplaceChildrenOwnedSuspensionFrame2 {
    pub(super) args: crate::runtime::kernel_data::rewrite_data::suspended_state::FrozenRewriteQueue<
        rum_core::Expr,
    >,
    pub(super) rule_plan: crate::runtime::RewriteRulePlan,
    pub(super) levels: crate::runtime::CasesLevelSpec,
    pub(super) child_level: usize,
    pub(super) include_heads: bool,
}

impl crate::runtime::suspension::SuspensionFrame for ReplaceChildrenOwnedSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => ReplaceOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_action(session.finish_suspended_replace_call_head(
            self.args.clone(),
            self.rule_plan.clone(),
            self.levels,
            self.child_level,
            self.include_heads,
            outcome,
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ReplaceCallChildrenOwnedSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::replace::once::SuspendedReplaceCallChildren,
}

impl crate::runtime::suspension::SuspensionFrame for ReplaceCallChildrenOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => ReplaceOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_action(
            session.resume_suspended_replace_call_children(self.state.clone(), outcome),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ReplaceAssociationChildrenOwnedSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::rewrite_data::replace::once::SuspendedReplaceAssociationChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for ReplaceAssociationChildrenOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => ReplaceOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_action(
            session.resume_suspended_replace_association_children(self.state.clone(), outcome),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DriveSuspendedReplaceCallChildrenSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::replace::once::SuspendedReplaceCallChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for DriveSuspendedReplaceCallChildrenSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => ReplaceOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_action(
            session.resume_suspended_replace_call_children(self.state.clone(), outcome),
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DriveSuspendedReplaceAssociationChildrenSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::rewrite_data::replace::once::SuspendedReplaceAssociationChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for DriveSuspendedReplaceAssociationChildrenSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(value) => ReplaceOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        finish_suspended_replace_action(
            session.resume_suspended_replace_association_children(self.state.clone(), outcome),
        )
    }
}

fn finish_suspended_replace_action(
    result: Result<SuspendedReplaceCompletion, EvalError>,
) -> crate::runtime::suspension::FrameAction {
    match result {
        Ok(completion) => {
            crate::runtime::suspension::FrameAction::finalize(FinishSuspendedReplaceFinalizer {
                completion,
            })
        }
        Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
    }
}

struct FinishSuspendedReplaceFinalizer {
    completion: SuspendedReplaceCompletion,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishSuspendedReplaceFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self.completion.finish().into_continuation_expr())
    }
}
