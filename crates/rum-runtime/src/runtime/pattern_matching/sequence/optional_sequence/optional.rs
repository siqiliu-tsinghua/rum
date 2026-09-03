mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct OptionalChoiceState {
    flat_head: Option<Expr>,
    spec: CompiledOptionalPattern,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    bindings: SymbolExprMap,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOptionalChoiceState {
    flat_head: Option<Expr>,
    spec: FrozenMatcherValue<CompiledOptionalPattern>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: FrozenMatchBindings,
}

impl SuspendedOptionalChoiceState {
    fn freeze(state: OptionalChoiceState) -> Self {
        Self {
            flat_head: state.flat_head,
            spec: FrozenMatcherValue::freeze(state.spec),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            bindings: FrozenMatchBindings::freeze(state.bindings),
        }
    }

    fn into_owned(self) -> OptionalChoiceState {
        OptionalChoiceState {
            flat_head: self.flat_head,
            spec: self.spec.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            bindings: self.bindings.into_owned(),
        }
    }
}

impl OptionalChoiceState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationOptionalChoiceState"),
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
        let Expr::Call { head, args } = expr else {
            panic!("unexpected optional-choice continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationOptionalChoiceState"),
            "unexpected optional-choice continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let state = Self {
            flat_head: Option::<Expr>::from_continuation_expr(
                args.next().expect("optional choice stores its flat head"),
            ),
            spec: optional_pattern_from_continuation_expr(
                args.next()
                    .expect("optional choice stores its specification"),
            ),
            rest_patterns: matcher_args_from_continuation_expr(
                args.next()
                    .expect("optional choice stores its remaining patterns"),
            ),
            value_args: Vec::<Expr>::from_continuation_expr(
                args.next().expect("optional choice stores its values"),
            ),
            bindings: binding_map_from_continuation_expr(
                args.next().expect("optional choice stores its bindings"),
            ),
        };
        debug_assert!(args.next().is_none());
        state
    }
}

struct OptionalOmissionState {
    flat_head: Option<Expr>,
    inner: MatcherArg,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    bindings: SymbolExprMap,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOptionalOmissionState {
    flat_head: Option<Expr>,
    inner: FrozenMatcherValue<MatcherArg>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: FrozenMatchBindings,
}

impl SuspendedOptionalOmissionState {
    fn freeze(state: OptionalOmissionState) -> Self {
        Self {
            flat_head: state.flat_head,
            inner: FrozenMatcherValue::freeze(state.inner),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            bindings: FrozenMatchBindings::freeze(state.bindings),
        }
    }

    fn into_owned(self) -> OptionalOmissionState {
        OptionalOmissionState {
            flat_head: self.flat_head,
            inner: self.inner.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            bindings: self.bindings.into_owned(),
        }
    }
}

impl OptionalOmissionState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationOptionalOmissionState"),
            vec![
                self.flat_head.into_continuation_expr(),
                matcher_arg_into_continuation_expr(self.inner),
                matcher_args_into_continuation_expr(self.rest_patterns),
                self.value_args.into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected optional-omission continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationOptionalOmissionState"),
            "unexpected optional-omission continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let state = Self {
            flat_head: Option::<Expr>::from_continuation_expr(
                args.next().expect("optional omission stores its flat head"),
            ),
            inner: matcher_arg_from_continuation_expr(
                args.next()
                    .expect("optional omission stores its inner matcher"),
            ),
            rest_patterns: matcher_args_from_continuation_expr(
                args.next()
                    .expect("optional omission stores its remaining patterns"),
            ),
            value_args: Vec::<Expr>::from_continuation_expr(
                args.next().expect("optional omission stores its values"),
            ),
            bindings: binding_map_from_continuation_expr(
                args.next().expect("optional omission stores its bindings"),
            ),
        };
        debug_assert!(args.next().is_none());
        state
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOptionalCollection {
    present: PersistentMatchResults,
}

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_optional_choices(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.match_optional_choices(OptionalChoiceState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_flat_optional_choices(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.match_optional_choices(OptionalChoiceState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    fn match_optional_choices(
        &mut self,
        state: OptionalChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if state.value_args.is_empty() {
            return self.match_optional_omission(state);
        }

        let present_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_flat_arg_and_rest_owned(
                flat_head,
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_arg_and_rest_owned(
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        };
        match present_result {
            Ok(outcome) => self.finish_optional_present(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOptionalChoiceState::freeze(state);
                crate::resume_or_suspend!(insert self, MatchOptionalChoicesSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOptionalPresentFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_optional_present(
        &mut self,
        state: OptionalChoiceState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if outcome.matched {
            Ok(outcome)
        } else {
            self.match_optional_omission(state)
        }
    }

    fn match_optional_omission(
        &mut self,
        state: OptionalChoiceState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if let Some(flat_head) = state.flat_head {
            self.match_compiled_flat_optional_omission_owned(
                flat_head,
                state.spec,
                state.rest_patterns,
                state.value_args,
                state.bindings,
            )
        } else {
            self.match_compiled_optional_omission_owned(
                state.spec,
                state.rest_patterns,
                state.value_args,
                state.bindings,
            )
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_optional_matches_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_optional_matches(OptionalChoiceState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_flat_optional_matches_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_optional_matches(OptionalChoiceState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            bindings,
        })
    }

    fn collect_optional_matches(
        &mut self,
        state: OptionalChoiceState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let present_result = if state.value_args.is_empty() {
            Ok(Vec::new())
        } else if let Some(flat_head) = state.flat_head.clone() {
            self.collect_compiled_flat_arg_and_rest_matches_owned(
                flat_head,
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        } else {
            self.collect_compiled_arg_and_rest_matches_owned(
                (*state.spec.inner).clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        };
        match present_result {
            Ok(present) => self.finish_optional_present_collection(state, present),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOptionalChoiceState::freeze(state);
                crate::resume_or_suspend!(insert self, CollectOptionalMatchesSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOptionalPresentCollectionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_optional_present_collection(
        &mut self,
        state: OptionalChoiceState,
        present: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let omission_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_flat_optional_omission_owned(
                flat_head,
                state.spec.clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_optional_omission_owned(
                state.spec.clone(),
                state.rest_patterns.clone(),
                state.value_args.clone(),
                state.bindings.clone(),
            )
        };
        match omission_result {
            Ok(outcome) => Ok(finish_optional_collection(present, outcome)),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOptionalCollection {
                    present: PersistentMatchResults::freeze(present),
                };
                crate::resume_or_suspend!(
                    insert self,
                    FinishOptionalPresentCollectionSuspensionFrame {
                        present: state.present,
                    },
                );
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_optional_omission_owned(
        &mut self,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let CompiledOptionalPattern { inner, default } = spec;
        self.resolve_optional_omission(
            default,
            OptionalOmissionState {
                flat_head: None,
                inner: *inner,
                rest_patterns,
                value_args,
                bindings,
            },
        )
    }

    pub(in crate::runtime::pattern_matching::sequence) fn resolve_optional_default_expr_owned(
        &mut self,
        default: CompiledOptionalDefault,
    ) -> Result<Option<Expr>, EvalError> {
        match default {
            CompiledOptionalDefault::Explicit(expr) => Ok(Some(expr)),
            CompiledOptionalDefault::UnavailableImplicit => Ok(None),
            CompiledOptionalDefault::Implicit { head, position } => {
                self.resolve_optional_implicit_default(head, position)
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_flat_optional_omission_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledOptionalPattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let CompiledOptionalPattern { inner, default } = spec;
        self.resolve_optional_omission(
            default,
            OptionalOmissionState {
                flat_head: Some(flat_head),
                inner: *inner,
                rest_patterns,
                value_args,
                bindings,
            },
        )
    }

    fn resolve_optional_omission(
        &mut self,
        default: CompiledOptionalDefault,
        state: OptionalOmissionState,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let default_result = self.resolve_optional_default_expr_owned(default);
        match default_result {
            Ok(default_expr) => self.finish_optional_default(state, default_expr),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOptionalOmissionState::freeze(state);
                crate::resume_or_suspend!(insert self, ResolveOptionalOmissionSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOptionalDefaultFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_optional_default(
        &mut self,
        state: OptionalOmissionState,
        default_expr: Option<Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(default_expr) = default_expr else {
            return Ok(match_outcome(false, state.bindings));
        };

        let default_result = if let Some(flat_head) = state.flat_head.clone() {
            self.match_compiled_flat_arg_and_rest_owned(
                flat_head,
                state.inner.clone(),
                Vec::new(),
                optional_default_values(default_expr),
                state.bindings.clone(),
            )
        } else {
            self.match_compiled_arg_and_rest_owned(
                state.inner.clone(),
                Vec::new(),
                optional_default_values(default_expr),
                state.bindings.clone(),
            )
        };
        match default_result {
            Ok(outcome) => self.finish_optional_default_match(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOptionalOmissionState::freeze(state);
                crate::resume_or_suspend!(insert self, FinishOptionalDefaultSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOptionalDefaultMatchFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_optional_default_match(
        &mut self,
        state: OptionalOmissionState,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        if let Some(flat_head) = state.flat_head {
            self.match_compiled_args_with_flat_head_owned(
                state.rest_patterns,
                state.value_args,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.match_compiled_args_owned(state.rest_patterns, state.value_args, outcome.bindings)
        }
    }
}

fn finish_optional_collection(
    mut present: Vec<SymbolExprMap>,
    outcome: CompiledMatchOutcome,
) -> Vec<SymbolExprMap> {
    if outcome.matched {
        present.push(outcome.bindings);
    }
    present
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedOptionalChoiceState>();
    assert_traces::<SuspendedOptionalCollection>();
    assert_traces::<SuspendedOptionalOmissionState>();
};
