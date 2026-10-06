use super::*;
mod leaf;
mod state;
#[cfg(test)]
mod tests;

use leaf::{finish_leaf, start_leaf};
pub(super) use state::TraceFilterState;
use state::{CandidateWork, FilterWork, LeafWork, LevelWork, PatternWork};

impl Session {
    pub(super) fn filter_trace_result(
        &mut self,
        trace: Expr,
        state: TraceEvalState,
        original_value: Expr,
    ) -> Result<Expr, EvalError> {
        let Expr::Call { head, args } = trace else {
            return self.finish_trace_filter_root(trace, state.output, original_value);
        };
        if !is_system_symbol_name(&head, "List") {
            return self.finish_trace_filter_root(
                Expr::call_shared(head, args),
                state.output,
                original_value,
            );
        }
        let work = FilterWork {
            mode: state.output,
            original_value,
            pattern: state.pattern,
            levels: state.levels,
            include_heads: state.include_heads,
            current: LevelWork::from_items(args.into_vec()),
            parents: Vec::new(),
        };
        self.drive_trace_filter(work, None)
    }

    fn drive_trace_filter(
        &mut self,
        mut work: FilterWork,
        mut pending: Option<LeafWork>,
    ) -> Result<Expr, EvalError> {
        loop {
            let mut leaf = if let Some(leaf) = pending.take() {
                leaf
            } else {
                loop {
                    match work.current.remaining.next() {
                        Some(Expr::Call { head, args }) if is_system_symbol_name(&head, "List") => {
                            let child = LevelWork::from_items(args.into_vec());
                            work.parents
                                .push(std::mem::replace(&mut work.current, child));
                        }
                        Some(item) => {
                            if let Some(leaf) = start_leaf(&mut work, item) {
                                break leaf;
                            }
                        }
                        None => {
                            let Some(mut parent) = work.parents.pop() else {
                                let trace = Expr::call(system_expr("List"), work.current.emitted);
                                return self.finish_trace_filter_root(
                                    trace,
                                    work.mode,
                                    work.original_value,
                                );
                            };
                            append_child(&mut parent.emitted, work.current.emitted);
                            work.current = parent;
                        }
                    }
                }
            };
            let matched = match self.match_trace_leaf(&work, &mut leaf) {
                Ok(matched) => matched,
                Err(EvalError::InputRequired { request }) => {
                    return suspend_trace_filter(self, work, leaf, request);
                }
                Err(error) => return Err(error),
            };
            finish_leaf(&mut work, leaf, matched);
        }
    }

    fn finish_trace_filter_root(
        &mut self,
        trace: Expr,
        mode: TraceOutput,
        original_value: Expr,
    ) -> Result<Expr, EvalError> {
        if mode == TraceOutput::Print {
            for line in render_structured_trace_lines(&trace) {
                self.push_print(line);
            }
            Ok(original_value)
        } else {
            Ok(trace)
        }
    }
}

fn append_child(emitted: &mut Vec<Expr>, nested: Vec<Expr>) {
    if nested.is_empty() {
        return;
    }
    if nested.len() == 1 {
        if let Expr::Call { head, args } = &nested[0] {
            if is_system_symbol_name(head, "List") {
                emitted.push(Expr::call_with_shared_args(
                    system_expr("List"),
                    args.clone(),
                ));
                return;
            }
        }
    }
    emitted.push(Expr::call(system_expr("List"), nested));
}

#[cold]
#[inline(never)]
fn suspend_trace_filter(
    session: &mut Session,
    work: FilterWork,
    leaf: LeafWork,
    request: crate::RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    let state = work.freeze_filter(leaf);
    crate::resume_or_suspend!(insert session, TraceFilterSuspensionFrame { state });
    Err(EvalError::InputRequired { request })
}

#[cold]
pub(super) fn resume_trace_filter(
    session: &mut Session,
    frozen: &TraceFilterState,
    value: Expr,
) -> Result<Expr, EvalError> {
    let outcome = CompiledMatchOutcome::from_continuation_expr(value);
    let (mut work, mut leaf) = frozen.thaw_filter();
    let pending = if leaf.apply_match(outcome.matched, work.mode == TraceOutput::Detail) {
        finish_leaf(&mut work, leaf, true);
        None
    } else {
        Some(leaf)
    };
    session.drive_trace_filter(work, pending)
}
