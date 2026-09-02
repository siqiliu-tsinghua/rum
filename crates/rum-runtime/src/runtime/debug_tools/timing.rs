use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

const REPEATED_TIMING_SAMPLES: usize = 3;

#[derive(Clone, FrameRootTrace)]
struct RepeatedTimingEvalState {
    expr: Expr,
}

impl Session {
    pub(in crate::runtime) fn eval_absolute_time(
        &mut self,
        _head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("AbsoluteTime", args, 0));
        }

        Ok(unix_time_expr(SystemTime::now()))
    }

    pub(in crate::runtime) fn eval_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_timing_like("Timing", args)
    }

    pub(in crate::runtime) fn eval_absolute_timing(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_timing_like("AbsoluteTiming", args)
    }

    pub(in crate::runtime) fn eval_repeated_timing(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("RepeatedTiming", args, 1));
        };

        self.resume_repeated_timing_from_sample(
            RepeatedTimingEvalState { expr: expr.clone() },
            0,
            None,
            system_expr("Null"),
        )
    }

    fn eval_timing_like(&mut self, symbol: &str, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count(symbol, args, 1));
        };

        let started = Instant::now();
        let evaluated = self.evaluate(expr.clone());
        let value = crate::resume_or_suspend!(
            self,
            evaluated,
            ok(value) => Ok(value),
            suspend => {
        let symbol = std::sync::Arc::new(symbol.to_string());
        EvalTimingLikeSuspensionFrame { symbol, started }
        },
        )?;

        self.finish_timing_like(symbol, started, value)
    }

    fn finish_timing_like(
        &mut self,
        _symbol: &str,
        started: Instant,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("List"),
            vec![elapsed_duration_expr(started.elapsed()), value],
        ))
    }

    fn resume_repeated_timing_from_sample(
        &mut self,
        state: RepeatedTimingEvalState,
        sample: usize,
        best_elapsed: Option<Duration>,
        last_value: Expr,
    ) -> Result<Expr, EvalError> {
        if sample >= REPEATED_TIMING_SAMPLES {
            return Ok(Expr::call(
                system_expr("List"),
                vec![
                    elapsed_duration_expr(best_elapsed.unwrap_or_default()),
                    last_value,
                ],
            ));
        }

        let started = Instant::now();
        let value_result = self.evaluate(state.expr.clone());
        let value = crate::resume_or_suspend!(
            self,
            value_result,
            ok(value) => Ok(value),
            suspend => {
        let suspended_state = state.clone();
        ResumeRepeatedTimingFromSampleSuspensionFrame { suspended_state, sample, best_elapsed, started }
        },
        )?;

        self.resume_repeated_timing_with_value(state, sample, best_elapsed, started, value)
    }

    fn resume_repeated_timing_with_value(
        &mut self,
        state: RepeatedTimingEvalState,
        sample: usize,
        best_elapsed: Option<Duration>,
        started: Instant,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let elapsed = started.elapsed();
        let best_elapsed = Some(best_elapsed.map_or(elapsed, |best| best.min(elapsed)));
        self.resume_repeated_timing_from_sample(state, sample + 1, best_elapsed, value)
    }
}

pub(in crate::runtime) fn unix_time_expr(time: SystemTime) -> Expr {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let numerator = BigInt::from(duration.as_nanos());
            let denominator = BigInt::from(1_000_000_000_u64);
            Expr::rational(
                Rational::new(numerator, denominator)
                    .expect("positive denominator should always build a rational"),
            )
        }
        Err(error) => {
            let duration = error.duration();
            let numerator = -BigInt::from(duration.as_nanos());
            let denominator = BigInt::from(1_000_000_000_u64);
            Expr::rational(
                Rational::new(numerator, denominator)
                    .expect("positive denominator should always build a rational"),
            )
        }
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<RepeatedTimingEvalState>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalTimingLikeSuspensionFrame {
    symbol: std::sync::Arc<std::string::String>,
    started: std::time::Instant,
}

impl crate::runtime::suspension::SuspensionFrame for EvalTimingLikeSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut symbol = self.symbol.clone();
        #[allow(unused_mut)]
        let mut started = self.started.clone();
        let resumed = match outcome {
            Ok(value) => (|| session.finish_timing_like(symbol.as_str(), started, value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeRepeatedTimingFromSampleSuspensionFrame {
    pub(super) suspended_state: crate::runtime::debug_tools::timing::RepeatedTimingEvalState,
    pub(super) sample: usize,
    pub(super) best_elapsed: std::option::Option<std::time::Duration>,
    pub(super) started: std::time::Instant,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeRepeatedTimingFromSampleSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_state = self.suspended_state.clone();
        #[allow(unused_mut)]
        let mut sample = self.sample.clone();
        #[allow(unused_mut)]
        let mut best_elapsed = self.best_elapsed.clone();
        #[allow(unused_mut)]
        let mut started = self.started.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                session.resume_repeated_timing_with_value(
                    suspended_state,
                    sample,
                    best_elapsed,
                    started,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
