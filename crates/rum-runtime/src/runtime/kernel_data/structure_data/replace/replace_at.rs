use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
struct SuspendedReplaceAtPaths {
    expr: Expr,
    paths: FrozenReplaceSequence<Vec<isize>>,
    rule_plan: RewriteRulePlan,
    index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_replace_at(
        &mut self,
        _head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        match args.as_ref() {
            [rules_expr, position_spec] => Ok(replace_at_operator_form(
                rules_expr.clone(),
                position_spec.clone(),
            )),
            [target_expr, rules_expr, position_spec] => {
                let target_result = self.evaluate(target_expr.clone());
                let target = match target_result {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        let args = args.clone();
                        crate::resume_or_suspend!(insert self, EvalReplaceAtSuspensionFrame { args });
                        crate::resume_or_suspend!(insert self, ContinueReplaceAtSetupFrame);
                        return Err(EvalError::InputRequired { request });
                    }
                    Err(error) => return Err(error),
                };

                self.resume_replace_at_after_target(
                    target,
                    rules_expr.clone(),
                    position_spec.clone(),
                    args.as_ref().to_vec(),
                )
            }
            _ => Ok(self.invalid_call_with_arg_range("ReplaceAt", args, 2, 3)),
        }
    }
    fn resume_replace_at_after_target(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        position_spec: Expr,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let positions_result = self.evaluate(position_spec);
        match positions_result {
            Ok(resolved_positions) => {
                self.finish_replace_at_positions(target, rules_expr, call_args, resolved_positions)
            }
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                crate::resume_or_suspend!(insert self, ResumeReplaceAtAfterTargetSuspensionFrame { target, rules_expr, call_args });
                crate::resume_or_suspend!(insert self, ContinueReplaceAtSetupFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_replace_at_positions(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        call_args: Vec<Expr>,
        resolved_positions: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(paths) = exact_position_paths_from_expr(&target, &resolved_positions) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAt",
                &call_args,
                "expected an exact part specification or list of exact part specifications as the third argument",
            ));
        };
        self.resume_replace_at_after_positions(target, rules_expr, paths, call_args)
    }

    fn resume_replace_at_after_positions(
        &mut self,
        target: Expr,
        rules_expr: Expr,
        paths: Vec<Vec<isize>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(
            rules_expr,
            RewriteRuleExprContinuation::ReplaceAt {
                target,
                paths,
                call_args,
            },
        )
    }

    pub(in crate::runtime) fn resume_replace_at_after_rule_exprs(
        &mut self,
        target: Expr,
        paths: Vec<Vec<isize>>,
        normalized_rule_exprs: Option<Vec<Expr>>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(rule_exprs) = normalized_rule_exprs else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAt",
                &call_args,
                "expected a rule or list of rules as the second argument",
            ));
        };

        let rules = self
            .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
            .expect("normalized rewrite rule expressions should recompile");
        let rule_plan = Self::shared_rewrite_rule_plan(rules);

        self.replace_at_paths_owned(target, paths, rule_plan, 0)
    }

    fn replace_at_paths_owned(
        &mut self,
        expr: Expr,
        paths: Vec<Vec<isize>>,
        rule_plan: RewriteRulePlan,
        index: usize,
    ) -> Result<Expr, EvalError> {
        let Some(path) = paths.get(index).cloned() else {
            return Ok(expr);
        };

        let Some(part) = exact_expr_at_path(&expr, &path) else {
            return self.replace_at_paths_owned(expr, paths, rule_plan, index + 1);
        };

        let replaced_result = self.apply_rewrite_rule_plan_once(
            part,
            rule_plan.clone(),
            RuleApplicationSource::ReplaceAt,
        );
        match replaced_result {
            Ok(replaced) => {
                let expr = match replaced {
                    Some(replaced) => replace_exact_path(expr, &path, replaced),
                    None => expr,
                };
                self.replace_at_paths_owned(expr, paths, rule_plan, index + 1)
            }
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedReplaceAtPaths {
                    expr,
                    paths: FrozenReplaceSequence::freeze(paths),
                    rule_plan,
                    index,
                };
                crate::resume_or_suspend!(insert self, ReplaceAtPathsOwnedSuspensionFrame {
                    state,
                    path: std::sync::Arc::from(path),
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_replace_at_paths(
        &mut self,
        mut state: SuspendedReplaceAtPaths,
        path: std::sync::Arc<[isize]>,
        replaced: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        if let Some(replaced) = replaced {
            state.expr = replace_exact_path(state.expr, path.as_ref(), replaced);
        }
        state.index += 1;
        loop {
            let Some(path) = state.paths.as_slice().get(state.index).cloned() else {
                return Ok(state.expr);
            };
            let Some(part) = exact_expr_at_path(&state.expr, &path) else {
                state.index += 1;
                continue;
            };
            let replaced_result = self.apply_rewrite_rule_plan_once(
                part,
                state.rule_plan.clone(),
                RuleApplicationSource::ReplaceAt,
            );
            match replaced_result {
                Ok(Some(replaced)) => {
                    state.expr = replace_exact_path(state.expr, &path, replaced);
                    state.index += 1;
                }
                Ok(None) => state.index += 1,
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedReplaceAtPathsSuspensionFrame {
                        state,
                        path: std::sync::Arc::from(path),
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

fn replace_at_operator_form(rules_expr: Expr, position_spec: Expr) -> Expr {
    let parameter = Expr::symbol("rum$replaceAtArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(
                system_expr("ReplaceAt"),
                vec![parameter, rules_expr, position_spec],
            ),
        ],
    )
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedReplaceAtPaths>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalReplaceAtSuspensionFrame {
    args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalReplaceAtSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareReplaceAtSetupFinalizer {
                    result,
                    target: None,
                    rules_expr: None,
                    args: Some(self.args.clone()),
                    call_args: None,
                    kind: ReplaceAtSetupContinuationKind::Target,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeReplaceAtAfterTargetSuspensionFrame {
    target: rum_core::Expr,
    rules_expr: rum_core::Expr,
    call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeReplaceAtAfterTargetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(result) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareReplaceAtSetupFinalizer {
                    result,
                    target: Some(self.target.clone()),
                    rules_expr: Some(self.rules_expr.clone()),
                    args: None,
                    call_args: Some(self.call_args.clone()),
                    kind: ReplaceAtSetupContinuationKind::Positions,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(Clone, Copy)]
enum ReplaceAtSetupContinuationKind {
    Target,
    Positions,
}

struct PrepareReplaceAtSetupFinalizer {
    result: Expr,
    target: Option<Expr>,
    rules_expr: Option<Expr>,
    args: Option<SharedArgs>,
    call_args: Option<SharedArgs>,
    kind: ReplaceAtSetupContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareReplaceAtSetupFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let tag = match self.kind {
            ReplaceAtSetupContinuationKind::Target => "InternalContinuationReplaceAtTarget",
            ReplaceAtSetupContinuationKind::Positions => "InternalContinuationReplaceAtPositions",
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
struct ContinueReplaceAtSetupFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceAtSetupFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected ReplaceAt setup continuation payload")
            };
            let mut args = args.into_iter();
            let result = args
                .next()
                .expect("ReplaceAt setup continuation stores its result");
            let target = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceAt setup continuation stores target"),
            );
            let rules_expr = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceAt setup continuation stores rules"),
            );
            let original_args = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("ReplaceAt setup continuation stores original arguments"),
            );
            let call_args = Option::<Vec<Expr>>::from_continuation_expr(
                args.next()
                    .expect("ReplaceAt setup continuation stores call arguments"),
            );
            debug_assert!(args.next().is_none());
            if is_system_symbol_name(&head, "InternalContinuationReplaceAtTarget") {
                let original_args =
                    original_args.expect("ReplaceAt target continuation stores arguments");
                session.resume_replace_at_after_target(
                    result,
                    original_args[1].clone(),
                    original_args[2].clone(),
                    original_args,
                )
            } else if is_system_symbol_name(&head, "InternalContinuationReplaceAtPositions") {
                session.finish_replace_at_positions(
                    target.expect("ReplaceAt positions continuation stores target"),
                    rules_expr.expect("ReplaceAt positions continuation stores rules"),
                    call_args.expect("ReplaceAt positions continuation stores call arguments"),
                    result,
                )
            } else {
                panic!("unexpected ReplaceAt setup continuation tag: {head:?}")
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ReplaceAtPathsOwnedSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::structure_data::replace::replace_at::SuspendedReplaceAtPaths,
    pub(super) path: std::sync::Arc<[isize]>,
}

impl crate::runtime::suspension::SuspensionFrame for ReplaceAtPathsOwnedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut path = self.path.clone();
        let value = outcome;
        let resumed = (|| {
            let replaced = Option::<Expr>::from_continuation_expr(value?);
            session.resume_suspended_replace_at_paths(state, path, replaced)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeSuspendedReplaceAtPathsSuspensionFrame {
    pub(super) state:
        crate::runtime::kernel_data::structure_data::replace::replace_at::SuspendedReplaceAtPaths,
    pub(super) path: std::sync::Arc<[isize]>,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSuspendedReplaceAtPathsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut path = self.path.clone();
        let value = outcome;
        let resumed = (|| {
            let replaced = Option::<Expr>::from_continuation_expr(value?);
            session.resume_suspended_replace_at_paths(state, path, replaced)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
