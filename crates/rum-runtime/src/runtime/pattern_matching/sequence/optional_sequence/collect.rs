use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct SequenceChoiceCollectionState {
    flat_head: Option<Expr>,
    spec: CompiledSequencePattern,
    rest_patterns: Vec<MatcherArg>,
    value_args: Vec<Expr>,
    bindings: SymbolExprMap,
    consumes: Vec<usize>,
    index: usize,
    collected: Vec<SymbolExprMap>,
}

impl SequenceChoiceCollectionState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationSequenceChoiceCollectionState"),
            vec![
                self.flat_head.into_continuation_expr(),
                sequence_pattern_into_continuation_expr(self.spec),
                matcher_args_into_continuation_expr(self.rest_patterns),
                self.value_args.into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings),
                Expr::call(
                    system_expr("InternalContinuationUsizeVec"),
                    self.consumes
                        .into_iter()
                        .map(usize_into_continuation_expr)
                        .collect(),
                ),
                usize_into_continuation_expr(self.index),
                self.collected.into_continuation_expr(),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected sequence-choice-collection continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationSequenceChoiceCollectionState"),
            "unexpected sequence-choice-collection continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let flat_head = Option::<Expr>::from_continuation_expr(
            args.next()
                .expect("sequence choice collection stores its flat head"),
        );
        let spec = sequence_pattern_from_continuation_expr(
            args.next()
                .expect("sequence choice collection stores its specification"),
        );
        let rest_patterns = matcher_args_from_continuation_expr(
            args.next()
                .expect("sequence choice collection stores remaining patterns"),
        );
        let value_args = Vec::<Expr>::from_continuation_expr(
            args.next()
                .expect("sequence choice collection stores its values"),
        );
        let bindings = binding_map_from_continuation_expr(
            args.next()
                .expect("sequence choice collection stores its bindings"),
        );
        let consumes_expr = args
            .next()
            .expect("sequence choice collection stores candidate lengths");
        let Expr::Call {
            head: consumes_head,
            args: consumes,
        } = consumes_expr
        else {
            panic!("unexpected sequence candidate-length continuation payload")
        };
        assert!(
            is_system_symbol_name(&consumes_head, "InternalContinuationUsizeVec"),
            "unexpected sequence candidate-length tag: {consumes_head:?}"
        );
        let consumes = consumes
            .into_iter()
            .map(usize_from_continuation_expr)
            .collect();
        let index = usize_from_continuation_expr(
            args.next()
                .expect("sequence choice collection stores its cursor"),
        );
        let collected = Vec::<SymbolExprMap>::from_continuation_expr(
            args.next()
                .expect("sequence choice collection stores collected bindings"),
        );
        debug_assert!(args.next().is_none());
        Self {
            flat_head,
            spec,
            rest_patterns,
            value_args,
            bindings,
            consumes,
            index,
            collected,
        }
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSequenceChoiceCollection {
    flat_head: Option<Expr>,
    spec: FrozenMatcherValue<CompiledSequencePattern>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: FrozenMatchBindings,
    consumes: FrozenMatcherSequence<usize>,
    index: usize,
    collected: PersistentMatchResults,
}

impl SuspendedSequenceChoiceCollection {
    fn freeze(state: SequenceChoiceCollectionState) -> Self {
        Self {
            flat_head: state.flat_head,
            spec: FrozenMatcherValue::freeze(state.spec),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            value_args: SharedArgs::new(state.value_args),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            consumes: FrozenMatcherSequence::freeze(state.consumes),
            index: state.index,
            collected: PersistentMatchResults::freeze(state.collected),
        }
    }

    fn into_owned(self) -> SequenceChoiceCollectionState {
        SequenceChoiceCollectionState {
            flat_head: self.flat_head,
            spec: self.spec.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            value_args: self.value_args.into_vec(),
            bindings: self.bindings.into_owned(),
            consumes: self.consumes.into_vec(),
            index: self.index,
            collected: self.collected.into_vec(),
        }
    }
}

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_sequence_choice_matches_owned(
        &mut self,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consumes: Vec<usize>,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_sequence_choice_matches(SequenceChoiceCollectionState {
            flat_head: None,
            spec,
            rest_patterns,
            value_args,
            bindings,
            consumes,
            index,
            collected,
        })
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_flat_sequence_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        spec: CompiledSequencePattern,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: SymbolExprMap,
        consumes: Vec<usize>,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.collect_sequence_choice_matches(SequenceChoiceCollectionState {
            flat_head: Some(flat_head),
            spec,
            rest_patterns,
            value_args,
            bindings,
            consumes,
            index,
            collected,
        })
    }

    fn collect_sequence_choice_matches(
        &mut self,
        state: SequenceChoiceCollectionState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let Some(consume) = state.consumes.get(state.index).copied() else {
            return Ok(state.collected);
        };

        let consumed_values = state
            .value_args
            .iter()
            .take(consume)
            .cloned()
            .collect::<Vec<_>>();
        let rest_values = state
            .value_args
            .iter()
            .skip(consume)
            .cloned()
            .collect::<Vec<_>>();
        let sequence_result = self.match_compiled_sequence_pattern_owned(
            state.spec.clone(),
            consumed_values,
            state.bindings.clone(),
        );
        match sequence_result {
            Ok(outcome) => self.finish_sequence_choice_collection(state, rest_values, outcome),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceCollection::freeze(state);
                let rest_values = SharedArgs::new(rest_values);
                crate::resume_or_suspend!(insert self, CollectSequenceChoiceMatchesSuspensionFrame { state, rest_values });
                crate::resume_or_suspend!(insert self, ContinueSequenceChoiceCollectionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice_collection(
        &mut self,
        mut state: SequenceChoiceCollectionState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            state.index += 1;
            return self.collect_sequence_choice_matches(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.collect_compiled_args_matches_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.collect_compiled_args_matches_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(matches) => self.finish_sequence_choice_rest_collection(state, matches),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSequenceChoiceCollection::freeze(state);
                crate::resume_or_suspend!(insert self, FinishSequenceChoiceCollectionSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueSequenceChoiceCollectionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_sequence_choice_rest_collection(
        &mut self,
        mut state: SequenceChoiceCollectionState,
        mut matches: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        state.collected.append(&mut matches);
        state.index += 1;
        self.collect_sequence_choice_matches(state)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedSequenceChoiceCollection>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct CollectSequenceChoiceMatchesSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::collect::SuspendedSequenceChoiceCollection,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for CollectSequenceChoiceMatchesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => crate::runtime::suspension::FrameAction::finalize(
                PrepareSequenceChoiceCollectionFinalizer {
                    state: self.state.clone(),
                    rest_values: Some(self.rest_values.clone()),
                    result,
                    kind: SequenceChoiceCollectionContinuationKind::Pattern,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishSequenceChoiceCollectionSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::collect::SuspendedSequenceChoiceCollection,
}

impl crate::runtime::suspension::SuspensionFrame for FinishSequenceChoiceCollectionSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => crate::runtime::suspension::FrameAction::finalize(
                PrepareSequenceChoiceCollectionFinalizer {
                    state: self.state.clone(),
                    rest_values: None,
                    result,
                    kind: SequenceChoiceCollectionContinuationKind::Rest,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

enum SequenceChoiceCollectionContinuationKind {
    Pattern,
    Rest,
}

struct PrepareSequenceChoiceCollectionFinalizer {
    state: SuspendedSequenceChoiceCollection,
    rest_values: Option<SharedArgs>,
    result: Expr,
    kind: SequenceChoiceCollectionContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareSequenceChoiceCollectionFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let tag = match self.kind {
            SequenceChoiceCollectionContinuationKind::Pattern => {
                "InternalContinuationSequenceChoiceCollectionPattern"
            }
            SequenceChoiceCollectionContinuationKind::Rest => {
                "InternalContinuationSequenceChoiceCollectionRest"
            }
        };
        Ok(Expr::call(
            system_expr(tag),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.rest_values
                    .map(SharedArgs::into_vec)
                    .into_continuation_expr(),
                self.result,
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueSequenceChoiceCollectionFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueSequenceChoiceCollectionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected sequence-choice-collection continuation payload")
            };
            let is_pattern =
                is_system_symbol_name(&head, "InternalContinuationSequenceChoiceCollectionPattern");
            assert!(
                is_pattern
                    || is_system_symbol_name(
                        &head,
                        "InternalContinuationSequenceChoiceCollectionRest"
                    ),
                "unexpected sequence-choice-collection continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let state = SequenceChoiceCollectionState::from_continuation_expr(
                args.next()
                    .expect("sequence choice collection stores its state"),
            );
            let rest_values = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("sequence choice collection stores remaining values"),
            );
            let result = args
                .next()
                .expect("sequence choice collection stores its result");
            debug_assert!(args.next().is_none());
            if is_pattern {
                session
                    .finish_sequence_choice_collection(
                        state,
                        rest_values.expect("pattern result stores remaining values"),
                        CompiledMatchOutcome::from_continuation_expr(result),
                    )
                    .map(Vec::<SymbolExprMap>::into_continuation_expr)
            } else {
                debug_assert!(rest_values.is_none());
                session
                    .finish_sequence_choice_rest_collection(
                        state,
                        Vec::<SymbolExprMap>::from_continuation_expr(result),
                    )
                    .map(Vec::<SymbolExprMap>::into_continuation_expr)
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
