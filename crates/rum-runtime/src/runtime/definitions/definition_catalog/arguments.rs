use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use crate::runtime::suspension::{
    ContinueToken, ResumeStop, SlotFinalizer, SlotWitness, TypedAction, TypedFrame, slot_witness,
};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct ResolvedArgumentAccumulator {
    prefix: SharedArgs,
    tail: AccumulatorChain<ResolvedArgumentNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct ResolvedArgumentNode {
    #[accumulator_link]
    previous: AccumulatorChain<ResolvedArgumentNode>,
    value: Expr,
}

impl ResolvedArgumentAccumulator {
    fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(ResolvedArgumentNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed argument accumulator must be unique"));
            suffix.push(current.value);
            node = current.previous.into_inner();
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedArgumentResolution {
    exprs: SharedArgs,
    cursor: usize,
    resolved: ResolvedArgumentAccumulator,
}

impl Session {
    pub(in crate::runtime) fn resolve_argument_exprs_owned(
        &mut self,
        exprs: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        self.resolve_argument_exprs_from_cursor(exprs, 0, Vec::new())
    }

    fn resolve_argument_exprs_from_cursor(
        &mut self,
        exprs: Vec<Expr>,
        mut cursor: usize,
        mut resolved: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        loop {
            let Some(expr) = exprs.get(cursor) else {
                return Ok(resolved);
            };

            crate::resume_typed_or_suspend!(
                self,
                self.evaluate(expr.clone()),
                ok(value) => {
                    resolved.push(value);
                    cursor += 1;
                    Ok(())
                },
                suspend => ResolveArgumentExprsFromCursorSuspensionFrame {
                    suspended: SuspendedArgumentResolution {
                        exprs: SharedArgs::new(exprs),
                        cursor,
                        resolved: ResolvedArgumentAccumulator::freeze(resolved),
                    },
                },
            )?;
        }
    }

    pub(in crate::runtime) fn resolve_argument_exprs_shared_from(
        &mut self,
        exprs: SharedArgs,
        mut cursor: usize,
    ) -> Result<Vec<Expr>, EvalError> {
        let mut resolved = Vec::new();
        loop {
            let Some(expr) = exprs.get(cursor) else {
                return Ok(resolved);
            };

            crate::resume_typed_or_suspend!(
                self,
                self.evaluate(expr.clone()),
                ok(value) => {
                    resolved.push(value);
                    cursor += 1;
                    Ok(())
                },
                suspend => ResolveArgumentExprsFromCursorSuspensionFrame {
                    suspended: SuspendedArgumentResolution {
                        exprs,
                        cursor,
                        resolved: ResolvedArgumentAccumulator::freeze(resolved),
                    },
                },
            )?;
        }
    }

    fn resume_suspended_argument_resolution<'resume>(
        &mut self,
        mut state: SuspendedArgumentResolution,
        value: Expr,
        token: ContinueToken<'resume, ResolveArgumentExprsFromCursorSuspensionFrame>,
    ) -> Result<ResolvedArgumentAccumulator, ResumeStop<Vec<Expr>>> {
        state.resolved = state.resolved.appended(value);
        state.cursor += 1;

        loop {
            let Some(expr) = state.exprs.get(state.cursor) else {
                return Ok(state.resolved);
            };

            let evaluated_result = self.evaluate(expr.clone());
            let witness: SlotWitness<
                <ResolveArgumentExprsFromCursorSuspensionFrame as TypedFrame>::Input,
            > = slot_witness(&evaluated_result);
            match evaluated_result {
                Ok(value) => {
                    state.resolved = state.resolved.appended(value);
                    state.cursor += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    return Err(token.suspend(
                        self,
                        witness,
                        request,
                        ResolveArgumentExprsFromCursorSuspensionFrame { suspended: state },
                    ));
                }
                Err(error) => return Err(ResumeStop::failure(error)),
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ResolvedArgumentAccumulator>();
    assert_traces::<SuspendedArgumentResolution>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResolveArgumentExprsFromCursorSuspensionFrame {
    pub(super) suspended:
        crate::runtime::definitions::definition_catalog::arguments::SuspendedArgumentResolution,
}

impl TypedFrame for ResolveArgumentExprsFromCursorSuspensionFrame {
    type Input = Expr;
    type Output = Vec<Expr>;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Vec<Expr>> {
        #[allow(unused_mut)]
        let mut suspended = self.suspended.clone();
        let value = match outcome {
            Ok(value) => value,
            Err(error) => return TypedAction::complete(Err(error)),
        };
        TypedAction::finish_work(
            session.resume_suspended_argument_resolution(suspended, value, token),
            |resolved| FinishArgumentResolutionFinalizer { resolved },
        )
    }
}

struct FinishArgumentResolutionFinalizer {
    resolved: ResolvedArgumentAccumulator,
}

impl SlotFinalizer for FinishArgumentResolutionFinalizer {
    type Output = Vec<Expr>;

    fn finish(self: Box<Self>) -> Result<Vec<Expr>, EvalError> {
        Ok(self.resolved.into_vec())
    }
}
