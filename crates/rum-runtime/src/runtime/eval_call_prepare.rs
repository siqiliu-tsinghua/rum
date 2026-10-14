mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{
    AccumulatorChain, AccumulatorChainNode, FrameRootTrace, FrameRootVisitor,
};
use crate::runtime::suspension::{
    ContinueToken, ResumeStop, SlotWitness, TypedFrame, slot_witness,
};
use std::sync::Arc;

pub(super) struct PreparedCallArguments {
    pub(super) evaluated_args: SharedArgs,
    pub(super) unevaluated_restore: Option<Box<UnevaluatedRestorePlan>>,
}

impl PreparedCallArguments {
    // The existing Legacy wire; typed frames use this same single provider.
    pub(in crate::runtime) fn into_continuation_expr(self) -> Expr {
        let restores = self
            .unevaluated_restore
            .map(|restore| restore.wrappers)
            .unwrap_or_default()
            .into_iter()
            .map(|(position, wrapper)| {
                Expr::call(
                    system_expr("InternalContinuationUnevaluatedRestore"),
                    vec![Expr::integer(BigInt::from(position)), wrapper],
                )
            })
            .collect();
        Expr::call(
            system_expr("InternalContinuationPreparedCallArguments"),
            vec![
                Expr::call_shared(SharedExpr::new(system_expr("List")), self.evaluated_args),
                Expr::call(system_expr("List"), restores),
            ],
        )
    }
}

#[derive(FrameRootTrace)]
pub(super) struct UnevaluatedRestorePlan {
    pub(super) prepared_args: SharedArgs,
    pub(super) wrappers: Vec<(usize, Expr)>,
}

#[derive(Clone, FrameRootTrace)]
struct UnevaluatedRestoreAccumulator {
    prefix: Arc<[(usize, Expr)]>,
    tail: AccumulatorChain<UnevaluatedRestoreNode>,
    len: usize,
}

#[derive(FrameRootTrace, AccumulatorChainNode)]
struct UnevaluatedRestoreNode {
    #[accumulator_link]
    previous: AccumulatorChain<UnevaluatedRestoreNode>,
    value: (usize, Expr),
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedCallArgumentPreparation {
    head: Expr,
    attributes: AttributeSet,
    args: SharedArgs,
    cursor: usize,
    prepared: PreparedArgumentAccumulator,
    unevaluated_restores: Option<UnevaluatedRestoreAccumulator>,
}

struct SuspendedPreparedCallArguments {
    evaluated_args: PreparedArgumentAccumulator,
    unevaluated_restores: Option<UnevaluatedRestoreAccumulator>,
}

impl UnevaluatedRestoreAccumulator {
    fn freeze(values: Vec<(usize, Expr)>) -> Self {
        let len = values.len();
        Self {
            prefix: Arc::from(values),
            tail: AccumulatorChain::default(),
            len,
        }
    }

    fn appended(&self, value: (usize, Expr)) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: AccumulatorChain::new(Some(Arc::new(UnevaluatedRestoreNode {
                previous: self.tail.clone(),
                value,
            }))),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<(usize, Expr)> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail.into_inner();
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed Unevaluated restore state must be unique"));
            suffix.push(current.value);
            node = current.previous.into_inner();
        }
        suffix.reverse();

        let mut values = self.prefix.iter().cloned().collect::<Vec<_>>();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

#[cfg(test)]
mod abandonment_tests {
    use super::*;
    use crate::runtime::suspension::AbandonReason;
    use crate::runtime::test_builtins::session_with_test_control_builtins;

    #[test]
    fn abandoning_prepared_arguments_and_restores_skips_remaining_arguments() {
        let mut session = session_with_test_control_builtins();
        let first = session
            .eval_request(
                "hits = 0; f[InputString[\"first\"], Unevaluated[kept], \
                 InputString[\"stop\"], hits++]",
            )
            .unwrap_err();
        assert!(matches!(
            first.error,
            SessionRequestError::InputRequired { request } if request.prompt == "first"
        ));
        let second = session.resume_eval_request_with_input("ready").unwrap_err();
        assert!(matches!(
            second.error,
            SessionRequestError::InputRequired { request } if request.prompt == "stop"
        ));
        assert_eq!(
            session.abandon_pending_suspension(AbandonReason::CancelledRequest),
            AbandonReason::CancelledRequest
        );
        assert!(session.pending_input_request().is_none());
        assert_eq!(session.eval_text("hits").unwrap().value.to_string(), "0");
    }
}

fn finish_suspended_prepared_call(state: SuspendedPreparedCallArguments) -> PreparedCallArguments {
    let evaluated_args = SharedArgs::new(state.evaluated_args.into_vec());
    let unevaluated_restore = state.unevaluated_restores.map(|restores| {
        Box::new(UnevaluatedRestorePlan {
            prepared_args: evaluated_args.clone(),
            wrappers: restores.into_vec(),
        })
    });
    PreparedCallArguments {
        evaluated_args,
        unevaluated_restore,
    }
}

pub(super) fn prepared_call_arguments_from_continuation_expr(
    expr: Expr,
) -> Result<PreparedCallArguments, Expr> {
    let Expr::Call { head, args } = expr else {
        return Err(expr);
    };
    if !is_system_symbol_name(&head, "InternalContinuationPreparedCallArguments") {
        return Err(Expr::Call { head, args });
    }
    let mut parts = args.into_iter();
    let evaluated_expr = parts
        .next()
        .expect("prepared-call continuation stores evaluated arguments");
    let restores_expr = parts
        .next()
        .expect("prepared-call continuation stores Unevaluated wrappers");
    assert!(parts.next().is_none());

    let Expr::Call {
        head: evaluated_head,
        args: evaluated_args,
    } = evaluated_expr
    else {
        panic!("prepared-call arguments should be stored as a List")
    };
    assert!(is_system_symbol_name(&evaluated_head, "List"));
    let Expr::Call {
        head: restores_head,
        args: restores,
    } = restores_expr
    else {
        panic!("prepared-call restore entries should be stored as a List")
    };
    assert!(is_system_symbol_name(&restores_head, "List"));

    let wrappers = restores
        .into_iter()
        .map(|entry| {
            let Expr::Call { head, args } = entry else {
                panic!("prepared-call restore entry should be a call")
            };
            assert!(is_system_symbol_name(
                &head,
                "InternalContinuationUnevaluatedRestore"
            ));
            let mut fields = args.into_iter();
            let position = fields
                .next()
                .as_ref()
                .and_then(expr_as_non_negative_usize)
                .expect("prepared-call restore position should be non-negative");
            let wrapper = fields
                .next()
                .expect("prepared-call restore entry stores its wrapper");
            assert!(fields.next().is_none());
            (position, wrapper)
        })
        .collect::<Vec<_>>();
    let unevaluated_restore = (!wrappers.is_empty()).then(|| {
        Box::new(UnevaluatedRestorePlan {
            prepared_args: evaluated_args.clone(),
            wrappers,
        })
    });
    Ok(PreparedCallArguments {
        evaluated_args,
        unevaluated_restore,
    })
}

impl Session {
    pub(super) fn prepare_arguments_for_call_with_unevaluated_restore(
        &mut self,
        head: Expr,
        attributes: AttributeSet,
        args: SharedArgs,
    ) -> Result<PreparedCallArguments, EvalError> {
        let runtime_profile = runtime_path_timing_start("prepare_arguments.total");
        let result = (|| {
            let sequence_hold = attributes.sequence_hold();
            let list_drops_nothing = is_system_symbol_name(&head, heads::structure::LIST);
            let mut built: Option<Vec<Expr>> = None;
            let mut unevaluated_restores: Option<Vec<(usize, Expr)>> = None;

            for index in 0..args.len() {
                let runtime_profile_plan = runtime_path_timing_start("prepare_arguments.plan_arg");
                let raw_arg = args[index].clone();
                let plan = plan_call_argument_preparation(attributes, index, raw_arg.clone());
                record_profile_runtime_path_timing(runtime_profile_plan, RuntimeProfileOutcome::Ok);

                let prepared_arg = match plan.mode {
                    CallArgumentMode::Evaluate => {
                        let allow_sequence_splice = plan.allow_sequence_splice;
                        let runtime_profile_evaluate =
                            runtime_path_timing_start("prepare_arguments.evaluate_arg");
                        let evaluated_result = self.evaluate_atom_fast_path(plan.expr);
                        record_profile_runtime_path_timing(
                            runtime_profile_evaluate,
                            runtime_profile_outcome_for_expr_result(&evaluated_result),
                        );
                        'argument_value: {
                            return crate::resume_typed_or_suspend!(
                                self,
                                evaluated_result,
                                ok(value) => break 'argument_value value,
                                suspend => PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame::freeze(
                                    head, attributes, args, index, built.take(),
                                    unevaluated_restores, allow_sequence_splice,
                                ),
                            );
                        }
                    }
                    CallArgumentMode::Hold => plan.expr,
                    CallArgumentMode::StrippedUnevaluated => {
                        let splices = prepared_argument_splices(
                            &plan.expr,
                            plan.allow_sequence_splice,
                            sequence_hold,
                        );
                        let drops_nothing = list_drops_nothing
                            && prepared_list_argument_drops_nothing(
                                &plan.expr,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        if !splices && !drops_nothing {
                            let prepared_position = built.as_ref().map_or(index, Vec::len);
                            unevaluated_restores
                                .get_or_insert_with(Vec::new)
                                .push((prepared_position, raw_arg));
                        }
                        plan.expr
                    }
                };

                let splices = prepared_argument_splices(
                    &prepared_arg,
                    plan.allow_sequence_splice,
                    sequence_hold,
                );
                let drops_nothing = list_drops_nothing
                    && prepared_list_argument_drops_nothing(
                        &prepared_arg,
                        plan.allow_sequence_splice,
                        sequence_hold,
                    );
                let changed = splices || drops_nothing || prepared_arg != args[index];

                match (built.as_mut(), changed) {
                    (None, false) => {}
                    (None, true) => {
                        let mut prepared_args = args[..index].to_vec();
                        let runtime_profile_append =
                            runtime_path_timing_start("prepare_arguments.append_arg");
                        if list_drops_nothing {
                            append_prepared_list_argument(
                                &mut prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        } else {
                            append_prepared_argument(
                                &mut prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        }
                        record_profile_runtime_path_timing(
                            runtime_profile_append,
                            RuntimeProfileOutcome::Ok,
                        );
                        built = Some(prepared_args);
                    }
                    (Some(prepared_args), _) => {
                        let runtime_profile_append =
                            runtime_path_timing_start("prepare_arguments.append_arg");
                        if list_drops_nothing {
                            append_prepared_list_argument(
                                prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        } else {
                            append_prepared_argument(
                                prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        }
                        record_profile_runtime_path_timing(
                            runtime_profile_append,
                            RuntimeProfileOutcome::Ok,
                        );
                    }
                }
            }

            let evaluated_args = built.map(SharedArgs::new).unwrap_or(args);
            let unevaluated_restore = unevaluated_restores.map(|wrappers| {
                Box::new(UnevaluatedRestorePlan {
                    prepared_args: evaluated_args.clone(),
                    wrappers,
                })
            });

            Ok(PreparedCallArguments {
                evaluated_args,
                unevaluated_restore,
            })
        })();
        record_profile_runtime_path_timing(
            runtime_profile,
            runtime_profile_outcome_for_shared_args_result(&result),
        );
        result
    }

    fn resume_suspended_call_after_argument<'resume>(
        &mut self,
        mut state: SuspendedCallArgumentPreparation,
        value: Expr,
        allow_sequence_splice: bool,
        token: ContinueToken<'resume, PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame>,
    ) -> Result<SuspendedPreparedCallArguments, ResumeStop<PreparedCallArguments>> {
        let list_drops_nothing = is_system_symbol_name(&state.head, heads::structure::LIST);
        state.prepared = append_suspended_prepared_argument(
            state.prepared,
            value,
            allow_sequence_splice,
            state.attributes.sequence_hold(),
            list_drops_nothing,
        );
        self.resume_suspended_call_argument_preparation(state, token)
    }

    fn resume_suspended_call_argument_preparation<'resume>(
        &mut self,
        mut state: SuspendedCallArgumentPreparation,
        token: ContinueToken<'resume, PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame>,
    ) -> Result<SuspendedPreparedCallArguments, ResumeStop<PreparedCallArguments>> {
        let sequence_hold = state.attributes.sequence_hold();
        let list_drops_nothing = is_system_symbol_name(&state.head, heads::structure::LIST);

        loop {
            let Some(arg) = state.args.get(state.cursor).cloned() else {
                return Ok(SuspendedPreparedCallArguments {
                    evaluated_args: state.prepared,
                    unevaluated_restores: state.unevaluated_restores,
                });
            };

            let index = state.cursor;
            let raw_arg = arg.clone();
            let runtime_profile_plan = runtime_path_timing_start("prepare_arguments.plan_arg");
            let plan = plan_call_argument_preparation(state.attributes, index, arg);
            record_profile_runtime_path_timing(runtime_profile_plan, RuntimeProfileOutcome::Ok);

            let prepared_arg = match plan.mode {
                CallArgumentMode::Evaluate => {
                    let allow_sequence_splice = plan.allow_sequence_splice;
                    let runtime_profile_evaluate =
                        runtime_path_timing_start("prepare_arguments.evaluate_arg");
                    let evaluated_result = self.evaluate_atom_fast_path(plan.expr);
                    record_profile_runtime_path_timing(
                        runtime_profile_evaluate,
                        runtime_profile_outcome_for_expr_result(&evaluated_result),
                    );
                    // The witness is derived before matching the actual evaluator Result.
                    let witness: SlotWitness<
                        <PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame as TypedFrame>::Input,
                    > = slot_witness(&evaluated_result);
                    match evaluated_result {
                        Ok(value) => value,
                        Err(EvalError::InputRequired { request }) => {
                            state.cursor += 1;
                            return Err(token.suspend(
                                self,
                                witness,
                                request,
                                PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame {
                                    state,
                                    allow_sequence_splice,
                                },
                            ));
                        }
                        Err(error) => return Err(ResumeStop::failure(error)),
                    }
                }
                CallArgumentMode::Hold => plan.expr,
                CallArgumentMode::StrippedUnevaluated => {
                    let splices = prepared_argument_splices(
                        &plan.expr,
                        plan.allow_sequence_splice,
                        sequence_hold,
                    );
                    let drops_nothing = list_drops_nothing
                        && prepared_list_argument_drops_nothing(
                            &plan.expr,
                            plan.allow_sequence_splice,
                            sequence_hold,
                        );
                    if !splices && !drops_nothing {
                        let entry = (state.prepared.len, raw_arg);
                        state.unevaluated_restores = Some(
                            state
                                .unevaluated_restores
                                .take()
                                .unwrap_or_else(
                                    || UnevaluatedRestoreAccumulator::freeze(Vec::new()),
                                )
                                .appended(entry),
                        );
                    }
                    plan.expr
                }
            };

            let runtime_profile_append = runtime_path_timing_start("prepare_arguments.append_arg");
            state.prepared = append_suspended_prepared_argument(
                state.prepared,
                prepared_arg,
                plan.allow_sequence_splice,
                sequence_hold,
                list_drops_nothing,
            );
            state.cursor += 1;
            record_profile_runtime_path_timing(runtime_profile_append, RuntimeProfileOutcome::Ok);
        }
    }

    pub(super) fn resume_call_from_prepared_args(
        &mut self,
        head: Expr,
        prepared_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.resume_call_from_prepared_args_with_unevaluated_restore(
            head,
            PreparedCallArguments {
                evaluated_args: SharedArgs::new(prepared_args),
                unevaluated_restore: None,
            },
        )
    }

    fn resume_call_from_prepared_args_with_unevaluated_restore(
        &mut self,
        head: Expr,
        prepared: PreparedCallArguments,
    ) -> Result<Expr, EvalError> {
        let plan = self.build_call_dispatch_plan(&head);
        let state = CallDispatchState::from_prepared(self.normalize_call_args_shared(
            &head,
            prepared.evaluated_args,
            plan.attributes,
        ));

        self.resume_call_with_dispatch_plan(head, plan, state, 0, prepared.unevaluated_restore)
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_shared_args_result<T>(
    result: &Result<T, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<UnevaluatedRestoreAccumulator>();
    assert_traces::<SuspendedCallArgumentPreparation>();
    assert_traces::<UnevaluatedRestorePlan>();
};

#[cfg(test)]
pub(in crate::runtime) fn prepared_slot_finalizer_fixture(
    prepared: &PreparedCallArguments,
) -> Expr {
    finish_suspended_prepared_call(SuspendedPreparedCallArguments {
        evaluated_args: PreparedArgumentAccumulator::freeze(
            prepared.evaluated_args.iter().cloned().collect(),
        ),
        unevaluated_restores: prepared
            .unevaluated_restore
            .as_ref()
            .map(|restore| UnevaluatedRestoreAccumulator::freeze(restore.wrappers.clone())),
    })
    .into_continuation_expr()
}
