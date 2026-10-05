mod options;
mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
struct SuspendedCasesTraversal {
    walker: PostOrderTraversal,
    spec: FrozenRewriteValue<CasesSpec>,
    traversal: CasesTraversalConfig,
    matches: PersistentRewriteExprs,
    current_depth: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedCasesRuleMatch {
    spec: FrozenRewriteValue<CasesSpec>,
}

impl Session {
    pub(in crate::runtime) fn eval_cases(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("Cases", args_slice));
        }
        if !(2..=5).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("Cases", args_slice, 2, 5));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = match target_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                let args = args.clone();
                crate::resume_or_suspend!(insert self, EvalCasesSuspensionFrame { args });
                crate::resume_or_suspend!(insert self, ContinueCasesFrame);
                return Err(EvalError::InputRequired { request });
            }
            Err(error) => return Err(error),
        };

        self.resume_cases_after_target(
            target,
            args_slice[1].clone(),
            args_slice[2..].to_vec(),
            args_slice.to_vec(),
        )
    }

    fn resume_cases_after_target(
        &mut self,
        target: Expr,
        spec_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let spec_result = self.resolve_cases_spec_expr_for_matching_owned(spec_expr.clone());
        match spec_result {
            Ok(resolved_spec) => self.resume_cases_after_resolved_spec(
                target,
                resolved_spec,
                option_exprs,
                call_args,
            ),
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                crate::resume_or_suspend!(insert self, ResumeCasesAfterTargetSuspensionFrame { target, call_args });
                crate::resume_or_suspend!(insert self, ContinueCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_cases_after_target_suspended(
        &mut self,
        target: Expr,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let spec_expr = call_args
            .get(1)
            .expect("Cases suspended call stores its specification")
            .clone();
        let spec_result = self.resolve_cases_spec_expr_for_matching_owned(spec_expr);
        match spec_result {
            Ok(resolved_spec) => {
                self.resume_cases_after_resolved_spec_suspended(target, resolved_spec, call_args)
            }
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeCasesAfterTargetSuspensionFrame { target, call_args });
                crate::resume_or_suspend!(insert self, ContinueCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_cases_after_resolved_spec_suspended(
        &mut self,
        target: Expr,
        resolved_spec: Expr,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let spec =
            FrozenRewriteValue::freeze(self.compile_cases_spec_from_resolved_expr(resolved_spec));
        self.system_option_bool(
            "Cases",
            "Heads",
            false,
            options::CasesOptionWorking::Suspended {
                target,
                spec,
                call_args,
            },
        )
    }

    fn resume_cases_after_resolved_spec(
        &mut self,
        target: Expr,
        resolved_spec: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let spec = self.compile_cases_spec_from_resolved_expr(resolved_spec);
        self.system_option_bool(
            "Cases",
            "Heads",
            false,
            options::CasesOptionWorking::Owned {
                target,
                spec,
                option_exprs,
                call_args,
            },
        )
    }

    #[inline(always)]
    fn finish_cases_after_resolved_traversal(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalResolution,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("Cases", &call_args, error));
            }
        };

        self.finish_cases_with_traversal(target, spec, traversal)
    }

    fn finish_cases_after_resolved_traversal_suspended(
        &mut self,
        target: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
        traversal: CasesTraversalResolution,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("Cases", call_args.as_ref(), error));
            }
        };
        if traversal.limit_reached(0) {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let matches_result = self.collect_cases_with_suspended_spec(target, spec, traversal);
        crate::resume_or_suspend!(
            self,
            matches_result,
            ok(outcome) => {
                Ok(Expr::call(system_expr("List"), outcome.matches))
            },
            suspend => FinishCasesWithTraversalSuspensionFrame,
        )
    }

    fn finish_cases_with_traversal(
        &mut self,
        target: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
    ) -> Result<Expr, EvalError> {
        if traversal.limit_reached(0) {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let matches_result = self.collect_cases_owned(target, spec, traversal);
        crate::resume_or_suspend!(
            self,
            matches_result,
            ok(outcome) => { (|| {
                    Ok(Expr::call(system_expr("List"), outcome.matches))
                })() },
            suspend => FinishCasesWithTraversalSuspensionFrame,
        )
    }

    fn collect_cases_owned(
        &mut self,
        expr: Expr,
        spec: CasesSpec,
        traversal: CasesTraversalConfig,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        let mut walker = PostOrderTraversal::new(expr, traversal.include_heads, 0);
        let mut matches = Vec::new();
        let mut current_depth = 1;
        while let Some(node) = walker.next() {
            current_depth = node.depth;
            if traversal.limit_reached(matches.len()) {
                return Ok(CasesCollectionOutcome::new(matches, node.depth, true));
            }
            if !traversal.levels.matches(node.level, node.depth) {
                continue;
            }

            let case_result = self.try_collect_case_owned(node.expr, &spec);
            match case_result {
                Ok(Some(case)) => {
                    matches.push(case);
                    if traversal.limit_reached(matches.len()) {
                        return Ok(CasesCollectionOutcome::new(matches, node.depth, true));
                    }
                }
                Ok(None) => {}
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedCasesTraversal {
                        walker,
                        spec: FrozenRewriteValue::freeze(spec),
                        traversal,
                        matches: PersistentRewriteExprs::freeze(matches),
                        current_depth: node.depth,
                    };
                    crate::resume_or_suspend!(insert self, ContinueCasesTraversalSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }

        Ok(CasesCollectionOutcome::new(matches, current_depth, false))
    }

    fn collect_cases_with_suspended_spec(
        &mut self,
        expr: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
        traversal: CasesTraversalConfig,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        self.drive_suspended_cases_traversal(SuspendedCasesTraversal {
            walker: PostOrderTraversal::new(expr, traversal.include_heads, 0),
            spec,
            traversal,
            matches: PersistentRewriteExprs::freeze(Vec::new()),
            current_depth: 1,
        })
    }

    fn resume_cases_traversal(
        &mut self,
        mut state: SuspendedCasesTraversal,
        case: Option<Expr>,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        if let Some(case) = case {
            state.matches = state.matches.appended(case);
            if state.traversal.limit_reached(state.matches.len()) {
                return Ok(CasesCollectionOutcome::new(
                    state.matches.into_vec(),
                    state.current_depth,
                    true,
                ));
            }
        }
        self.drive_suspended_cases_traversal(state)
    }

    fn drive_suspended_cases_traversal(
        &mut self,
        mut state: SuspendedCasesTraversal,
    ) -> Result<CasesCollectionOutcome, EvalError> {
        while let Some(node) = state.walker.next() {
            state.current_depth = node.depth;
            if state.traversal.limit_reached(state.matches.len()) {
                return Ok(CasesCollectionOutcome::new(
                    state.matches.into_vec(),
                    node.depth,
                    true,
                ));
            }
            if !state.traversal.levels.matches(node.level, node.depth) {
                continue;
            }

            let case_result = self.try_collect_case_suspended(node.expr, state.spec.clone());
            match case_result {
                Ok(Some(case)) => {
                    state.matches = state.matches.appended(case);
                    if state.traversal.limit_reached(state.matches.len()) {
                        return Ok(CasesCollectionOutcome::new(
                            state.matches.into_vec(),
                            node.depth,
                            true,
                        ));
                    }
                }
                Ok(None) => {}
                Err(EvalError::InputRequired { request }) => {
                    let matches = std::mem::replace(
                        &mut state.matches,
                        PersistentRewriteExprs::freeze(Vec::new()),
                    )
                    .into_vec();
                    state.matches = PersistentRewriteExprs::freeze(matches);
                    crate::resume_or_suspend!(insert self, ContinueCasesTraversalSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
        Ok(CasesCollectionOutcome::new(
            state.matches.into_vec(),
            state.current_depth,
            false,
        ))
    }

    fn try_collect_case_owned(
        &mut self,
        expr: Expr,
        spec: &CasesSpec,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);

        match spec {
            CasesSpec::Pattern(pattern) => {
                let match_result =
                    self.match_expr_owned(pattern.clone(), expr.clone(), SymbolExprMap::default());
                self.finish_cases_pattern_match(match_result, expr)
            }
            CasesSpec::Rule(rule) => {
                let match_result = self.match_compiled_owned(
                    rule.matcher.clone(),
                    normalized_expr,
                    SymbolExprMap::default(),
                );
                match match_result {
                    Ok(outcome) => self.finish_cases_rule_match(rule, outcome),
                    Err(EvalError::InputRequired { request }) => {
                        let state = SuspendedCasesRuleMatch {
                            spec: FrozenRewriteValue::freeze(CasesSpec::Rule(rule.clone())),
                        };
                        crate::resume_or_suspend!(insert self, TryCollectCaseOwnedSuspensionFrame { state });
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn try_collect_case_suspended(
        &mut self,
        expr: Expr,
        spec: FrozenRewriteValue<CasesSpec>,
    ) -> Result<Option<Expr>, EvalError> {
        let normalized_expr = self.normalize_match_expr(&expr);
        match spec.as_ref() {
            CasesSpec::Pattern(pattern) => {
                let match_result =
                    self.match_expr_owned(pattern.clone(), expr.clone(), SymbolExprMap::default());
                self.finish_cases_pattern_match(match_result, expr)
            }
            CasesSpec::Rule(rule) => {
                let match_result = self.match_compiled_owned(
                    rule.matcher.clone(),
                    normalized_expr,
                    SymbolExprMap::default(),
                );
                match match_result {
                    Ok(outcome) => self.finish_cases_rule_match(rule, outcome),
                    Err(EvalError::InputRequired { request }) => {
                        let state = SuspendedCasesRuleMatch { spec };
                        crate::resume_or_suspend!(insert self, TryCollectCaseSuspendedSuspensionFrame { state });
                        Err(EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }
        }
    }

    fn finish_cases_pattern_match(
        &mut self,
        match_result: Result<CompiledMatchOutcome, EvalError>,
        expr: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        match match_result {
            Ok(outcome) => Ok(outcome.matched.then_some(expr)),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, FinishCasesPatternMatchSuspensionFrame { expr });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_suspended_cases_rule_match(
        &mut self,
        state: SuspendedCasesRuleMatch,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        let CasesSpec::Rule(rule) = state.spec.as_ref() else {
            unreachable!("suspended Cases rule match retains a rule spec");
        };
        self.finish_cases_rule_match(rule, outcome)
    }

    fn finish_cases_rule_match(
        &mut self,
        rule: &RewriteRule,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        if !outcome.matched {
            return Ok(None);
        }

        let transformed = substitute_rule_bindings_for_rhs(
            &rule.rhs,
            &outcome.bindings,
            rule.rhs_may_splice_without_bindings,
        );
        let transformed_result = self.evaluate(transformed);
        crate::resume_or_suspend!(
            self,
            transformed_result,
            ok(value) => Ok(value),
            suspend => FinishCasesRuleMatchSuspensionFrame,
        )
        .map(Some)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedCasesTraversal>();
    assert_traces::<SuspendedCasesRuleMatch>();
};
