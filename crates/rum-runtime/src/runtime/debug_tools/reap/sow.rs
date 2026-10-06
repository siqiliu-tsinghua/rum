use super::*;
use crate::runtime::suspension::{FrameAction, SuspensionFrame};
use std::sync::Arc;

#[cfg(test)]
mod tests;

// Frozen only when a tag match asks for input. Reap scopes remain Session-owned.
#[derive(Clone, rum_derive::FrameRootTrace)]
struct SowMatchState {
    value: Expr,
    tag: Expr,
    scope_limit: usize,
    scope_index: usize,
    pattern_index: usize,
    pattern_count: usize,
    matched: Vec<(usize, usize)>,
}

impl Session {
    pub(in crate::runtime) fn eval_sow(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let (value, tag) = match args.as_ref() {
            [value] => (value.clone(), system_expr("None")),
            [value, tag] => (value.clone(), tag.clone()),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };
        let scope_limit = self.reap_scopes.len();
        let pattern_count = if scope_limit == 0 {
            0
        } else {
            self.reap_scopes[0].patterns.len()
        };
        self.drive_sow_matches(value, tag, scope_limit, 0, 0, pattern_count, Vec::new())
    }

    #[inline(always)]
    fn drive_sow_matches(
        &mut self,
        value: Expr,
        tag: Expr,
        scope_limit: usize,
        mut scope_index: usize,
        mut pattern_index: usize,
        mut pattern_count: usize,
        mut matched: Vec<(usize, usize)>,
    ) -> Result<Expr, EvalError> {
        while scope_index < scope_limit {
            while pattern_index < pattern_count {
                let pattern = self.reap_scopes[scope_index].patterns[pattern_index].clone();
                let outcome =
                    match self.match_expr_owned(pattern, tag.clone(), SymbolExprMap::default()) {
                        Ok(outcome) => outcome,
                        Err(EvalError::InputRequired { request }) => {
                            let state = SowMatchState {
                                value,
                                tag,
                                scope_limit,
                                scope_index,
                                pattern_index,
                                pattern_count,
                                matched,
                            };
                            return suspend_sow_match(self, state, request);
                        }
                        Err(error) => return Err(error),
                    };
                if outcome.matched {
                    matched.push((scope_index, pattern_index));
                }
                pattern_index += 1;
            }
            scope_index += 1;
            pattern_index = 0;
            if scope_index < scope_limit {
                pattern_count = self.reap_scopes[scope_index].patterns.len();
            }
        }

        // No bucket write is allowed until every enclosing match has completed.
        for (scope_index, pattern_index) in matched {
            let added_tag = record_sown_value(
                &mut self.reap_scopes[scope_index].buckets_by_pattern[pattern_index],
                tag.clone(),
                value.clone(),
            );
            if self.current_memory_limit.is_some() {
                self.charge_reap_occurrence(&value);
                if added_tag {
                    self.charge_reap_occurrence(&tag);
                }
            }
        }

        Ok(value)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct SowMatchSuspensionFrame {
    state: SowMatchState,
}

impl SuspensionFrame for SowMatchSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(match outcome {
            Ok(value) => resume_sow_match(session, &self.state, value),
            Err(error) => Err(error),
        })
    }
}

#[cold]
#[inline(never)]
fn suspend_sow_match(
    session: &mut Session,
    state: SowMatchState,
    request: crate::RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    crate::resume_or_suspend!(insert session, SowMatchSuspensionFrame { state });
    Err(EvalError::InputRequired { request })
}

#[cold]
fn resume_sow_match(
    session: &mut Session,
    frozen: &SowMatchState,
    value: Expr,
) -> Result<Expr, EvalError> {
    let outcome = CompiledMatchOutcome::from_continuation_expr(value);
    let mut state = frozen.clone();
    if outcome.matched {
        state.matched.push((state.scope_index, state.pattern_index));
    }
    state.pattern_index += 1;
    session.drive_sow_matches(
        state.value,
        state.tag,
        state.scope_limit,
        state.scope_index,
        state.pattern_index,
        state.pattern_count,
        state.matched,
    )
}
