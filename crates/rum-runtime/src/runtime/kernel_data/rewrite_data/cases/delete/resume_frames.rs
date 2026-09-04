use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalDeleteCasesSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalDeleteCasesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareDeleteCasesFinalizer {
                    result,
                    target: None,
                    pattern: None,
                    args: Some(self.args.clone()),
                    call_args: None,
                    kind: DeleteCasesContinuationKind::Target,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeDeleteCasesAfterTargetSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeDeleteCasesAfterTargetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareDeleteCasesFinalizer {
                    result,
                    target: Some(self.target.clone()),
                    pattern: None,
                    args: None,
                    call_args: Some(self.call_args.clone()),
                    kind: DeleteCasesContinuationKind::Pattern,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeDeleteCasesAfterPatternSuspensionFrame {
    pub(super) target: rum_core::Expr,
    pub(super) pattern: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeDeleteCasesAfterPatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareDeleteCasesFinalizer {
                    result,
                    target: Some(self.target.clone()),
                    pattern: Some(self.pattern.clone()),
                    args: None,
                    call_args: Some(self.call_args.clone()),
                    kind: DeleteCasesContinuationKind::Traversal,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(Clone, Copy)]
enum DeleteCasesContinuationKind {
    Target,
    Pattern,
    Traversal,
}

struct PrepareDeleteCasesFinalizer {
    result: Expr,
    target: Option<Expr>,
    pattern: Option<Expr>,
    args: Option<SharedArgs>,
    call_args: Option<SharedArgs>,
    kind: DeleteCasesContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareDeleteCasesFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let tag = match self.kind {
            DeleteCasesContinuationKind::Target => "InternalContinuationDeleteCasesTarget",
            DeleteCasesContinuationKind::Pattern => "InternalContinuationDeleteCasesPattern",
            DeleteCasesContinuationKind::Traversal => "InternalContinuationDeleteCasesTraversal",
        };
        Ok(Expr::call(
            system_expr(tag),
            vec![
                self.result,
                self.target.into_continuation_expr(),
                self.pattern.into_continuation_expr(),
                self.args.map(SharedArgs::into_vec).into_continuation_expr(),
                self.call_args
                    .map(SharedArgs::into_vec)
                    .into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueDeleteCasesFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueDeleteCasesFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected DeleteCases continuation payload")
            };
            let mut args = args.into_iter();
            let result = args
                .next()
                .expect("DeleteCases continuation stores its result");
            let target = Option::<Expr>::from_continuation_expr(
                args.next().expect("DeleteCases continuation stores target"),
            );
            let pattern = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("DeleteCases continuation stores pattern"),
            );
            let call = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("DeleteCases continuation stores original arguments"),
            );
            let call_args = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("DeleteCases continuation stores call arguments"),
            );
            debug_assert!(args.next().is_none());

            if is_system_symbol_name(&head, "InternalContinuationDeleteCasesTarget") {
                let call = call.expect("DeleteCases target continuation stores arguments");
                session.resume_delete_cases_after_target_suspended(result, SharedArgs::new(call))
            } else if is_system_symbol_name(&head, "InternalContinuationDeleteCasesPattern") {
                session.resume_delete_cases_after_pattern_suspended(
                    target.expect("DeleteCases pattern continuation stores target"),
                    result,
                    SharedArgs::new(
                        call_args.expect("DeleteCases pattern continuation stores call arguments"),
                    ),
                )
            } else if is_system_symbol_name(&head, "InternalContinuationDeleteCasesTraversal") {
                session.finish_delete_cases_after_resolved_traversal_suspended(
                    target.expect("DeleteCases traversal continuation stores target"),
                    pattern.expect("DeleteCases traversal continuation stores pattern"),
                    CasesTraversalResolution::from_continuation_expr(result),
                    SharedArgs::new(
                        call_args
                            .expect("DeleteCases traversal continuation stores call arguments"),
                    ),
                )
            } else {
                panic!("unexpected DeleteCases continuation tag: {head:?}")
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishDeleteCasesAfterResolvedTraversalSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame
    for FinishDeleteCasesAfterResolvedTraversalSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(match outcome.result {
                DeleteCasesResult::Keep(expr) => expr,
                DeleteCasesResult::Delete => Expr::call(system_expr("Sequence"), Vec::new()),
                DeleteCasesResult::Splice(items) => Expr::call(system_expr("Sequence"), items),
            })
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishDeleteCasesAtCurrentNodeSuspensionFrame {
    pub(super) depth: usize,
    pub(super) traversal: crate::runtime::CasesTraversalConfig,
    pub(super) deleted_before: usize,
    pub(super) expr: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for FinishDeleteCasesAtCurrentNodeSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut depth = self.depth.clone();
        #[allow(unused_mut)]
        let mut traversal = self.traversal.clone();
        #[allow(unused_mut)]
        let mut deleted_before = self.deleted_before.clone();
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rewrite_data::DeleteCasesOutcome, EvalError> {
            if outcome.matched {
                Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Delete,
                    depth,
                    deleted_count: 1,
                    exhausted: traversal.limit_reached(deleted_before + 1),
                })
            } else {
                Ok(DeleteCasesOutcome {
                    result: DeleteCasesResult::Keep(expr),
                    depth,
                    deleted_count: 0,
                    exhausted: false,
                })
            }
        })()
                    .map(|value| <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DriveDeleteCasesTraversalSuspensionFrame {
    pub(super) state: DeleteCasesTraversalState,
    pub(super) added: usize,
}

impl crate::runtime::suspension::SuspensionFrame for DriveDeleteCasesTraversalSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let state = self.state.clone();
        let added = self.added;
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                session
                    .resume_delete_cases_traversal(state, outcome, added)
                    .map(|value| <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DeleteCasesCallChildrenFromQueueSuspensionFrame1 {
    pub(super) deleted_before: usize,
    pub(super) subtree_start: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for DeleteCasesCallChildrenFromQueueSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut deleted_before = self.deleted_before.clone();
        #[allow(unused_mut)]
        let mut subtree_start = self.subtree_start.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let mut outcome = <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rewrite_data::DeleteCasesOutcome, EvalError> {
                    outcome.deleted_count += deleted_before - subtree_start;
                    Ok(outcome)
                })()
                    .map(|value| <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DeleteCasesCallChildrenFromQueueSuspensionFrame2 {
    pub(super) state: crate::runtime::kernel_data::rewrite_data::cases::delete::suspended_state::SuspendedDeleteCallChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for DeleteCasesCallChildrenFromQueueSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = DeleteCasesOutcome::from_continuation_expr(value?);
            session
                .resume_suspended_delete_call_children(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DeleteCasesAssociationChildrenFromQueueSuspensionFrame1 {
    pub(super) deleted_before: usize,
    pub(super) subtree_start: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for DeleteCasesAssociationChildrenFromQueueSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut deleted_before = self.deleted_before.clone();
        #[allow(unused_mut)]
        let mut subtree_start = self.subtree_start.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let mut outcome = <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rewrite_data::DeleteCasesOutcome, EvalError> {
                    outcome.deleted_count += deleted_before - subtree_start;
                    Ok(outcome)
                })()
                    .map(|value| <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct DeleteCasesAssociationChildrenFromQueueSuspensionFrame2 {
    pub(super) state: crate::runtime::kernel_data::rewrite_data::cases::delete::suspended_state::SuspendedDeleteAssociationChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for DeleteCasesAssociationChildrenFromQueueSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = DeleteCasesOutcome::from_continuation_expr(value?);
            session
                .resume_suspended_delete_association_children(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteCallChildrenSuspensionFrame1 {
    pub(super) added: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedDeleteCallChildrenSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut added = self.added.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let mut outcome = <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rewrite_data::DeleteCasesOutcome, EvalError> {
                    outcome.deleted_count += added;
                    Ok(outcome)
                })()
                    .map(|value| <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteCallChildrenSuspensionFrame2 {
    pub(super) state: crate::runtime::kernel_data::rewrite_data::cases::delete::suspended_state::SuspendedDeleteCallChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedDeleteCallChildrenSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = DeleteCasesOutcome::from_continuation_expr(value?);
            session
                .resume_suspended_delete_call_children(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteAssociationChildrenSuspensionFrame1 {
    pub(super) added: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedDeleteAssociationChildrenSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut added = self.added.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let mut outcome = <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rewrite_data::DeleteCasesOutcome, EvalError> {
                    outcome.deleted_count += added;
                    Ok(outcome)
                })()
                    .map(|value| <rewrite_data::DeleteCasesOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteAssociationChildrenSuspensionFrame2 {
    pub(super) state: crate::runtime::kernel_data::rewrite_data::cases::delete::suspended_state::SuspendedDeleteAssociationChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedDeleteAssociationChildrenSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = DeleteCasesOutcome::from_continuation_expr(value?);
            session
                .resume_suspended_delete_association_children(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
