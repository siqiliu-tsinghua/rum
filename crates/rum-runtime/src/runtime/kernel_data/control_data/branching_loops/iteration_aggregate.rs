use super::iteration::{
    DoSpec, IterationConsumer, PersistentExprAccumulator, SuspendedDoSpec,
    build_iteration_body_or_nested, evaluate_do_spec_expr, parse_do_spec,
};
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum IterationAggregate {
    Product,
    Sum,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedIterationAggregate {
    aggregate: IterationAggregate,
    args: SharedArgs,
    spec_index: usize,
    spec: SuspendedDoSpec,
    iteration: usize,
    collected: PersistentExprAccumulator,
}

impl IterationAggregate {
    fn name(self) -> &'static str {
        match self {
            Self::Product => "Product",
            Self::Sum => "Sum",
        }
    }

    fn combiner_name(self) -> &'static str {
        match self {
            Self::Product => "Times",
            Self::Sum => "Plus",
        }
    }

    fn consumer(self) -> IterationConsumer {
        match self {
            Self::Product => IterationConsumer::Product,
            Self::Sum => IterationConsumer::Sum,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_product(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        self.eval_iteration_aggregate(IterationAggregate::Product, args)
    }

    pub(in crate::runtime) fn eval_sum(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        self.eval_iteration_aggregate(IterationAggregate::Sum, args)
    }

    fn eval_iteration_aggregate(
        &mut self,
        aggregate: IterationAggregate,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                aggregate.name(),
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_iteration_aggregate_after_spec_eval(aggregate, args, 0)
    }

    fn resume_iteration_aggregate_after_spec_eval(
        &mut self,
        aggregate: IterationAggregate,
        args: &SharedArgs,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = crate::resume_or_suspend!(
            self,
            spec_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        ResumeIterationAggregateAfterSpecEvalSuspensionFrame { aggregate, args, spec_index }
        },
        )?;

        self.finish_iteration_aggregate_spec_eval(aggregate, args, spec_index, spec_expr)
    }

    fn finish_iteration_aggregate_spec_eval(
        &mut self,
        aggregate: IterationAggregate,
        args: &SharedArgs,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let spec = match parse_do_spec(self, &spec_expr, aggregate.consumer())? {
            Some(spec) => spec,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    aggregate.name(),
                    args,
                    "expected a supported repetition count or iterator specification",
                ));
            }
        };

        let capacity = spec.iteration_count();
        self.resume_iteration_aggregate_iterations(
            aggregate,
            args,
            spec_index,
            spec,
            0,
            Vec::with_capacity(capacity),
        )
    }

    fn resume_iteration_aggregate_iterations(
        &mut self,
        aggregate: IterationAggregate,
        args: &SharedArgs,
        spec_index: usize,
        spec: DoSpec,
        mut iteration: usize,
        mut collected: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &spec {
                DoSpec::Count(count) => {
                    if iteration >= *count {
                        return self.finish_iteration_aggregate(aggregate, collected);
                    }
                    let body = build_iteration_body_or_nested(args, spec_index, aggregate.name());
                    self.evaluate(body)
                }
                DoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(iteration).cloned() else {
                        return self.finish_iteration_aggregate(aggregate, collected);
                    };
                    let body = build_iteration_body_or_nested(args, spec_index, aggregate.name());
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(value) => {
                    collected.push(value);
                    iteration += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedIterationAggregate {
                        aggregate,
                        args: args.clone(),
                        spec_index,
                        spec: spec.into(),
                        iteration,
                        collected: PersistentExprAccumulator::freeze(collected),
                    };
                    crate::resume_or_suspend!(insert self, ResumeIterationAggregateIterationsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_iteration_aggregate_after_iteration(
        &mut self,
        mut state: SuspendedIterationAggregate,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                state.collected = state.collected.appended(value);
                state.iteration += 1;
                self.resume_suspended_iteration_aggregate(state)
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_iteration_aggregate(
        &mut self,
        mut state: SuspendedIterationAggregate,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &state.spec {
                SuspendedDoSpec::Count(count) => {
                    if state.iteration >= *count {
                        return self.finish_iteration_aggregate(
                            state.aggregate,
                            state.collected.into_vec(),
                        );
                    }
                    let body = build_iteration_body_or_nested(
                        state.args.as_ref(),
                        state.spec_index,
                        state.aggregate.name(),
                    );
                    self.evaluate(body)
                }
                SuspendedDoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(state.iteration).cloned() else {
                        return self.finish_iteration_aggregate(
                            state.aggregate,
                            state.collected.into_vec(),
                        );
                    };
                    let body = build_iteration_body_or_nested(
                        state.args.as_ref(),
                        state.spec_index,
                        state.aggregate.name(),
                    );
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(value) => {
                    state.collected = state.collected.appended(value);
                    state.iteration += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedIterationAggregateSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn finish_iteration_aggregate(
        &mut self,
        aggregate: IterationAggregate,
        values: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.evaluate(Expr::call(system_expr(aggregate.combiner_name()), values))
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedIterationAggregate>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeIterationAggregateAfterSpecEvalSuspensionFrame {
    pub(super) aggregate: crate::runtime::kernel_data::control_data::branching_loops::iteration_aggregate::IterationAggregate,
    pub(super) args: rum_core::SharedArgs,
    pub(super) spec_index: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeIterationAggregateAfterSpecEvalSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut aggregate = self.aggregate.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        let resumed = match outcome {
            Ok(spec_expr) => (|| {
                session
                    .finish_iteration_aggregate_spec_eval(aggregate, &args, spec_index, spec_expr)
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeIterationAggregateIterationsSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::control_data::branching_loops::iteration_aggregate::SuspendedIterationAggregate,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeIterationAggregateIterationsSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed =
            (|| session.resume_suspended_iteration_aggregate_after_iteration(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeSuspendedIterationAggregateSuspensionFrame {
    pub(super) state: crate::runtime::kernel_data::control_data::branching_loops::iteration_aggregate::SuspendedIterationAggregate,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedIterationAggregateSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed =
            (|| session.resume_suspended_iteration_aggregate_after_iteration(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
