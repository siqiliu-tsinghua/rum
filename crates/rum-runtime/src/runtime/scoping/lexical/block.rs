mod continuation;
mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct SuspendedBlockInitializers {
    items: Arc<Vec<(Symbol, Option<Expr>)>>,
    body: Expr,
    saved_state: Arc<BTreeMap<Symbol, BlockSymbolState>>,
    dynamic_depth: usize,
    index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedBlockScope {
    saved_state: Arc<BTreeMap<Symbol, BlockSymbolState>>,
    dynamic_depth: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedSingleBlockScope {
    symbol: Symbol,
    body: Expr,
    saved_state: Arc<BlockSymbolState>,
    dynamic_depth: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_block(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Block", args, 2));
        }

        let config = match self.compile_scoped_call_from_raw_args(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Block", args, error.message()));
            }
        };
        let items = config.items;

        if items.len() == 1 {
            let (symbol, init) = items
                .into_iter()
                .next()
                .expect("single Block item should exist");
            return self.eval_single_symbol_block(symbol, init, args[1].clone());
        }

        let saved_state = self.capture_block_symbol_states(
            &items
                .iter()
                .map(|(symbol, _)| symbol.clone())
                .collect::<Vec<_>>(),
        );
        let mut attributes_changed = false;
        let mut messages_changed = false;
        for (symbol, _) in &items {
            if self.definitions.remove(symbol).is_some() {
                self.bump_evaluation_epoch_for_symbol(symbol);
            }
            attributes_changed |= self.attributes.remove(symbol).is_some();
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
            messages_changed |= self.message_definitions.len() != messages_before;
        }
        if !items.is_empty() {
            self.refresh_has_up_value_definitions();
        }
        if attributes_changed {
            self.bump_attribute_revision();
        }
        if messages_changed {
            self.rebuild_message_symbol_counts();
        }

        let dynamic_depth = self.push_block_dynamic_shadow_bindings(&items);
        self.resume_block_initializers(items, args[1].clone(), saved_state, dynamic_depth, 0)
    }

    fn eval_single_symbol_block(
        &mut self,
        symbol: Symbol,
        init: Option<Expr>,
        body: Expr,
    ) -> Result<Expr, EvalError> {
        let saved_state = self.capture_single_block_symbol_state(&symbol);
        self.clear_single_block_symbol_state(&symbol, &saved_state);
        let dynamic_depth = self.push_dynamic_own_value_binding(symbol, None);

        if let Some(init) = init {
            return match self.evaluate(init) {
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedSingleBlockScope {
                        symbol,
                        body,
                        saved_state: Arc::new(saved_state),
                        dynamic_depth,
                    };
                    crate::resume_or_suspend!(insert self, EvalSingleSymbolBlockSuspensionFrame { state });
                    crate::resume_or_suspend!(insert self, ContinueSingleBlockInitializerFrame);
                    Err(EvalError::InputRequired { request })
                }
                outcome => self.resume_single_symbol_block_after_initializer(
                    symbol,
                    body,
                    saved_state,
                    dynamic_depth,
                    outcome,
                ),
            };
        }

        self.resume_single_symbol_block_body(symbol, body, saved_state, dynamic_depth)
    }

    fn push_block_dynamic_shadow_bindings(&mut self, items: &[(Symbol, Option<Expr>)]) -> usize {
        let dynamic_depth = self.dynamic_own_values.len();
        for (symbol, _) in items {
            self.push_dynamic_own_value_binding(*symbol, None);
        }
        dynamic_depth
    }

    fn capture_single_block_symbol_state(&self, symbol: &Symbol) -> BlockSymbolState {
        BlockSymbolState {
            definition: self.definitions.get(symbol).cloned(),
            attributes: self.attributes.get(symbol).copied(),
            options: self.options.get(symbol).cloned(),
            syntax_information: self.syntax_information.get(symbol).cloned(),
            messages: self.single_symbol_message_definitions(symbol),
        }
    }

    fn single_symbol_message_definitions(
        &self,
        symbol: &Symbol,
    ) -> Vec<(MessageNameKey, MessageDefinition)> {
        if self
            .message_symbol_counts
            .get(symbol)
            .copied()
            .unwrap_or_default()
            == 0
        {
            return Vec::new();
        }

        self.message_definitions
            .iter()
            .filter(|(key, _)| key.symbol == *symbol)
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }

    fn clear_single_block_symbol_state(&mut self, symbol: &Symbol, saved_state: &BlockSymbolState) {
        let removed_up_values = saved_state
            .definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());

        if self.definitions.remove(symbol).is_some() {
            self.bump_evaluation_epoch_for_symbol(symbol);
        }
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

    fn capture_block_symbol_states(
        &self,
        symbols: &[Symbol],
    ) -> BTreeMap<Symbol, BlockSymbolState> {
        let mut states = BTreeMap::new();

        for symbol in symbols {
            states
                .entry(symbol.clone())
                .or_insert_with(|| BlockSymbolState {
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
                });
        }

        states
    }

    fn restore_block_symbol_states(&mut self, states: BTreeMap<Symbol, BlockSymbolState>) {
        let mut attributes_changed = false;
        let mut messages_changed = false;
        for (symbol, state) in states {
            let definition_changed = match state.definition {
                Some(definition) => {
                    self.definitions.insert(symbol, definition);
                    true
                }
                None => self.definitions.remove(&symbol).is_some(),
            };
            if definition_changed {
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }

            match state.attributes {
                Some(attributes) => {
                    self.attributes.insert(symbol, attributes);
                    attributes_changed = true;
                }
                None => {
                    attributes_changed |= self.attributes.remove(&symbol).is_some();
                }
            }

            match state.options {
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

            match state.syntax_information {
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
            messages_changed |=
                self.message_definitions.len() != messages_before || !state.messages.is_empty();
            for (key, definition) in state.messages {
                self.message_definitions.insert(key, definition);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
                );
            }
        }
        self.refresh_has_up_value_definitions();
        if attributes_changed {
            self.bump_attribute_revision();
        }
        if messages_changed {
            self.rebuild_message_symbol_counts();
        }
    }

    fn resume_block_initializers(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
        mut index: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some((symbol, init)) = items.get(index) else {
                return self.resume_block_body(body, saved_state, dynamic_depth);
            };

            let symbol = *symbol;
            if let Some(init) = init {
                match self.evaluate(init.clone()) {
                    Ok(value) => {
                        let definition = self.definitions.entry(symbol).or_default();
                        definition.store_own_value(value, Box::default(), false);
                        self.bump_evaluation_epoch_for_symbol(&symbol);
                        index += 1;
                    }
                    Err(EvalError::InputRequired { request }) => {
                        let state = SuspendedBlockInitializers {
                            items: Arc::new(items),
                            body,
                            saved_state: Arc::new(saved_state),
                            dynamic_depth,
                            index,
                        };
                        crate::resume_or_suspend!(insert self, ResumeBlockInitializersSuspensionFrame { state });
                        crate::resume_or_suspend!(insert self, ContinueBlockInitializerFrame);
                        return Err(EvalError::InputRequired { request });
                    }
                    Err(error) => {
                        self.truncate_dynamic_own_value_bindings(dynamic_depth);
                        self.restore_block_symbol_states(saved_state);
                        return Err(error);
                    }
                }
            } else {
                index += 1;
            }
        }
    }

    fn resume_block_after_initializer(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
        index: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let symbol = items[index].0;
                let definition = self.definitions.entry(symbol).or_default();
                definition.store_own_value(value, Box::default(), false);
                self.bump_evaluation_epoch_for_symbol(&symbol);
                self.resume_block_initializers(items, body, saved_state, dynamic_depth, index + 1)
            }
            Err(error) => {
                self.truncate_dynamic_own_value_bindings(dynamic_depth);
                self.restore_block_symbol_states(saved_state);
                Err(error)
            }
        }
    }

    fn resume_single_symbol_block_after_initializer(
        &mut self,
        symbol: Symbol,
        body: Expr,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.store_own_value(value, Box::default(), false);
                self.bump_evaluation_epoch_for_symbol(&symbol);
                self.resume_single_symbol_block_body(symbol, body, saved_state, dynamic_depth)
            }
            Err(error) => {
                self.truncate_dynamic_own_value_bindings(dynamic_depth);
                self.restore_single_block_symbol_state(symbol, saved_state);
                Err(error)
            }
        }
    }

    fn resume_single_symbol_block_body(
        &mut self,
        symbol: Symbol,
        body: Expr,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
    ) -> Result<Expr, EvalError> {
        match self.evaluate(body) {
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedSingleBlockScope {
                    symbol,
                    body: system_expr("Null"),
                    saved_state: Arc::new(saved_state),
                    dynamic_depth,
                };
                crate::resume_or_suspend!(insert self, ResumeSingleSymbolBlockBodySuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueSingleBlockBodyFrame);
                Err(EvalError::InputRequired { request })
            }
            outcome => {
                self.finish_single_symbol_block_scope(symbol, saved_state, dynamic_depth, outcome)
            }
        }
    }

    fn finish_single_symbol_block_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.restore_single_block_symbol_state(symbol, saved_state);
        outcome
    }

    pub(in crate::runtime) fn abandon_single_block_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
        dynamic_depth: usize,
    ) {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.restore_single_block_symbol_state(symbol, saved_state);
    }

    fn restore_single_block_symbol_state(&mut self, symbol: Symbol, saved_state: BlockSymbolState) {
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

        let current_message_count = self
            .message_symbol_counts
            .get(&symbol)
            .copied()
            .unwrap_or_default();
        if current_message_count > 0 || !saved_state.messages.is_empty() {
            self.message_definitions
                .retain(|key, _| key.symbol != symbol);
            self.refresh_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
            );
            for (key, definition) in saved_state.messages {
                self.message_definitions.insert(key, definition);
                self.refresh_memory_owner(
                    crate::runtime::debug_tools::resources::MemoryOwner::MessageDefinitions,
                );
            }
            self.rebuild_message_symbol_counts();
        }
    }

    fn resume_block_body(
        &mut self,
        body: Expr,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
    ) -> Result<Expr, EvalError> {
        match self.evaluate(body) {
            Err(EvalError::InputRequired { request }) => {
                let state = SuspendedBlockScope {
                    saved_state: Arc::new(saved_state),
                    dynamic_depth,
                };
                crate::resume_or_suspend!(insert self, ResumeBlockBodySuspensionFrame { state });
                crate::resume_or_suspend!(insert self, ContinueBlockBodyFrame);
                Err(EvalError::InputRequired { request })
            }
            outcome => self.finish_block_scope(saved_state, dynamic_depth, outcome),
        }
    }

    fn finish_block_scope(
        &mut self,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.restore_block_symbol_states(saved_state);
        outcome
    }

    pub(in crate::runtime) fn abandon_block_scope(
        &mut self,
        saved_state: BTreeMap<Symbol, BlockSymbolState>,
        dynamic_depth: usize,
    ) {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.restore_block_symbol_states(saved_state);
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedBlockInitializers>();
    assert_traces::<SuspendedBlockScope>();
    assert_traces::<SuspendedSingleBlockScope>();
};
