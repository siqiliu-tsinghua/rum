use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_sequence_items_owned(
        &mut self,
        matcher: MatcherNode,
        values: Vec<Expr>,
        bindings: SymbolExprMap,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(value) = values.get(index).cloned() else {
            return Ok(match_outcome(true, bindings));
        };

        let bindings_for_next = bindings.clone();
        let matcher_for_next = matcher.clone();
        let values_for_next = values.clone();
        let item_result = self.match_compiled_owned(matcher, value, bindings);
        match item_result {
            Ok(outcome) => self.finish_compiled_sequence_item(
                matcher_for_next,
                values_for_next,
                bindings_for_next,
                index,
                outcome,
            ),
            Err(EvalError::InputRequired { request }) => {
                let matcher = FrozenMatcherValue::freeze(matcher_for_next);
                let values = SharedArgs::new(values_for_next);
                let bindings = FrozenMatchBindings::freeze(bindings_for_next);
                crate::resume_or_suspend!(insert self, MatchCompiledSequenceItemsOwnedSuspensionFrame { matcher, values, bindings, index });
                crate::resume_or_suspend!(insert self, ContinueCompiledSequenceItemsFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_compiled_sequence_item(
        &mut self,
        matcher: MatcherNode,
        values: Vec<Expr>,
        bindings: SymbolExprMap,
        index: usize,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        self.match_compiled_sequence_items_owned(matcher, values, bindings, index + 1)
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_sequence_pattern_owned(
        &mut self,
        spec: CompiledSequencePattern,
        values: Vec<Expr>,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let snapshot = bindings.clone();

        if values.len() < spec.min_len {
            return Ok(match_outcome(false, snapshot));
        }

        if spec.max_len.is_some_and(|max_len| values.len() > max_len) {
            return Ok(match_outcome(false, snapshot));
        }

        let matcher_result = self.match_compiled_sequence_items_owned(
            spec.matcher.clone(),
            values.clone(),
            bindings,
            0,
        );
        match matcher_result {
            Ok(outcome) => self.finish_compiled_sequence_pattern(spec, values, snapshot, outcome),
            Err(EvalError::InputRequired { request }) => {
                let spec = FrozenMatcherValue::freeze(spec);
                let values = SharedArgs::new(values);
                let snapshot = FrozenMatchBindings::freeze(snapshot);
                crate::resume_or_suspend!(insert self, MatchCompiledSequencePatternOwnedSuspensionFrame { spec, values, snapshot });
                crate::resume_or_suspend!(insert self, ContinueCompiledSequencePatternFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_compiled_sequence_pattern(
        &mut self,
        spec: CompiledSequencePattern,
        values: Vec<Expr>,
        snapshot: SymbolExprMap,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(match_outcome(false, snapshot));
        }

        let mut bindings = outcome.bindings;
        let sequence_value = sequence_expr(values);

        if let Some(name) = &spec.name {
            if !bind_pattern_name(name, sequence_value.clone(), &mut bindings) {
                return Ok(match_outcome(false, snapshot));
            }
        }

        if !spec.checks.is_empty() {
            return self.evaluate_sequence_pattern_checks_owned(
                spec.checks,
                sequence_value,
                bindings,
                snapshot,
                0,
            );
        }

        Ok(match_outcome(true, bindings))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct MatchCompiledSequenceItemsOwnedSuspensionFrame {
    matcher: crate::runtime::pattern_matching::suspended_state::FrozenMatcherValue<
        crate::runtime::session::session_state::MatcherNode,
    >,
    values: rum_core::SharedArgs,
    bindings: crate::runtime::pattern_matching::suspended_state::FrozenMatchBindings,
    index: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchCompiledSequenceItemsOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareCompiledSequenceItemsFinalizer {
                    matcher: self.matcher.clone(),
                    values: self.values.clone(),
                    bindings: self.bindings.clone(),
                    index: self.index,
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareCompiledSequenceItemsFinalizer {
    matcher: FrozenMatcherValue<MatcherNode>,
    values: SharedArgs,
    bindings: FrozenMatchBindings,
    index: usize,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareCompiledSequenceItemsFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationCompiledSequenceItems"),
            vec![
                matcher_node_into_continuation_expr(self.matcher.into_inner()),
                self.values.into_vec().into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings.into_owned()),
                usize_into_continuation_expr(self.index),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueCompiledSequenceItemsFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCompiledSequenceItemsFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected compiled-sequence-items continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationCompiledSequenceItems"),
                "unexpected compiled-sequence-items continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let matcher = matcher_node_from_continuation_expr(
                args.next().expect("sequence items store their matcher"),
            );
            let values = Vec::<Expr>::from_continuation_expr(
                args.next().expect("sequence items store their values"),
            );
            let bindings = binding_map_from_continuation_expr(
                args.next().expect("sequence items store their bindings"),
            );
            let index = usize_from_continuation_expr(
                args.next().expect("sequence items store their cursor"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("sequence items store their match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_compiled_sequence_item(matcher, values, bindings, index, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct MatchCompiledSequencePatternOwnedSuspensionFrame {
    spec: crate::runtime::pattern_matching::suspended_state::FrozenMatcherValue<
        crate::runtime::session::session_state::CompiledSequencePattern,
    >,
    values: rum_core::SharedArgs,
    snapshot: crate::runtime::pattern_matching::suspended_state::FrozenMatchBindings,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchCompiledSequencePatternOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareCompiledSequencePatternFinalizer {
                    spec: self.spec.clone(),
                    values: self.values.clone(),
                    snapshot: self.snapshot.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareCompiledSequencePatternFinalizer {
    spec: FrozenMatcherValue<CompiledSequencePattern>,
    values: SharedArgs,
    snapshot: FrozenMatchBindings,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareCompiledSequencePatternFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationCompiledSequencePattern"),
            vec![
                sequence_pattern_into_continuation_expr(self.spec.into_inner()),
                self.values.into_vec().into_continuation_expr(),
                binding_map_into_continuation_expr(self.snapshot.into_owned()),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueCompiledSequencePatternFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCompiledSequencePatternFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected compiled-sequence-pattern continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationCompiledSequencePattern"),
                "unexpected compiled-sequence-pattern continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let spec = sequence_pattern_from_continuation_expr(
                args.next()
                    .expect("sequence pattern stores its specification"),
            );
            let values = Vec::<Expr>::from_continuation_expr(
                args.next().expect("sequence pattern stores its values"),
            );
            let snapshot = binding_map_from_continuation_expr(
                args.next().expect("sequence pattern stores its snapshot"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("sequence pattern stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_compiled_sequence_pattern(spec, values, snapshot, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
