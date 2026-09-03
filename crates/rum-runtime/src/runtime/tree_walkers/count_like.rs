mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, FrameRootTrace)]
enum CountLikeFinish {
    Count,
    MemberQ,
    FreeQ,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedCountTraversal {
    walker: PostOrderTraversal,
    pattern: Expr,
    traversal: CasesTraversalConfig,
    count: usize,
    current_depth: usize,
}

impl CountLikeFinish {
    fn finish(self, count: usize) -> Expr {
        match self {
            Self::Count => Expr::integer(BigInt::from(count)),
            Self::MemberQ => system_expr(if count > 0 { "True" } else { "False" }),
            Self::FreeQ => system_expr(if count == 0 { "True" } else { "False" }),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_count(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("Count", args_slice));
        }
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("Count", args_slice, 2, 4));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalCountSuspensionFrame { args }
        },
        )?;

        self.resume_count_after_target(target, args)
    }

    pub(in crate::runtime) fn eval_member_q(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("MemberQ", args_slice));
        }
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("MemberQ", args_slice, 2, 4));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalMemberQSuspensionFrame { args }
        },
        )?;

        self.resume_member_q_after_target(target, args)
    }

    pub(in crate::runtime) fn eval_free_q(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("FreeQ", args_slice));
        }
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("FreeQ", args_slice, 2, 4));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalFreeQSuspensionFrame { args }
        },
        )?;

        self.resume_free_q_after_target(target, args)
    }

    fn resume_count_after_target(
        &mut self,
        target: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(args[1].clone());
        let pattern = crate::resume_or_suspend!(
            self,
            pattern_result,
            ok(value) => Ok(value),
            suspend => {
        let target = target.clone();
                    let args = args.clone();
        ResumeCountAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_count_after_pattern(target, pattern, args)
    }

    fn resume_count_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("Count", "Heads", false)?;
        self.finish_count_like_after_pattern(
            "Count",
            target,
            pattern,
            args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_cases()
            },
            CountLikeFinish::Count,
        )
    }

    fn resume_member_q_after_target(
        &mut self,
        target: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(args[1].clone());
        let pattern = crate::resume_or_suspend!(
            self,
            pattern_result,
            ok(value) => Ok(value),
            suspend => {
        let target = target.clone();
                    let args = args.clone();
        ResumeMemberQAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_member_q_after_pattern(target, pattern, args)
    }

    fn resume_member_q_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("MemberQ", "Heads", false)?;
        self.finish_count_like_after_pattern(
            "MemberQ",
            target,
            pattern,
            args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_member_q()
            },
            CountLikeFinish::MemberQ,
        )
    }

    fn resume_free_q_after_target(
        &mut self,
        target: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(args[1].clone());
        let pattern = crate::resume_or_suspend!(
            self,
            pattern_result,
            ok(value) => Ok(value),
            suspend => {
        let target = target.clone();
                    let args = args.clone();
        ResumeFreeQAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_free_q_after_pattern(target, pattern, args)
    }

    fn resume_free_q_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let include_heads = self.system_option_bool("FreeQ", "Heads", true)?;
        self.finish_count_like_after_pattern(
            "FreeQ",
            target,
            pattern,
            args,
            CasesTraversalConfig {
                include_heads,
                ..CasesTraversalConfig::default_free_q()
            },
            CountLikeFinish::FreeQ,
        )
    }

    fn finish_count_like_after_pattern(
        &mut self,
        builtin_name: &'static str,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
        defaults: CasesTraversalConfig,
        finish: CountLikeFinish,
    ) -> Result<Expr, EvalError> {
        let traversal_result =
            self.resolve_traversal_without_limit_owned(args[2..].to_vec(), defaults);
        match traversal_result {
            Ok(traversal) => self.finish_count_like_after_traversal(
                builtin_name,
                target,
                pattern,
                args,
                finish,
                traversal,
            ),
            Err(EvalError::InputRequired { request }) => {
                let args = args.clone();
                crate::resume_or_suspend!(insert self, FinishCountLikeAfterPatternSuspensionFrame { builtin_name, target, pattern, args, finish });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_count_like_after_traversal(
        &mut self,
        builtin_name: &'static str,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
        finish: CountLikeFinish,
        traversal: TraversalWithoutLimitResolution,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            TraversalWithoutLimitResolution::Valid(traversal) => traversal,
            TraversalWithoutLimitResolution::Invalid(error) => {
                return Ok(self.invalid_traversal_without_limit_call(
                    builtin_name,
                    args.as_ref(),
                    error,
                ));
            }
        };

        let outcome_result = self.count_pattern_matches_owned(target, pattern, traversal);
        crate::resume_or_suspend!(
            self,
            outcome_result,
            ok(outcome) => { (|| {
                    Ok(finish.finish(outcome.count))
                })() },
            suspend => FinishCountLikeAfterTraversalSuspensionFrame { finish },
        )
    }

    fn count_pattern_matches_owned(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
    ) -> Result<TraversalCountOutcome, EvalError> {
        let state = SuspendedCountTraversal {
            walker: PostOrderTraversal::new(expr, traversal.include_heads, 0),
            pattern,
            traversal,
            count: 0,
            current_depth: 1,
        };
        self.drive_count_pattern_matches(state)
    }

    fn drive_count_pattern_matches(
        &mut self,
        mut state: SuspendedCountTraversal,
    ) -> Result<TraversalCountOutcome, EvalError> {
        while let Some(node) = state.walker.next() {
            state.current_depth = node.depth;
            if state.traversal.limit_reached(state.count) {
                return Ok(TraversalCountOutcome::new(state.count, node.depth, true));
            }
            if !state.traversal.levels.matches(node.level, node.depth) {
                continue;
            }

            let match_result =
                self.match_expr_owned(state.pattern.clone(), node.expr, SymbolExprMap::default());
            match match_result {
                Ok(outcome) => {
                    state.count += usize::from(outcome.matched);
                    if state.traversal.limit_reached(state.count) {
                        return Ok(TraversalCountOutcome::new(state.count, node.depth, true));
                    }
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ContinueCountTraversalSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }

        Ok(TraversalCountOutcome::new(
            state.count,
            state.current_depth,
            false,
        ))
    }

    fn resume_count_traversal(
        &mut self,
        mut state: SuspendedCountTraversal,
        outcome: CompiledMatchOutcome,
    ) -> Result<TraversalCountOutcome, EvalError> {
        state.count += usize::from(outcome.matched);
        if state.traversal.limit_reached(state.count) {
            return Ok(TraversalCountOutcome::new(
                state.count,
                state.current_depth,
                true,
            ));
        }
        self.drive_count_pattern_matches(state)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<CountLikeFinish>();
    assert_traces::<SuspendedCountTraversal>();
};
