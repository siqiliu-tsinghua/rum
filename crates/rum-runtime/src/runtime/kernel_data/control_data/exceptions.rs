mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Debug, FrameRootTrace)]
struct ExceptionHandler {
    pattern: Expr,
    rhs: Expr,
    rhs_may_splice_without_bindings: bool,
    guards: Box<[Expr]>,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedExceptionDispatch {
    exception: Expr,
    payload: Expr,
    type_chain: Arc<[String]>,
    handlers: Arc<[ExceptionHandler]>,
    handler_index: usize,
    ancestor_index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_register_exception_type(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("RegisterExceptionType", args, 1, 2));
        }

        let Some(type_name) = expr_as_string(&args[0]).map(str::to_owned) else {
            return Ok(self.invalid_call_with_arg_form(
                "RegisterExceptionType",
                args,
                "exception type must be a string",
            ));
        };
        let requested_parent = match args.get(1) {
            None => None,
            Some(parent) => {
                let Some(parent) = expr_as_string(parent) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "RegisterExceptionType",
                        args,
                        "exception parent type must be a string",
                    ));
                };
                (parent != "Exception").then(|| parent.to_owned())
            }
        };

        let existing = if type_name == "Exception" {
            Some(None)
        } else {
            self.exception_types.get(&type_name).cloned()
        };
        if let Some(stored_parent) = existing {
            if stored_parent != requested_parent {
                self.push_message(
                    "RegisterExceptionType",
                    "reregister",
                    format!(
                        "Exception type {type_name:?} is already registered with parent {}.",
                        display_exception_parent(stored_parent.as_deref())
                    ),
                );
            }
            return Ok(exception_registration_expr(
                &type_name,
                stored_parent.as_deref(),
            ));
        }

        self.exception_types
            .insert(type_name.clone(), requested_parent.clone());
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::ExceptionTypes,
        );
        Ok(exception_registration_expr(
            &type_name,
            requested_parent.as_deref(),
        ))
    }

    pub(in crate::runtime) fn eval_exception_types(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("ExceptionTypes", args, 0));
        }

        Ok(list_expr(
            self.exception_types
                .iter()
                .map(|(type_name, parent)| {
                    exception_registration_expr(type_name, parent.as_deref())
                })
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_throw_exception(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ThrowException", args, 1, 2));
        }

        let Some(type_name) = expr_as_string(&args[0]).map(str::to_owned) else {
            return Ok(self.invalid_call_with_arg_form(
                "ThrowException",
                args,
                "exception type must be a string",
            ));
        };
        let payload = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| association_expr(Vec::new()));
        if !is_association_expr(&payload) {
            return Ok(self.invalid_call_with_arg_form(
                "ThrowException",
                args,
                "exception payload must be an Association",
            ));
        }

        let exception =
            if type_name == "Exception" || self.exception_types.contains_key(type_name.as_str()) {
                exception_expr(&type_name, payload)
            } else {
                self.push_message(
                    "ThrowException",
                    "unreg",
                    format!("Exception type {type_name:?} is not registered."),
                );
                exception_expr(
                    "Exception",
                    association_expr(vec![
                        string_rule_expr("OriginalType", Expr::string(type_name)),
                        string_rule_expr("OriginalPayload", payload),
                    ]),
                )
            };

        Err(EvalError::ExceptionThrown { exception })
    }

    pub(in crate::runtime) fn eval_catch_exceptions(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("CatchExceptions", args, 1, 2));
        }

        let mut handlers = match args.get(1) {
            None => None,
            Some(expr) => match parse_exception_handlers(expr) {
                Some(handlers) => Some(handlers),
                None => {
                    return Ok(self.invalid_call_with_arg_form(
                        "CatchExceptions",
                        args,
                        "handlers must be a List of rules",
                    ));
                }
            },
        };

        let body_result = self.evaluate(args[0].clone());
        let outcome = crate::resume_or_suspend!(
            self,
            body_result,
            ok(value) => Ok(value),
            suspend => {
        let handlers = handlers.take().map(Arc::<[ExceptionHandler]>::from);
        EvalCatchExceptionsSuspensionFrame { handlers }
        },
        );

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_catch_exceptions_after_body(handlers, outcome),
        }
    }

    fn resume_catch_exceptions_after_body(
        &mut self,
        handlers: Option<Vec<ExceptionHandler>>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::ExceptionThrown { exception }) => {
                let Some(handlers) = handlers else {
                    return Ok(exception);
                };
                let Some((type_name, payload)) = exception_parts(&exception) else {
                    return Err(EvalError::ExceptionThrown { exception });
                };
                let type_chain = self.exception_type_chain(&type_name);
                self.dispatch_exception_handlers_owned(
                    exception, payload, type_chain, handlers, 0, 0,
                )
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_catch_exceptions_after_body(
        &mut self,
        handlers: Option<Arc<[ExceptionHandler]>>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::ExceptionThrown { exception }) => {
                let Some(handlers) = handlers else {
                    return Ok(exception);
                };
                let Some((type_name, payload)) = exception_parts(&exception) else {
                    return Err(EvalError::ExceptionThrown { exception });
                };
                let state = SuspendedExceptionDispatch {
                    exception,
                    payload,
                    type_chain: Arc::from(self.exception_type_chain(&type_name)),
                    handlers,
                    handler_index: 0,
                    ancestor_index: 0,
                };
                self.dispatch_suspended_exception_handlers(state)
            }
            Err(error) => Err(error),
        }
    }

    fn exception_type_chain(&self, type_name: &str) -> Vec<String> {
        let mut chain = vec![type_name.to_owned()];
        let mut seen = BTreeSet::from([type_name.to_owned()]);
        let mut current = type_name.to_owned();

        while let Some(Some(parent)) = self.exception_types.get(&current) {
            if parent == "Exception" || !seen.insert(parent.clone()) {
                break;
            }
            chain.push(parent.clone());
            current = parent.clone();
        }

        if chain.last().is_none_or(|name| name != "Exception") {
            chain.push("Exception".to_string());
        }
        chain
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch_exception_handlers_owned(
        &mut self,
        exception: Expr,
        payload: Expr,
        type_chain: Vec<String>,
        handlers: Vec<ExceptionHandler>,
        mut handler_index: usize,
        mut ancestor_index: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(handler) = handlers.get(handler_index) else {
                return Err(EvalError::ExceptionThrown { exception });
            };
            if ancestor_index >= type_chain.len() {
                handler_index += 1;
                ancestor_index = 0;
                continue;
            }

            let candidate = exception_expr(&type_chain[ancestor_index], payload.clone());
            let match_result =
                self.match_expr_owned(handler.pattern.clone(), candidate, SymbolExprMap::default());
            match match_result {
                Ok(outcome) => {
                    if !outcome.matched {
                        ancestor_index += 1;
                        continue;
                    }
                    if handler.guards.is_empty() {
                        return self.evaluate_exception_handler_rhs(handler, outcome.bindings);
                    }

                    let false_bindings = outcome.bindings.clone();
                    let guard_result = self.evaluate_guard_sequence_owned(
                        handler.guards.clone(),
                        outcome.bindings,
                        false_bindings,
                        None,
                    );
                    match guard_result {
                        Ok(outcome) if outcome.matched => {
                            return self.evaluate_exception_handler_rhs(handler, outcome.bindings);
                        }
                        Ok(_) => {
                            ancestor_index += 1;
                        }
                        Err(EvalError::InputRequired { request }) => {
                            let state = SuspendedExceptionDispatch {
                                exception,
                                payload,
                                type_chain: Arc::from(type_chain),
                                handlers: Arc::from(handlers),
                                handler_index,
                                ancestor_index,
                            };
                            crate::resume_or_suspend!(insert self, DispatchExceptionHandlersOwnedSuspensionFrame1 { state });
                            return Err(EvalError::InputRequired { request });
                        }
                        Err(error) => return Err(error),
                    }
                }
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedExceptionDispatch {
                        exception,
                        payload,
                        type_chain: Arc::from(type_chain),
                        handlers: Arc::from(handlers),
                        handler_index,
                        ancestor_index,
                    };
                    crate::resume_or_suspend!(insert self, DispatchExceptionHandlersOwnedSuspensionFrame2 { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn dispatch_suspended_exception_handlers(
        &mut self,
        mut state: SuspendedExceptionDispatch,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(handler) = state.handlers.get(state.handler_index) else {
                return Err(EvalError::ExceptionThrown {
                    exception: state.exception,
                });
            };
            if state.ancestor_index >= state.type_chain.len() {
                state.handler_index += 1;
                state.ancestor_index = 0;
                continue;
            }

            let candidate = exception_expr(
                &state.type_chain[state.ancestor_index],
                state.payload.clone(),
            );
            let match_result =
                self.match_expr_owned(handler.pattern.clone(), candidate, SymbolExprMap::default());
            match match_result {
                Ok(outcome) => {
                    return self.resume_suspended_exception_match_outcome(state, outcome);
                }
                Err(EvalError::InputRequired { request }) => {
                    crate::resume_or_suspend!(insert self, DispatchSuspendedExceptionHandlersSuspensionFrame { state });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_exception_match(
        &mut self,
        state: SuspendedExceptionDispatch,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = CompiledMatchOutcome::from_continuation_expr(outcome?);
        self.resume_suspended_exception_match_outcome(state, outcome)
    }

    fn resume_suspended_exception_match_outcome(
        &mut self,
        mut state: SuspendedExceptionDispatch,
        outcome: CompiledMatchOutcome,
    ) -> Result<Expr, EvalError> {
        if !outcome.matched {
            state.ancestor_index += 1;
            return self.dispatch_suspended_exception_handlers(state);
        }

        let handler = &state.handlers[state.handler_index];
        if handler.guards.is_empty() {
            return self.evaluate_exception_handler_rhs(handler, outcome.bindings);
        }

        let false_bindings = outcome.bindings.clone();
        let guard_result = self.evaluate_guard_sequence_owned(
            handler.guards.clone(),
            outcome.bindings,
            false_bindings,
            None,
        );
        match guard_result {
            Ok(outcome) => self.resume_suspended_exception_guard_outcome(state, outcome),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, ResumeSuspendedExceptionMatchOutcomeSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn resume_suspended_exception_guard(
        &mut self,
        state: SuspendedExceptionDispatch,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = CompiledMatchOutcome::from_continuation_expr(outcome?);
        self.resume_suspended_exception_guard_outcome(state, outcome)
    }

    fn resume_suspended_exception_guard_outcome(
        &mut self,
        mut state: SuspendedExceptionDispatch,
        outcome: CompiledMatchOutcome,
    ) -> Result<Expr, EvalError> {
        if outcome.matched {
            let handler = &state.handlers[state.handler_index];
            self.evaluate_exception_handler_rhs(handler, outcome.bindings)
        } else {
            state.ancestor_index += 1;
            self.dispatch_suspended_exception_handlers(state)
        }
    }

    fn evaluate_exception_handler_rhs(
        &mut self,
        handler: &ExceptionHandler,
        bindings: SymbolExprMap,
    ) -> Result<Expr, EvalError> {
        let rhs = substitute_rule_bindings_for_rhs(
            &handler.rhs,
            &bindings,
            handler.rhs_may_splice_without_bindings,
        );
        let rhs_result = self.evaluate(rhs);
        crate::resume_or_suspend!(
            self,
            rhs_result,
            ok(value) => Ok(value),
            suspend => EvaluateExceptionHandlerRhsSuspensionFrame,
        )
    }
}

fn parse_exception_handlers(expr: &Expr) -> Option<Vec<ExceptionHandler>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(parse_exception_handler).collect()
}

fn parse_exception_handler(expr: &Expr) -> Option<ExceptionHandler> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }
    let (rhs, guards) = split_definition_rhs_guards(args[1].clone());
    let rhs_may_splice_without_bindings = rule_rhs_may_splice_without_bindings(&rhs);

    Some(ExceptionHandler {
        pattern: args[0].clone(),
        rhs,
        rhs_may_splice_without_bindings,
        guards,
    })
}

fn exception_parts(exception: &Expr) -> Option<(String, Expr)> {
    let Expr::Call { head, args } = exception else {
        return None;
    };
    if !is_system_symbol_name(head, "Exception") || args.len() != 2 {
        return None;
    }

    Some((expr_as_string(&args[0])?.to_owned(), args[1].clone()))
}

fn exception_registration_expr(type_name: &str, parent: Option<&str>) -> Expr {
    Expr::call(
        system_expr("Rule"),
        vec![
            Expr::string(type_name),
            Expr::string(display_exception_parent(parent)),
        ],
    )
}

fn display_exception_parent(parent: Option<&str>) -> &str {
    parent.unwrap_or("Exception")
}

fn exception_expr(type_name: &str, payload: Expr) -> Expr {
    Expr::call(
        system_expr("Exception"),
        vec![Expr::string(type_name), payload],
    )
}

fn association_expr(entries: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Association"), entries)
}

fn string_rule_expr(key: &str, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![Expr::string(key), value])
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn is_association_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Association"))
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ExceptionHandler>();
    assert_traces::<SuspendedExceptionDispatch>();
};
