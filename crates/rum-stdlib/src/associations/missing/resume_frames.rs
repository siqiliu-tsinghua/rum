use super::target::{DeleteMissingTargetWork, resume_suspended_delete_missing_target_work};
use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

fn resume_target_work(
    session: &mut Session,
    work: FrozenAssociationSequence<DeleteMissingTargetWork>,
    outcome: Result<Expr, EvalError>,
) -> rum_runtime::kernel_api::FrameAction {
    let resumed = match outcome {
        Ok(value) => resume_suspended_delete_missing_target_work(session, work, value),
        Err(error) => Err(error),
    };
    rum_runtime::kernel_api::FrameAction::complete(resumed)
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrame1 {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingListAtTargetLevelSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrame2 {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingListAtTargetLevelSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrame {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1 {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame2 {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame3 {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame3
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame1 {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame2 {
    pub(super) work: FrozenAssociationSequence<DeleteMissingTargetWork>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        resume_target_work(session, self.work.clone(), outcome)
    }
}
