use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor, SlotWitness, slot_witness};
use crate::runtime::profiling::logical::{complete_phase_error_at_finish, pause_phase_at_finish};
use crate::runtime::suspension::{ContinueToken, TypedAction, TypedFrame};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
pub(in crate::runtime) struct SuspendedUnevaluatedRestorePlan {
    pub(in crate::runtime) prepared_args: SharedArgs,
    pub(in crate::runtime) wrappers: Arc<[(usize, Expr)]>,
}

impl SuspendedUnevaluatedRestorePlan {
    fn freeze(plan: UnevaluatedRestorePlan) -> Self {
        Self {
            prepared_args: plan.prepared_args,
            wrappers: Arc::from(plan.wrappers),
        }
    }
}

#[cold]
#[inline(never)]
fn suspend_prepared_stage_with_boundary(
    session: &mut Session,
    evaluated_head: SharedExpr,
    plan: SuspendedCallDispatchPlan,
    state: CallDispatchState,
    stage_index: usize,
    token: Option<crate::runtime::profiling::RuntimePathTimingToken>,
    witness: SlotWitness<PreparedCallArguments>,
    request: crate::runtime::RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    record_profile_runtime_path_timing(token, RuntimeProfileOutcome::InputRequired);
    witness.suspend(
        session,
        request,
        PreparedDispatchFrame::new(evaluated_head, plan, state, stage_index),
    )
}

#[cold]
#[inline(never)]
fn suspend_resumed_prepared_stage_with_boundary(
    session: &mut Session,
    evaluated_head: SharedExpr,
    plan: SuspendedCallDispatchPlan,
    state: CallDispatchState,
    stage_index: usize,
    token: Option<crate::runtime::profiling::RuntimePathTimingToken>,
    witness: SlotWitness<PreparedCallArguments>,
    request: crate::runtime::RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    record_profile_runtime_path_timing(token, RuntimeProfileOutcome::InputRequired);
    witness.suspend(
        session,
        request,
        PreparedDispatchFrame::new(evaluated_head, plan, state, stage_index),
    )
}

#[cold]
#[inline(never)]
fn suspend_call_stage_with_boundary(
    session: &mut Session,
    evaluated_head: SharedExpr,
    plan: CallDispatchPlan,
    state: CallDispatchState,
    stage_index: usize,
    unevaluated_restore: Option<Box<UnevaluatedRestorePlan>>,
    witness: SlotWitness<Option<Expr>>,
    request: crate::runtime::RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        ResumeCallWithDispatchPlanSuspensionFrame::freeze((
            evaluated_head,
            plan,
            state,
            stage_index,
            unevaluated_restore,
        )),
    )
}

#[cold]
#[inline(never)]
fn suspend_resumed_call_stage_with_boundary(
    session: &mut Session,
    evaluated_head: SharedExpr,
    plan: SuspendedCallDispatchPlan,
    state: CallDispatchState,
    stage_index: usize,
    unevaluated_restore: Option<SuspendedUnevaluatedRestorePlan>,
    witness: SlotWitness<Option<Expr>>,
    request: crate::runtime::RuntimeInputRequest,
) -> Result<Expr, EvalError> {
    witness.suspend(
        session,
        request,
        ResumeSuspendedCallWithDispatchPlanSuspensionFrame::freeze((
            evaluated_head,
            plan,
            state,
            stage_index,
            unevaluated_restore,
        )),
    )
}

impl Session {
    pub(super) fn evaluate_call(
        &mut self,
        head: SharedExpr,
        args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_total = runtime_path_timing_start_adopted(self, "evaluate_call.total");
        let runtime_profile_head =
            runtime_path_timing_start_adopted(self, "evaluate_call.head_eval");
        let head_kind = EvaluateAtomFastPathKind::classify(self, head.as_ref());
        let runtime_profile_fast_path =
            runtime_path_timing_start_adopted(self, "evaluate_atom_fast_path.total");
        let runtime_profile_kind =
            runtime_path_timing_start_adopted(self, head_kind.profile_name());
        // Only the evaluating-head branch needs a return boundary for the
        // witnessed macro. Stable heads recurse directly through the shared entry.
        let result = match head_kind {
            EvaluateAtomFastPathKind::StableSymbol | EvaluateAtomFastPathKind::Atom => {
                record_profile_runtime_path_timing_adopted_ok(self, runtime_profile_kind);
                record_profile_runtime_path_timing_adopted_ok(self, runtime_profile_fast_path);
                record_profile_runtime_path_timing_adopted_ok(self, runtime_profile_head);
                self.resume_call_after_head(head, args)
            }
            EvaluateAtomFastPathKind::UnstableSymbol | EvaluateAtomFastPathKind::Call => self
                .evaluate_unstable_call_head(
                    head,
                    args,
                    runtime_profile_kind,
                    runtime_profile_fast_path,
                    runtime_profile_head,
                ),
        };
        record_profile_runtime_path_timing_adopted(self, runtime_profile_total, &result);
        result
    }

    fn evaluate_unstable_call_head(
        &mut self,
        head: SharedExpr,
        args: SharedArgs,
        runtime_profile_kind: Option<crate::runtime::profiling::RuntimePathTimingToken>,
        runtime_profile_fast_path: Option<crate::runtime::profiling::RuntimePathTimingToken>,
        runtime_profile_head: Option<crate::runtime::profiling::RuntimePathTimingToken>,
    ) -> Result<Expr, EvalError> {
        let produced = self.evaluate(head.into_owned());
        let witness = slot_witness(&produced);
        record_profile_runtime_path_timing_adopted(self, runtime_profile_kind, &produced);
        record_profile_runtime_path_timing_adopted(self, runtime_profile_fast_path, &produced);
        record_profile_runtime_path_timing_adopted(self, runtime_profile_head, &produced);
        match produced {
            Ok(value) => self.resume_call_after_head(value, args),
            Err(EvalError::InputRequired { request }) => witness.suspend(
                self,
                request,
                EvaluateCallSuspensionFrame {
                    args_for_resume: args,
                },
            ),
            Err(error) => Err(error),
        }
    }

    pub(super) fn resume_call_after_head(
        &mut self,
        evaluated_head: impl Into<SharedExpr>,
        args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let evaluated_head = evaluated_head.into();
        let runtime_profile_total =
            runtime_path_timing_start_adopted(self, "evaluate_call.resume_after_head");
        let runtime_profile_plan = runtime_path_timing_start("evaluate_call.build_dispatch_plan");
        let plan = self.build_call_dispatch_plan(evaluated_head.as_ref());
        record_profile_runtime_path_timing(runtime_profile_plan, RuntimeProfileOutcome::Ok);
        let state = CallDispatchState::new(args);

        let runtime_profile_dispatch =
            runtime_path_timing_start_adopted(self, "evaluate_call.dispatch");
        let result = self.resume_call_with_dispatch_plan(evaluated_head, plan, state, 0, None);
        record_profile_runtime_path_timing_adopted(self, runtime_profile_dispatch, &result);
        record_profile_runtime_path_timing_adopted(self, runtime_profile_total, &result);
        result
    }

    pub(in crate::runtime) fn resume_call_with_dispatch_plan(
        &mut self,
        evaluated_head: impl Into<SharedExpr>,
        plan: CallDispatchPlan,
        mut state: CallDispatchState,
        mut stage_index: usize,
        mut unevaluated_restore: Option<Box<UnevaluatedRestorePlan>>,
    ) -> Result<Expr, EvalError> {
        let evaluated_head = evaluated_head.into();
        loop {
            if stage_index >= plan.len() {
                let final_args = match unevaluated_restore {
                    Some(restore) => {
                        let mut restored_args = restore.prepared_args.into_vec();
                        for (position, wrapper) in restore.wrappers {
                            restored_args[position] = wrapper;
                        }
                        self.normalize_call_args_shared(
                            evaluated_head.as_ref(),
                            SharedArgs::new(restored_args),
                            plan.attributes,
                        )
                    }
                    None => state.into_final_shared_args(),
                };
                return Ok(Expr::call_shared(evaluated_head, final_args));
            }

            let stage = plan.stage(stage_index);
            if stage == CallDispatchStage::PrepareArguments && state.is_prepared() {
                stage_index += 1;
                continue;
            }

            let runtime_profile_stage =
                runtime_path_timing_start_adopted(self, stage.profile_name());
            if stage == CallDispatchStage::PrepareArguments {
                let raw_args = state.take_raw_args();
                let produced = self.prepare_arguments_for_call_with_unevaluated_restore(
                    evaluated_head.as_ref().clone(),
                    plan.attributes,
                    raw_args,
                );
                let witness = slot_witness(&produced);
                debug_assert!(unevaluated_restore.is_none());
                unevaluated_restore = 'prepared_arguments: {
                    let failed: Result<Expr, EvalError> = match produced {
                        Err(EvalError::InputRequired { request }) => {
                            let failed = suspend_prepared_stage_with_boundary(
                                self,
                                evaluated_head,
                                plan.into_suspended(),
                                state,
                                stage_index,
                                runtime_profile_stage,
                                witness,
                                request,
                            );
                            pause_phase_at_finish(self, stage.profile_name());
                            return failed;
                        }
                        Err(error) => {
                            complete_phase_error_at_finish(self, &runtime_profile_stage, &error);
                            Err(error)
                        }
                        Ok(prepared) => {
                            break 'prepared_arguments self.install_prepared_call_arguments(
                                evaluated_head.as_ref(),
                                plan.attributes,
                                &mut state,
                                prepared,
                            );
                        }
                    };
                    record_profile_runtime_path_timing(
                        runtime_profile_stage,
                        runtime_profile_outcome_for_expr_result(&failed),
                    );
                    return failed;
                };
                record_profile_runtime_path_timing_adopted_none(self, runtime_profile_stage);
                stage_index += 1;
                continue;
            }
            let produced = self.run_call_dispatch_stage(
                stage,
                evaluated_head.as_ref(),
                plan.attributes,
                plan.builtin_spec,
                &mut state,
            );
            let witness = slot_witness(&produced);
            record_profile_runtime_path_timing_adopted_defer_pause(
                self,
                runtime_profile_stage,
                &produced,
            );
            return match produced {
                Ok(value) => match value {
                    Some(value) => Ok(value),
                    None => {
                        stage_index += 1;
                        continue;
                    }
                },
                Err(EvalError::InputRequired { request }) => {
                    let failed = suspend_call_stage_with_boundary(
                        self,
                        evaluated_head,
                        plan,
                        state,
                        stage_index,
                        unevaluated_restore,
                        witness,
                        request,
                    );
                    pause_phase_at_finish(self, stage.profile_name());
                    failed
                }
                Err(error) => Err(error),
            };
        }
    }

    pub(in crate::runtime) fn resume_suspended_call_with_dispatch_plan(
        &mut self,
        evaluated_head: SharedExpr,
        plan: SuspendedCallDispatchPlan,
        mut state: CallDispatchState,
        mut stage_index: usize,
        mut unevaluated_restore: Option<SuspendedUnevaluatedRestorePlan>,
    ) -> Result<Expr, EvalError> {
        loop {
            if stage_index >= plan.len() {
                let final_args = match unevaluated_restore {
                    Some(restore) => {
                        let mut restored_args = restore.prepared_args.into_vec();
                        for (position, wrapper) in restore.wrappers.iter() {
                            restored_args[*position] = wrapper.clone();
                        }
                        self.normalize_call_args_shared(
                            evaluated_head.as_ref(),
                            SharedArgs::new(restored_args),
                            plan.attributes,
                        )
                    }
                    None => state.into_final_shared_args(),
                };
                return Ok(Expr::call_shared(evaluated_head, final_args));
            }

            let stage = plan.stage(stage_index);
            if stage == CallDispatchStage::PrepareArguments && state.is_prepared() {
                stage_index += 1;
                continue;
            }

            let runtime_profile_stage =
                runtime_path_timing_start_adopted(self, stage.profile_name());
            if stage == CallDispatchStage::PrepareArguments {
                let raw_args = state.take_raw_args();
                let produced = self.prepare_arguments_for_call_with_unevaluated_restore(
                    evaluated_head.as_ref().clone(),
                    plan.attributes,
                    raw_args,
                );
                let witness = slot_witness(&produced);
                debug_assert!(unevaluated_restore.is_none());
                unevaluated_restore = 'prepared_arguments: {
                    let failed: Result<Expr, EvalError> = match produced {
                        Err(EvalError::InputRequired { request }) => {
                            let failed = suspend_resumed_prepared_stage_with_boundary(
                                self,
                                evaluated_head,
                                plan,
                                state,
                                stage_index,
                                runtime_profile_stage,
                                witness,
                                request,
                            );
                            pause_phase_at_finish(self, stage.profile_name());
                            return failed;
                        }
                        Err(error) => {
                            complete_phase_error_at_finish(self, &runtime_profile_stage, &error);
                            Err(error)
                        }
                        Ok(prepared) => {
                            break 'prepared_arguments self
                                .install_prepared_call_arguments(
                                    evaluated_head.as_ref(),
                                    plan.attributes,
                                    &mut state,
                                    prepared,
                                )
                                .map(|restore| SuspendedUnevaluatedRestorePlan::freeze(*restore));
                        }
                    };
                    record_profile_runtime_path_timing(
                        runtime_profile_stage,
                        runtime_profile_outcome_for_expr_result(&failed),
                    );
                    return failed;
                };
                record_profile_runtime_path_timing_adopted_none(self, runtime_profile_stage);
                stage_index += 1;
                continue;
            }
            let produced = self.run_call_dispatch_stage(
                stage,
                evaluated_head.as_ref(),
                plan.attributes,
                plan.builtin_spec,
                &mut state,
            );
            let witness = slot_witness(&produced);
            record_profile_runtime_path_timing_adopted_defer_pause(
                self,
                runtime_profile_stage,
                &produced,
            );
            return match produced {
                Ok(value) => match value {
                    Some(value) => Ok(value),
                    None => {
                        stage_index += 1;
                        continue;
                    }
                },
                Err(EvalError::InputRequired { request }) => {
                    let failed = suspend_resumed_call_stage_with_boundary(
                        self,
                        evaluated_head,
                        plan,
                        state,
                        stage_index,
                        unevaluated_restore,
                        witness,
                        request,
                    );
                    pause_phase_at_finish(self, stage.profile_name());
                    failed
                }
                Err(error) => Err(error),
            };
        }
    }

    fn install_prepared_call_arguments(
        &mut self,
        head: &Expr,
        attributes: AttributeSet,
        state: &mut CallDispatchState,
        prepared: PreparedCallArguments,
    ) -> Option<Box<UnevaluatedRestorePlan>> {
        state.set_prepared_args(self.normalize_call_args_shared(
            head,
            prepared.evaluated_args,
            attributes,
        ));
        prepared.unevaluated_restore
    }

    fn resume_prepared_call_dispatch_frame(
        &mut self,
        evaluated_head: SharedExpr,
        plan: SuspendedCallDispatchPlan,
        mut state: CallDispatchState,
        stage_index: usize,
        outcome: Result<PreparedCallArguments, EvalError>,
    ) -> Result<Expr, EvalError> {
        let prepared = outcome?;
        let restore = self
            .install_prepared_call_arguments(
                evaluated_head.as_ref(),
                plan.attributes,
                &mut state,
                prepared,
            )
            .map(|restore| SuspendedUnevaluatedRestorePlan::freeze(*restore));
        self.resume_suspended_call_with_dispatch_plan(
            evaluated_head,
            plan,
            state,
            stage_index + 1,
            restore,
        )
    }

    fn resume_suspended_call_dispatch_frame(
        &mut self,
        evaluated_head: SharedExpr,
        plan: SuspendedCallDispatchPlan,
        state: CallDispatchState,
        stage_index: usize,
        unevaluated_restore: Option<SuspendedUnevaluatedRestorePlan>,
        outcome: Result<Option<Expr>, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome? {
            Some(result) => Ok(result),
            None => self.resume_suspended_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
                unevaluated_restore,
            ),
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedUnevaluatedRestorePlan>();
};

impl CallDispatchStage {
    fn profile_name(self) -> &'static str {
        match self {
            CallDispatchStage::SpecialForm => "call.stage.special_form",
            CallDispatchStage::PrepareArguments => "call.stage.prepare_arguments",
            CallDispatchStage::Listable => "call.stage.listable",
            CallDispatchStage::UpValues => "call.stage.up_values",
            CallDispatchStage::DownValues => "call.stage.down_values",
            CallDispatchStage::SubValues => "call.stage.sub_values",
            CallDispatchStage::OrdinaryBuiltin => "call.stage.ordinary_builtin",
            CallDispatchStage::Function => "call.stage.function",
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvaluateCallSuspensionFrame {
    args_for_resume: rum_core::SharedArgs,
}

impl TypedFrame for EvaluateCallSuspensionFrame {
    type Input = Expr;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut args_for_resume = self.args_for_resume.clone();
        let resumed = match outcome {
            Ok(value) => (|| session.resume_call_after_head(value, args_for_resume))(),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeCallWithDispatchPlanSuspensionFrame {
    pub(super) evaluated_head: rum_core::SharedExpr,
    pub(super) plan: crate::runtime::session::session_state::SuspendedCallDispatchPlan,
    pub(super) state: crate::runtime::session::session_state::CallDispatchState,
    pub(super) stage_index: usize,
    pub(super) unevaluated_restore:
        std::option::Option<crate::runtime::eval_core::call::SuspendedUnevaluatedRestorePlan>,
}

impl ResumeCallWithDispatchPlanSuspensionFrame {
    #[cold]
    #[inline(never)]
    fn freeze(
        (evaluated_head, plan, state, stage_index, unevaluated_restore): (
            SharedExpr,
            CallDispatchPlan,
            CallDispatchState,
            usize,
            Option<Box<UnevaluatedRestorePlan>>,
        ),
    ) -> Self {
        Self {
            evaluated_head,
            plan: plan.into_suspended(),
            state,
            stage_index,
            unevaluated_restore: unevaluated_restore
                .map(|restore| SuspendedUnevaluatedRestorePlan::freeze(*restore)),
        }
    }
}

impl TypedFrame for ResumeCallWithDispatchPlanSuspensionFrame {
    type Input = Option<Expr>;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Option<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut evaluated_head = self.evaluated_head.clone();
        #[allow(unused_mut)]
        let mut plan = self.plan.clone();
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut stage_index = self.stage_index.clone();
        #[allow(unused_mut)]
        let mut unevaluated_restore = self.unevaluated_restore.clone();
        let resumed = session.resume_suspended_call_dispatch_frame(
            evaluated_head,
            plan,
            state,
            stage_index,
            unevaluated_restore,
            outcome,
        );
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeSuspendedCallWithDispatchPlanSuspensionFrame {
    pub(super) evaluated_head: rum_core::SharedExpr,
    pub(super) plan: crate::runtime::session::session_state::SuspendedCallDispatchPlan,
    pub(super) state: crate::runtime::session::session_state::CallDispatchState,
    pub(super) stage_index: usize,
    pub(super) unevaluated_restore:
        std::option::Option<crate::runtime::eval_core::call::SuspendedUnevaluatedRestorePlan>,
}

impl ResumeSuspendedCallWithDispatchPlanSuspensionFrame {
    #[cold]
    #[inline(never)]
    fn freeze(
        (evaluated_head, plan, state, stage_index, unevaluated_restore): (
            SharedExpr,
            SuspendedCallDispatchPlan,
            CallDispatchState,
            usize,
            Option<SuspendedUnevaluatedRestorePlan>,
        ),
    ) -> Self {
        Self {
            evaluated_head,
            plan,
            state,
            stage_index,
            unevaluated_restore,
        }
    }
}

impl TypedFrame for ResumeSuspendedCallWithDispatchPlanSuspensionFrame {
    type Input = Option<Expr>;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Option<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut evaluated_head = self.evaluated_head.clone();
        #[allow(unused_mut)]
        let mut plan = self.plan.clone();
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut stage_index = self.stage_index.clone();
        #[allow(unused_mut)]
        let mut unevaluated_restore = self.unevaluated_restore.clone();
        let resumed = session.resume_suspended_call_dispatch_frame(
            evaluated_head,
            plan,
            state,
            stage_index,
            unevaluated_restore,
            outcome,
        );
        TypedAction::complete(resumed)
    }
}

// T0025/T0026: this consumer is distinct from the Option<Expr> stage consumer.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct PreparedDispatchFrame {
    evaluated_head: SharedExpr,
    plan: SuspendedCallDispatchPlan,
    state: CallDispatchState,
    stage_index: usize,
}

impl PreparedDispatchFrame {
    #[cold]
    #[inline(never)]
    fn new(
        evaluated_head: SharedExpr,
        plan: SuspendedCallDispatchPlan,
        state: CallDispatchState,
        stage_index: usize,
    ) -> Self {
        Self {
            evaluated_head,
            plan,
            state,
            stage_index,
        }
    }
}

impl TypedFrame for PreparedDispatchFrame {
    type Input = PreparedCallArguments;
    type Output = Expr;

    fn resume<'resume>(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<PreparedCallArguments, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        TypedAction::complete(session.resume_prepared_call_dispatch_frame(
            self.evaluated_head.clone(),
            self.plan.clone(),
            self.state.clone(),
            self.stage_index,
            outcome,
        ))
    }
}
