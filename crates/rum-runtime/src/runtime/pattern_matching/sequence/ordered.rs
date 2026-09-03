use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    // This is a multi-argument shape specialization. Keep it out of the common
    // atomic/single-argument matcher layout; at its intended widths, the
    // iterator loop makes one out-of-line call immaterial.
    #[cold]
    #[inline(never)]
    pub(super) fn match_compiled_ordered_args_iter_owned(
        &mut self,
        mut pattern_args: std::vec::IntoIter<MatcherArg>,
        mut value_args: std::vec::IntoIter<Expr>,
        mut bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        loop {
            let Some(pattern_arg) = pattern_args.next() else {
                return Ok(match_outcome(value_args.len() == 0, bindings));
            };
            let MatcherArg::Single(pattern) = pattern_arg else {
                return self.match_compiled_arg_and_rest_owned(
                    pattern_arg,
                    pattern_args.collect(),
                    value_args.collect(),
                    bindings,
                );
            };
            let Some(value) = value_args.next() else {
                return Ok(match_outcome(false, bindings));
            };

            match self.match_compiled_owned(pattern, value, bindings) {
                Ok(outcome) if outcome.matched => bindings = outcome.bindings,
                Ok(outcome) => return Ok(outcome),
                Err(error) => {
                    return self.resume_match_compiled_ordered_args_iter_owned(
                        error,
                        pattern_args,
                        value_args,
                    );
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn resume_match_compiled_ordered_args_iter_owned(
        &mut self,
        error: EvalError,
        pattern_args: std::vec::IntoIter<MatcherArg>,
        value_args: std::vec::IntoIter<Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match error {
            EvalError::InputRequired { request } => {
                let pattern_args = FrozenMatcherSequence::freeze(pattern_args.collect());
                let value_args = SharedArgs::new(value_args.collect());
                crate::resume_or_suspend!(insert self, ResumeMatchCompiledOrderedArgsIterOwnedSuspensionFrame { pattern_args, value_args });
                crate::resume_or_suspend!(insert self, ContinueMatchCompiledOrderedArgsFrame);
                Err(EvalError::InputRequired { request })
            }
            error => Err(error),
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn collect_compiled_ordered_args_iter_owned(
        &mut self,
        mut pattern_args: std::vec::IntoIter<MatcherArg>,
        mut value_args: std::vec::IntoIter<Expr>,
        mut bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        loop {
            let Some(pattern_arg) = pattern_args.next() else {
                return Ok(if value_args.len() == 0 {
                    vec![bindings]
                } else {
                    Vec::new()
                });
            };
            let MatcherArg::Single(pattern) = pattern_arg else {
                return self.collect_compiled_arg_and_rest_matches_owned(
                    pattern_arg,
                    pattern_args.collect(),
                    value_args.collect(),
                    bindings,
                );
            };
            let Some(value) = value_args.next() else {
                return Ok(Vec::new());
            };

            match self.match_compiled_owned(pattern, value, bindings) {
                Ok(outcome) if outcome.matched => bindings = outcome.bindings,
                Ok(_) => return Ok(Vec::new()),
                Err(error) => {
                    return self.resume_collect_compiled_ordered_args_iter_owned(
                        error,
                        pattern_args,
                        value_args,
                    );
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn resume_collect_compiled_ordered_args_iter_owned(
        &mut self,
        error: EvalError,
        pattern_args: std::vec::IntoIter<MatcherArg>,
        value_args: std::vec::IntoIter<Expr>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        match error {
            EvalError::InputRequired { request } => {
                let pattern_args = FrozenMatcherSequence::freeze(pattern_args.collect());
                let value_args = SharedArgs::new(value_args.collect());
                crate::resume_or_suspend!(insert self, ResumeCollectCompiledOrderedArgsIterOwnedSuspensionFrame { pattern_args, value_args });
                crate::resume_or_suspend!(insert self, ContinueCollectCompiledOrderedArgsFrame);
                Err(EvalError::InputRequired { request })
            }
            error => Err(error),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeMatchCompiledOrderedArgsIterOwnedSuspensionFrame {
    pattern_args: crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
        crate::runtime::session::session_state::MatcherArg,
    >,
    value_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeMatchCompiledOrderedArgsIterOwnedSuspensionFrame
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
        crate::runtime::suspension::FrameAction::finalize(PrepareOrderedArgsFinalizer {
            pattern_args: self.pattern_args.clone(),
            value_args: self.value_args.clone(),
            bindings: outcome.bindings,
        })
    }
}

struct PrepareOrderedArgsFinalizer {
    pattern_args: FrozenMatcherSequence<MatcherArg>,
    value_args: SharedArgs,
    bindings: SymbolExprMap,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOrderedArgsFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationOrderedArgs"),
            vec![
                matcher_args_into_continuation_expr(self.pattern_args.into_vec()),
                self.value_args.into_vec().into_continuation_expr(),
                binding_map_into_continuation_expr(self.bindings),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueMatchCompiledOrderedArgsFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueMatchCompiledOrderedArgsFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                return Ok(payload);
            };
            if !is_system_symbol_name(&head, "InternalContinuationOrderedArgs") {
                return Ok(Expr::call_shared(head, args));
            }
            let mut args = args.into_iter();
            let pattern_args = matcher_args_from_continuation_expr(
                args.next()
                    .expect("ordered matcher stores its remaining patterns"),
            );
            let value_args = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ordered matcher stores its remaining values"),
            );
            let bindings = binding_map_from_continuation_expr(
                args.next().expect("ordered matcher stores its bindings"),
            );
            debug_assert!(args.next().is_none());
            session
                .match_compiled_ordered_args_iter_owned(
                    pattern_args.into_iter(),
                    value_args.into_iter(),
                    bindings,
                )
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeCollectCompiledOrderedArgsIterOwnedSuspensionFrame {
    pattern_args: crate::runtime::pattern_matching::suspended_state::FrozenMatcherSequence<
        crate::runtime::session::session_state::MatcherArg,
    >,
    value_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCollectCompiledOrderedArgsIterOwnedSuspensionFrame
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
        crate::runtime::suspension::FrameAction::finalize(PrepareOrderedArgsFinalizer {
            pattern_args: self.pattern_args.clone(),
            value_args: self.value_args.clone(),
            bindings: outcome.bindings,
        })
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueCollectCompiledOrderedArgsFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCollectCompiledOrderedArgsFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                return Ok(payload);
            };
            if !is_system_symbol_name(&head, "InternalContinuationOrderedArgs") {
                return Ok(Expr::call_shared(head, args));
            }
            let mut args = args.into_iter();
            let pattern_args = matcher_args_from_continuation_expr(
                args.next()
                    .expect("ordered collector stores its remaining patterns"),
            );
            let value_args = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ordered collector stores its remaining values"),
            );
            let bindings = binding_map_from_continuation_expr(
                args.next().expect("ordered collector stores its bindings"),
            );
            debug_assert!(args.next().is_none());
            session
                .collect_compiled_ordered_args_iter_owned(
                    pattern_args.into_iter(),
                    value_args.into_iter(),
                    bindings,
                )
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
