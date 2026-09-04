mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod iterative;
use iterative::*;

mod suspended_state;
use suspended_state::*;

mod traversal;

const RECURSIVE_DELETE_CASES_CALL_DEPTH: usize = 16;

impl Session {
    pub(in crate::runtime) fn eval_delete_cases(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("DeleteCases", args_slice));
        }
        if !(2..=5).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("DeleteCases", args_slice, 2, 5));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = match target_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                let args = args.clone();
                crate::resume_or_suspend!(insert self, EvalDeleteCasesSuspensionFrame { args });
                crate::resume_or_suspend!(insert self, ContinueDeleteCasesFrame);
                return Err(EvalError::InputRequired { request });
            }
            Err(error) => return Err(error),
        };

        self.resume_delete_cases_after_target(
            target,
            args_slice[1].clone(),
            args_slice[2..].to_vec(),
            args_slice.to_vec(),
        )
    }

    fn resume_delete_cases_after_target(
        &mut self,
        target: Expr,
        pattern_expr: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        match pattern_result {
            Ok(pattern) => {
                self.resume_delete_cases_after_pattern(target, pattern, option_exprs, call_args)
            }
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                crate::resume_or_suspend!(insert self, ResumeDeleteCasesAfterTargetSuspensionFrame { target, call_args });
                crate::resume_or_suspend!(insert self, ContinueDeleteCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_delete_cases_after_target_suspended(
        &mut self,
        target: Expr,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let pattern_expr = call_args
            .get(1)
            .expect("DeleteCases suspended call stores its pattern")
            .clone();
        let pattern_result = self.resolve_matcher_pattern_expr_owned(pattern_expr);
        match pattern_result {
            Ok(pattern) => {
                self.resume_delete_cases_after_pattern_suspended(target, pattern, call_args)
            }
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeDeleteCasesAfterTargetSuspensionFrame { target, call_args });
                crate::resume_or_suspend!(insert self, ContinueDeleteCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_delete_cases_after_pattern_suspended(
        &mut self,
        target: Expr,
        pattern: Expr,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let mut defaults = CasesTraversalConfig::default_cases();
        defaults.include_heads = self.system_option_bool("DeleteCases", "Heads", false)?;
        let traversal_result =
            self.resolve_cases_traversal_shared_from(call_args.clone(), 2, defaults);
        match traversal_result {
            Ok(traversal) => self.finish_delete_cases_after_resolved_traversal_suspended(
                target, pattern, traversal, call_args,
            ),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeDeleteCasesAfterPatternSuspensionFrame { target, pattern, call_args });
                crate::resume_or_suspend!(insert self, ContinueDeleteCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_delete_cases_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        option_exprs: Vec<Expr>,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let mut defaults = CasesTraversalConfig::default_cases();
        defaults.include_heads = self.system_option_bool("DeleteCases", "Heads", false)?;
        let traversal_result = self.resolve_cases_traversal_owned(option_exprs, defaults);
        match traversal_result {
            Ok(traversal) => self.finish_delete_cases_after_resolved_traversal(
                target, pattern, traversal, call_args,
            ),
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                crate::resume_or_suspend!(insert self, ResumeDeleteCasesAfterPatternSuspensionFrame { target, pattern, call_args });
                crate::resume_or_suspend!(insert self, ContinueDeleteCasesFrame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn finish_delete_cases_after_resolved_traversal(
        &mut self,
        target: Expr,
        pattern: Expr,
        traversal: CasesTraversalResolution,
        call_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("DeleteCases", &call_args, error));
            }
        };

        let deleted_result = self.delete_cases_owned(target, pattern, traversal, 0, 0);
        crate::resume_or_suspend!(
            self,
            deleted_result,
            ok(outcome) => { (|| {
                    Ok(match outcome.result {
                        DeleteCasesResult::Keep(expr) => expr,
                        DeleteCasesResult::Delete => Expr::call(system_expr("Sequence"), Vec::new()),
                        DeleteCasesResult::Splice(items) => Expr::call(system_expr("Sequence"), items),
                    })
                })() },
            suspend => FinishDeleteCasesAfterResolvedTraversalSuspensionFrame,
        )
    }

    fn finish_delete_cases_after_resolved_traversal_suspended(
        &mut self,
        target: Expr,
        pattern: Expr,
        traversal: CasesTraversalResolution,
        call_args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call(
                    "DeleteCases",
                    call_args.as_ref(),
                    error,
                ));
            }
        };

        let deleted_result = self.delete_cases_owned(target, pattern, traversal, 0, 0);
        crate::resume_or_suspend!(
            self,
            deleted_result,
            ok(outcome) => { (|| {
                    Ok(match outcome.result {
                        DeleteCasesResult::Keep(expr) => expr,
                        DeleteCasesResult::Delete => Expr::call(system_expr("Sequence"), Vec::new()),
                        DeleteCasesResult::Splice(items) => Expr::call(system_expr("Sequence"), items),
                    })
                })() },
            suspend => FinishDeleteCasesAfterResolvedTraversalSuspensionFrame,
        )
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedDeleteAssociationChildren>();
    assert_traces::<SuspendedDeleteCallChildren>();
};
