use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct SuspendReplaceStringOutputSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) literal_match_index: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
}

impl rum_runtime::kernel_api::SuspensionFrame for SuspendReplaceStringOutputSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut replacements = self.replacements.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut output = self.output.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let rendered_expr = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<std::string::String, EvalError> {
        resume_suspended_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            literal_match_index,
            output,
            rendered_expr,
        )
    })()
                    .map(|value| <std::string::String as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct DriveSuspendedReplaceStringOutputResultSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) literal_match_index: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for DriveSuspendedReplaceStringOutputResultSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut replacements = self.replacements.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut output = self.output.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let rendered_expr = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<std::string::String, EvalError> {
                    resume_suspended_replace_string_value_with_output(
                        session,
                        state,
                        index,
                        end,
                        replacements,
                        literal_match_index,
                        output,
                        rendered_expr,
                    )
                })()
                    .map(|value| <std::string::String as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeReplaceStringValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) rendered:
        crate::strings::suspended_state::PersistentStringSequence<std::string::String>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeReplaceStringValuesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut rendered = self.rendered.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let value = <std::string::String as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_replace_string_values_with_value(
                            session, state, index, rendered, value,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) rendered:
        crate::strings::suspended_state::PersistentStringSequence<std::string::String>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut rendered = self.rendered.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let value = <std::string::String as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_replace_string_values_with_value(
                            session, state, index, rendered, value,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct DriveStringCasesOutputResultSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) literal_match_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for DriveStringCasesOutputResultSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output,
                    )
                })()
                    .map(|value| <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct DriveSuspendedStringCasesOutputResultSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) literal_match_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for DriveSuspendedStringCasesOutputResultSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output,
                    )
                })()
                    .map(|value| <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeCollectStringCasesValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeCollectStringCasesValuesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let items = <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_collect_string_cases_values_with_value(
                            session, state, index, outputs, items,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let items = <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_collect_string_cases_values_with_value(
                            session, state, index, outputs, items,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
