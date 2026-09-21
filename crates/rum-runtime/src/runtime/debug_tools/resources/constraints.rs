use super::memory::expr_memory_proxy_bytes;
use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_time_constrained(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        let (body, limit_expr, fallback) = match args_slice {
            [body, limit] => (body.clone(), limit, None),
            [body, limit, fallback] => (body.clone(), limit, Some(fallback.clone())),
            _ => {
                return Ok(self.invalid_call_with_arg_range("TimeConstrained", args_slice, 2, 3));
            }
        };

        let resolved_result = self.evaluate(limit_expr.clone());
        let resolved_limit = crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
                    let suspended_body = body.clone();
                    let suspended_fallback = fallback.clone();
        EvalTimeConstrainedSuspensionFrame { suspended_body, suspended_fallback, args }
        },
        )?;
        self.resume_time_constrained_with_limit(body, fallback, args_slice, resolved_limit)
    }

    fn resume_time_constrained_with_limit(
        &mut self,
        body: Expr,
        fallback: Option<Expr>,
        raw_args: &[Expr],
        resolved_limit: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(limit) = parse_time_constraint_duration(&resolved_limit) else {
            return Ok(self.invalid_call_with_arg_form(
                "TimeConstrained",
                raw_args,
                "expected the timeout argument to be a non-negative exact number in the current first-pass subset",
            ));
        };

        let prior_deadline = self.current_time_deadline;
        let next_deadline = combine_time_deadlines(prior_deadline, limit);
        self.current_time_deadline = Some(next_deadline);

        let body_result = self.evaluate(body);
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => {
        let fallback = fallback.clone();
        ResumeTimeConstrainedWithLimitSuspensionFrame { prior_deadline, fallback }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_time_constrained(outcome, prior_deadline, fallback),
        }
    }

    pub(in crate::runtime) fn eval_memory_constrained(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        let (body, limit_expr, fallback) = match args_slice {
            [body, limit] => (body.clone(), limit, None),
            [body, limit, fallback] => (body.clone(), limit, Some(fallback.clone())),
            _ => {
                return Ok(self.invalid_call_with_arg_range("MemoryConstrained", args_slice, 2, 3));
            }
        };

        let resolved_result = self.evaluate(limit_expr.clone());
        let resolved_limit = crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
                    let suspended_body = body.clone();
                    let suspended_fallback = fallback.clone();
        EvalMemoryConstrainedSuspensionFrame { suspended_body, suspended_fallback, args }
        },
        )?;
        self.resume_memory_constrained_with_limit(body, fallback, args_slice, resolved_limit)
    }

    fn resume_memory_constrained_with_limit(
        &mut self,
        body: Expr,
        fallback: Option<Expr>,
        raw_args: &[Expr],
        resolved_limit: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(limit) = expr_as_non_negative_usize(&resolved_limit) else {
            return Ok(self.invalid_call_with_arg_form(
                "MemoryConstrained",
                raw_args,
                "expected the memory limit argument to be a non-negative exact integer",
            ));
        };

        let prior_limit = self.current_memory_limit;
        self.current_memory_limit = Some(prior_limit.map_or(limit, |current| current.min(limit)));

        let body_result = self.evaluate(body);
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => {
        let fallback = fallback.clone();
        ResumeMemoryConstrainedWithLimitSuspensionFrame { prior_limit, fallback }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_memory_constrained(outcome, prior_limit, fallback),
        }
    }

    /// Every armed checkpoint checks the session-plus-expression proxy against
    /// the memory limit. A single construction larger than the limit is rejected
    /// before allocation only where a projected-byte precheck exists; otherwise
    /// the excess is detected at the next existing checkpoint that observes it.
    /// This structural model is not an allocator or process RSS bound.
    pub(in crate::runtime) fn enforce_active_resource_limits_for_expr(
        &self,
        expr: &Expr,
    ) -> Result<(), EvalError> {
        self.enforce_active_time_limit()?;

        if let Some(limit) = self.current_memory_limit {
            self.enforce_memory_limit_for_expr(expr, limit)?;
        }

        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn enforce_memory_limit_for_expr(&self, expr: &Expr, limit: usize) -> Result<(), EvalError> {
        let projected = self
            .current_memory_proxy_bytes()
            .saturating_add(expr_memory_proxy_bytes(expr));
        if projected > limit {
            return Err(EvalError::MemoryConstraintExceeded { limit });
        }
        Ok(())
    }

    pub(in crate::runtime) fn enforce_active_time_limit(&self) -> Result<(), EvalError> {
        if let Some(deadline) = self.current_time_deadline {
            if Instant::now() >= deadline {
                return Err(EvalError::TimeConstraintExceeded);
            }
        }

        Ok(())
    }

    /// Enforces the active memory limit for a projected additional byte count.
    pub fn enforce_active_memory_limit_for_projected_bytes(
        &self,
        bytes: usize,
    ) -> Result<(), EvalError> {
        self.record_builtin_memory_projection_for_growth_test();

        if let Some(limit) = self.current_memory_limit {
            let projected = self.current_memory_proxy_bytes().saturating_add(bytes);
            if projected > limit {
                return Err(EvalError::MemoryConstraintExceeded { limit });
            }
        }

        Ok(())
    }

    /// Enforces the configured generated-element limit for a concrete count.
    pub fn enforce_generated_element_limit_for_count(&self, count: usize) -> Result<(), EvalError> {
        let limit = self.resource_limits.generated_element_limit;
        if count > limit {
            return Err(EvalError::GeneratedElementLimitExceeded { limit });
        }

        Ok(())
    }

    /// Enforces the configured generated-element limit for an arbitrary-size count.
    pub fn enforce_generated_element_limit_for_big_count(
        &self,
        count: &BigInt,
    ) -> Result<(), EvalError> {
        let limit = self.resource_limits.generated_element_limit;
        if count > &BigInt::from(limit) {
            return Err(EvalError::GeneratedElementLimitExceeded { limit });
        }

        Ok(())
    }

    /// Enforces the active iteration-work limit for a potentially large count.
    pub fn enforce_iteration_work_limit_for_big_count(
        &self,
        count: &BigInt,
    ) -> Result<(), EvalError> {
        let limit = self.resource_limits.iteration_work_limit;
        if count > &BigInt::from(limit) {
            return Err(EvalError::IterationWorkLimitExceeded { limit });
        }

        Ok(())
    }

    fn finish_time_constrained(
        &mut self,
        outcome: Result<Expr, EvalError>,
        prior_deadline: Option<Instant>,
        fallback: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.current_time_deadline = prior_deadline;

        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::TimeConstraintExceeded) => match fallback {
                Some(expr) => {
                    let fallback_result = self.evaluate(expr);
                    crate::resume_or_suspend!(
                        self,
                        fallback_result,
                        ok(value) => Ok(value),
                        suspend => FinishTimeConstrainedSuspensionFrame,
                    )
                }
                None => Ok(system_expr("$Aborted")),
            },
            Err(error) => Err(error),
        }
    }

    fn finish_memory_constrained(
        &mut self,
        outcome: Result<Expr, EvalError>,
        prior_limit: Option<usize>,
        fallback: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.current_memory_limit = prior_limit;

        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::MemoryConstraintExceeded { .. }) => match fallback {
                Some(expr) => {
                    let fallback_result = self.evaluate(expr);
                    crate::resume_or_suspend!(
                        self,
                        fallback_result,
                        ok(value) => Ok(value),
                        suspend => FinishMemoryConstrainedSuspensionFrame,
                    )
                }
                None => Ok(system_expr("$Aborted")),
            },
            Err(error) => Err(error),
        }
    }
}

fn parse_time_constraint_duration(expr: &Expr) -> Option<Duration> {
    let rational = expr_as_exact_rational(expr)?;
    if rational.numerator() < &BigInt::from(0) {
        return None;
    }

    let nanos_numerator = rational.numerator().clone() * BigInt::from(1_000_000_000_u64);
    let nanos = nanos_numerator / rational.denominator().clone();
    let nanos = nanos
        .to_string()
        .parse::<u64>()
        .unwrap_or(u64::MAX.saturating_sub(1));

    Some(Duration::from_nanos(nanos))
}

fn combine_time_deadlines(prior: Option<Instant>, limit: Duration) -> Instant {
    let candidate = Instant::now()
        .checked_add(limit)
        .unwrap_or_else(Instant::now);
    prior.map_or(candidate, |current| current.min(candidate))
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalTimeConstrainedSuspensionFrame {
    suspended_body: rum_core::Expr,
    suspended_fallback: std::option::Option<rum_core::Expr>,
    args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalTimeConstrainedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_body = self.suspended_body.clone();
        #[allow(unused_mut)]
        let mut suspended_fallback = self.suspended_fallback.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(resolved) => (|| {
                session.resume_time_constrained_with_limit(
                    suspended_body,
                    suspended_fallback,
                    args.as_ref(),
                    resolved,
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct ResumeTimeConstrainedWithLimitSuspensionFrame {
    prior_deadline: std::option::Option<std::time::Instant>,
    fallback: std::option::Option<rum_core::Expr>,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeTimeConstrainedWithLimitSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut prior_deadline = self.prior_deadline.clone();
        #[allow(unused_mut)]
        let mut fallback = self.fallback.clone();
        let resumed = (|| session.finish_time_constrained(outcome, prior_deadline, fallback))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeTimeConstrainedWithLimitSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_time_deadline(self.prior_deadline);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalMemoryConstrainedSuspensionFrame {
    suspended_body: rum_core::Expr,
    suspended_fallback: std::option::Option<rum_core::Expr>,
    args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalMemoryConstrainedSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_body = self.suspended_body.clone();
        #[allow(unused_mut)]
        let mut suspended_fallback = self.suspended_fallback.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(resolved) => (|| {
                session.resume_memory_constrained_with_limit(
                    suspended_body,
                    suspended_fallback,
                    args.as_ref(),
                    resolved,
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct ResumeMemoryConstrainedWithLimitSuspensionFrame {
    prior_limit: std::option::Option<usize>,
    fallback: std::option::Option<rum_core::Expr>,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeMemoryConstrainedWithLimitSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut prior_limit = self.prior_limit.clone();
        #[allow(unused_mut)]
        let mut fallback = self.fallback.clone();
        let resumed = (|| session.finish_memory_constrained(outcome, prior_limit, fallback))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

impl crate::runtime::suspension::FrameAbandon for ResumeMemoryConstrainedWithLimitSuspensionFrame {
    fn abandon(
        self: std::sync::Arc<Self>,
        context: &mut crate::runtime::suspension::AbandonmentContext<'_>,
    ) -> crate::runtime::suspension::FrameAbandonAction {
        context.restore_memory_limit(self.prior_limit);
        crate::runtime::suspension::FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishTimeConstrainedSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishTimeConstrainedSuspensionFrame {
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

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishMemoryConstrainedSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishMemoryConstrainedSuspensionFrame {
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
