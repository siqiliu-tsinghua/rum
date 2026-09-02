use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
pub(super) struct ResumeCollectStringCasesFromCandidateSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) rule_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
    pub(super) bindings: crate::strings::suspended_state::FrozenStringValue<
        std::collections::HashMap<
            rum_core::Symbol,
            rum_core::Expr,
            std::hash::BuildHasherDefault<rum_runtime::RuntimeCacheHasher>,
        >,
    >,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeCollectStringCasesFromCandidateSuspensionFrame
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
        let mut rule_index = self.rule_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        #[allow(unused_mut)]
        let mut bindings = self.bindings.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let accepted =
                    <bool as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(
                        resumed_value,
                    );
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                            resume_suspended_collect_string_cases_after_guard(
                                session, state, index, end, rule_index, matches, bindings, accepted,
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
pub(super) struct EvaluateStringCasesOutputSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvaluateStringCasesOutputSuspensionFrame {
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
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session, state, index, end, matches, output,
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
pub(super) struct ResumeSuspendedCollectStringCasesFromCandidateSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) rule_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
    pub(super) bindings: crate::strings::suspended_state::FrozenStringValue<
        std::collections::HashMap<
            rum_core::Symbol,
            rum_core::Expr,
            std::hash::BuildHasherDefault<rum_runtime::RuntimeCacheHasher>,
        >,
    >,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCollectStringCasesFromCandidateSuspensionFrame
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
        let mut rule_index = self.rule_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        #[allow(unused_mut)]
        let mut bindings = self.bindings.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let accepted =
                    <bool as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(
                        resumed_value,
                    );
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                            resume_suspended_collect_string_cases_after_guard(
                                session, state, index, end, rule_index, matches, bindings, accepted,
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
pub(super) struct EvaluateSuspendedStringCasesOutputSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for EvaluateSuspendedStringCasesOutputSuspensionFrame
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
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session, state, index, end, matches, output,
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
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesValuesEvalState,
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
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesValuesEvalState,
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
