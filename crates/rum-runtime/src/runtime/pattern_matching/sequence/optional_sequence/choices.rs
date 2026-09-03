use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct SequenceChoiceState {
    flat_head: Option<Expr>,
    spec: CompiledSequencePattern,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    snapshot: SymbolExprMap,
    consume: usize,
    max_len: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSequenceChoiceState {
    flat_head: Option<Expr>,
    spec: FrozenMatcherValue<CompiledSequencePattern>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    snapshot: FrozenMatchBindings,
    consume: usize,
    max_len: usize,
}

impl SuspendedSequenceChoiceState {
    fn freeze(state: SequenceChoiceState) -> Self {
        Self {
            flat_head: state.flat_head,
            spec: FrozenMatcherValue::freeze(state.spec),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            snapshot: FrozenMatchBindings::freeze(state.snapshot),
            consume: state.consume,
            max_len: state.max_len,
        }
    }

    fn into_owned(self) -> SequenceChoiceState {
        SequenceChoiceState {
            flat_head: self.flat_head,
            spec: self.spec.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            snapshot: self.snapshot.into_owned(),
            consume: self.consume,
            max_len: self.max_len,
        }
    }
}

impl SequenceChoiceState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationSequenceChoiceState"),
            vec![
                self.flat_head.into_continuation_expr(),
                sequence_pattern_into_continuation_expr(self.spec),
                matcher_args_into_continuation_expr(self.rest_patterns),
                self.value_args.into_continuation_expr(),
                binding_map_into_continuation_expr(self.snapshot),
                usize_into_continuation_expr(self.consume),
                usize_into_continuation_expr(self.max_len),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args, .. } = expr else {
            panic!("unexpected sequence-choice continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationSequenceChoiceState"),
            "unexpected sequence-choice continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let state = Self {
            flat_head: Option::<Expr>::from_continuation_expr(
                args.next().expect("sequence choice stores its flat head"),
            ),
            spec: sequence_pattern_from_continuation_expr(
                args.next()
                    .expect("sequence choice stores its specification"),
            ),
            rest_patterns: matcher_args_from_continuation_expr(
                args.next()
                    .expect("sequence choice stores its remaining patterns"),
            ),
            value_args: Vec::<Expr>::from_continuation_expr(
                args.next().expect("sequence choice stores its values"),
            ),
            snapshot: binding_map_from_continuation_expr(
                args.next()
                    .expect("sequence choice stores its binding snapshot"),
            ),
            consume: usize_from_continuation_expr(
                args.next()
                    .expect("sequence choice stores its consume cursor"),
            ),
            max_len: usize_from_continuation_expr(
                args.next()
                    .expect("sequence choice stores its maximum length"),
            ),
        };
        debug_assert!(args.next().is_none());
        state
    }
}

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_sequence_choices(
        &mut self,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consume: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let min_rest = min_required_matcher_arg_count(&rest_patterns);
        let Some(max_len) = max_sequence_consume(&spec, min_rest, value_args.len()) else {
            return Ok(match_outcome(false, bindings));
        };
        if !(spec.min_len..=max_len).contains(&consume) {
            return Ok(match_outcome(false, bindings));
        }

        let consumed_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let state = SequenceChoiceState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            snapshot: bindings.clone(),
            consume,
            max_len,
        };
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            state.spec.clone(),
            consumed_values,
            bindings,
        );
        self.drive_sequence_choice_result(sequence_result, state, rest_values)
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_flat_sequence_choices(
        &mut self,
        flat_head: Expr,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consume: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let min_rest = min_required_matcher_arg_count(&rest_patterns);
        let Some(max_len) = max_sequence_consume(&spec, min_rest, value_args.len()) else {
            return Ok(match_outcome(false, bindings));
        };
        if !(spec.min_len..=max_len).contains(&consume) {
            return Ok(match_outcome(false, bindings));
        }

        let consumed_values = value_args.iter().take(consume).cloned().collect::<Vec<_>>();
        let rest_values = value_args.iter().skip(consume).cloned().collect::<Vec<_>>();
        let state = SequenceChoiceState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            snapshot: bindings.clone(),
            consume,
            max_len,
        };
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            state.spec.clone(),
            consumed_values,
            bindings,
        );
        self.drive_sequence_choice_result(sequence_result, state, rest_values)
    }

    fn drive_sequence_choice_result(
        &mut self,
        result: Result<CompiledMatchOutcome, EvalError>,
        state: SequenceChoiceState,
        rest_values: Vec<Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match result {
            Ok(outcome) => self.finish_sequence_choice(state, rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceState::freeze(state);
                let rest_values = SharedArgs::new(rest_values);
                crate::resume_or_suspend!(insert self, DriveSequenceChoiceResultSuspensionFrame { state, rest_values });
                crate::resume_or_suspend!(insert self, ContinueSequenceChoiceResultFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice(
        &mut self,
        state: SequenceChoiceState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return self.retry_sequence_choice(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_args_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.match_compiled_args_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(outcome) => self.finish_sequence_choice_rest(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceState::freeze(state);
                crate::resume_or_suspend!(insert self, FinishSequenceChoiceSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueSequenceChoiceRestFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice_rest(
        &mut self,
        state: SequenceChoiceState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.retry_sequence_choice(state)
        }
    }

    fn retry_sequence_choice(
        &mut self,
        state: SequenceChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(next_consume) = next_sequence_consume(
            state.spec.greediness,
            state.consume,
            state.spec.min_len,
            state.max_len,
        ) else {
            return Ok(match_outcome(false, state.snapshot));
        };

        if let Some(flat_head) = state.flat_head {
            self.match_compiled_flat_sequence_choices(
                flat_head,
                state.spec,
                state.rest_patterns,
                state.value_args,
                state.snapshot,
                next_consume,
            )
        } else {
            self.match_compiled_sequence_choices(
                state.spec,
                state.rest_patterns,
                state.value_args,
                state.snapshot,
                next_consume,
            )
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedSequenceChoiceState>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct DriveSequenceChoiceResultSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::choices::SuspendedSequenceChoiceState,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for DriveSequenceChoiceResultSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareSequenceChoiceResultFinalizer {
                    state: self.state.clone(),
                    rest_values: self.rest_values.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareSequenceChoiceResultFinalizer {
    state: SuspendedSequenceChoiceState,
    rest_values: SharedArgs,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareSequenceChoiceResultFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationSequenceChoiceResult"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.rest_values.into_vec().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueSequenceChoiceResultFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueSequenceChoiceResultFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected sequence-choice result continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationSequenceChoiceResult"),
                "unexpected sequence-choice result continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let state = SequenceChoiceState::from_continuation_expr(
                args.next()
                    .expect("sequence-choice result stores its state"),
            );
            let rest_values = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("sequence-choice result stores its remaining values"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("sequence-choice result stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_sequence_choice(state, rest_values, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishSequenceChoiceSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::choices::SuspendedSequenceChoiceState,
}

impl crate::runtime::suspension::SuspensionFrame for FinishSequenceChoiceSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareSequenceChoiceRestFinalizer {
                    state: self.state.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareSequenceChoiceRestFinalizer {
    state: SuspendedSequenceChoiceState,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareSequenceChoiceRestFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationSequenceChoiceRest"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueSequenceChoiceRestFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueSequenceChoiceRestFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected sequence-choice-rest continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationSequenceChoiceRest"),
                "unexpected sequence-choice-rest continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let state = SequenceChoiceState::from_continuation_expr(
                args.next().expect("sequence-choice rest stores its state"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("sequence-choice rest stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_sequence_choice_rest(state, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
