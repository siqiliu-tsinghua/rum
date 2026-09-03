use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod matching;

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedReplaceListScan {
    pub(super) target: Expr,
    pub(super) normalized_target: Expr,
    pub(super) rules: RewriteRulePlan,
    pub(super) max_results: Option<usize>,
    pub(super) collected: PersistentRewriteExprs,
    pub(super) index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_replace_list(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        match args_slice {
            [rules_expr] => Ok(replace_list_operator_form(rules_expr.clone())),
            [target_expr, rules_expr] | [target_expr, rules_expr, _] => {
                let target_result = self.evaluate(target_expr.clone());
                let target = match target_result {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        let args = args.clone();
                        crate::resume_or_suspend!(insert self, EvalReplaceListSuspensionFrame { args });
                        crate::resume_or_suspend!(insert self, ContinueReplaceListSetupFrame);
                        return Err(EvalError::InputRequired { request });
                    }
                    Err(error) => return Err(error),
                };

                self.resume_replace_list_after_target(
                    target,
                    rules_expr.clone(),
                    args_slice.get(2).cloned(),
                    args_slice.to_vec(),
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("ReplaceList", args_slice, 1, 3)),
        }
    }

    fn resume_replace_list_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        max_expr: Option<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(max_expr) = max_expr else {
            return self.resume_replace_list_after_limit(target, rules_expr, None, call_args);
        };

        let max_result = self.evaluate(max_expr);
        match max_result {
            Ok(resolved_max) => {
                self.finish_replace_list_limit(target, rules_expr, resolved_max, call_args)
            }
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                crate::resume_or_suspend!(insert self, ResumeReplaceListAfterTargetSuspensionFrame { target, rules_expr, call_args });
                crate::resume_or_suspend!(insert self, ContinueReplaceListSetupFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_replace_list_limit(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        resolved_max: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(max_results) = parse_replace_list_max_results(&resolved_max) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceList",
                &call_args,
                "expected a non-negative integer or Infinity as the optional third argument",
            ));
        };
        self.resume_replace_list_after_limit(target, rules_expr, max_results, call_args)
    }

    fn resume_replace_list_after_limit(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            rules_expr,
            RewriteRuleExprContinuation::ReplaceList {
                target,
                max_results,
                call_args,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_list_after_rule_exprs(
        &mut self,
        target: Expr,
        normalized_rule_exprs: Option<Vec<Expr>>,
        max_results: Option<usize>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceList",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");

        let normalized_target = self.normalize_match_expr(&target);
        let rules = Self::shared_rewrite_rule_plan(rules);
        let results_result = self.collect_replace_list_results_owned(
            target,
            normalized_target,
            rules,
            max_results,
            Vec::new(),
            0,
        );
        crate::resume_or_suspend!(
            self,
            results_result,
            ok(results) => { (|| {
                    Ok(Expr::call(system_expr("List"), results))
                })() },
            suspend => ResumeReplaceListAfterRuleExprsSuspensionFrame,
        )
    }

    fn collect_replace_list_results_owned(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: RewriteRulePlan,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        index: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        self.collect_replace_list_results_with_pending(
            target,
            normalized_target,
            rules,
            max_results,
            collected,
            index,
            None,
        )
    }

    /// Walks the rules one per loop iteration instead of recursing per rule,
    /// so a wide rule list no longer grows the native stack. `pending` carries
    /// already-collected bindings for the rule at `index` across a suspension
    /// resume, so a resumed traversal skips re-collecting them and keeps the
    /// accumulated results in rule order.
    fn collect_replace_list_results_with_pending(
        &mut self,
        target: Expr,
        normalized_target: Expr,
        rules: RewriteRulePlan,
        max_results: Option<usize>,
        collected: Vec<Expr>,
        mut index: usize,
        mut pending: Option<Vec<SymbolExprMap>>,
    ) -> Result<Vec<Expr>, EvalError> {
        loop {
            if max_results.is_some_and(|limit| collected.len() >= limit) {
                return Ok(collected);
            }

            let Some(rule) = rules.get(index) else {
                return Ok(collected);
            };

            let bindings = if let Some(bindings) = pending.take() {
                bindings
            } else {
                let bindings_result = self.collect_replace_list_rule_bindings_owned(
                    rule.matcher.clone(),
                    normalized_target.clone(),
                );
                match bindings_result {
                    Ok(bindings) => bindings,
                    Err(EvalError::InputRequired { request }) => {
                        let state = SuspendedReplaceListScan {
                            target,
                            normalized_target,
                            rules,
                            max_results,
                            collected: PersistentRewriteExprs::freeze(collected),
                            index,
                        };
                        let next = ContinueReplaceListResultsWithPendingFrame {
                            target: state.target.clone(),
                            normalized_target: state.normalized_target.clone(),
                            rules: state.rules.clone(),
                            max_results: state.max_results,
                            index: state.index,
                        };
                        crate::resume_or_suspend!(insert self, CollectReplaceListResultsWithPendingSuspensionFrame { state });
                        crate::resume_or_suspend!(insert self, next);
                        return Err(EvalError::InputRequired { request });
                    }
                    Err(error) => return Err(error),
                }
            };

            if bindings.is_empty() {
                index += 1;
                continue;
            }

            return self.collect_replace_list_rule_results_owned(
                target,
                normalized_target,
                rules,
                max_results,
                collected,
                index,
                bindings,
            );
        }
    }
}

fn parse_replace_list_max_results(expr: &Expr) -> Option<Option<usize>> {
    if is_system_symbol_name(expr, "Infinity") {
        Some(None)
    } else {
        expr_as_non_negative_usize(expr).map(Some)
    }
}

fn replace_list_operator_form(rules_expr: Expr) -> Expr {
    let parameter = Expr::symbol("rum$replaceListArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(system_expr("ReplaceList"), vec![parameter, rules_expr]),
        ],
    )
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplaceListScan>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalReplaceListSuspensionFrame {
    args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalReplaceListSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceListSetupFinalizer {
                    result,
                    target: None,
                    rules_expr: None,
                    args: Some(self.args.clone()),
                    call_args: None,
                    kind: ReplaceListSetupContinuationKind::Target,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReplaceListAfterTargetSuspensionFrame {
    target: rum_core::Expr,
    rules_expr: rum_core::Expr,
    call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeReplaceListAfterTargetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceListSetupFinalizer {
                    result,
                    target: Some(self.target.clone()),
                    rules_expr: Some(self.rules_expr.clone()),
                    args: None,
                    call_args: Some(self.call_args.clone()),
                    kind: ReplaceListSetupContinuationKind::Limit,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(Clone, Copy)]
enum ReplaceListSetupContinuationKind {
    Target,
    Limit,
}

struct PrepareReplaceListSetupFinalizer {
    result: Expr,
    target: Option<Expr>,
    rules_expr: Option<Expr>,
    args: Option<SharedArgs>,
    call_args: Option<SharedArgs>,
    kind: ReplaceListSetupContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareReplaceListSetupFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let tag = match self.kind {
            ReplaceListSetupContinuationKind::Target => "InternalContinuationReplaceListTarget",
            ReplaceListSetupContinuationKind::Limit => "InternalContinuationReplaceListLimit",
        };
        Ok(Expr::call(
            system_expr(tag),
            vec![
                self.result,
                self.target.into_continuation_expr(),
                self.rules_expr.into_continuation_expr(),
                self.args.map(SharedArgs::into_vec).into_continuation_expr(),
                self.call_args
                    .map(SharedArgs::into_vec)
                    .into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueReplaceListSetupFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceListSetupFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected ReplaceList setup continuation payload")
            };
            let mut args = args.into_iter();
            let result = args
                .next()
                .expect("ReplaceList setup continuation stores its result");
            let target = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList setup continuation stores target"),
            );
            let rules_expr = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList setup continuation stores rule expression"),
            );
            let original_args = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList setup continuation stores original arguments"),
            );
            let call_args = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList setup continuation stores call arguments"),
            );
            debug_assert!(args.next().is_none());
            if is_system_symbol_name(&head, "InternalContinuationReplaceListTarget") {
                let original_args =
                    original_args.expect("ReplaceList target continuation stores arguments");
                session.resume_replace_list_after_target(
                    result,
                    original_args[1].clone(),
                    original_args.get(2).cloned(),
                    original_args,
                )
            } else if is_system_symbol_name(&head, "InternalContinuationReplaceListLimit") {
                session.finish_replace_list_limit(
                    target.expect("ReplaceList limit continuation stores target"),
                    rules_expr.expect("ReplaceList limit continuation stores rule expression"),
                    result,
                    call_args.expect("ReplaceList limit continuation stores call arguments"),
                )
            } else {
                panic!("unexpected ReplaceList setup continuation tag: {head:?}")
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReplaceListAfterRuleExprsSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame
    for ResumeReplaceListAfterRuleExprsSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let results = <Vec<rum_core::Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(Expr::call(system_expr("List"), results))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct CollectReplaceListResultsWithPendingSuspensionFrame {
    state: crate::runtime::kernel_data::rewrite_data::replace::list::SuspendedReplaceListScan,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectReplaceListResultsWithPendingSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(bindings) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceListResultsWithPendingFinalizer {
                    state: self.state.clone(),
                    bindings,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareReplaceListResultsWithPendingFinalizer {
    state: SuspendedReplaceListScan,
    bindings: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer
    for PrepareReplaceListResultsWithPendingFinalizer
{
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationReplaceListScan"),
            vec![
                self.state.collected.into_vec().into_continuation_expr(),
                self.bindings,
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ContinueReplaceListResultsWithPendingFrame {
    target: Expr,
    normalized_target: Expr,
    rules: RewriteRulePlan,
    max_results: Option<usize>,
    index: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceListResultsWithPendingFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected ReplaceList scan continuation payload")
            };
            assert!(
                is_system_symbol_name(&head, "InternalContinuationReplaceListScan"),
                "unexpected ReplaceList scan continuation tag: {head:?}"
            );
            let mut args = args.into_iter();
            let collected = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList scan stores collected values"),
            );
            let bindings = Vec::<SymbolExprMap>::from_continuation_expr(
                args.next().expect("ReplaceList scan stores rule bindings"),
            );
            debug_assert!(args.next().is_none());
            session
                .collect_replace_list_results_with_pending(
                    self.target.clone(),
                    self.normalized_target.clone(),
                    self.rules.clone(),
                    self.max_results,
                    collected,
                    self.index,
                    Some(bindings),
                )
                .map(Vec::<Expr>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
