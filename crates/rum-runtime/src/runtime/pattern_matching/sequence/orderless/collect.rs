use super::candidates::OrderlessCandidateChoice;
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct OrderlessChoiceCollectionState {
    flat_head: Option<Expr>,
    pattern: MatcherArg,
    rest_patterns: Vec<MatcherArg>,
    choices: Vec<OrderlessCandidateChoice>,
    bindings: SymbolExprMap,
    index: usize,
    collected: Vec<SymbolExprMap>,
}

impl OrderlessChoiceCollectionState {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationOrderlessChoiceCollectionState"),
            vec![
                self.flat_head.into_continuation_expr(),
                matcher_arg_into_continuation_expr(self.pattern),
                matcher_args_into_continuation_expr(self.rest_patterns),
                Expr::call(
                    system_expr("InternalContinuationOrderlessChoices"),
                    self.choices
                        .into_iter()
                        .map(OrderlessCandidateChoice::into_continuation_expr)
                        .collect(),
                ),
                binding_map_into_continuation_expr(self.bindings),
                usize_into_continuation_expr(self.index),
                self.collected.into_continuation_expr(),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected orderless-choice-collection continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationOrderlessChoiceCollectionState"),
            "unexpected orderless-choice-collection continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let flat_head = Option::<Expr>::from_continuation_expr(
            args.next()
                .expect("orderless choice collection stores its flat head"),
        );
        let pattern = matcher_arg_from_continuation_expr(
            args.next()
                .expect("orderless choice collection stores its pattern"),
        );
        let rest_patterns = matcher_args_from_continuation_expr(
            args.next()
                .expect("orderless choice collection stores remaining patterns"),
        );
        let choices_expr = args
            .next()
            .expect("orderless choice collection stores candidate choices");
        let Expr::Call {
            head: choices_head,
            args: choices,
        } = choices_expr
        else {
            panic!("unexpected orderless choices continuation payload")
        };
        assert!(
            is_system_symbol_name(&choices_head, "InternalContinuationOrderlessChoices"),
            "unexpected orderless choices continuation tag: {choices_head:?}"
        );
        let choices = choices
            .into_iter()
            .map(OrderlessCandidateChoice::from_continuation_expr)
            .collect();
        let bindings = binding_map_from_continuation_expr(
            args.next()
                .expect("orderless choice collection stores its bindings"),
        );
        let index = usize_from_continuation_expr(
            args.next()
                .expect("orderless choice collection stores its cursor"),
        );
        let collected = Vec::<SymbolExprMap>::from_continuation_expr(
            args.next()
                .expect("orderless choice collection stores collected bindings"),
        );
        debug_assert!(args.next().is_none());
        Self {
            flat_head,
            pattern,
            rest_patterns,
            choices,
            bindings,
            index,
            collected,
        }
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedOrderlessChoiceCollection {
    flat_head: Option<Expr>,
    pattern: FrozenMatcherValue<MatcherArg>,
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    choices: FrozenMatcherSequence<OrderlessCandidateChoice>,
    bindings: FrozenMatchBindings,
    index: usize,
    collected: PersistentMatchResults,
}

impl SuspendedOrderlessChoiceCollection {
    fn freeze(state: OrderlessChoiceCollectionState) -> Self {
        Self {
            flat_head: state.flat_head,
            pattern: FrozenMatcherValue::freeze(state.pattern),
            rest_patterns: FrozenMatcherSequence::freeze(state.rest_patterns),
            choices: FrozenMatcherSequence::freeze(state.choices),
            bindings: FrozenMatchBindings::freeze(state.bindings),
            index: state.index,
            collected: PersistentMatchResults::freeze(state.collected),
        }
    }

    fn into_owned(self) -> OrderlessChoiceCollectionState {
        OrderlessChoiceCollectionState {
            flat_head: self.flat_head,
            pattern: self.pattern.into_inner(),
            rest_patterns: self.rest_patterns.into_vec(),
            choices: self.choices.into_vec(),
            bindings: self.bindings.into_owned(),
            index: self.index,
            collected: self.collected.into_vec(),
        }
    }
}

impl Session {
    pub(super) fn collect_compiled_orderless_choices_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.with_match_depth_guard(move |session| {
            session.collect_orderless_choice_matches(OrderlessChoiceCollectionState {
                flat_head: None,
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
                collected,
            })
        })
    }

    pub(super) fn collect_compiled_orderless_flat_choices_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        choices: Vec<OrderlessCandidateChoice>,
        bindings: SymbolExprMap,
        index: usize,
        collected: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.with_match_depth_guard(move |session| {
            session.collect_orderless_choice_matches(OrderlessChoiceCollectionState {
                flat_head: Some(flat_head),
                pattern,
                rest_patterns,
                choices,
                bindings,
                index,
                collected,
            })
        })
    }

    fn collect_orderless_choice_matches(
        &mut self,
        state: OrderlessChoiceCollectionState,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        let Some(choice) = state.choices.get(state.index).cloned() else {
            return Ok(state.collected);
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
            Ok(outcome) => {
                self.finish_orderless_choice_collection(state, choice.rest_values, outcome)
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceCollection::freeze(state);
                let rest_values = SharedArgs::new(choice.rest_values);
                crate::resume_or_suspend!(insert self, CollectOrderlessChoiceMatchesSuspensionFrame { state, rest_values });
                crate::resume_or_suspend!(insert self, ContinueOrderlessChoiceCollectionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_choice_collection(
        &mut self,
        mut state: OrderlessChoiceCollectionState,
        rest_values: Vec<Expr>,
        outcome: CompiledMatchOutcome,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        if !outcome.matched {
            state.index += 1;
            return self.collect_orderless_choice_matches(state);
        }

        let rest_result = if let Some(flat_head) = state.flat_head.clone() {
            self.collect_compiled_orderless_arg_matches_with_flat_head_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
                flat_head,
            )
        } else {
            self.collect_compiled_orderless_arg_matches_owned(
                state.rest_patterns.clone(),
                rest_values,
                outcome.bindings,
            )
        };
        match rest_result {
            Ok(matches) => self.finish_orderless_choice_rest_collection(state, matches),
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedOrderlessChoiceCollection::freeze(state);
                crate::resume_or_suspend!(insert self, FinishOrderlessChoiceCollectionSuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueOrderlessChoiceCollectionFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_orderless_choice_rest_collection(
        &mut self,
        mut state: OrderlessChoiceCollectionState,
        mut matches: Vec<SymbolExprMap>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        state.collected.append(&mut matches);
        state.index += 1;
        self.collect_orderless_choice_matches(state)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedOrderlessChoiceCollection>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct CollectOrderlessChoiceMatchesSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::orderless::collect::SuspendedOrderlessChoiceCollection,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for CollectOrderlessChoiceMatchesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => crate::runtime::suspension::FrameAction::finalize(
                PrepareOrderlessChoiceCollectionFinalizer {
                    state: self.state.clone(),
                    rest_values: Some(self.rest_values.clone()),
                    result,
                    kind: OrderlessChoiceCollectionContinuationKind::Pattern,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishOrderlessChoiceCollectionSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::orderless::collect::SuspendedOrderlessChoiceCollection,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishOrderlessChoiceCollectionSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => crate::runtime::suspension::FrameAction::finalize(
                PrepareOrderlessChoiceCollectionFinalizer {
                    state: self.state.clone(),
                    rest_values: None,
                    result,
                    kind: OrderlessChoiceCollectionContinuationKind::Rest,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

enum OrderlessChoiceCollectionContinuationKind {
    Pattern,
    Rest,
}

struct PrepareOrderlessChoiceCollectionFinalizer {
    state: SuspendedOrderlessChoiceCollection,
    rest_values: Option<SharedArgs>,
    result: Expr,
    kind: OrderlessChoiceCollectionContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOrderlessChoiceCollectionFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let tag = match self.kind {
            OrderlessChoiceCollectionContinuationKind::Pattern => {
                "InternalContinuationOrderlessChoiceCollectionPattern"
            }
            OrderlessChoiceCollectionContinuationKind::Rest => {
                "InternalContinuationOrderlessChoiceCollectionRest"
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
struct ContinueOrderlessChoiceCollectionFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOrderlessChoiceCollectionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected orderless-choice-collection continuation payload")
            };
            let is_pattern = is_system_symbol_name(
                &head,
                "InternalContinuationOrderlessChoiceCollectionPattern",
            );
            assert!(
                is_pattern
                    || is_system_symbol_name(
                        &head,
                        "InternalContinuationOrderlessChoiceCollectionRest"
                    ),
                "unexpected orderless-choice-collection continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let state = OrderlessChoiceCollectionState::from_continuation_expr(
                args.next()
                    .expect("orderless choice collection stores its state"),
            );
            let rest_values = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("orderless choice collection stores remaining values"),
            );
            let result = args
                .next()
                .expect("orderless choice collection stores its result");
            debug_assert!(args.next().is_none());
            if is_pattern {
                session
                    .finish_orderless_choice_collection(
                        state,
                        rest_values.expect("pattern result stores remaining values"),
                        CompiledMatchOutcome::from_continuation_expr(result),
                    )
                    .map(Vec::<SymbolExprMap>::into_continuation_expr)
            } else {
                debug_assert!(rest_values.is_none());
                session
                    .finish_orderless_choice_rest_collection(
                        state,
                        Vec::<SymbolExprMap>::from_continuation_expr(result),
                    )
                    .map(Vec::<SymbolExprMap>::into_continuation_expr)
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
