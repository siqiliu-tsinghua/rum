use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
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

impl Session {
    pub(super) fn evaluate_call(
        &mut self,
        head: SharedExpr,
        args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_total = runtime_path_timing_start("evaluate_call.total");
        let runtime_profile_head = runtime_path_timing_start("evaluate_call.head_eval");
        let head_kind = EvaluateAtomFastPathKind::classify(self, head.as_ref());
        let runtime_profile_fast_path = runtime_path_timing_start("evaluate_atom_fast_path.total");
        let runtime_profile_kind = runtime_path_timing_start(head_kind.profile_name());
        let evaluated_head_result = match head_kind {
            EvaluateAtomFastPathKind::StableSymbol | EvaluateAtomFastPathKind::Atom => Ok(head),
            EvaluateAtomFastPathKind::UnstableSymbol | EvaluateAtomFastPathKind::Call => {
                self.evaluate(head.into_owned()).map(SharedExpr::new)
            }
        };
        record_profile_runtime_path_timing(
            runtime_profile_kind,
            runtime_profile_outcome_for_shared_expr_result(&evaluated_head_result),
        );
        record_profile_runtime_path_timing(
            runtime_profile_fast_path,
            runtime_profile_outcome_for_shared_expr_result(&evaluated_head_result),
        );
        record_profile_runtime_path_timing(
            runtime_profile_head,
            runtime_profile_outcome_for_shared_expr_result(&evaluated_head_result),
        );
        let evaluated_head = match evaluated_head_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                let result = self.suspend_evaluate_call(args, request);
                record_profile_runtime_path_timing(
                    runtime_profile_total,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }
            Err(error) => {
                let result = Err(error);
                record_profile_runtime_path_timing(
                    runtime_profile_total,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }
        };
        let result = self.resume_call_after_head(evaluated_head, args);
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    #[cold]
    #[inline(never)]
    fn suspend_evaluate_call(
        &mut self,
        args_for_resume: SharedArgs,
        request: RuntimeInputRequest,
    ) -> Result<Expr, EvalError> {
        crate::resume_or_suspend!(insert self, EvaluateCallSuspensionFrame { args_for_resume });
        Err(EvalError::InputRequired { request })
    }

    pub(super) fn resume_call_after_head(
        &mut self,
        evaluated_head: SharedExpr,
        args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_total = runtime_path_timing_start("evaluate_call.resume_after_head");
        let runtime_profile_plan = runtime_path_timing_start("evaluate_call.build_dispatch_plan");
        let plan = self.build_call_dispatch_plan(evaluated_head.as_ref());
        record_profile_runtime_path_timing(runtime_profile_plan, RuntimeProfileOutcome::Ok);
        let state = CallDispatchState::new(args);

        let runtime_profile_dispatch = runtime_path_timing_start("evaluate_call.dispatch");
        let result = self.resume_call_with_dispatch_plan(evaluated_head, plan, state, 0, None);
        record_profile_runtime_path_timing(
            runtime_profile_dispatch,
            runtime_profile_outcome_for_expr_result(&result),
        );
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(in crate::runtime) fn resume_call_with_dispatch_plan(
        &mut self,
        evaluated_head: impl Into<SharedExpr>,
        plan: CallDispatchPlan,
        state: CallDispatchState,
        stage_index: usize,
        unevaluated_restore: Option<Box<UnevaluatedRestorePlan>>,
    ) -> Result<Expr, EvalError> {
        let evaluated_head = evaluated_head.into();
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
        let mut state = state;
        if stage == CallDispatchStage::PrepareArguments && state.is_prepared() {
            return self.resume_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
                unevaluated_restore,
            );
        }

        let runtime_profile_stage =
            runtime_path_timing_start(call_dispatch_stage_profile_name(stage));
        let mut unevaluated_restore = unevaluated_restore;
        let stage_result = if stage == CallDispatchStage::PrepareArguments {
            let raw_args = state.take_raw_args();
            self.prepare_arguments_for_call_with_unevaluated_restore(
                evaluated_head.as_ref().clone(),
                plan.attributes,
                raw_args,
            )
            .map(|prepared| {
                state.set_prepared_args(self.normalize_call_args_shared(
                    evaluated_head.as_ref(),
                    prepared.evaluated_args,
                    plan.attributes,
                ));
                unevaluated_restore = prepared.unevaluated_restore;
                None
            })
        } else {
            self.run_call_dispatch_stage(
                stage,
                evaluated_head.as_ref(),
                plan.attributes,
                plan.builtin_spec,
                &mut state,
            )
        };
        record_profile_runtime_path_timing(
            runtime_profile_stage,
            runtime_profile_outcome_for_optional_result(&stage_result),
        );

        match stage_result {
            Ok(Some(result)) => Ok(result),
            Ok(None) => self.resume_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
                unevaluated_restore,
            ),
            Err(EvalError::InputRequired { request }) => {
                let unevaluated_restore = unevaluated_restore
                    .map(|restore| SuspendedUnevaluatedRestorePlan::freeze(*restore));
                crate::resume_or_suspend!(insert self, ResumeCallWithDispatchPlanSuspensionFrame { evaluated_head, plan: plan.into_suspended(), state, stage_index, unevaluated_restore });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn resume_suspended_call_with_dispatch_plan(
        &mut self,
        evaluated_head: SharedExpr,
        plan: SuspendedCallDispatchPlan,
        mut state: CallDispatchState,
        stage_index: usize,
        unevaluated_restore: Option<SuspendedUnevaluatedRestorePlan>,
    ) -> Result<Expr, EvalError> {
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
            return self.resume_suspended_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
                unevaluated_restore,
            );
        }

        let runtime_profile_stage =
            runtime_path_timing_start(call_dispatch_stage_profile_name(stage));
        let mut unevaluated_restore = unevaluated_restore;
        let stage_result = if stage == CallDispatchStage::PrepareArguments {
            let raw_args = state.take_raw_args();
            self.prepare_arguments_for_call_with_unevaluated_restore(
                evaluated_head.as_ref().clone(),
                plan.attributes,
                raw_args,
            )
            .map(|prepared| {
                state.set_prepared_args(self.normalize_call_args_shared(
                    evaluated_head.as_ref(),
                    prepared.evaluated_args,
                    plan.attributes,
                ));
                unevaluated_restore = prepared
                    .unevaluated_restore
                    .map(|restore| SuspendedUnevaluatedRestorePlan::freeze(*restore));
                None
            })
        } else {
            self.run_call_dispatch_stage(
                stage,
                evaluated_head.as_ref(),
                plan.attributes,
                plan.builtin_spec,
                &mut state,
            )
        };
        record_profile_runtime_path_timing(
            runtime_profile_stage,
            runtime_profile_outcome_for_optional_result(&stage_result),
        );

        match stage_result {
            Ok(Some(result)) => Ok(result),
            Ok(None) => self.resume_suspended_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
                unevaluated_restore,
            ),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeSuspendedCallWithDispatchPlanSuspensionFrame { evaluated_head, plan, state, stage_index, unevaluated_restore });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_call_dispatch_frame(
        &mut self,
        evaluated_head: SharedExpr,
        plan: SuspendedCallDispatchPlan,
        mut state: CallDispatchState,
        stage_index: usize,
        unevaluated_restore: Option<SuspendedUnevaluatedRestorePlan>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let value = outcome?;
        if plan.stage(stage_index) == CallDispatchStage::PrepareArguments {
            match prepared_call_arguments_from_continuation_expr(value) {
                Ok(prepared) => {
                    debug_assert!(unevaluated_restore.is_none());
                    state.set_prepared_args(self.normalize_call_args_shared(
                        evaluated_head.as_ref(),
                        prepared.evaluated_args,
                        plan.attributes,
                    ));
                    let unevaluated_restore = prepared
                        .unevaluated_restore
                        .map(|restore| SuspendedUnevaluatedRestorePlan::freeze(*restore));
                    return self.resume_suspended_call_with_dispatch_plan(
                        evaluated_head,
                        plan,
                        state,
                        stage_index + 1,
                        unevaluated_restore,
                    );
                }
                Err(value) => {
                    return Ok(Option::<Expr>::from_continuation_expr(value)
                        .expect("a completed prepare stage returns its call result"));
                }
            }
        }

        match Option::<Expr>::from_continuation_expr(value) {
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

fn call_dispatch_stage_profile_name(stage: CallDispatchStage) -> &'static str {
    match stage {
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

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvaluateCallSuspensionFrame {
    args_for_resume: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvaluateCallSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args_for_resume = self.args_for_resume.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| session.resume_call_after_head(SharedExpr::new(value), args_for_resume))()
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame for ResumeCallWithDispatchPlanSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
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
        crate::runtime::suspension::FrameAction::complete(resumed)
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

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedCallWithDispatchPlanSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
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
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
