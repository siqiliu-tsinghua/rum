use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSelectFromIndexSuspensionFrame {
    pub(super) source: crate::functional_ops::sequence::select::SuspendedSelectSource,
    pub(super) criterion: rum_core::Expr,
    pub(super) limit: crate::functional_ops::sequence::select::SelectLimit,
    pub(super) index: usize,
    pub(super) selected:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSelectFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut limit = self.limit.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut selected = self.selected.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_select_with_value(
                    session, source, criterion, limit, index, selected, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedSelectFromIndexSuspensionFrame {
    pub(super) source: crate::functional_ops::sequence::select::SuspendedSelectSource,
    pub(super) criterion: rum_core::Expr,
    pub(super) limit: crate::functional_ops::sequence::select::SelectLimit,
    pub(super) index: usize,
    pub(super) selected:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedSelectFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut limit = self.limit.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut selected = self.selected.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_select_with_value(
                    session, source, criterion, limit, index, selected, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSelectFirstFromIndexSuspensionFrame {
    pub(super) source: crate::functional_ops::sequence::select::SuspendedSelectSource,
    pub(super) criterion: rum_core::Expr,
    pub(super) default: rum_core::Expr,
    pub(super) index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSelectFirstFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut default = self.default.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_select_first_with_value(
                    session, source, criterion, default, index, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedSelectFirstFromIndexSuspensionFrame {
    pub(super) source: crate::functional_ops::sequence::select::SuspendedSelectSource,
    pub(super) criterion: rum_core::Expr,
    pub(super) default: rum_core::Expr,
    pub(super) index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedSelectFirstFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut source = self.source.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut default = self.default.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_select_first_with_value(
                    session, source, criterion, default, index, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
