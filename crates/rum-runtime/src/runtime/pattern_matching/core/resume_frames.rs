use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledOwnedSuspensionFrame {
    pub(super) snapshot: crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
        (rum_core::Symbol, std::option::Option<rum_core::Expr>),
    >,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(value) => crate::runtime::suspension::FrameAction::finalize(
                FinishSuspendedCompiledMatchFinalizer {
                    snapshot: self.snapshot.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(value),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluatePatternTestOwnedSuspensionFrame {
    pub(super) bindings_for_true:
        crate::runtime::pattern_matching::suspended_state::FrozenMatchBindings,
    pub(super) bindings_for_false:
        crate::runtime::pattern_matching::suspended_state::FrozenMatchBindings,
}

impl crate::runtime::suspension::SuspensionFrame for EvaluatePatternTestOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(FinishPatternTestFinalizer {
                    matched: is_system_symbol_name(&result, "True"),
                    bindings_for_true: self.bindings_for_true.clone(),
                    bindings_for_false: self.bindings_for_false.clone(),
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct FinishSuspendedCompiledMatchFinalizer {
    snapshot: FrozenMatcherSequence<(Symbol, Option<Expr>)>,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishSuspendedCompiledMatchFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(finish_suspended_compiled_match(self.snapshot, self.outcome).into_continuation_expr())
    }
}

struct FinishPatternTestFinalizer {
    matched: bool,
    bindings_for_true: FrozenMatchBindings,
    bindings_for_false: FrozenMatchBindings,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishPatternTestFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let bindings = if self.matched {
            self.bindings_for_true.into_owned()
        } else {
            self.bindings_for_false.into_owned()
        };
        Ok(match_outcome(self.matched, bindings).into_continuation_expr())
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledAlternativesOwnedSuspensionFrame {
    pub(super) alternatives:
        crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
            crate::runtime::session::session_state::MatcherNode,
        >,
    pub(super) value_for_retry: rum_core::Expr,
    pub(super) index: usize,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledAlternativesOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(match_outcome) => crate::runtime::suspension::FrameAction::finalize(
                PrepareCompiledAlternativeFinalizer {
                    alternatives: self.alternatives.clone(),
                    value_for_retry: self.value_for_retry.clone(),
                    index: self.index,
                    match_outcome,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareCompiledAlternativeFinalizer {
    alternatives: FrozenMatcherSequence<MatcherNode>,
    value_for_retry: Expr,
    index: usize,
    match_outcome: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareCompiledAlternativeFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationCompiledAlternative"),
            vec![
                matcher_args_into_continuation_expr(
                    self.alternatives
                        .into_vec()
                        .into_iter()
                        .map(MatcherArg::Single)
                        .collect(),
                ),
                self.value_for_retry,
                usize_into_continuation_expr(self.index),
                self.match_outcome,
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCompiledAlternativeFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCompiledAlternativeFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected compiled-alternative continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationCompiledAlternative"),
                "unexpected compiled-alternative continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let alternatives = matcher_args_from_continuation_expr(
                args.next()
                    .expect("compiled alternative stores its alternatives"),
            )
            .into_iter()
            .map(|arg| match arg {
                MatcherArg::Single(matcher) => matcher,
                _ => panic!("compiled alternative stores only matcher nodes"),
            })
            .collect();
            let value_for_retry = args
                .next()
                .expect("compiled alternative stores its retry value");
            let index = usize_from_continuation_expr(
                args.next().expect("compiled alternative stores its cursor"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("compiled alternative stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_compiled_alternative(alternatives, value_for_retry, index, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledNamedOwnedSuspensionFrame {
    pub(super) name: rum_core::Symbol,
    pub(super) value: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledNamedOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut name = self.name.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<CompiledMatchOutcome, EvalError> {
            if !outcome.matched {
                return Ok(outcome);
            }

            let mut bindings = outcome.bindings;
            Ok(match_outcome(
                bind_pattern_name(&name, value.clone(), &mut bindings),
                bindings,
            ))
        })()
                    .map(|value| <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledConditionOwnedSuspensionFrame {
    pub(super) condition: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledConditionOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut condition = self.condition.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<CompiledMatchOutcome, EvalError> {
            if !outcome.matched {
                return Ok(outcome);
            }

            let snapshot = outcome.bindings.clone();
            let bindings = outcome.bindings;
            session.evaluate_guard_owned(condition, bindings, snapshot, None)
        })()
                    .map(|value| <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledPatternTestOwnedSuspensionFrame {
    pub(super) test: rum_core::Expr,
    pub(super) value: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledPatternTestOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut test = self.test.clone();
        #[allow(unused_mut)]
        let mut value = self.value.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<CompiledMatchOutcome, EvalError> {
            if !outcome.matched {
                return Ok(outcome);
            }

            let false_bindings = outcome.bindings.clone();
            session.evaluate_pattern_test_owned(
                test,
                value.clone(),
                outcome.bindings,
                false_bindings,
            )
        })()
                    .map(|value| <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledExceptOwnedSuspensionFrame {
    pub(super) excluded: crate::runtime::pattern_matching::suspended_state::FrozenMatcherValue<
        crate::runtime::session::session_state::MatcherNode,
    >,
    pub(super) value: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledExceptOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(match_outcome) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareCompiledExceptFinalizer {
                    excluded: self.excluded.clone(),
                    value: self.value.clone(),
                    match_outcome,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareCompiledExceptFinalizer {
    excluded: FrozenMatcherValue<MatcherNode>,
    value: Expr,
    match_outcome: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareCompiledExceptFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationCompiledExcept"),
            vec![
                matcher_node_into_continuation_expr(self.excluded.into_inner()),
                self.value,
                self.match_outcome,
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCompiledExceptFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCompiledExceptFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected compiled-except continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationCompiledExcept"),
                "unexpected compiled-except continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let excluded = matcher_node_from_continuation_expr(
                args.next()
                    .expect("compiled except stores its excluded matcher"),
            );
            let value = args.next().expect("compiled except stores its value");
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("compiled except stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_compiled_except_inner(excluded, value, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishCompiledExceptInnerSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishCompiledExceptInnerSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let payload = outcome;
        let resumed = (|| {
            let excluded_outcome = CompiledMatchOutcome::from_continuation_expr(payload?);
            Ok(
                match_outcome(!excluded_outcome.matched, excluded_outcome.bindings)
                    .into_continuation_expr(),
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledCallOwnedSuspensionFrame {
    pub(super) args: crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
        crate::runtime::session::session_state::MatcherArg,
    >,
    pub(super) value_args: rum_core::SharedArgs,
    pub(super) flat_head: rum_core::Expr,
    pub(super) one_identity_head: std::option::Option<rum_core::Symbol>,
    pub(super) orderless: bool,
    pub(super) flat: bool,
}

impl crate::runtime::suspension::SuspensionFrame for MatchCompiledCallOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(match_outcome) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareCompiledCallFinalizer {
                    args: self.args.clone(),
                    value_args: self.value_args.clone(),
                    flat_head: self.flat_head.clone(),
                    one_identity_head: self.one_identity_head,
                    orderless: self.orderless,
                    flat: self.flat,
                    match_outcome,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareCompiledCallFinalizer {
    args: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    flat_head: Expr,
    one_identity_head: Option<Symbol>,
    orderless: bool,
    flat: bool,
    match_outcome: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareCompiledCallFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let one_identity_head = self
            .one_identity_head
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .into_continuation_expr();
        Ok(Expr::call(
            system_expr("InternalContinuationCompiledCall"),
            vec![
                matcher_args_into_continuation_expr(self.args.into_vec()),
                self.value_args.into_vec().into_continuation_expr(),
                self.flat_head,
                one_identity_head,
                self.orderless.into_continuation_expr(),
                self.flat.into_continuation_expr(),
                self.match_outcome,
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueCompiledCallFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCompiledCallFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected compiled-call continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationCompiledCall"),
                "unexpected compiled-call continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let matcher_args = matcher_args_from_continuation_expr(
                args.next().expect("compiled call stores matcher arguments"),
            );
            let value_args = Vec::<Expr>::from_continuation_expr(
                args.next().expect("compiled call stores value arguments"),
            );
            let flat_head = args.next().expect("compiled call stores its flat head");
            let one_identity_head = Option::<Expr>::from_continuation_expr(
                args.next().expect("compiled call stores OneIdentity head"),
            )
            .map(|expr| match expr {
                Expr::Atom(Atom::Symbol(symbol)) => symbol,
                other => panic!("compiled-call OneIdentity head should be a symbol: {other:?}"),
            });
            let orderless = bool::from_continuation_expr(
                args.next()
                    .expect("compiled call stores its Orderless flag"),
            );
            let flat = bool::from_continuation_expr(
                args.next().expect("compiled call stores its Flat flag"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next().expect("compiled call stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_compiled_call_head(
                    matcher_args,
                    value_args,
                    flat_head,
                    one_identity_head,
                    orderless,
                    flat,
                    match_outcome,
                )
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchCompiledKeyValuePatternChoiceOwnedSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::core::SuspendedKeyValueChoice,
    pub(super) candidate_value: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchCompiledKeyValuePatternChoiceOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(match_outcome) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareKeyValueMatchFinalizer {
                    state: self.state.clone(),
                    candidate_value: Some(self.candidate_value.clone()),
                    match_outcome,
                    kind: KeyValueContinuationKind::Key,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishKeyValueKeyMatchSuspensionFrame {
    pub(super) state: crate::runtime::pattern_matching::core::SuspendedKeyValueChoice,
}

impl crate::runtime::suspension::SuspensionFrame for FinishKeyValueKeyMatchSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(match_outcome) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareKeyValueMatchFinalizer {
                    state: self.state.clone(),
                    candidate_value: None,
                    match_outcome,
                    kind: KeyValueContinuationKind::Value,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

enum KeyValueContinuationKind {
    Key,
    Value,
}

struct PrepareKeyValueMatchFinalizer {
    state: SuspendedKeyValueChoice,
    candidate_value: Option<Expr>,
    match_outcome: Expr,
    kind: KeyValueContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareKeyValueMatchFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let tag = match self.kind {
            KeyValueContinuationKind::Key => "InternalContinuationKeyValueKeyMatch",
            KeyValueContinuationKind::Value => "InternalContinuationKeyValueValueMatch",
        };
        Ok(Expr::call(
            system_expr(tag),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.candidate_value.into_continuation_expr(),
                self.match_outcome,
            ],
        ))
    }
}

fn decode_key_value_match_payload(
    payload: Expr,
    tag: &'static str,
) -> (KeyValueChoiceState, Option<Expr>, CompiledMatchOutcome) {
    let Expr::Call { head, args, .. } = payload else {
        panic!("unexpected {tag} continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, tag),
        "unexpected {tag} continuation tag: {head:?}"
    );
    let mut args = args.into_iter();
    let state = KeyValueChoiceState::from_continuation_expr(
        args.next()
            .expect("key-value continuation stores its state"),
    );
    let candidate_value = Option::<Expr>::from_continuation_expr(
        args.next()
            .expect("key-value continuation stores its candidate value"),
    );
    let match_outcome = CompiledMatchOutcome::from_continuation_expr(
        args.next()
            .expect("key-value continuation stores its match outcome"),
    );
    debug_assert!(args.next().is_none());
    (state, candidate_value, match_outcome)
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueKeyValueKeyMatchFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueKeyValueKeyMatchFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (state, candidate_value, match_outcome) =
                decode_key_value_match_payload(payload, "InternalContinuationKeyValueKeyMatch");
            session
                .finish_key_value_key_match(
                    state,
                    candidate_value.expect("key match stores a candidate value"),
                    match_outcome,
                )
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueKeyValueValueMatchFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueKeyValueValueMatchFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (state, candidate_value, match_outcome) =
                decode_key_value_match_payload(payload, "InternalContinuationKeyValueValueMatch");
            debug_assert!(candidate_value.is_none());
            session
                .finish_key_value_value_match(state, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateSequencePatternChecksOwnedSuspensionFrame1 {
    pub(super) checks: crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
        crate::runtime::session::session_state::SequencePatternCheck,
    >,
    pub(super) sequence_for_next: rum_core::Expr,
    pub(super) snapshot: crate::runtime::pattern_matching::suspended_state::FrozenMatchBindings,
    pub(super) index: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateSequencePatternChecksOwnedSuspensionFrame1
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
                outcome.into_continuation_expr()
            ));
        }
        plan_suspended_sequence_check(SuspendedSequenceCheckState {
            checks: self.checks.clone(),
            sequence: self.sequence_for_next.clone(),
            bindings: FrozenMatchBindings::freeze(outcome.bindings),
            snapshot: self.snapshot.clone(),
            index: self.index + 1,
        })
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvaluateSequencePatternChecksOwnedSuspensionFrame2 {
    pub(super) checks: crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
        crate::runtime::session::session_state::SequencePatternCheck,
    >,
    pub(super) sequence_for_next: rum_core::Expr,
    pub(super) bindings: crate::runtime::pattern_matching::suspended_state::FrozenMatchBindings,
    pub(super) snapshot: crate::runtime::pattern_matching::suspended_state::FrozenMatchBindings,
    pub(super) index: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateSequencePatternChecksOwnedSuspensionFrame2
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
                outcome.into_continuation_expr()
            ));
        }
        plan_suspended_sequence_check(SuspendedSequenceCheckState {
            checks: self.checks.clone(),
            sequence: self.sequence_for_next.clone(),
            bindings: FrozenMatchBindings::freeze(outcome.bindings),
            snapshot: self.snapshot.clone(),
            index: self.index + 1,
        })
    }
}

#[derive(Clone, rum_derive::FrameRootTrace)]
struct SuspendedSequenceCheckState {
    checks: FrozenMatcherSequence<SequencePatternCheck>,
    sequence: Expr,
    bindings: FrozenMatchBindings,
    snapshot: FrozenMatchBindings,
    index: usize,
}

fn plan_suspended_sequence_check(
    state: SuspendedSequenceCheckState,
) -> crate::runtime::suspension::FrameAction {
    let Some(check) = state.checks.as_slice().get(state.index) else {
        return crate::runtime::suspension::FrameAction::finalize(
            FinishFrozenMatchOutcomeFinalizer {
                matched: true,
                bindings: state.bindings,
            },
        );
    };
    let expr = match check {
        SequencePatternCheck::Condition(condition) => {
            substitute_bindings(condition, state.bindings.as_map())
        }
        SequencePatternCheck::PatternTest(test) => {
            Expr::call(test.clone(), vec![state.sequence.clone()])
        }
    };
    crate::runtime::suspension::FrameAction::evaluate(
        expr,
        ContinueSuspendedSequenceCheckFrame { state },
    )
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueSuspendedSequenceCheckFrame {
    state: SuspendedSequenceCheckState,
}

impl crate::runtime::suspension::SuspensionFrame for ContinueSuspendedSequenceCheckFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let result = match outcome {
            Ok(result) => result,
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        if !is_system_symbol_name(&result, "True") {
            return crate::runtime::suspension::FrameAction::finalize(
                FinishFrozenMatchOutcomeFinalizer {
                    matched: false,
                    bindings: self.state.snapshot.clone(),
                },
            );
        }
        let mut state = self.state.clone();
        state.index += 1;
        plan_suspended_sequence_check(state)
    }
}

struct FinishFrozenMatchOutcomeFinalizer {
    matched: bool,
    bindings: FrozenMatchBindings,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishFrozenMatchOutcomeFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(match_outcome(self.matched, self.bindings.into_owned()).into_continuation_expr())
    }
}
