use super::candidates::OrderlessCandidateChoice;
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

const MAX_MATCH_DEPTH: usize = 512;

struct OrderlessChoiceState {
    flat_head: Option<Expr>,
    pattern: MatcherArg,
    rest_patterns: Vec<MatcherArg>,
    choices: Vec<OrderlessCandidateChoice>,
    bindings: SymbolExprMap,
    index: usize,
}

impl OrderlessChoiceState {
    fn into_continuation_expr(self) -> Expr {
        let choices = self
            .choices
            .into_iter()
            .map(|choice| {
                Expr::call(
                    system_expr("InternalContinuationOrderlessCandidateChoice"),
                    vec![
                        choice.selected_values.into_continuation_expr(),
                        choice.rest_values.into_continuation_expr(),
                    ],
                )
            })
            .collect();
        Expr::call(
            system_expr("InternalContinuationOrderlessChoiceState"),
            vec![
                self.flat_head.into_continuation_expr(),
                matcher_arg_into_continuation_expr(self.pattern),
                matcher_args_into_continuation_expr(self.rest_patterns),
                Expr::call(system_expr("InternalContinuationOrderlessChoices"), choices),
                binding_map_into_continuation_expr(self.bindings),
                usize_into_continuation_expr(self.index),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected orderless-choice continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationOrderlessChoiceState"),
            "unexpected orderless-choice continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let flat_head = Option::<Expr>::from_continuation_expr(
            args.next().expect("orderless choice stores its flat head"),
        );
        let pattern = matcher_arg_from_continuation_expr(
            args.next().expect("orderless choice stores its pattern"),
        );
        let rest_patterns = matcher_args_from_continuation_expr(
            args.next()
                .expect("orderless choice stores its remaining patterns"),
        );
        let choices_expr = args.next().expect("orderless choice stores its candidates");
        let Expr::Call {
            head: choices_head,
            args: choices,
        } = choices_expr
        else {
            panic!("unexpected orderless-candidate continuation payload")
        };
        assert!(
            is_system_symbol_name(&choices_head, "InternalContinuationOrderlessChoices"),
            "unexpected orderless-candidate continuation tag: {choices_head:?}"
        );
        let choices = choices
            .into_iter()
            .map(|choice| {
                let Expr::Call { head, args } = choice else {
                    panic!("unexpected orderless-candidate continuation entry")
                };
                assert!(
                    is_system_symbol_name(&head, "InternalContinuationOrderlessCandidateChoice"),
                    "unexpected orderless-candidate entry tag: {head:?}"
                );
                let mut args = args.into_iter();
                let selected_values = Vec::<Expr>::from_continuation_expr(
                    args.next()
                        .expect("orderless candidate stores selected values"),
                );
                let rest_values = Vec::<Expr>::from_continuation_expr(
                    args.next().expect("orderless candidate stores rest values"),
                );
                debug_assert!(args.next().is_none());
                OrderlessCandidateChoice {
                    selected_values,
                    rest_values,
                }
            })
            .collect();
        let bindings = binding_map_from_continuation_expr(
            args.next().expect("orderless choice stores its bindings"),
        );
        let index =
            usize_from_continuation_expr(args.next().expect("orderless choice stores its cursor"));
        debug_assert!(args.next().is_none());
        Self {
            flat_head,
            pattern,
            rest_patterns,
            choices,
            bindings,
            index,
        }
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOrderlessChoiceState {
    flat_head: Option<Expr>,
    pattern: FrozenMatcherValue<MatcherArg>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    choices: FrozenMatcherSequence<OrderlessCandidateChoice>,
    bindings: FrozenMatchBindings,
    index: usize,
}

impl SuspendedOrderlessChoiceState {
    fn freeze(state: OrderlessChoiceState) -> Self {
        Self {
            flat_head: state.flat_head,
            pattern: FrozenMatcherValue::freeze(state.pattern),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            choices: FrozenMatcherSequence::freeze(state.choices),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            index: state.index,
        }
    }

    fn into_owned(self) -> OrderlessChoiceState {
        OrderlessChoiceState {
            flat_head: self.flat_head,
            pattern: self.pattern.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            choices: self.choices.into_vec(),
            bindings: self.bindings.into_owned(),
            index: self.index,
        }
    }
}

impl Session {
    pub(in crate::runtime::pattern_matching::sequence::orderless) fn with_match_depth_guard<T>(
        &mut self,
        body: impl FnOnce(&mut Session) -> Result<T, EvalError>,
    ) -> Result<T, EvalError> {
        if self.match_depth >= MAX_MATCH_DEPTH {
            return Err(EvalError::MatchRecursionLimitExceeded {
                limit: MAX_MATCH_DEPTH,
            });
        }

        self.match_depth += 1;
        let result = body(self);
        self.match_depth -= 1;
        result
    }

    pub(super) fn match_compiled_orderless_choices_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.with_match_depth_guard(move |session| {
            session.match_orderless_choice(OrderlessChoiceState {
                flat_head: None,
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
            })
        })
    }

    pub(super) fn match_compiled_orderless_flat_choices_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.with_match_depth_guard(move |session| {
            session.match_orderless_choice(OrderlessChoiceState {
                flat_head: Some(flat_head),
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
            })
        })
    }

    fn match_orderless_choice(
        &mut self,
        state: OrderlessChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(choice) = state.choices.get(state.index).cloned() else {
            return Ok(match_outcome(false, state.bindings));
        };

        let match_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_orderless_flat_selected_values_owned(
                flat_head,
                state.pattern.clone(),
                choice.selected_values,
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_orderless_selected_values_owned(
                state.pattern.clone(),
                choice.selected_values,
                state.bindings.clone(),
            )
        };
        match match_result {
            Ok(outcome) => self.finish_orderless_choice_pattern(state, choice.rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceState::freeze(state);
                let rest_values = SharedArgs::new(choice.rest_values);
                crate::resume_or_suspend!(insert self, MatchOrderlessChoiceSuspensionFrame { state, rest_values });
                crate::resume_or_suspend!(insert self, ContinueOrderlessChoicePatternFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_choice_pattern(
        &mut self,
        state: OrderlessChoiceState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return self.retry_orderless_choice(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_orderless_args_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.match_compiled_orderless_args_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(outcome) => self.finish_orderless_choice_rest(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceState::freeze(state);
                crate::resume_or_suspend!(insert self, FinishOrderlessChoicePatternSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessChoiceRestFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_choice_rest(
        &mut self,
        state: OrderlessChoiceState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.retry_orderless_choice(state)
        }
    }

    fn retry_orderless_choice(
        &mut self,
        state: OrderlessChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if let Some(flat_head) = state.flat_head {
            self.match_compiled_orderless_flat_choices_owned(
                flat_head,
                state.pattern,
                state.rest_patterns,
                state.choices,
                state.bindings,
                state.index + 1,
            )
        } else {
            self.match_compiled_orderless_choices_owned(
                state.pattern,
                state.rest_patterns,
                state.choices,
                state.bindings,
                state.index + 1,
            )
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedOrderlessChoiceState>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct MatchOrderlessChoiceSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::orderless::match_choices::SuspendedOrderlessChoiceState,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for MatchOrderlessChoiceSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareOrderlessChoicePatternFinalizer {
                    state: self.state.clone(),
                    rest_values: self.rest_values.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareOrderlessChoicePatternFinalizer {
    state: SuspendedOrderlessChoiceState,
    rest_values: SharedArgs,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOrderlessChoicePatternFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationOrderlessChoicePattern"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.rest_values.into_vec().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueOrderlessChoicePatternFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOrderlessChoicePatternFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected orderless-choice-pattern continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationOrderlessChoicePattern"),
                "unexpected orderless-choice-pattern continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let state = OrderlessChoiceState::from_continuation_expr(
                args.next()
                    .expect("orderless choice pattern stores its state"),
            );
            let rest_values = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("orderless choice pattern stores its remaining values"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("orderless choice pattern stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_orderless_choice_pattern(state, rest_values, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishOrderlessChoicePatternSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::orderless::match_choices::SuspendedOrderlessChoiceState,
}

impl crate::runtime::suspension::SuspensionFrame for FinishOrderlessChoicePatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareOrderlessChoiceRestFinalizer {
                    state: self.state.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareOrderlessChoiceRestFinalizer {
    state: SuspendedOrderlessChoiceState,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOrderlessChoiceRestFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationOrderlessChoiceRest"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueOrderlessChoiceRestFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOrderlessChoiceRestFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected orderless-choice-rest continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationOrderlessChoiceRest"),
                "unexpected orderless-choice-rest continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let state = OrderlessChoiceState::from_continuation_expr(
                args.next().expect("orderless choice rest stores its state"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("orderless choice rest stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_orderless_choice_rest(state, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
