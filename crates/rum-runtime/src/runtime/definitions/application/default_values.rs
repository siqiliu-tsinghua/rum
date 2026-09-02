use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct SuspendedExactDefaultValueScan {
    down_values: Arc<[DownValue]>,
    lookup_order: SharedArgs,
    lookup_index: usize,
    candidate_index: usize,
    matched_index: usize,
}

impl Session {
    pub(super) fn try_apply_direct_default_value(
        &mut self,
        args: &[Expr],
        down_values: &[DownValue],
        has_guarded_default_values: bool,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(target) = args.first().cloned() else {
            return Ok(None);
        };

        let lookup_order = match args {
            [_] => vec![
                default_value_lhs(target.clone(), None),
                default_value_lhs(target, Some(1)),
            ],
            [_, index] => {
                let Some(index) = expr_as_non_negative_usize(index).filter(|index| *index > 0)
                else {
                    return Ok(None);
                };
                vec![
                    default_value_lhs(target.clone(), None),
                    default_value_lhs(target, Some(index)),
                ]
            }
            _ => return Ok(None),
        };

        self.evaluate_first_exact_default_value(
            down_values,
            lookup_order,
            has_guarded_default_values,
        )
    }

    pub(in crate::runtime) fn resolve_optional_implicit_default(
        &mut self,
        head: Expr,
        position: usize,
    ) -> Result<Option<Expr>, EvalError> {
        let default_symbol = Symbol::system(heads::patterns::DEFAULT);
        let Some(definition) = self.definitions.get(&default_symbol) else {
            return Ok(None);
        };
        let has_guarded_default_values = definition.has_non_ground_down_values();
        let down_values = definition.down_values.clone();

        self.evaluate_first_exact_default_value(
            &down_values,
            vec![
                default_value_lhs(head.clone(), Some(position)),
                default_value_lhs(head, None),
            ],
            has_guarded_default_values,
        )
    }

    fn evaluate_first_exact_default_value(
        &mut self,
        down_values: &[DownValue],
        lookup_order: Vec<Expr>,
        has_guarded_default_values: bool,
    ) -> Result<Option<Expr>, EvalError> {
        if !has_guarded_default_values {
            for lhs in lookup_order {
                let Some(rhs) = exact_down_value_rhs(down_values, &lhs) else {
                    continue;
                };
                return self.evaluate_definition_rhs(None, rhs).map(Some);
            }
            return Ok(None);
        }

        for (lookup_index, lhs) in lookup_order.iter().enumerate() {
            let Some((candidate_index, down_value)) = exact_down_value(down_values, lhs) else {
                continue;
            };
            if down_value.guards.is_empty() {
                return self
                    .evaluate_definition_rhs(None, down_value.rhs.clone())
                    .map(Some);
            }

            return self.evaluate_first_exact_default_value_from_cursor(
                down_values.to_vec(),
                lookup_order,
                lookup_index,
                candidate_index,
                None,
            );
        }

        Ok(None)
    }

    fn evaluate_first_exact_default_value_from_cursor(
        &mut self,
        down_values: Vec<DownValue>,
        lookup_order: Vec<Expr>,
        mut lookup_index: usize,
        mut candidate_index: usize,
        mut pending: Option<(usize, CompiledMatchOutcome)>,
    ) -> Result<Option<Expr>, EvalError> {
        loop {
            if let Some((index, outcome)) = pending.take() {
                if outcome.matched {
                    let rhs = down_values[index].rhs.clone();
                    return self.evaluate_definition_rhs(None, rhs).map(Some);
                }
                candidate_index = index + 1;
            }

            let Some(lhs) = lookup_order.get(lookup_index) else {
                return Ok(None);
            };
            let Some((relative_index, down_value)) =
                exact_down_value(&down_values[candidate_index..], lhs)
            else {
                lookup_index += 1;
                candidate_index = 0;
                continue;
            };
            let index = candidate_index + relative_index;

            if down_value.guards.is_empty() {
                return self
                    .evaluate_definition_rhs(None, down_value.rhs.clone())
                    .map(Some);
            }

            let guard_result = self.evaluate_guard_sequence_owned(
                down_value.guards.clone(),
                SymbolExprMap::default(),
                SymbolExprMap::default(),
                None,
            );
            match guard_result {
                Ok(outcome) => pending = Some((index, outcome)),
                Err(EvalError::InputRequired { request }) => {
                    return self.suspend_exact_default_value_scan_from_owned(
                        down_values,
                        lookup_order,
                        lookup_index,
                        candidate_index,
                        index,
                        request,
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_exact_default_value_scan_from_owned(
        &mut self,
        down_values: Vec<DownValue>,
        lookup_order: Vec<Expr>,
        lookup_index: usize,
        candidate_index: usize,
        matched_index: usize,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        self.suspend_exact_default_value_scan(
            SuspendedExactDefaultValueScan {
                down_values: Arc::from(down_values),
                lookup_order: SharedArgs::new(lookup_order),
                lookup_index,
                candidate_index,
                matched_index,
            },
            request,
        )
    }

    #[cold]
    #[inline(never)]
    fn suspend_exact_default_value_scan(
        &mut self,
        state: SuspendedExactDefaultValueScan,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        crate::resume_or_suspend!(insert self, EvaluateFirstExactDefaultValueFromCursorSuspensionFrame { state });
        Err(EvalError::InputRequired { request })
    }

    fn resume_suspended_exact_default_value_scan(
        &mut self,
        mut state: SuspendedExactDefaultValueScan,
        mut outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        loop {
            if outcome.matched {
                let rhs = state.down_values[state.matched_index].rhs.clone();
                return self.evaluate_definition_rhs(None, rhs).map(Some);
            }
            state.candidate_index = state.matched_index + 1;

            loop {
                let Some(lhs) = state.lookup_order.get(state.lookup_index) else {
                    return Ok(None);
                };
                let Some((relative_index, down_value)) =
                    exact_down_value(&state.down_values[state.candidate_index..], lhs)
                else {
                    state.lookup_index += 1;
                    state.candidate_index = 0;
                    continue;
                };
                state.matched_index = state.candidate_index + relative_index;

                if down_value.guards.is_empty() {
                    return self
                        .evaluate_definition_rhs(None, down_value.rhs.clone())
                        .map(Some);
                }

                let guard_result = self.evaluate_guard_sequence_owned(
                    down_value.guards.clone(),
                    SymbolExprMap::default(),
                    SymbolExprMap::default(),
                    None,
                );
                match guard_result {
                    Ok(next_outcome) => {
                        outcome = next_outcome;
                        break;
                    }
                    Err(EvalError::InputRequired { request }) => {
                        return self.suspend_exact_default_value_scan(state, request);
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }
}

fn exact_down_value<'a>(
    down_values: &'a [DownValue],
    lhs: &Expr,
) -> Option<(usize, &'a DownValue)> {
    down_values
        .iter()
        .enumerate()
        .find(|(_, down_value)| down_value.lhs == *lhs)
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedExactDefaultValueScan>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvaluateFirstExactDefaultValueFromCursorSuspensionFrame {
    pub(super) state:
        crate::runtime::definitions::application::default_values::SuspendedExactDefaultValueScan,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateFirstExactDefaultValueFromCursorSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .resume_suspended_exact_default_value_scan(state, outcome)
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
