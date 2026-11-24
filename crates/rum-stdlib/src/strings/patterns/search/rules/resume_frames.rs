use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
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

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon for SuspendReplaceStringOutputSuspensionFrame {
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

impl rum_runtime::kernel_api::SuspensionFrame for SuspendReplaceStringOutputSuspensionFrame {
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
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

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon
    for DriveSuspendedReplaceStringOutputResultSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

impl rum_runtime::kernel_api::SuspensionFrame
    for DriveSuspendedReplaceStringOutputResultSuspensionFrame
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeReplaceStringValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceValuesEvalState,
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
        crate::strings::patterns::search::rules::StringReplaceValuesEvalState,
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct DriveStringCasesOutputResultSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) literal_match_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon for DriveStringCasesOutputResultSuspensionFrame {
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::cm740_probe::q25::retire_abandon(
            std::sync::Arc::as_ptr(&self) as *const () as usize
        );
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

fn challenge_equal_siblings(origin: &std::sync::Arc<DriveStringCasesOutputResultSuspensionFrame>) {
    let sibling = |source: &DriveStringCasesOutputResultSuspensionFrame| {
        std::sync::Arc::new(DriveStringCasesOutputResultSuspensionFrame {
            state: source.state.clone(),
            index: source.index,
            end: source.end,
            literal_match_index: source.literal_match_index,
            matches: source.matches.clone(),
        })
    };
    let left = sibling(origin);
    let right = sibling(origin);
    let fields_equal = origin.index == left.index
        && origin.end == left.end
        && origin.literal_match_index == left.literal_match_index
        && origin.index == right.index
        && std::ptr::eq(origin.state.as_ref(), left.state.as_ref())
        && std::ptr::eq(origin.state.as_ref(), right.state.as_ref())
        && origin.matches.len() == left.matches.len()
        && left.matches.len() == right.matches.len()
        && !std::ptr::eq(
            std::sync::Arc::as_ptr(origin),
            std::sync::Arc::as_ptr(&left),
        )
        && !std::ptr::eq(
            std::sync::Arc::as_ptr(origin),
            std::sync::Arc::as_ptr(&right),
        )
        && !std::ptr::eq(
            std::sync::Arc::as_ptr(&left),
            std::sync::Arc::as_ptr(&right),
        );
    rum_runtime::cm740_probe::q25::reject_equal_siblings(origin, &left, &right, fields_equal);
}

impl rum_runtime::kernel_api::SuspensionFrame for DriveStringCasesOutputResultSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let _q25_claim = rum_runtime::cm740_probe::q25::begin_concrete(&self);
        let _class_row = rum_runtime::cm740_probe::class_lease::begin_row(&self);

        if rum_runtime::cm740_probe::q25::siblings_mode() {
            challenge_equal_siblings(&self);
        }
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct DriveSuspendedStringCasesOutputResultSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) literal_match_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon
    for DriveSuspendedStringCasesOutputResultSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
}

impl rum_runtime::kernel_api::SuspensionFrame
    for DriveSuspendedStringCasesOutputResultSuspensionFrame
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeCollectStringCasesValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon
    for ResumeCollectStringCasesValuesFromIndexSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
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

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrame {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

// CM4 copied carrier: actual one-use retirement before the
// unconditional Discard (Q26/Q27 substitution render; the derive
// expansion is unconditional Discard).
impl rum_runtime::kernel_api::FrameAbandon
    for ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrame
{
    fn abandon(
        self: ::std::sync::Arc<Self>,
        _context: &mut rum_runtime::kernel_api::AbandonmentContext<'_>,
    ) -> rum_runtime::kernel_api::FrameAbandonAction {
        rum_runtime::kernel_api::FrameAbandonAction::Discard
    }
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
