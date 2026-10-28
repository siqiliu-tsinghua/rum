use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

enum OwnValueRead {
    Unguarded(Expr),
    Guarded { body: Expr, guards: Box<[Expr]> },
}

impl Session {
    pub(super) fn evaluate_symbol(&mut self, symbol: Symbol) -> Result<Expr, EvalError> {
        if symbol.context() == SYSTEM_CONTEXT {
            match symbol.name() {
                "$Context" => return Ok(Expr::string(self.current_context.clone())),
                "$ContextPath" => {
                    let contexts = self
                        .context_path
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), contexts));
                }
                "$Packages" => {
                    let packages = self
                        .loaded_packages
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), packages));
                }
                "$InputFileName" => {
                    return Ok(Expr::string(
                        self.load_stack
                            .last()
                            .map(|path| path.display().to_string())
                            .unwrap_or_default(),
                    ));
                }
                "$Path" => {
                    let paths = self
                        .package_search_paths
                        .iter()
                        .map(|path| Expr::string(path.display().to_string()))
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), paths));
                }
                "$MessageList" => {
                    let messages = self.current_message_list();
                    return Ok(Expr::call(system_expr("List"), messages));
                }
                _ => {}
            }
        }

        if self.declared_stubs.contains(&symbol)
            && !self
                .loaded_packages
                .iter()
                .any(|loaded| loaded == symbol.context())
        {
            let outcome = self.eval_needs(&[Expr::string(symbol.context().to_string())]);
            return crate::resume_or_suspend!(
                self, outcome,
                ok(value) => self.finish_declared_stub_load(symbol, value),
                suspend => crate::runtime::builtins_context_loader::DeclaredStubLoadFrame { symbol },
            );
        }

        self.evaluate_symbol_after_autoload(symbol)
    }

    pub(in crate::runtime) fn finish_declared_stub_load(
        &mut self,
        symbol: Symbol,
        load_result: Expr,
    ) -> Result<Expr, EvalError> {
        if !self.capability_policy.is_unrestricted()
            && !self.capability_policy.allows(Capability::FileRead)
        {
            debug_assert!(is_system_symbol_name(&load_result, "$Failed"));
            return Ok(load_result);
        }
        self.evaluate_symbol_after_autoload(symbol)
    }

    #[inline(always)]
    fn evaluate_symbol_after_autoload(&mut self, symbol: Symbol) -> Result<Expr, EvalError> {
        if let Some(value) = self.definitions.get(&symbol).and_then(|definition| {
            own_value_read(&definition.own_value, &definition.own_value_guards)
        }) {
            return self.evaluate_stored_own_value(symbol, value, false);
        }

        if let Some(value) = self.definitions.get(&symbol).and_then(|definition| {
            own_value_read(&definition.delayed_own_value, &definition.own_value_guards)
        }) {
            return self.evaluate_stored_own_value(symbol, value, true);
        }

        self.evaluate_symbol_after_stored_own_value(symbol)
    }

    fn evaluate_stored_own_value(
        &mut self,
        symbol: Symbol,
        value: OwnValueRead,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        match value {
            OwnValueRead::Unguarded(value) => {
                self.evaluate_accepted_own_value(&symbol, value, delayed)
            }
            OwnValueRead::Guarded { body, guards } => {
                let symbol_expr = Expr::Atom(Atom::Symbol(symbol));
                let option_scope =
                    self.option_value_scope_for_definition(&symbol_expr, &symbol_expr);
                let guard_result = self.evaluate_guard_sequence_owned(
                    guards,
                    SymbolExprMap::default(),
                    SymbolExprMap::default(),
                    option_scope,
                );
                crate::resume_typed_or_suspend!(
                    self,
                    guard_result,
                    ok(outcome) => { let session = self; (|| {
                                    if outcome.matched {
                                        session.evaluate_accepted_own_value(&symbol, body, delayed)
                                    } else {
                                        session.evaluate_symbol_after_stored_own_value(symbol)
                                    }
                                })() },
                    suspend => EvaluateStoredOwnValueSuspensionFrame { symbol, body, delayed },
                )
            }
        }
    }

    fn evaluate_accepted_own_value(
        &mut self,
        symbol: &Symbol,
        value: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        if matches!(&value, Expr::Atom(Atom::Symbol(value_symbol)) if value_symbol == symbol) {
            return Ok(value);
        }
        if delayed {
            self.evaluate_definition_rhs(None, value)
        } else {
            self.evaluate_own_value(value)
        }
    }

    fn evaluate_symbol_after_stored_own_value(
        &mut self,
        symbol: Symbol,
    ) -> Result<Expr, EvalError> {
        if let Some(value) = self.dynamic_own_value_binding(&symbol) {
            match value {
                Some(value) => self.evaluate_own_value(value),
                None => Ok(Expr::Atom(Atom::Symbol(symbol))),
            }
        } else if self.removed_symbols.contains(&symbol) {
            Ok(removed_symbol_expr(&symbol))
        } else {
            Ok(Expr::Atom(Atom::Symbol(symbol)))
        }
    }

    fn evaluate_own_value(&mut self, value: Expr) -> Result<Expr, EvalError> {
        match &value {
            Expr::Atom(Atom::Integer(_) | Atom::Rational(_) | Atom::String(_)) => Ok(value),
            _ => self.evaluate(value),
        }
    }

    pub(in crate::runtime) fn push_dynamic_own_value_binding(
        &mut self,
        symbol: Symbol,
        value: Option<Expr>,
    ) -> usize {
        let depth = self.dynamic_own_values.len();
        self.dynamic_own_values
            .push(DynamicOwnValueFrame { symbol, value });
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::DynamicOwnValues,
        );
        depth
    }

    pub(in crate::runtime) fn truncate_dynamic_own_value_bindings(&mut self, depth: usize) {
        if depth >= self.dynamic_own_values.len() {
            return;
        }

        self.dynamic_own_values.truncate(depth);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::DynamicOwnValues,
        );
    }

    pub(in crate::runtime) fn dynamic_own_value_binding(
        &self,
        symbol: &Symbol,
    ) -> Option<Option<Expr>> {
        self.dynamic_own_values
            .iter()
            .rev()
            .find(|frame| frame.symbol == *symbol)
            .map(|frame| frame.value.clone())
    }

    pub(in crate::runtime) fn dynamic_scope_symbol_state_is_clean(&self, symbol: &Symbol) -> bool {
        self.definitions.get(symbol).is_none()
            && !self.attributes.contains_key(symbol)
            && !self.options.contains_key(symbol)
            && !self.syntax_information.contains_key(symbol)
            && self
                .message_symbol_counts
                .get(symbol)
                .copied()
                .unwrap_or_default()
                == 0
    }

    pub(in crate::runtime) fn clear_dynamic_scope_symbol_side_effects(&mut self, symbol: &Symbol) {
        let current_had_up_values = self
            .definitions
            .get(symbol)
            .is_some_and(|definition| !definition.up_values.is_empty());
        if self.definitions.remove(symbol).is_some() {
            self.bump_evaluation_epoch_for_symbol(symbol);
        }
        if current_had_up_values {
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

        if self
            .message_symbol_counts
            .get(symbol)
            .copied()
            .unwrap_or_default()
            > 0
        {
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
            );
            self.rebuild_message_symbol_counts();
        }
    }

    pub(super) fn has_dynamic_own_value_binding(&self, symbol: &Symbol) -> bool {
        self.dynamic_own_values
            .iter()
            .rev()
            .any(|frame| frame.symbol == *symbol)
    }

    fn current_message_list(&self) -> Vec<Expr> {
        if let Some(checkpoint) = self
            .current_message_list_checkpoint
            .filter(|_| self.current_message_list_dirty)
        {
            return self
                .messages
                .after(checkpoint)
                .cloned()
                .map(held_runtime_message_name_expr)
                .collect();
        }

        self.last_message_list
            .iter()
            .cloned()
            .map(held_runtime_message_name_expr)
            .collect()
    }
}

fn own_value_read(value: &Option<Expr>, guards: &Option<Box<[Expr]>>) -> Option<OwnValueRead> {
    value.as_ref().map(|value| match guards {
        Some(guards) => OwnValueRead::Guarded {
            body: value.clone(),
            guards: guards.clone(),
        },
        None => OwnValueRead::Unguarded(value.clone()),
    })
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvaluateStoredOwnValueSuspensionFrame {
    symbol: rum_core::Symbol,
    body: rum_core::Expr,
    delayed: bool,
}

impl crate::runtime::suspension::TypedFrame for EvaluateStoredOwnValueSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        _token: crate::runtime::suspension::ContinueToken<'resume, Self>,
    ) -> crate::runtime::suspension::TypedAction<Expr> {
        #[allow(unused_mut)]
        let mut symbol = self.symbol.clone();
        #[allow(unused_mut)]
        let mut body = self.body.clone();
        #[allow(unused_mut)]
        let mut delayed = self.delayed.clone();
        let resumed = match outcome {
            Ok(outcome) => (|| -> Result<rum_core::Expr, EvalError> {
                if outcome.matched {
                    session.evaluate_accepted_own_value(&symbol, body, delayed)
                } else {
                    session.evaluate_symbol_after_stored_own_value(symbol)
                }
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::TypedAction::complete(resumed)
    }
}
