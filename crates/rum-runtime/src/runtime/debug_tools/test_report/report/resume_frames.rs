use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeTestReportFromIndexSuspensionFrame {
    pub(super) case_state:
        crate::runtime::debug_tools::test_report::report::SuspendedTestReportCase,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeTestReportFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut case_state = self.case_state.clone();
        let resumed = (|| {
            let evaluation = outcome.map(EvalCapturedOutcome::from_continuation_expr)?;
            session.resume_suspended_test_report_after_evaluation(case_state, evaluation)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeTestReportAfterEvaluationSuspensionFrame {
    pub(super) output_state:
        crate::runtime::debug_tools::test_report::report::SuspendedTestReportOutput,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeTestReportAfterEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut output_state = self.output_state.clone();
        let resumed = (|| {
            let output_match = outcome.map(bool::from_continuation_expr)?;
            session.resume_suspended_test_report_after_output_match(output_state, output_match)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedTestReportAfterEvaluationSuspensionFrame {
    pub(super) output: crate::runtime::debug_tools::test_report::report::SuspendedTestReportOutput,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedTestReportAfterEvaluationSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut output = self.output.clone();
        let resumed = (|| {
            let output_match = outcome.map(bool::from_continuation_expr)?;
            session.resume_suspended_test_report_after_output_match(output, output_match)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ResumeSuspendedTestReportFromIndexSuspensionFrame {
    pub(super) state: crate::runtime::debug_tools::test_report::report::SuspendedTestReportCase,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedTestReportFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let evaluation = outcome.map(EvalCapturedOutcome::from_continuation_expr)?;
            session.resume_suspended_test_report_after_evaluation(state, evaluation)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
