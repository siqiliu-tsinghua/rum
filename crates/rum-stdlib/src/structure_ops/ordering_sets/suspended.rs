use super::*;
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
pub(super) struct SuspendedSetSameTestState {
    expr_head: Expr,
    same_test: Expr,
    scan: FrozenStructureSequence<Expr>,
    index: usize,
    kept: PersistentStructureSequence<Expr>,
    kind: FrozenStructureValue<SetOperationKind>,
    candidate: Option<Expr>,
    phase: SetSameTestPhase,
}

impl SuspendedSetSameTestState {
    pub(super) fn freeze(state: SetSameTestState) -> Self {
        Self {
            expr_head: state.expr_head,
            same_test: state.same_test,
            scan: FrozenStructureSequence::freeze(state.scan),
            index: state.index,
            kept: PersistentStructureSequence::freeze(state.kept),
            kind: FrozenStructureValue::freeze(state.kind),
            candidate: state.candidate,
            phase: state.phase,
        }
    }
}

fn resume_suspended_set_same_test(
    session: &mut Session,
    mut state: SuspendedSetSameTestState,
) -> Result<Expr, EvalError> {
    loop {
        match state.phase {
            SetSameTestPhase::Scan => {
                let Some(candidate) = state.scan.get(state.index).cloned() else {
                    return Ok(Expr::call(state.expr_head, state.kept.to_vec()));
                };
                state.candidate = Some(candidate);
                if state.kept.len() == 0 {
                    begin_suspended_set_same_test_membership(&mut state);
                } else {
                    state.phase = SetSameTestPhase::Deduplicate { kept_index: 0 };
                }
            }
            SetSameTestPhase::Deduplicate { kept_index } => {
                let Some(existing) = state.kept.get(kept_index) else {
                    begin_suspended_set_same_test_membership(&mut state);
                    continue;
                };
                let candidate = state
                    .candidate
                    .as_ref()
                    .expect("deduplication should have a candidate");
                let value =
                    match eval_binary_predicate(session, &state.same_test, candidate, existing) {
                        Ok(value) => value,
                        Err(EvalError::InputRequired { request }) => {
                            return rum_runtime::resume_or_suspend!(
                                session,
                                Err(EvalError::InputRequired { request }),
                                ok(value) => Ok(value),
                                suspend => ResumeSuspendedSetSameTestSuspensionFrame1 { state },
                            );
                        }
                        Err(error) => return Err(error),
                    };
                advance_suspended_set_same_test_after_value(&mut state, &value);
            }
            SetSameTestPhase::Intersection {
                group_index,
                item_index,
            } => {
                let SetOperationKind::Intersection { groups } = state.kind.as_ref() else {
                    unreachable!("intersection phase requires intersection state")
                };
                let Some(group) = groups.get(group_index) else {
                    finish_suspended_set_same_test_candidate(&mut state, true);
                    continue;
                };
                let Some(item) = group.get(item_index) else {
                    finish_suspended_set_same_test_candidate(&mut state, false);
                    continue;
                };
                let candidate = state
                    .candidate
                    .as_ref()
                    .expect("intersection should have a candidate");
                let value = match eval_binary_predicate(session, &state.same_test, candidate, item)
                {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return rum_runtime::resume_or_suspend!(
                            session,
                            Err(EvalError::InputRequired { request }),
                            ok(value) => Ok(value),
                            suspend => ResumeSuspendedSetSameTestSuspensionFrame2 { state },
                        );
                    }
                    Err(error) => return Err(error),
                };
                advance_suspended_set_same_test_after_value(&mut state, &value);
            }
            SetSameTestPhase::Complement { excluded_index } => {
                let SetOperationKind::Complement { excluded } = state.kind.as_ref() else {
                    unreachable!("complement phase requires complement state")
                };
                let Some(item) = excluded.get(excluded_index) else {
                    finish_suspended_set_same_test_candidate(&mut state, true);
                    continue;
                };
                let candidate = state
                    .candidate
                    .as_ref()
                    .expect("complement should have a candidate");
                let value = match eval_binary_predicate(session, &state.same_test, candidate, item)
                {
                    Ok(value) => value,
                    Err(EvalError::InputRequired { request }) => {
                        return rum_runtime::resume_or_suspend!(
                            session,
                            Err(EvalError::InputRequired { request }),
                            ok(value) => Ok(value),
                            suspend => ResumeSuspendedSetSameTestSuspensionFrame3 { state },
                        );
                    }
                    Err(error) => return Err(error),
                };
                advance_suspended_set_same_test_after_value(&mut state, &value);
            }
        }
    }
}

pub(super) fn resume_suspended_set_same_test_with_value(
    session: &mut Session,
    mut state: SuspendedSetSameTestState,
    value: Expr,
) -> Result<Expr, EvalError> {
    advance_suspended_set_same_test_after_value(&mut state, &value);
    resume_suspended_set_same_test(session, state)
}

fn begin_suspended_set_same_test_membership(state: &mut SuspendedSetSameTestState) {
    match state.kind.as_ref() {
        SetOperationKind::Union => finish_suspended_set_same_test_candidate(state, true),
        SetOperationKind::Intersection { .. } => {
            state.phase = SetSameTestPhase::Intersection {
                group_index: 0,
                item_index: 0,
            };
        }
        SetOperationKind::Complement { .. } => {
            state.phase = SetSameTestPhase::Complement { excluded_index: 0 };
        }
    }
}

fn advance_suspended_set_same_test_after_value(
    state: &mut SuspendedSetSameTestState,
    value: &Expr,
) {
    let matched = is_true_expr(value);
    match state.phase {
        SetSameTestPhase::Scan => unreachable!("scan phase does not evaluate SameTest"),
        SetSameTestPhase::Deduplicate { kept_index } => {
            if matched {
                finish_suspended_set_same_test_candidate(state, false);
            } else {
                state.phase = SetSameTestPhase::Deduplicate {
                    kept_index: kept_index + 1,
                };
            }
        }
        SetSameTestPhase::Intersection {
            group_index,
            item_index,
        } => {
            state.phase = if matched {
                SetSameTestPhase::Intersection {
                    group_index: group_index + 1,
                    item_index: 0,
                }
            } else {
                SetSameTestPhase::Intersection {
                    group_index,
                    item_index: item_index + 1,
                }
            };
        }
        SetSameTestPhase::Complement { excluded_index } => {
            if matched {
                finish_suspended_set_same_test_candidate(state, false);
            } else {
                state.phase = SetSameTestPhase::Complement {
                    excluded_index: excluded_index + 1,
                };
            }
        }
    }
}

fn finish_suspended_set_same_test_candidate(state: &mut SuspendedSetSameTestState, keep: bool) {
    let candidate = state
        .candidate
        .take()
        .expect("SameTest candidate should exist while finishing it");
    if keep {
        state.kept.push(candidate);
    }
    state.index += 1;
    state.phase = SetSameTestPhase::Scan;
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedSetSameTestState>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedSetSameTestSuspensionFrame1 {
    state: crate::structure_ops::ordering_sets::suspended::SuspendedSetSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedSetSameTestSuspensionFrame1 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_set_same_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedSetSameTestSuspensionFrame2 {
    state: crate::structure_ops::ordering_sets::suspended::SuspendedSetSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedSetSameTestSuspensionFrame2 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_set_same_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedSetSameTestSuspensionFrame3 {
    state: crate::structure_ops::ordering_sets::suspended::SuspendedSetSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedSetSameTestSuspensionFrame3 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_set_same_test_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
