use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeReplaceStringValueFromCandidateSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::replace::GuardedStringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) rule_index: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
    pub(super) bindings: crate::strings::suspended_state::FrozenStringValue<
        std::collections::HashMap<
            rum_core::Symbol,
            rum_core::Expr,
            std::hash::BuildHasherDefault<rum_runtime::RuntimeCacheHasher>,
        >,
    >,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon
    for ResumeReplaceStringValueFromCandidateSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeReplaceStringValueFromCandidateSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let _class_row = rum_runtime::cm740_probe::class_lease::begin_row(&self);
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut replacements = self.replacements.clone();
        #[allow(unused_mut)]
        let mut rule_index = self.rule_index.clone();
        #[allow(unused_mut)]
        let mut output = self.output.clone();
        #[allow(unused_mut)]
        let mut bindings = self.bindings.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let accepted =
                    <bool as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(
                        resumed_value,
                    );
                (|| -> Result<std::string::String, EvalError> {
                            resume_suspended_replace_string_value_after_guard(
                                session,
                                state,
                                index,
                                end,
                                replacements,
                                rule_index,
                                output,
                                bindings,
                                accepted,
                            )
                        })()
                    .map(|value| <std::string::String as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct EvaluateReplaceStringOutputSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::replace::GuardedStringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon for EvaluateReplaceStringOutputSuspensionFrame {
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

impl rum_runtime::kernel_api::SuspensionFrame for EvaluateReplaceStringOutputSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let _class_row = rum_runtime::cm740_probe::class_lease::begin_row(&self);
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut replacements = self.replacements.clone();
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedReplaceStringValueFromCandidateSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::replace::GuardedStringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) rule_index: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
    pub(super) bindings: crate::strings::suspended_state::FrozenStringValue<
        std::collections::HashMap<
            rum_core::Symbol,
            rum_core::Expr,
            std::hash::BuildHasherDefault<rum_runtime::RuntimeCacheHasher>,
        >,
    >,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon
    for ResumeSuspendedReplaceStringValueFromCandidateSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedReplaceStringValueFromCandidateSuspensionFrame
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
        let mut rule_index = self.rule_index.clone();
        #[allow(unused_mut)]
        let mut output = self.output.clone();
        #[allow(unused_mut)]
        let mut bindings = self.bindings.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let accepted =
                    <bool as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(
                        resumed_value,
                    );
                (|| -> Result<std::string::String, EvalError> {
                            resume_suspended_replace_string_value_after_guard(
                                session,
                                state,
                                index,
                                end,
                                replacements,
                                rule_index,
                                output,
                                bindings,
                                accepted,
                            )
                        })()
                    .map(|value| <std::string::String as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct EvaluateSuspendedReplaceStringOutputSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::replace::GuardedStringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon for EvaluateSuspendedReplaceStringOutputSuspensionFrame {
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

impl rum_runtime::kernel_api::SuspensionFrame
    for EvaluateSuspendedReplaceStringOutputSuspensionFrame
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeReplaceStringValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::replace::GuardedStringReplaceValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) rendered:
        crate::strings::suspended_state::PersistentStringSequence<std::string::String>,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon for ResumeReplaceStringValuesFromIndexSuspensionFrame {
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::replace::GuardedStringReplaceValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) rendered:
        crate::strings::suspended_state::PersistentStringSequence<std::string::String>,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon
    for ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
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
