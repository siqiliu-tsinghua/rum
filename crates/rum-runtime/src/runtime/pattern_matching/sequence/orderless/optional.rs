mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct OrderlessOptionalState {
    flat_head: Option<Expr>,
    spec: CompiledOptionalPattern,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    bindings: SymbolExprMap,
}

impl OrderlessOptionalState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationOrderlessOptionalState"),
            vec![
                self.flat_head.into_continuation_expr(),
                optional_pattern_into_continuation_expr(self.spec),
                matcher_args_into_continuation_expr(self.rest_patterns),
                self.value_args.into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args, .. } = expr else {
            panic!("unexpected orderless-optional continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationOrderlessOptionalState"),
            "unexpected orderless-optional continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let state = Self {
            flat_head: Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("orderless optional stores its flat head"),
            ),
            spec: optional_pattern_from_continuation_expr(
                args.next()
                    .expect("orderless optional stores its specification"),
            ),
            rest_patterns: matcher_args_from_continuation_expr(
                args.next()
                    .expect("orderless optional stores remaining patterns"),
            ),
            value_args: Vec::<Expr>::from_continuation_expr(
                args.next().expect("orderless optional stores its values"),
            ),
            bindings: binding_map_from_continuation_expr(
                args.next().expect("orderless optional stores its bindings"),
            ),
        };
        debug_assert!(args.next().is_none());
        state
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOrderlessOptionalState {
    flat_head: Option<Expr>,
    spec: FrozenMatcherValue<CompiledOptionalPattern>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: FrozenMatchBindings,
}

impl SuspendedOrderlessOptionalState {
    fn freeze(state: OrderlessOptionalState) -> Self {
        Self {
            flat_head: state.flat_head,
            spec: FrozenMatcherValue::freeze(state.spec),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            bindings: FrozenMatchBindings::freeze(state.bindings),
        }
    }

    fn into_owned(self) -> OrderlessOptionalState {
        OrderlessOptionalState {
            flat_head: self.flat_head,
            spec: self.spec.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            bindings: self.bindings.into_owned(),
        }
    }
}

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_optional_choices(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.match_orderless_optional_choices(OrderlessOptionalState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_optional_choices(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.match_orderless_optional_choices(OrderlessOptionalState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    fn match_orderless_optional_choices(
        &mut self,
        state: OrderlessOptionalState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if state.value_args.is_empty() {
            return self.match_orderless_optional_omission(state);
        }

        let present_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_orderless_flat_arg_choices_owned(
                flat_head,
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_orderless_arg_choices_owned(
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        };
        match present_result {
            Ok(outcome) => self.finish_orderless_optional_present(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessOptionalState::freeze(state);
                crate::resume_or_suspend!(insert self, MatchOrderlessOptionalChoicesSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessOptionalFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_optional_present(
        &mut self,
        state: OrderlessOptionalState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.match_orderless_optional_omission(state)
        }
    }

    fn match_orderless_optional_omission(
        &mut self,
        state: OrderlessOptionalState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let default_result = self.resolve_optional_default_expr_owned(state.spec.default.clone());
        match default_result {
            Ok(default_expr) => self.finish_orderless_optional_default(state, default_expr),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessOptionalState::freeze(state);
                crate::resume_or_suspend!(insert self, MatchOrderlessOptionalOmissionSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessOptionalFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_optional_default(
        &mut self,
        state: OrderlessOptionalState,
        default_expr: Option<Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(default_expr) = default_expr else {
            return Ok(match_outcome(false, state.bindings));
        };

        let default_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_orderless_flat_selected_values_owned(
                flat_head,
                (*state.spec.inner).clone(),
                optional_default_values(default_expr),
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_orderless_selected_values_owned(
                (*state.spec.inner).clone(),
                optional_default_values(default_expr),
                state.bindings.clone(),
            )
        };
        match default_result {
            Ok(outcome) => self.finish_orderless_optional_default_match(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessOptionalState::freeze(state);
                crate::resume_or_suspend!(insert self, FinishOrderlessOptionalDefaultSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessOptionalFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_optional_default_match(
        &mut self,
        state: OrderlessOptionalState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        if let Some(flat_head) = state.flat_head {
            self.match_compiled_orderless_args_with_flat_head_owned(
                state.rest_patterns,
                state.value_args,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.match_compiled_orderless_args_owned(
                state.rest_patterns,
                state.value_args,
                outcome.bindings,
            )
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_optional_matches_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_orderless_optional_matches(OrderlessOptionalState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_flat_optional_matches_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_orderless_optional_matches(OrderlessOptionalState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    fn collect_orderless_optional_matches(
        &mut self,
        state: OrderlessOptionalState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let present_result = if state.value_args.is_empty() {
            Ok(Vec::new())
        } else if let Some(flat_head) = state.flat_head.clone() {
            self.collect_compiled_orderless_flat_arg_choice_matches_owned(
                flat_head,
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        } else {
            self.collect_compiled_orderless_arg_choice_matches_owned(
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        };
        match present_result {
            Ok(present) => self.finish_orderless_optional_present_collection(state, present),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessOptionalState::freeze(state);
                crate::resume_or_suspend!(insert self, CollectOrderlessOptionalMatchesSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessOptionalFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_optional_present_collection(
        &mut self,
        state: OrderlessOptionalState,
        present: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let omission_result = self.collect_orderless_optional_omission(state);
        match omission_result {
            Ok(omitted) => Ok(append_match_results(present, omitted)),
            Err(EvalError::InputRequired { request }) => {
                let present = PersistentMatchResults::freeze(present);
                crate::resume_or_suspend!(insert self, FinishOrderlessOptionalPresentCollectionSuspensionFrame { present });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn collect_orderless_optional_omission(
        &mut self,
        state: OrderlessOptionalState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let default_result = self.resolve_optional_default_expr_owned(state.spec.default.clone());
        match default_result {
            Ok(default_expr) => {
                self.finish_orderless_optional_collection_default(state, default_expr)
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessOptionalState::freeze(state);
                crate::resume_or_suspend!(insert self, CollectOrderlessOptionalOmissionSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessOptionalFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_optional_collection_default(
        &mut self,
        state: OrderlessOptionalState,
        default_expr: Option<Expr>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let Some(default_expr) = default_expr else {
            return Ok(Vec::new());
        };

        let default_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_orderless_flat_selected_values_owned(
                flat_head,
                (*state.spec.inner).clone(),
                optional_default_values(default_expr),
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_orderless_selected_values_owned(
                (*state.spec.inner).clone(),
                optional_default_values(default_expr),
                state.bindings.clone(),
            )
        };
        match default_result {
            Ok(outcome) => self.finish_orderless_optional_collection_match(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessOptionalState::freeze(state);
                crate::resume_or_suspend!(insert self, FinishOrderlessOptionalCollectionDefaultSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessOptionalFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_optional_collection_match(
        &mut self,
        state: OrderlessOptionalState,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            return Ok(Vec::new());
        }

        if let Some(flat_head) = state.flat_head {
            self.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                state.rest_patterns,
                state.value_args,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.collect_compiled_orderless_arg_matches_owned(
                state.rest_patterns,
                state.value_args,
                outcome.bindings,
            )
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_optional_omission_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.match_orderless_optional_omission(OrderlessOptionalState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_optional_omission_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.match_orderless_optional_omission(OrderlessOptionalState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }
}

fn append_match_results(
    mut first: Vec<SymbolExprMap>,
    mut second: Vec<SymbolExprMap>,
) -> Vec<SymbolExprMap> {
    first.append(&mut second);
    first
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedOrderlessOptionalState>();
};
