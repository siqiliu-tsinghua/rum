use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_replace_repeated(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("ReplaceRepeated", args_slice));
        }
        let max_iterations_expr = match args_slice {
            [_, _] => None,
            [_, _, option] if replace_repeated_max_iterations_option_value(option).is_some() => {
                Some(option.clone())
            }
            _ => {
                return Ok(self.invalid_call_with_exact_arg_count(
                    "ReplaceRepeated",
                    args_slice,
                    2,
                ));
            }
        };

        let current_result = self.evaluate(args_slice[0].clone());
        let current = match current_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                let args = args.clone();
                let max_iterations_expr = max_iterations_expr.clone();
                crate::resume_or_suspend!(insert self, EvalReplaceRepeatedSuspensionFrame { args, max_iterations_expr });
                crate::resume_or_suspend!(insert self, ContinueReplaceRepeatedTargetFrame);
                return Err(EvalError::InputRequired { request });
            }
            Err(error) => return Err(error),
        };

        self.resume_replace_repeated_after_current(
            current,
            args_slice[1].clone(),
            max_iterations_expr,
            args_slice.to_vec(),
        )
    }

    fn resume_replace_repeated_after_current(
        &mut self,
        current: Expr,
        rules_expr: Expr,
        max_iterations_expr: Option<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            rules_expr.clone(),
            RewriteRuleExprContinuation::ReplaceRepeated {
                current,
                rules_input: rules_expr,
                max_iterations_expr,
                call_args,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_repeated_after_rule_exprs(
        &mut self,
        current: Expr,
        rules_input: Expr,
        max_iterations_expr: Option<Expr>,
        normalized_rule_exprs: Option<Vec<Expr>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceRepeated",
                &[current, rules_input],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");
        let rules = Self::shared_rewrite_rule_plan(rules);

        let limit_result = self.resolve_replace_repeated_limit_owned(max_iterations_expr);
        crate::resume_or_suspend!(
            self,
            limit_result,
            ok(effective_limit) => { let session = self; (|| {
                    let Some(effective_limit_expr) = effective_limit else {
                        return Ok(session.invalid_call_with_arg_form(
                            "ReplaceRepeated",
                            &call_args,
                            "expected MaxIterations -> Automatic, Infinity, or a positive integer",
                        ));
                    };
                    let effective_limit = expr_as_non_negative_usize(&effective_limit_expr)
                        .expect("replace-repeated limit should be non-negative");

                    session.resume_replace_repeated_loop(current, rules, 0, effective_limit)
                })() },
            suspend => ResumeReplaceRepeatedAfterRuleExprsSuspensionFrame {
                call_args: SharedArgs::new(call_args),
                current,
                rules,
            },
        )
    }

    fn resolve_replace_repeated_limit_owned(
        &mut self,
        max_iterations_expr: Option<Expr>,
    ) -> Result<Option<Expr>, EvalError> {
        let ceiling = self.resource_limits.replace_repeated_limit;
        let Some(max_iterations_expr) = max_iterations_expr else {
            return Ok(Some(Expr::integer(BigInt::from(ceiling))));
        };

        let resolved_result = self.evaluate(max_iterations_expr);
        crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(resolved_expr) => { let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resolved_expr); (|| {
                    let Some(value) = replace_repeated_max_iterations_option_value(&resolved) else {
                        return Ok(None);
                    };
                    Ok(parse_replace_repeated_max_iterations_value(value, ceiling)
                        .map(|limit| Expr::integer(BigInt::from(limit))))
                })() },
            suspend => ResolveReplaceRepeatedLimitOwnedSuspensionFrame { ceiling },
        )
    }

    fn resume_replace_repeated_loop(
        &mut self,
        current: Expr,
        rules: RewriteRulePlan,
        iteration: usize,
        effective_limit: usize,
    ) -> Result<Expr, EvalError> {
        if iteration >= effective_limit {
            return Err(EvalError::IterationLimitExceeded {
                limit: effective_limit,
            });
        }

        let replaced = self.apply_replace_all_with_plan(
            current.clone(),
            rules.clone(),
            RuleApplicationSource::ReplaceRepeated,
        )?;
        let next_result = self.evaluate(replaced);
        let next = crate::resume_or_suspend!(
            self,
            next_result,
            ok(value) => Ok(value),
            suspend => {
        let current = current.clone();
                    let rules = rules.clone();
        ResumeReplaceRepeatedLoopSuspensionFrame { current, rules, iteration, effective_limit }
        },
        )?;

        self.resume_replace_repeated_after_next(current, rules, iteration, effective_limit, next)
    }

    fn resume_replace_repeated_after_next(
        &mut self,
        current: Expr,
        rules: RewriteRulePlan,
        iteration: usize,
        effective_limit: usize,
        next: Expr,
    ) -> Result<Expr, EvalError> {
        if next == current {
            Ok(next)
        } else {
            self.resume_replace_repeated_loop(next, rules, iteration + 1, effective_limit)
        }
    }
}

fn replace_repeated_max_iterations_option_value(expr: &Expr) -> Option<&Expr> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    max_iterations_option_symbol_matches(&args[0]).then_some(&args[1])
}

fn max_iterations_option_symbol_matches(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbol.name() == "MaxIterations",
        Expr::Atom(Atom::String(name)) => {
            name == "MaxIterations"
                || resolve_unqualified_symbol(name)
                    .is_some_and(|symbol| symbol.name() == "MaxIterations")
        }
        _ => false,
    }
}

fn parse_replace_repeated_max_iterations_value(value: &Expr, ceiling: usize) -> Option<usize> {
    if is_system_symbol_name(value, "Automatic") || is_system_symbol_name(value, "Infinity") {
        return Some(ceiling);
    }

    expr_as_non_negative_usize(value)
        .filter(|limit| *limit > 0)
        .map(|limit| limit.min(ceiling))
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalReplaceRepeatedSuspensionFrame {
    args: rum_core::SharedArgs,
    max_iterations_expr: std::option::Option<rum_core::Expr>,
}

impl crate::runtime::suspension::SuspensionFrame for EvalReplaceRepeatedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(current) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceRepeatedTargetFinalizer {
                    current,
                    args: self.args.clone(),
                    max_iterations_expr: self.max_iterations_expr.clone(),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareReplaceRepeatedTargetFinalizer {
    current: Expr,
    args: SharedArgs,
    max_iterations_expr: Option<Expr>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareReplaceRepeatedTargetFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationReplaceRepeatedTarget"),
            vec![
                self.current,
                self.args.into_vec().into_continuation_expr(),
                self.max_iterations_expr.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueReplaceRepeatedTargetFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceRepeatedTargetFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected ReplaceRepeated target continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationReplaceRepeatedTarget"),
                "unexpected ReplaceRepeated target continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let current = args
                .next()
                .expect("ReplaceRepeated target continuation stores current value");
            let call_args = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceRepeated target continuation stores call arguments"),
            );
            let max_iterations_expr = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceRepeated target continuation stores its option"),
            );
            debug_assert!(args.next().is_none());
            session.resume_replace_repeated_after_current(
                current,
                call_args[1].clone(),
                max_iterations_expr,
                call_args,
            )
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReplaceRepeatedAfterRuleExprsSuspensionFrame {
    call_args: rum_core::SharedArgs,
    current: rum_core::Expr,
    rules: crate::runtime::RewriteRulePlan,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeReplaceRepeatedAfterRuleExprsSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let call_args = self.call_args.clone();
        #[allow(unused_mut)]
        let mut current = self.current.clone();
        #[allow(unused_mut)]
        let mut rules = self.rules.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let effective_limit = <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            let Some(effective_limit_expr) = effective_limit else {
                return Ok(session.invalid_call_with_arg_form(
                    "ReplaceRepeated",
                    call_args.as_ref(),
                    "expected MaxIterations -> Automatic, Infinity, or a positive integer",
                ));
            };
            let effective_limit = expr_as_non_negative_usize(&effective_limit_expr)
                .expect("replace-repeated limit should be non-negative");

            session.resume_replace_repeated_loop(current, rules, 0, effective_limit)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResolveReplaceRepeatedLimitOwnedSuspensionFrame {
    ceiling: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResolveReplaceRepeatedLimitOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut ceiling = self.ceiling.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let resolved = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Option<rum_core::Expr>, EvalError> {
            let Some(value) = replace_repeated_max_iterations_option_value(&resolved) else {
                return Ok(None);
            };
            Ok(parse_replace_repeated_max_iterations_value(value, ceiling)
                .map(|limit| Expr::integer(BigInt::from(limit))))
        })()
                    .map(|value| <Option<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReplaceRepeatedLoopSuspensionFrame {
    current: rum_core::Expr,
    rules: crate::runtime::RewriteRulePlan,
    iteration: usize,
    effective_limit: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeReplaceRepeatedLoopSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut current = self.current.clone();
        #[allow(unused_mut)]
        let mut rules = self.rules.clone();
        #[allow(unused_mut)]
        let mut iteration = self.iteration.clone();
        #[allow(unused_mut)]
        let mut effective_limit = self.effective_limit.clone();
        let resumed = match outcome {
            Ok(next) => (|| {
                session.resume_replace_repeated_after_next(
                    current,
                    rules,
                    iteration,
                    effective_limit,
                    next,
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
