use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledArgAndRestOwnedSuspensionFrame {
    pub(super) rest_patterns:
        crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
            crate::runtime::session::session_state::MatcherArg,
        >,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledArgAndRestOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(payload) => CompiledMatchOutcome::from_continuation_expr(payload),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        if !outcome.matched {
            return crate::runtime::suspension::FrameAction::complete(Ok(
                outcome.into_continuation_expr()
            ));
        }
        if self.rest_patterns.as_slice().is_empty() {
            return crate::runtime::suspension::FrameAction::complete(Ok(match_outcome(
                self.rest_values.is_empty(),
                outcome.bindings,
            )
            .into_continuation_expr()));
        }
        crate::runtime::suspension::FrameAction::finalize(PrepareArgAndRestFinalizer {
            rest_patterns: self.rest_patterns.clone(),
            rest_values: self.rest_values.clone(),
            bindings: outcome.bindings,
        })
    }
}

struct PrepareArgAndRestFinalizer {
    rest_patterns: FrozenMatcherSequence<MatcherArg>,
    rest_values: SharedArgs,
    bindings: SymbolExprMap,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareArgAndRestFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationCompiledArgAndRest"),
            vec![
                matcher_args_into_continuation_expr(self.rest_patterns.into_vec()),
                self.rest_values.into_vec().into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueMatchCompiledArgAndRestFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueMatchCompiledArgAndRestFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Some((rest_patterns, rest_values, bindings)) =
                compiled_arg_and_rest_payload(payload.clone())
            else {
                return Ok(payload);
            };
            session
                .match_compiled_args_owned(rest_patterns, rest_values, bindings)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchFlatSingleChoiceSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::SuspendedFlatSingleChoiceState,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for MatchFlatSingleChoiceSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareFlatSinglePatternFinalizer {
                    state: self.state.clone(),
                    rest_values: self.rest_values.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareFlatSinglePatternFinalizer {
    state: SuspendedFlatSingleChoiceState,
    rest_values: SharedArgs,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareFlatSinglePatternFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationFlatSinglePattern"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.rest_values.into_vec().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueFlatSinglePatternFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueFlatSinglePatternFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args = sequence_frame_payload(payload, "InternalContinuationFlatSinglePattern");
            let state = FlatSingleChoiceState::from_continuation_expr(
                args.next().expect("flat-single pattern stores its state"),
            );
            let rest_values = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("flat-single pattern stores its remaining values"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next().expect("flat-single pattern stores its outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_flat_single_pattern(state, rest_values, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishFlatSinglePatternSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::sequence::SuspendedFlatSingleChoiceState,
}

impl crate::runtime::suspension::SuspensionFrame for FinishFlatSinglePatternSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareFlatSingleRestFinalizer {
                    state: self.state.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareFlatSingleRestFinalizer {
    state: SuspendedFlatSingleChoiceState,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareFlatSingleRestFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationFlatSingleRest"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueFlatSingleRestFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueFlatSingleRestFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args = sequence_frame_payload(payload, "InternalContinuationFlatSingleRest");
            let state = FlatSingleChoiceState::from_continuation_expr(
                args.next().expect("flat-single rest stores its state"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next().expect("flat-single rest stores its outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_flat_single_rest(state, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectCompiledArgAndRestMatchesOwnedSuspensionFrame {
    pub(super) rest_patterns:
        crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
            crate::runtime::session::session_state::MatcherArg,
        >,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectCompiledArgAndRestMatchesOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let outcome = match outcome {
            Ok(payload) => CompiledMatchOutcome::from_continuation_expr(payload),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        if !outcome.matched {
            return crate::runtime::suspension::FrameAction::complete(Ok(
                Vec::<SymbolExprMap>::new().into_continuation_expr(),
            ));
        }
        if self.rest_patterns.as_slice().is_empty() {
            return crate::runtime::suspension::FrameAction::complete(Ok(if self
                .rest_values
                .is_empty()
            {
                vec![outcome.bindings]
            } else {
                Vec::new()
            }
            .into_continuation_expr()));
        }
        crate::runtime::suspension::FrameAction::finalize(PrepareArgAndRestFinalizer {
            rest_patterns: self.rest_patterns.clone(),
            rest_values: self.rest_values.clone(),
            bindings: outcome.bindings,
        })
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCollectCompiledArgAndRestFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCollectCompiledArgAndRestFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Some((rest_patterns, rest_values, bindings)) =
                compiled_arg_and_rest_payload(payload.clone())
            else {
                return Ok(payload);
            };
            session
                .collect_compiled_args_matches_owned(rest_patterns, rest_values, bindings)
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectFlatSingleChoiceSuspensionFrame {
    pub(super) state:
        crate::runtime::pattern_matching::sequence::SuspendedFlatSingleCollectionState,
    pub(super) rest_values: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for CollectFlatSingleChoiceSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareFlatSingleCollectionPatternFinalizer {
                    state: self.state.clone(),
                    rest_values: self.rest_values.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareFlatSingleCollectionPatternFinalizer {
    state: SuspendedFlatSingleCollectionState,
    rest_values: SharedArgs,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer
    for PrepareFlatSingleCollectionPatternFinalizer
{
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationFlatSingleCollectionPattern"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.rest_values.into_vec().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueFlatSingleCollectionPatternFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueFlatSingleCollectionPatternFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args =
                sequence_frame_payload(payload, "InternalContinuationFlatSingleCollectionPattern");
            let state = FlatSingleCollectionState::from_continuation_expr(
                args.next()
                    .expect("flat-single collection pattern stores its state"),
            );
            let rest_values = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("flat-single collection pattern stores its remaining values"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("flat-single collection pattern stores its outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_flat_single_collection_pattern(state, rest_values, match_outcome)
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishFlatSingleCollectionPatternSuspensionFrame {
    pub(super) state:
        crate::runtime::pattern_matching::sequence::SuspendedFlatSingleCollectionState,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishFlatSingleCollectionPatternSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareFlatSingleCollectionRestFinalizer {
                    state: self.state.clone(),
                    matches: Vec::<SymbolExprMap>::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareFlatSingleCollectionRestFinalizer {
    state: SuspendedFlatSingleCollectionState,
    matches: Vec<SymbolExprMap>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareFlatSingleCollectionRestFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationFlatSingleCollectionRest"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.matches.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueFlatSingleCollectionRestFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueFlatSingleCollectionRestFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args =
                sequence_frame_payload(payload, "InternalContinuationFlatSingleCollectionRest");
            let state = FlatSingleCollectionState::from_continuation_expr(
                args.next()
                    .expect("flat-single collection rest stores its state"),
            );
            let matches = Vec::<SymbolExprMap>::from_continuation_expr(
                args.next()
                    .expect("flat-single collection rest stores its matches"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_flat_single_collection_rest(state, matches)
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

fn compiled_arg_and_rest_payload(
    payload: Expr,
) -> Option<(Vec<MatcherArg>, Vec<Expr>, SymbolExprMap)> {
    let Expr::Call { head, args, .. } = payload else {
        return None;
    };
    if !is_system_symbol_name(&head, "InternalContinuationCompiledArgAndRest") {
        return None;
    }
    let mut args = args.into_iter();
    let rest_patterns = matcher_args_from_continuation_expr(
        args.next()
            .expect("argument-and-rest continuation stores its remaining patterns"),
    );
    let rest_values = Vec::<Expr>::from_continuation_expr(
        args.next()
            .expect("argument-and-rest continuation stores its remaining values"),
    );
    let bindings = binding_map_from_continuation_expr(
        args.next()
            .expect("argument-and-rest continuation stores its bindings"),
    );
    debug_assert!(args.next().is_none());
    Some((rest_patterns, rest_values, bindings))
}

fn sequence_frame_payload(payload: Expr, tag: &'static str) -> std::vec::IntoIter<Expr> {
    let Expr::Call { head, args, .. } = payload else {
        panic!("unexpected {tag} continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, tag),
        "unexpected {tag} continuation tag: {head:?}"
    );
    args.into_iter()
}
