use super::super::*;
use super::definition_lhs_preparation::{
    DefinitionLhsPreparationContinuation, PreparedDefinitionLhs, PreparedUnsetDefinitionTarget,
};

impl Session {
    pub(in crate::runtime) fn refresh_has_up_value_definitions(&mut self) {
        self.has_up_value_definitions = self
            .definitions
            .values()
            .any(|definition| !definition.up_values.is_empty());
    }

    pub(in crate::runtime) fn eval_unset(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [target] => self.prepare_definition_lhs(
                target.clone(),
                DefinitionLhsPreparationContinuation::Unset {
                    source: target.clone(),
                },
            ),
            _ => Ok(self.invalid_call_with_exact_arg_count("Unset", args, 1)),
        }
    }

    pub(super) fn finish_prepared_unset(
        &mut self,
        prepared: PreparedDefinitionLhs,
        source: Expr,
    ) -> Result<Expr, EvalError> {
        match self.prepared_unset_target(prepared) {
            Ok(target) => {
                if self.ensure_symbol_not_protected_for_write("Unset", target.symbol()) {
                    self.unset_prepared_definition(target);
                }
                Ok(system_expr("Null"))
            }
            Err(error) => Ok(self.invalid_call_with_arg_form("Unset", &[source], error.message())),
        }
    }

    pub(in crate::runtime) fn eval_clear(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_clear_like("Clear", args, false, false, false, false, false)
    }

    pub(in crate::runtime) fn eval_clear_all(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_clear_like("ClearAll", args, true, true, true, true, true)
    }

    pub(in crate::runtime) fn eval_remove(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let symbols = match self.normalize_clear_targets(args) {
            Ok(symbols) => symbols,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Remove", args, error.message()));
            }
        };

        for symbol in symbols {
            if self.symbol_is_protected(&symbol) {
                self.push_message(
                    "Remove",
                    "rmptc",
                    format!("Symbol {symbol} is Protected and cannot be removed."),
                );
                continue;
            }

            self.remove_symbol_state(&symbol);
        }

        Ok(system_expr("Null"))
    }

    pub(in crate::runtime) fn eval_clear_like(
        &mut self,
        name: &str,
        args: &[Expr],
        clear_attributes: bool,
        clear_messages: bool,
        clear_options: bool,
        clear_syntax_information: bool,
        clear_default_values: bool,
    ) -> Result<Expr, EvalError> {
        let symbols = match self.normalize_clear_targets(args) {
            Ok(symbols) => symbols,
            Err(error) => return Ok(self.invalid_call_with_arg_form(name, args, error.message())),
        };

        for symbol in symbols {
            if self.ensure_symbol_not_protected_for_write(name, &symbol) {
                self.clear_symbol_state(
                    &symbol,
                    clear_attributes,
                    clear_messages,
                    clear_options,
                    clear_syntax_information,
                    clear_default_values,
                );
            }
        }

        Ok(system_expr("Null"))
    }

    fn unset_prepared_definition(&mut self, target: PreparedUnsetDefinitionTarget) {
        match target {
            PreparedUnsetDefinitionTarget::Message(key) => {
                if self.message_definitions.remove(&key).is_some() {
                    self.discount_message_symbol(&key.symbol);
                }
            }
            PreparedUnsetDefinitionTarget::Options(symbol) => {
                self.options.remove(&symbol);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::Options,
                );
            }
            PreparedUnsetDefinitionTarget::SyntaxInformation(symbol) => {
                self.syntax_information.remove(&symbol);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
                );
            }
            PreparedUnsetDefinitionTarget::OwnValue(symbol) => {
                let mut changed = false;
                if let Some(definition) = self.definitions.get_mut(&symbol) {
                    changed = definition.clear_own_values();
                }

                self.prune_definition_if_empty(&symbol);
                if changed {
                    self.bump_evaluation_epoch_for_symbol(&symbol);
                }
            }
            PreparedUnsetDefinitionTarget::Definition(prepared) => {
                self.remove_prepared_definition(prepared);
            }
            PreparedUnsetDefinitionTarget::TaggedNoop(_) => {}
        }
    }

    pub(in crate::runtime) fn prune_definition_if_empty(&mut self, symbol: &Symbol) {
        if self
            .definitions
            .get(symbol)
            .is_some_and(SymbolDefinition::is_empty)
        {
            self.definitions.remove(symbol);
            self.bump_evaluation_epoch_for_symbol(symbol);
        }
    }

    pub(in crate::runtime) fn clear_symbol_state(
        &mut self,
        symbol: &Symbol,
        clear_attributes: bool,
        clear_messages: bool,
        clear_options: bool,
        clear_syntax_information: bool,
        clear_default_values: bool,
    ) {
        let removed_definition = self.definitions.remove(symbol);
        let removed_up_values = removed_definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());
        let mut changed = removed_definition.is_some();

        if clear_attributes {
            if self.attributes.remove(symbol).is_some() {
                self.bump_attribute_revision();
            }
        }

        if clear_messages {
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
            );
        }

        if clear_options {
            self.options.remove(symbol);
            self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Options);
        }

        if clear_syntax_information {
            self.syntax_information.remove(symbol);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::SyntaxInformation,
            );
        }

        if clear_default_values {
            changed |= self.remove_default_values_for_symbol(symbol);
        }

        if removed_up_values {
            self.refresh_has_up_value_definitions();
        }

        if changed {
            self.bump_evaluation_epoch_for_symbol(symbol);
        }

        if clear_attributes
            && clear_messages
            && clear_options
            && clear_syntax_information
            && clear_default_values
            && !self.in_gensym_sweep
        {
            self.remove_gc_candidate_gensym(symbol);
        }
    }

    pub(in crate::runtime) fn remove_symbol_state(&mut self, symbol: &Symbol) {
        self.mark_symbol_removed(symbol);
        self.clear_symbol_state(symbol, true, true, true, true, true);
        if self.declared_stubs.remove(symbol) {
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::DeclaredStubs,
            );
            self.bump_evaluation_epoch_for_symbol(symbol);
        }
    }

    fn mark_symbol_removed(&mut self, symbol: &Symbol) {
        if self.removed_symbols.insert(*symbol) {
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::RemovedSymbols,
            );
            self.bump_evaluation_epoch_for_symbol(symbol);
        }

        if symbol.context() == SYSTEM_CONTEXT {
            return;
        }

        if crate::runtime::scoping::packages::symbol_is_sealed_private(symbol) {
            return;
        }

        let key = symbol_generation_key(symbol.context(), symbol.name());
        let current_generation = self.symbol_generations.get(&key).copied().unwrap_or(0);
        let removed_generation = symbol.identity().unwrap_or(0);
        self.symbol_generations
            .insert(key, current_generation.max(removed_generation) + 1);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::SymbolGenerations,
        );
    }

    pub(in crate::runtime) fn remove_default_values_for_symbol(&mut self, symbol: &Symbol) -> bool {
        let default_symbol = Symbol::system("Default");
        let Some(definition) = self.definitions.get_mut(&default_symbol) else {
            return false;
        };

        let original_len = definition.down_values.len();
        definition
            .down_values
            .retain(|entry| !default_value_lhs_targets_symbol(&entry.lhs, symbol));
        let changed = definition.down_values.len() != original_len;

        if definition.is_empty() {
            self.definitions.remove(&default_symbol);
        } else if changed {
            definition.note_down_values_rebuilt(&default_symbol);
        }

        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::Definitions);

        changed
    }
}
