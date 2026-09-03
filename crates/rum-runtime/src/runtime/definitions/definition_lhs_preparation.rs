mod prepared_store;
mod resume_frames;
mod syntax_policy;
pub(super) use prepared_store::{
    PreparedDefinitionAssignment, PreparedDefinitionStore, PreparedUnsetDefinitionTarget,
};
use resume_frames::*;
use syntax_policy::{DefinitionLhsSyntaxCallPlan, definition_lhs_syntax_call_plan};
pub(super) use syntax_policy::{
    definition_lhs_blank_head, definition_lhs_contains_pattern_syntax,
    definition_lhs_pattern_wrapper_head,
};

use super::super::*;
use super::value_table_assignment::ValueTableAssignmentState;
use crate::runtime::eval_core::call::SuspendedUnevaluatedRestorePlan;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

type PreparedDefinitionLhsCallback =
    fn(&mut Session, PreparedDefinitionLhs, SharedArgs) -> Result<Expr, EvalError>;

impl FrameRootTrace for PreparedDefinitionLhsCallback {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

#[doc(hidden)]
#[derive(Clone, FrameRootTrace)]
pub struct PreparedDefinitionLhsContinuation {
    callback: PreparedDefinitionLhsCallback,
    state: SharedArgs,
}

impl PreparedDefinitionLhsContinuation {
    #[doc(hidden)]
    pub fn new(
        callback: fn(&mut Session, PreparedDefinitionLhs, SharedArgs) -> Result<Expr, EvalError>,
        state: SharedArgs,
    ) -> Self {
        Self { callback, state }
    }

    fn resume(
        self,
        session: &mut Session,
        prepared: PreparedDefinitionLhs,
    ) -> Result<Expr, EvalError> {
        (self.callback)(session, prepared, self.state)
    }
}

#[derive(Clone, FrameRootTrace)]
struct PreparedDefinitionUnevaluatedRestore {
    prepared_args: SharedArgs,
    wrappers: Arc<[(usize, Expr)]>,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PreparedDefinitionOuterCall {
    head: SharedExpr,
    plan: SuspendedCallDispatchPlan,
    prepared_args: SharedArgs,
    unevaluated_restore: Option<PreparedDefinitionUnevaluatedRestore>,
}

#[derive(Clone, FrameRootTrace)]
pub struct PreparedDefinitionLhs {
    lhs: Expr,
    target: Option<DefinitionAssignmentTarget>,
    outer_call: Option<PreparedDefinitionOuterCall>,
}

impl PreparedDefinitionLhs {
    fn new(session: &Session, lhs: Expr, outer_call: Option<PreparedDefinitionOuterCall>) -> Self {
        let target = session.normalize_definition_assignment_target(&lhs);
        Self {
            lhs,
            target,
            outer_call,
        }
    }

    #[doc(hidden)]
    pub fn lhs(&self) -> &Expr {
        &self.lhs
    }

    #[cfg(test)]
    fn into_lhs(self) -> Expr {
        self.lhs
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) enum DefinitionLhsPreparationContinuation {
    Set {
        rhs: Expr,
        delayed: bool,
    },
    Unset {
        source: Expr,
    },
    ValueTableEntry {
        state: ValueTableAssignmentState,
    },
    TagSet {
        tag: Symbol,
        args: SharedArgs,
        rhs: Expr,
        delayed: bool,
    },
    TagUnset {
        tag: Symbol,
        args: SharedArgs,
    },
    UpSet {
        rhs: Expr,
        delayed: bool,
    },
    External(PreparedDefinitionLhsContinuation),
    #[cfg(test)]
    ReturnPrepared,
}

pub(super) enum DefinitionLhsPreparationContinuationResult {
    Complete(Expr),
    PrepareNext {
        lhs: Expr,
        continuation: DefinitionLhsPreparationContinuation,
    },
}

impl DefinitionLhsPreparationContinuation {
    fn resume(
        self,
        session: &mut Session,
        prepared: PreparedDefinitionLhs,
    ) -> Result<DefinitionLhsPreparationContinuationResult, EvalError> {
        match self {
            Self::Set { rhs, delayed } => session
                .finish_prepared_definition_assignment(prepared, rhs, delayed)
                .map(DefinitionLhsPreparationContinuationResult::Complete),
            Self::Unset { source } => session
                .finish_prepared_unset(prepared, source)
                .map(DefinitionLhsPreparationContinuationResult::Complete),
            Self::ValueTableEntry { state } => {
                session.resume_prepared_value_table_entry(state, prepared)
            }
            Self::TagSet {
                tag,
                args,
                rhs,
                delayed,
            } => session
                .finish_prepared_tag_set(prepared, tag, args, rhs, delayed)
                .map(DefinitionLhsPreparationContinuationResult::Complete),
            Self::TagUnset { tag, args } => session
                .finish_prepared_tag_unset(prepared, tag, args)
                .map(DefinitionLhsPreparationContinuationResult::Complete),
            Self::UpSet { rhs, delayed } => session
                .finish_prepared_up_set(prepared, rhs, delayed)
                .map(DefinitionLhsPreparationContinuationResult::Complete),
            Self::External(continuation) => continuation
                .resume(session, prepared)
                .map(DefinitionLhsPreparationContinuationResult::Complete),
            #[cfg(test)]
            Self::ReturnPrepared => Ok(DefinitionLhsPreparationContinuationResult::Complete(
                prepared.into_lhs(),
            )),
        }
    }

    fn captures_outer_call(&self) -> bool {
        matches!(self, Self::External(_))
    }
}

impl Session {
    #[cold]
    #[inline(never)]
    pub(super) fn prepare_definition_lhs(
        &mut self,
        lhs: Expr,
        continuation: DefinitionLhsPreparationContinuation,
    ) -> Result<Expr, EvalError> {
        self.drive_definition_lhs_preparation(DefinitionLhsPreparationState {
            work: DefinitionLhsPreparationWork::PrepareNode {
                expr: lhs,
                region: DefinitionLhsRegion::Root,
            },
            parents: DefinitionLhsParentStack::default(),
            continuation,
            prepared_outer_call: None,
        })
    }

    #[doc(hidden)]
    #[cold]
    #[inline(never)]
    pub fn prepare_definition_lhs_for_consumer(
        &mut self,
        lhs: Expr,
        continuation: PreparedDefinitionLhsContinuation,
    ) -> Result<Expr, EvalError> {
        self.prepare_definition_lhs(
            lhs,
            DefinitionLhsPreparationContinuation::External(continuation),
        )
    }

    #[doc(hidden)]
    pub fn evaluate_prepared_definition_lhs(
        &mut self,
        prepared: &PreparedDefinitionLhs,
    ) -> Result<Expr, EvalError> {
        let Some(outer_call) = &prepared.outer_call else {
            return self.evaluate(prepared.lhs.clone());
        };
        let unevaluated_restore = outer_call.unevaluated_restore.as_ref().map(|restore| {
            SuspendedUnevaluatedRestorePlan {
                prepared_args: restore.prepared_args.clone(),
                wrappers: restore.wrappers.clone(),
            }
        });
        self.resume_suspended_call_with_dispatch_plan(
            outer_call.head.clone(),
            outer_call.plan.clone(),
            CallDispatchState::from_prepared(outer_call.prepared_args.clone()),
            0,
            unevaluated_restore,
        )
    }

    #[cold]
    #[inline(never)]
    fn drive_definition_lhs_preparation(
        &mut self,
        mut state: DefinitionLhsPreparationState,
    ) -> Result<Expr, EvalError> {
        loop {
            let work = std::mem::replace(&mut state.work, DefinitionLhsPreparationWork::Vacant);
            match work {
                DefinitionLhsPreparationWork::Vacant => {
                    unreachable!("definition-lhs work is replaced only while it is being driven")
                }
                DefinitionLhsPreparationWork::PrepareNode { expr, region } => match expr {
                    Expr::Atom(_)
                        if matches!(
                            region,
                            DefinitionLhsRegion::Root | DefinitionLhsRegion::RawPattern
                        ) =>
                    {
                        state.work = DefinitionLhsPreparationWork::CompleteNode(expr);
                    }
                    Expr::Atom(_) => {
                        let source = expr;
                        match self.evaluate_definition_lhs_active_expr(&source) {
                            Ok(value) => {
                                state.work = DefinitionLhsPreparationWork::CompleteNode(value);
                            }
                            Err(EvalError::InputRequired { request }) => {
                                state.work =
                                    DefinitionLhsPreparationWork::AwaitPatternAtom { source };
                                crate::resume_or_suspend!(insert self, DefinitionLhsPreparationSuspensionFrame { state });
                                return Err(EvalError::InputRequired { request });
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    Expr::Call { head, args, .. } => {
                        match definition_lhs_syntax_call_plan(&head, args.len(), region) {
                            DefinitionLhsSyntaxCallPlan::Opaque => {
                                state.work = DefinitionLhsPreparationWork::CompleteNode(
                                    Expr::call_shared(head, args),
                                );
                            }
                            DefinitionLhsSyntaxCallPlan::Structural {
                                selection,
                                child_region,
                            } => {
                                self.start_definition_lhs_structural_call(
                                    &mut state,
                                    head,
                                    args,
                                    selection,
                                    child_region,
                                );
                            }
                            DefinitionLhsSyntaxCallPlan::Ordinary
                                if matches!(region, DefinitionLhsRegion::RawPattern) =>
                            {
                                self.start_definition_lhs_structural_call(
                                    &mut state,
                                    head,
                                    args,
                                    DefinitionLhsStructuralSelection::RawTree {
                                        head_pending: true,
                                        cursor: 0,
                                    },
                                    DefinitionLhsRegion::RawPattern,
                                );
                            }
                            DefinitionLhsSyntaxCallPlan::Ordinary => {
                                let capture_outer_call =
                                    matches!(region, DefinitionLhsRegion::Root)
                                        && state.continuation.captures_outer_call();
                                let source_head = head.clone();
                                match self.evaluate_definition_lhs_call_head(head) {
                                    Ok(evaluated_head) => {
                                        state.work =
                                            DefinitionLhsPreparationWork::PrepareCallArguments(
                                                self.definition_lhs_call_state(
                                                    evaluated_head,
                                                    args,
                                                    capture_outer_call,
                                                ),
                                            );
                                    }
                                    Err(EvalError::InputRequired { request }) => {
                                        state.work = DefinitionLhsPreparationWork::AwaitCallHead {
                                            source_head,
                                            args,
                                            capture_outer_call,
                                        };
                                        crate::resume_or_suspend!(insert self, DefinitionLhsPreparationSuspensionFrame { state });
                                        return Err(EvalError::InputRequired { request });
                                    }
                                    Err(error) => return Err(error),
                                }
                            }
                        }
                    }
                },
                DefinitionLhsPreparationWork::PrepareCallArguments(mut call) => {
                    let Some(raw_arg) = call.args.get(call.cursor).cloned() else {
                        let (value, outer_call) = self.finish_definition_lhs_call(call);
                        if let Some(outer_call) = outer_call {
                            state.prepared_outer_call = Some(outer_call);
                        }
                        state.work = DefinitionLhsPreparationWork::CompleteNode(value);
                        continue;
                    };

                    let source_index = call.cursor;
                    let restore_wrapper = (call.outer_dispatch_plan.is_some()
                        && unevaluated_wrapper_arg(&raw_arg).is_some())
                    .then(|| raw_arg.clone());
                    let plan =
                        plan_call_argument_preparation(call.attributes, source_index, raw_arg);
                    call.cursor += 1;
                    let argument = DefinitionLhsCallArgumentContinuation {
                        source_index,
                        allow_sequence_splice: plan.allow_sequence_splice,
                    };

                    match plan.mode {
                        CallArgumentMode::Evaluate
                            if definition_lhs_contains_pattern_syntax(&plan.expr) =>
                        {
                            state
                                .parents
                                .push(DefinitionLhsParentContext::CallArgument { call, argument });
                            state.work = DefinitionLhsPreparationWork::PrepareNode {
                                expr: plan.expr,
                                region: DefinitionLhsRegion::ActivePattern,
                            };
                        }
                        CallArgumentMode::Evaluate => {
                            let source = plan.expr;
                            match self.evaluate_definition_lhs_active_expr(&source) {
                                Ok(value) => {
                                    self.merge_definition_lhs_call_argument(
                                        &mut call, argument, value,
                                    );
                                    state.work =
                                        DefinitionLhsPreparationWork::PrepareCallArguments(call);
                                }
                                Err(EvalError::InputRequired { request }) => {
                                    state.work = DefinitionLhsPreparationWork::AwaitCallArgument {
                                        call,
                                        argument,
                                        source,
                                    };
                                    crate::resume_or_suspend!(insert self, DefinitionLhsPreparationSuspensionFrame { state });
                                    return Err(EvalError::InputRequired { request });
                                }
                                Err(error) => return Err(error),
                            }
                        }
                        CallArgumentMode::Hold | CallArgumentMode::StrippedUnevaluated => {
                            if matches!(plan.mode, CallArgumentMode::StrippedUnevaluated) {
                                if let Some(wrapper) = restore_wrapper {
                                    self.record_definition_lhs_unevaluated_restore(
                                        &mut call, argument, &plan.expr, wrapper,
                                    );
                                }
                            }
                            self.merge_definition_lhs_call_argument(&mut call, argument, plan.expr);
                            state.work = DefinitionLhsPreparationWork::PrepareCallArguments(call);
                        }
                    }
                }
                DefinitionLhsPreparationWork::CompleteNode(value) => {
                    let Some(parent) = state.parents.pop() else {
                        let prepared = PreparedDefinitionLhs::new(
                            self,
                            value,
                            state.prepared_outer_call.take(),
                        );
                        match state.continuation.resume(self, prepared)? {
                            DefinitionLhsPreparationContinuationResult::Complete(value) => {
                                return Ok(value);
                            }
                            DefinitionLhsPreparationContinuationResult::PrepareNext {
                                lhs,
                                continuation,
                            } => {
                                state = DefinitionLhsPreparationState {
                                    work: DefinitionLhsPreparationWork::PrepareNode {
                                        expr: lhs,
                                        region: DefinitionLhsRegion::Root,
                                    },
                                    parents: DefinitionLhsParentStack::default(),
                                    continuation,
                                    prepared_outer_call: None,
                                };
                                continue;
                            }
                        }
                    };

                    match parent {
                        DefinitionLhsParentContext::CallArgument { mut call, argument } => {
                            self.merge_definition_lhs_call_argument(&mut call, argument, value);
                            state.work = DefinitionLhsPreparationWork::PrepareCallArguments(call);
                        }
                        DefinitionLhsParentContext::StructuralNode { mut call, source } => {
                            merge_definition_lhs_structural_node(&mut call, source, value);
                            self.continue_definition_lhs_structural_call(&mut state, call);
                        }
                    }
                }
                DefinitionLhsPreparationWork::AwaitPatternAtom { .. }
                | DefinitionLhsPreparationWork::AwaitCallHead { .. }
                | DefinitionLhsPreparationWork::AwaitCallArgument { .. } => {
                    unreachable!("only a suspension frame resumes awaited lhs preparation work")
                }
            }
        }
    }

    fn resume_definition_lhs_preparation(
        &mut self,
        mut state: DefinitionLhsPreparationState,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let value = outcome?;
        state.work =
            match state.work {
                DefinitionLhsPreparationWork::AwaitPatternAtom { .. } => {
                    DefinitionLhsPreparationWork::CompleteNode(value)
                }
                DefinitionLhsPreparationWork::AwaitCallHead {
                    source_head,
                    args,
                    capture_outer_call,
                } => {
                    let evaluated_head = reuse_definition_lhs_call_head(source_head, value);
                    DefinitionLhsPreparationWork::PrepareCallArguments(
                        self.definition_lhs_call_state(evaluated_head, args, capture_outer_call),
                    )
                }
                DefinitionLhsPreparationWork::AwaitCallArgument {
                    mut call, argument, ..
                } => {
                    self.merge_definition_lhs_call_argument(&mut call, argument, value);
                    DefinitionLhsPreparationWork::PrepareCallArguments(call)
                }
                _ => unreachable!("a definition-lhs frame stores awaited work"),
            };
        self.drive_definition_lhs_preparation(state)
    }

    fn evaluate_definition_lhs_call_head(
        &mut self,
        source_head: SharedExpr,
    ) -> Result<SharedExpr, EvalError> {
        let evaluated = self.evaluate_definition_lhs_active_expr(&source_head)?;
        Ok(reuse_definition_lhs_call_head(source_head, evaluated))
    }

    fn evaluate_definition_lhs_active_expr(&mut self, source: &Expr) -> Result<Expr, EvalError> {
        match self.evaluate_atom_fast_path(source.clone()) {
            Err(EvalError::EvaluationCycleDetected { expr })
                if matches!(source, Expr::Atom(Atom::Symbol(_))) && expr == source.to_string() =>
            {
                Ok(source.clone())
            }
            result => result,
        }
    }

    fn definition_lhs_call_state(
        &mut self,
        head: SharedExpr,
        args: SharedArgs,
        capture_outer_call: bool,
    ) -> DefinitionLhsCallState {
        let (attributes, outer_dispatch_plan) = if capture_outer_call {
            let plan = self.build_call_dispatch_plan(&head);
            let attributes = plan.attributes;
            let has_prepare_arguments = (0..plan.len())
                .any(|index| plan.stage(index) == CallDispatchStage::PrepareArguments);
            (
                attributes,
                has_prepare_arguments.then(|| plan.into_suspended()),
            )
        } else {
            (self.call_preparation_metadata(&head).attributes(), None)
        };
        DefinitionLhsCallState {
            head,
            attributes,
            args,
            cursor: 0,
            prepared: None,
            unevaluated_restores: None,
            outer_dispatch_plan,
        }
    }

    fn merge_definition_lhs_call_argument(
        &self,
        call: &mut DefinitionLhsCallState,
        argument: DefinitionLhsCallArgumentContinuation,
        value: Expr,
    ) {
        let attributes = call.attributes;
        let sequence_hold = attributes.sequence_hold();
        let list_drops_nothing = is_system_symbol_name(&call.head, heads::structure::LIST);
        let splices =
            prepared_argument_splices(&value, argument.allow_sequence_splice, sequence_hold);
        let drops_nothing = list_drops_nothing
            && prepared_list_argument_drops_nothing(
                &value,
                argument.allow_sequence_splice,
                sequence_hold,
            );
        let changed = splices || drops_nothing || value != call.args[argument.source_index];

        if call.prepared.is_none() && changed {
            call.prepared = Some(PreparedArgumentAccumulator::freeze(
                call.args[..argument.source_index].to_vec(),
            ));
        }

        if let Some(prepared) = call.prepared.take() {
            call.prepared = Some(append_suspended_prepared_argument(
                prepared,
                value,
                argument.allow_sequence_splice,
                sequence_hold,
                list_drops_nothing,
            ));
        }
    }

    fn record_definition_lhs_unevaluated_restore(
        &self,
        call: &mut DefinitionLhsCallState,
        argument: DefinitionLhsCallArgumentContinuation,
        value: &Expr,
        wrapper: Expr,
    ) {
        let sequence_hold = call.attributes.sequence_hold();
        let list_drops_nothing = is_system_symbol_name(&call.head, heads::structure::LIST);
        let splices =
            prepared_argument_splices(value, argument.allow_sequence_splice, sequence_hold);
        let drops_nothing = list_drops_nothing
            && prepared_list_argument_drops_nothing(
                value,
                argument.allow_sequence_splice,
                sequence_hold,
            );
        if splices || drops_nothing {
            return;
        }

        let position = call
            .prepared
            .as_ref()
            .map_or(argument.source_index, |prepared| prepared.len);
        call.unevaluated_restores
            .get_or_insert_with(Vec::new)
            .push((position, wrapper));
    }

    fn finish_definition_lhs_call(
        &self,
        call: DefinitionLhsCallState,
    ) -> (Expr, Option<PreparedDefinitionOuterCall>) {
        let attributes = call.attributes;
        let args = match call.prepared {
            Some(prepared) => SharedArgs::new(prepared.into_vec_allow_shared()),
            None => call.args,
        };
        let unevaluated_restore =
            call.unevaluated_restores
                .map(|wrappers| PreparedDefinitionUnevaluatedRestore {
                    prepared_args: args.clone(),
                    wrappers: Arc::from(wrappers),
                });
        let normalized_args = self.normalize_call_args_shared(&call.head, args, attributes);
        let outer_call = call
            .outer_dispatch_plan
            .map(|plan| PreparedDefinitionOuterCall {
                head: call.head.clone(),
                plan,
                prepared_args: normalized_args.clone(),
                unevaluated_restore,
            });
        let identity = Expr::call_shared(call.head, normalized_args);
        (
            self.normalize_definition_identity_for_matching(&identity, attributes),
            outer_call,
        )
    }

    fn start_definition_lhs_structural_call(
        &self,
        state: &mut DefinitionLhsPreparationState,
        head: SharedExpr,
        args: SharedArgs,
        selection: DefinitionLhsStructuralSelection,
        child_region: DefinitionLhsRegion,
    ) {
        self.continue_definition_lhs_structural_call(
            state,
            DefinitionLhsStructuralCallState {
                head,
                args,
                selection,
                child_region,
                prepared: None,
            },
        );
    }

    fn continue_definition_lhs_structural_call(
        &self,
        state: &mut DefinitionLhsPreparationState,
        mut call: DefinitionLhsStructuralCallState,
    ) {
        if let Some(source) = call.selection.next(call.args.len()) {
            let expr = match source {
                DefinitionLhsStructuralSource::Head => call.head.as_ref().clone(),
                DefinitionLhsStructuralSource::Argument(source_index) => {
                    call.args[source_index].clone()
                }
            };
            let child_region = call.child_region;
            state
                .parents
                .push(DefinitionLhsParentContext::StructuralNode { call, source });
            state.work = DefinitionLhsPreparationWork::PrepareNode {
                expr,
                region: child_region,
            };
            return;
        }

        let args = match call.prepared {
            Some(mut prepared) => {
                for value in &call.args[prepared.len..] {
                    prepared = append_suspended_prepared_argument(
                        prepared,
                        value.clone(),
                        false,
                        true,
                        false,
                    );
                }
                SharedArgs::new(prepared.into_vec_allow_shared())
            }
            None => call.args,
        };
        state.work = DefinitionLhsPreparationWork::CompleteNode(Expr::call_shared(call.head, args));
    }
}

fn reuse_definition_lhs_call_head(source_head: SharedExpr, evaluated: Expr) -> SharedExpr {
    if source_head.as_ref() == &evaluated {
        source_head
    } else {
        SharedExpr::new(evaluated)
    }
}

fn merge_definition_lhs_structural_node(
    call: &mut DefinitionLhsStructuralCallState,
    source: DefinitionLhsStructuralSource,
    value: Expr,
) {
    let DefinitionLhsStructuralSource::Argument(source_index) = source else {
        if call.head.as_ref() != &value {
            call.head = SharedExpr::new(value);
        }
        return;
    };

    let changed = value != call.args[source_index];
    if call.prepared.is_none() && changed {
        call.prepared = Some(PreparedArgumentAccumulator::freeze(
            call.args[..source_index].to_vec(),
        ));
    }

    if let Some(prepared) = call.prepared.take() {
        call.prepared = Some(append_suspended_prepared_argument(
            prepared, value, false, true, false,
        ));
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DefinitionLhsPreparationContinuation>();
    assert_traces::<PreparedDefinitionLhs>();
    assert_traces::<PreparedDefinitionLhsContinuation>();
    assert_traces::<PreparedDefinitionOuterCall>();
    assert_traces::<PreparedDefinitionStore>();
    assert_traces::<PreparedDefinitionUnevaluatedRestore>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_canonical_lhs_reuses_its_shared_arguments() {
        let mut session = Session::new();
        let args = SharedArgs::new(vec![Expr::symbol("a"), Expr::symbol("b")]);
        let lhs = Expr::call_shared(SharedExpr::new(Expr::symbol("f")), args.clone());

        let prepared = session
            .prepare_definition_lhs(lhs, DefinitionLhsPreparationContinuation::ReturnPrepared)
            .unwrap();
        let Expr::Call {
            args: prepared_args,
            ..
        } = prepared
        else {
            panic!("prepared definition lhs should remain a call")
        };

        assert!(SharedArgs::ptr_eq(&args, &prepared_args));
    }
}
