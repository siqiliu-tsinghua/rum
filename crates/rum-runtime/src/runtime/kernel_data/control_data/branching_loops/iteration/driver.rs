mod continuation;
mod resume_frames;
use resume_frames::*;

use super::spec::{build_iteration_body_or_nested, evaluate_do_spec_expr, parse_do_spec};
use super::table_fast_path::{
    eval_single_spec_constant_table_fast_path, eval_single_spec_table_fast_path,
};
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct SuspendedDoIterations {
    args: SharedArgs,
    spec_index: usize,
    spec: SuspendedDoSpec,
    iteration: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedTableIterations {
    args: SharedArgs,
    spec_index: usize,
    spec: SuspendedDoSpec,
    iteration: usize,
    collected: PersistentExprAccumulator,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedLoopIteratorSymbolState(Arc<BlockSymbolState>);

impl SuspendedLoopIteratorSymbolState {
    fn freeze(state: BlockSymbolState) -> Self {
        Self(Arc::new(state))
    }
}

impl Session {
    pub(in crate::runtime) fn eval_do(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Do",
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_do_after_spec_eval(args, 0)
    }

    pub(in crate::runtime) fn eval_table(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Table",
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_table_after_spec_eval(args, 0)
    }

    fn resume_do_after_spec_eval(
        &mut self,
        args: &SharedArgs,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = crate::resume_or_suspend!(
            self,
            spec_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        ResumeDoAfterSpecEvalSuspensionFrame { args, spec_index }
        },
        )?;

        self.finish_do_spec_eval(args, spec_index, spec_expr)
    }

    fn finish_do_spec_eval(
        &mut self,
        args: &SharedArgs,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let spec = match parse_do_spec(self, &spec_expr, IterationConsumer::Do)? {
            Some(spec) => spec,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    "Do",
                    args,
                    "expected a supported repetition count or iterator specification",
                ));
            }
        };

        self.resume_do_iterations(args, spec_index, spec, 0)
    }

    fn resume_do_iterations(
        &mut self,
        args: &SharedArgs,
        spec_index: usize,
        spec: DoSpec,
        mut iteration: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &spec {
                DoSpec::Count(count) => {
                    if iteration >= *count {
                        return Ok(system_expr("Null"));
                    }
                    let body = build_iteration_body_or_nested(args, spec_index, "Do");
                    self.evaluate(body)
                }
                DoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(iteration).cloned() else {
                        return Ok(system_expr("Null"));
                    };
                    let body = build_iteration_body_or_nested(args, spec_index, "Do");
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(_) | Err(EvalError::Continue) => {
                    iteration += 1;
                }
                Err(EvalError::Break) => return Ok(system_expr("Null")),
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedDoIterations {
                        args: args.clone(),
                        spec_index,
                        spec: spec.into(),
                        iteration,
                    };
                    crate::resume_or_suspend!(insert self, ResumeDoIterationsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_do_after_iteration(
        &mut self,
        mut state: SuspendedDoIterations,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) | Err(EvalError::Continue) => {
                state.iteration += 1;
                self.resume_suspended_do_iterations(state)
            }
            Err(EvalError::Break) => Ok(system_expr("Null")),
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_do_iterations(
        &mut self,
        mut state: SuspendedDoIterations,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &state.spec {
                SuspendedDoSpec::Count(count) => {
                    if state.iteration >= *count {
                        return Ok(system_expr("Null"));
                    }
                    let body =
                        build_iteration_body_or_nested(state.args.as_ref(), state.spec_index, "Do");
                    self.evaluate(body)
                }
                SuspendedDoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(state.iteration).cloned() else {
                        return Ok(system_expr("Null"));
                    };
                    let body =
                        build_iteration_body_or_nested(state.args.as_ref(), state.spec_index, "Do");
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(_) | Err(EvalError::Continue) => state.iteration += 1,
                Err(EvalError::Break) => return Ok(system_expr("Null")),
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedDoIterationsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_table_after_spec_eval(
        &mut self,
        args: &SharedArgs,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = crate::resume_or_suspend!(
            self,
            spec_result,
            ok(value) => Ok(value),
            suspend => {
        let args = args.clone();
        ResumeTableAfterSpecEvalSuspensionFrame { args, spec_index }
        },
        )?;

        self.finish_table_spec_eval(args, spec_index, spec_expr)
    }

    fn finish_table_spec_eval(
        &mut self,
        args: &SharedArgs,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        if spec_index + 2 >= args.len() {
            if let Some(result) =
                eval_single_spec_constant_table_fast_path(self, &args[0], &spec_expr)?
            {
                return Ok(result);
            }
        }

        let spec = match parse_do_spec(self, &spec_expr, IterationConsumer::Table)? {
            Some(spec) => spec,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    "Table",
                    args,
                    "expected a supported repetition count or iterator specification",
                ));
            }
        };
        let capacity = spec.iteration_count();
        if spec_index + 2 >= args.len() {
            match eval_single_spec_table_fast_path(self, &args[0], spec) {
                Ok(result) => return Ok(result),
                Err(spec_after_fast_path) => {
                    return self.resume_table_iterations(
                        args,
                        spec_index,
                        spec_after_fast_path,
                        0,
                        Vec::with_capacity(capacity),
                    );
                }
            }
        }

        self.resume_table_iterations(args, spec_index, spec, 0, Vec::with_capacity(capacity))
    }

    fn resume_table_iterations(
        &mut self,
        args: &SharedArgs,
        spec_index: usize,
        spec: DoSpec,
        mut iteration: usize,
        mut collected: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &spec {
                DoSpec::Count(count) => {
                    if iteration >= *count {
                        return Ok(Expr::call(system_expr("List"), collected));
                    }
                    let body = build_iteration_body_or_nested(args, spec_index, "Table");
                    self.evaluate(body)
                }
                DoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(iteration).cloned() else {
                        return Ok(Expr::call(system_expr("List"), collected));
                    };
                    let body = build_iteration_body_or_nested(args, spec_index, "Table");
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(value) => {
                    collected.push(value);
                    iteration += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedTableIterations {
                        args: args.clone(),
                        spec_index,
                        spec: spec.into(),
                        iteration,
                        collected: PersistentExprAccumulator::freeze(collected),
                    };
                    crate::resume_or_suspend!(insert self, ResumeTableIterationsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_table_after_iteration(
        &mut self,
        mut state: SuspendedTableIterations,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                state.collected = state.collected.appended(value);
                state.iteration += 1;
                self.resume_suspended_table_iterations(state)
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_table_iterations(
        &mut self,
        mut state: SuspendedTableIterations,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &state.spec {
                SuspendedDoSpec::Count(count) => {
                    if state.iteration >= *count {
                        return Ok(Expr::call(system_expr("List"), state.collected.into_vec()));
                    }
                    let body = build_iteration_body_or_nested(
                        state.args.as_ref(),
                        state.spec_index,
                        "Table",
                    );
                    self.evaluate(body)
                }
                SuspendedDoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(state.iteration).cloned() else {
                        return Ok(Expr::call(system_expr("List"), state.collected.into_vec()));
                    };
                    let body = build_iteration_body_or_nested(
                        state.args.as_ref(),
                        state.spec_index,
                        "Table",
                    );
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(value) => {
                    state.collected = state.collected.appended(value);
                    state.iteration += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, ResumeSuspendedTableIterationsSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub(in crate::runtime::kernel_data::control_data::branching_loops) fn evaluate_loop_iterator_body(
        &mut self,
        symbol: Symbol,
        value: Expr,
        body: Expr,
    ) -> Result<Expr, EvalError> {
        if self.dynamic_scope_symbol_state_is_clean(&symbol) {
            return self.evaluate_loop_iterator_body_with_dynamic_value(symbol, value, body);
        }

        let mut saved_state = self.capture_loop_iterator_symbol_state(&symbol);
        self.install_loop_iterator_symbol_value(&symbol, value, &saved_state);

        let body_result = self.evaluate(body);
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => {
        let saved_state =
                        SuspendedLoopIteratorSymbolState::freeze(std::mem::take(&mut saved_state));
        EvaluateLoopIteratorBodySuspensionFrame { symbol, saved_state }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ContinueLoopIteratorScopeFrame);
                Err(EvalError::InputRequired { request })
            }
            outcome => self.finish_loop_iterator_scope(symbol, saved_state, outcome),
        }
    }

    fn evaluate_loop_iterator_body_with_dynamic_value(
        &mut self,
        symbol: Symbol,
        value: Expr,
        body: Expr,
    ) -> Result<Expr, EvalError> {
        let dynamic_depth = self.push_dynamic_own_value_binding(symbol, Some(value));
        let body_result = self.evaluate(body);
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrame { symbol, dynamic_depth },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_loop_iterator_dynamic_scope(symbol, dynamic_depth, outcome),
        }
    }

    fn finish_loop_iterator_dynamic_scope(
        &mut self,
        symbol: Symbol,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.clear_dynamic_scope_symbol_side_effects(&symbol);
        outcome
    }

    pub(in crate::runtime) fn abandon_loop_iterator_dynamic_scope(
        &mut self,
        symbol: Symbol,
        dynamic_depth: usize,
    ) {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.clear_dynamic_scope_symbol_side_effects(&symbol);
    }

    fn capture_loop_iterator_symbol_state(&self, symbol: &Symbol) -> BlockSymbolState {
        BlockSymbolState {
            definition: self.definitions.get(symbol).cloned(),
            attributes: self.attributes.get(symbol).copied(),
            options: self.options.get(symbol).cloned(),
            syntax_information: self.syntax_information.get(symbol).cloned(),
            messages: self
                .message_definitions
                .iter()
                .filter(|(key, _)| key.symbol == *symbol)
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        }
    }

    fn install_loop_iterator_symbol_value(
        &mut self,
        symbol: &Symbol,
        value: Expr,
        saved_state: &BlockSymbolState,
    ) {
        let removed_up_values = saved_state
            .definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());

        self.definitions
            .insert(*symbol, SymbolDefinition::with_own_value(value));
        self.bump_evaluation_epoch_for_symbol(symbol);

        if removed_up_values {
            self.refresh_has_up_value_definitions();
        }

        if self.attributes.remove(symbol).is_some() {
            self.bump_attribute_revision();
        }
        self.options.remove(symbol);
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Options);
        self.syntax_information.remove(symbol);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
        );

        let messages_before = self.message_definitions.len();
        self.message_definitions
            .retain(|key, _| key.symbol != *symbol);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
        );
        if self.message_definitions.len() != messages_before {
            self.rebuild_message_symbol_counts();
        }
    }

    fn finish_loop_iterator_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let current_had_up_values = self
            .definitions
            .get(&symbol)
            .is_some_and(|definition| !definition.up_values.is_empty());
        let restored_has_up_values = saved_state
            .definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());

        match saved_state.definition {
            Some(definition) => {
                self.definitions.insert(symbol, definition);
            }
            None => {
                self.definitions.remove(&symbol);
            }
        }
        self.bump_evaluation_epoch_for_symbol(&symbol);

        if current_had_up_values || restored_has_up_values {
            self.refresh_has_up_value_definitions();
        }

        let current_attributes = self.attributes.get(&symbol).copied();
        if current_attributes != saved_state.attributes {
            match saved_state.attributes {
                Some(attributes) => {
                    self.attributes.insert(symbol, attributes);
                }
                None => {
                    self.attributes.remove(&symbol);
                }
            }
            self.bump_attribute_revision();
        }

        match saved_state.options {
            Some(options) => {
                self.options.insert(symbol, options);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::Options,
                );
            }
            None => {
                self.options.remove(&symbol);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::Options,
                );
            }
        }

        match saved_state.syntax_information {
            Some(syntax_information) => {
                self.syntax_information.insert(symbol, syntax_information);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
                );
            }
            None => {
                self.syntax_information.remove(&symbol);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
                );
            }
        }

        let messages_before = self.message_definitions.len();
        self.message_definitions
            .retain(|key, _| key.symbol != symbol);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
        );
        let messages_changed =
            self.message_definitions.len() != messages_before || !saved_state.messages.is_empty();
        for (key, definition) in saved_state.messages {
            self.message_definitions.insert(key, definition);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
            );
        }
        if messages_changed {
            self.rebuild_message_symbol_counts();
        }

        outcome
    }

    pub(in crate::runtime) fn abandon_loop_iterator_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
    ) {
        let _ = self.finish_loop_iterator_scope(symbol, saved_state, Ok(system_expr("Null")));
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedDoIterations>();
    assert_traces::<SuspendedTableIterations>();
    assert_traces::<SuspendedLoopIteratorSymbolState>();
};
