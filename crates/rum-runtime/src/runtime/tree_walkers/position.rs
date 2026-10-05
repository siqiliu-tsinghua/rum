mod options;
mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct PersistentPositionAccumulator {
    prefix: SharedArgs,
    tail: AccumulatorChain<PersistentPositionNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct PersistentPositionNode {
    #[accumulator_link]
    previous: AccumulatorChain<PersistentPositionNode>,
    values: SharedArgs,
}

impl PersistentPositionAccumulator {
    fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    fn appended_values(&self, values: Vec<Expr>) -> Self {
        if values.is_empty() {
            return self.clone();
        }
        let added = values.len();
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(PersistentPositionNode {
                previous: self.tail.clone(),
                values: SharedArgs::new(values),
            }))),
            len: self.len + added,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        self.appended_values(vec![value])
    }

    fn len(&self) -> usize {
        self.len
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::new();
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            match Arc::try_unwrap(current) {
                Ok(current) => {
                    suffix.push(current.values.into_vec());
                    node = current.previous.into_inner();
                }
                Err(current) => {
                    suffix.push(current.values.as_ref().to_vec());
                    node = current.previous.clone().into_inner();
                }
            }
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(self.len.saturating_sub(values.len()));
        for chunk in suffix {
            values.extend(chunk);
        }
        values
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedPositionTraversal {
    walker: PostOrderPathTraversal,
    pattern: Expr,
    traversal: CasesTraversalConfig,
    positions: PersistentPositionAccumulator,
    current_path: Option<usize>,
    current_depth: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_position(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() == 1 {
            return Ok(self.invalid_call("Position", args_slice));
        }
        if !(2..=5).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("Position", args_slice, 2, 5));
        }

        let target_result = self.evaluate(args_slice[0].clone());
        let target = crate::resume_or_suspend!(
            self,
            target_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        EvalPositionSuspensionFrame { args }
        },
        )?;

        self.resume_position_after_target(target, args)
    }

    fn resume_position_after_target(
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
        ResumePositionAfterTargetSuspensionFrame { target, args }
        },
        )?;

        self.resume_position_after_pattern(target, pattern, args)
    }

    fn resume_position_after_pattern(
        &mut self,
        target: Expr,
        pattern: Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        self.system_option_bool(
            "Position",
            "Heads",
            true,
            options::PositionOptionWorking {
                target,
                pattern,
                args,
            },
        )
    }

    fn finish_position_after_resolved_traversal(
        &mut self,
        target: Expr,
        pattern: Expr,
        traversal: CasesTraversalResolution,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let traversal = match traversal {
            CasesTraversalResolution::Valid(traversal) => traversal,
            CasesTraversalResolution::Invalid(error) => {
                return Ok(self.invalid_cases_traversal_call("Position", args.as_ref(), error));
            }
        };

        if traversal.limit_reached(0) {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let positions_result = self.collect_positions_owned(target, pattern, traversal);
        crate::resume_or_suspend!(
            self,
            positions_result,
            ok(outcome) => { (|| {
                    Ok(Expr::call(system_expr("List"), outcome.positions))
                })() },
            suspend => FinishPositionAfterResolvedTraversalSuspensionFrame,
        )
    }

    fn collect_positions_owned(
        &mut self,
        expr: Expr,
        pattern: Expr,
        traversal: CasesTraversalConfig,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        let mut walker = PostOrderPathTraversal::new(expr, traversal.include_heads, 0);
        let mut positions = Vec::new();
        let mut current_depth = 1;

        while let Some(node) = walker.next() {
            current_depth = node.depth;
            if traversal.limit_reached(positions.len()) {
                return Ok(PositionCollectionOutcome::new(positions, node.depth, true));
            }
            if !traversal.levels.matches(node.level, node.depth) {
                continue;
            }

            let match_result =
                self.match_expr_owned(pattern.clone(), node.expr, SymbolExprMap::default());
            match match_result {
                Ok(outcome) => {
                    if outcome.matched {
                        positions.push(walker.position_expr(node.path));
                        if traversal.limit_reached(positions.len()) {
                            return Ok(PositionCollectionOutcome::new(positions, node.depth, true));
                        }
                    }
                }
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedPositionTraversal {
                        walker,
                        pattern,
                        traversal,
                        positions: PersistentPositionAccumulator::freeze(positions),
                        current_path: node.path,
                        current_depth: node.depth,
                    };
                    crate::resume_or_suspend!(insert self, ContinuePositionTraversalSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }

        Ok(PositionCollectionOutcome::new(
            positions,
            current_depth,
            false,
        ))
    }

    fn resume_position_traversal(
        &mut self,
        mut state: SuspendedPositionTraversal,
        outcome: CompiledMatchOutcome,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        if outcome.matched {
            state.positions = state
                .positions
                .appended(state.walker.position_expr(state.current_path));
            if state.traversal.limit_reached(state.positions.len()) {
                return Ok(PositionCollectionOutcome::new(
                    state.positions.into_vec(),
                    state.current_depth,
                    true,
                ));
            }
        }
        self.drive_suspended_position_traversal(state)
    }

    fn drive_suspended_position_traversal(
        &mut self,
        mut state: SuspendedPositionTraversal,
    ) -> Result<PositionCollectionOutcome, EvalError> {
        while let Some(node) = state.walker.next() {
            state.current_path = node.path;
            state.current_depth = node.depth;
            if state.traversal.limit_reached(state.positions.len()) {
                return Ok(PositionCollectionOutcome::new(
                    state.positions.into_vec(),
                    node.depth,
                    true,
                ));
            }
            if !state.traversal.levels.matches(node.level, node.depth) {
                continue;
            }
            let match_result =
                self.match_expr_owned(state.pattern.clone(), node.expr, SymbolExprMap::default());
            match match_result {
                Ok(outcome) => {
                    if outcome.matched {
                        state.positions = state
                            .positions
                            .appended(state.walker.position_expr(node.path));
                        if state.traversal.limit_reached(state.positions.len()) {
                            return Ok(PositionCollectionOutcome::new(
                                state.positions.into_vec(),
                                node.depth,
                                true,
                            ));
                        }
                    }
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ContinuePositionTraversalSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }

        Ok(PositionCollectionOutcome::new(
            state.positions.into_vec(),
            state.current_depth,
            false,
        ))
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PersistentPositionAccumulator>();
    assert_traces::<SuspendedPositionTraversal>();
};
