use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ThreadCallConfig {
    expr: Expr,
    thread_head: Expr,
    arg_spec: ThreadArgSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThreadCallNormalizationError {
    InvalidArgCount { actual: usize },
    InvalidArgSpec,
}

impl Session {
    pub(super) fn eval_release_hold(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("ReleaseHold", args, 1));
        }

        let released = release_hold_once(&args[0]).unwrap_or_else(|| args[0].clone());

        let released_result = self.evaluate(released);
        crate::resume_or_suspend!(
            self,
            released_result,
            ok(value) => Ok(value),
            suspend => EvalReleaseHoldSuspensionFrame,
        )
    }

    pub(super) fn eval_thread(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let ThreadCallConfig {
            expr,
            thread_head,
            arg_spec,
        } = match self.compile_thread_call_from_resolved_args(args) {
            Ok(config) => config,
            Err(ThreadCallNormalizationError::InvalidArgCount { actual }) => {
                return Ok(self.invalid_call_with_message(
                    "Thread",
                    args,
                    "argx",
                    format!("expected 1, 2, or 3 argument(s), got {actual}"),
                ));
            }
            Err(ThreadCallNormalizationError::InvalidArgSpec) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Thread",
                    args,
                    "expected the third argument to be a standard sequence specification such as All, None, n, -n, {n}, {m, n}, or {m, n, s}",
                ));
            }
        };

        let Expr::Call {
            head: threaded_head,
            args: threaded_args,
        } = &expr
        else {
            return Ok(expr);
        };

        match self.thread_call_args(
            threaded_head.as_ref(),
            threaded_args,
            &thread_head,
            &thread_head,
            &arg_spec,
        )? {
            ThreadingDispatch::NotApplicable => Ok(expr),
            ThreadingDispatch::Threaded(threaded) => Ok(threaded),
            ThreadingDispatch::Mismatch => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    fn compile_thread_call_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<ThreadCallConfig, ThreadCallNormalizationError> {
        match args {
            [expr] => Ok(ThreadCallConfig {
                expr: expr.clone(),
                thread_head: system_expr("List"),
                arg_spec: ThreadArgSpec::All,
            }),
            [expr, thread_head] => Ok(ThreadCallConfig {
                expr: expr.clone(),
                thread_head: thread_head.clone(),
                arg_spec: ThreadArgSpec::All,
            }),
            [expr, thread_head, spec] => {
                let Some(arg_spec) = parse_thread_arg_spec(spec) else {
                    return Err(ThreadCallNormalizationError::InvalidArgSpec);
                };

                Ok(ThreadCallConfig {
                    expr: expr.clone(),
                    thread_head: thread_head.clone(),
                    arg_spec,
                })
            }
            _ => Err(ThreadCallNormalizationError::InvalidArgCount { actual: args.len() }),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalReleaseHoldSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for EvalReleaseHoldSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
