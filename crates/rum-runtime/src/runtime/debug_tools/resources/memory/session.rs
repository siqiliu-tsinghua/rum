//! The complete observation walker, factored only at owner boundaries.
//! Each formula is preserved from the original session walk; MemoryInUse
//! always uses these actual owners, never maintained values.
use super::*;
use crate::runtime::debug_tools::resources::MemoryOwner;

impl Session {
    pub(in crate::runtime) fn walked_memory_owner_bytes(&self, owner: MemoryOwner) -> usize {
        match owner {
            MemoryOwner::AbandonmentWorklist => self
                .abandonment_worklist
                .iter()
                .map(render_suspended_evaluation_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.abandonment_worklist)),
            MemoryOwner::AccountingRoute => self
                .accounting_route
                .as_ref()
                .map_or(0, |route| route.memory_bytes()),
            MemoryOwner::AssociationLookupCache => {
                self.association_lookup_cache.memory_proxy_bytes()
            }
            MemoryOwner::AttributeSetCache => {
                render_symbol_keyed_hash_map_memory_proxy_bytes(&self.attribute_set_cache)
            }
            MemoryOwner::Attributes => self
                .attributes
                .keys()
                .map(|symbol| symbol.to_string().len())
                .sum::<usize>(),
            MemoryOwner::BuiltinDispatchSpecCache => {
                render_symbol_keyed_hash_map_memory_proxy_bytes(&self.builtin_dispatch_spec_cache)
            }
            MemoryOwner::Builtins => self
                .builtins
                .len()
                .saturating_mul(std::mem::size_of::<(Symbol, BuiltinSpec)>()),
            MemoryOwner::CallStabilityCache => {
                render_call_stability_cache_memory_proxy_bytes(&self.call_stability_cache)
            }
            MemoryOwner::ContextPath => render_string_vec_memory_proxy_bytes(&self.context_path),
            MemoryOwner::ContextStack => self
                .context_stack
                .iter()
                .map(render_context_frame_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.context_stack)),
            MemoryOwner::CurrentContext => self.current_context.capacity(),
            MemoryOwner::DeclaredStubs => self
                .declared_stubs
                .iter()
                .map(render_symbol_memory_proxy_bytes)
                .sum::<usize>(),
            MemoryOwner::Definitions => self
                .definitions
                .iter()
                .map(|(symbol, definition)| {
                    symbol.to_string().len() + render_definition_memory_proxy_bytes(definition)
                })
                .sum::<usize>(),
            MemoryOwner::DisabledMessageSelectors => self
                .disabled_message_selectors
                .iter()
                .map(render_message_selector_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.disabled_message_selectors)),
            MemoryOwner::DynamicOwnValues => self
                .dynamic_own_values
                .iter()
                .map(render_dynamic_own_value_frame_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.dynamic_own_values)),
            MemoryOwner::EntropySource => self
                .entropy_source
                .as_ref()
                .map_or(0, render_entropy_provider_memory_proxy_bytes),
            MemoryOwner::EvaluationStack => self.walked_evaluation_stack_bytes(),
            MemoryOwner::ExceptionTypes => {
                render_exception_types_memory_proxy_bytes(&self.exception_types)
            }
            MemoryOwner::FileIo => self
                .file_io
                .as_ref()
                .map_or(0, render_file_io_provider_memory_proxy_bytes),
            MemoryOwner::GcCandidateGensyms => {
                render_symbol_hash_set_memory_proxy_bytes(&self.gc_candidate_gensyms)
            }
            MemoryOwner::HandledNonLoaderEvalErrors => self
                .handled_non_loader_eval_errors
                .iter()
                .map(render_runtime_message_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(
                    &self.handled_non_loader_eval_errors,
                )),
            MemoryOwner::InstalledSystemDownValues => {
                render_installed_system_down_values_memory_proxy_bytes(
                    &self.installed_system_down_values,
                )
            }
            MemoryOwner::InstalledSystemOwnValues => {
                render_symbol_hash_set_memory_proxy_bytes(&self.installed_system_own_values)
            }
            MemoryOwner::LastMessageList => self
                .last_message_list
                .iter()
                .map(render_runtime_message_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.last_message_list)),
            MemoryOwner::LoadStack => render_path_buf_vec_memory_proxy_bytes(&self.load_stack),
            MemoryOwner::LoadedPackages => {
                render_string_vec_memory_proxy_bytes(&self.loaded_packages)
            }
            MemoryOwner::MessageDefinitions => self
                .message_definitions
                .iter()
                .map(|(key, definition)| {
                    key.symbol.to_string().len()
                        + key.tag.len()
                        + definition
                            .immediate
                            .as_ref()
                            .map_or(0, expr_memory_proxy_bytes)
                        + definition
                            .delayed
                            .as_ref()
                            .map_or(0, expr_memory_proxy_bytes)
                })
                .sum::<usize>(),
            MemoryOwner::MessageSymbolCounts => {
                render_symbol_keyed_hash_map_memory_proxy_bytes(&self.message_symbol_counts)
            }
            MemoryOwner::Messages => self
                .messages
                .iter()
                .map(render_runtime_message_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.messages)),
            MemoryOwner::ModuleActivations => self
                .module_activations
                .iter()
                .map(render_module_activation_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.module_activations)),
            MemoryOwner::MonitorScopes => self
                .monitor_scopes
                .iter()
                .map(render_monitor_scope_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.monitor_scopes)),
            MemoryOwner::OnceValues => self
                .once_values
                .iter()
                .map(|(key, value)| key.capacity() + expr_memory_proxy_bytes(value))
                .sum::<usize>(),
            MemoryOwner::OptionValueScopes => self
                .option_value_scopes
                .iter()
                .map(render_option_value_scope_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.option_value_scopes)),
            MemoryOwner::Options => self
                .options
                .iter()
                .map(|(symbol, options)| {
                    symbol.to_string().len() + render_symbol_options_memory_proxy_bytes(options)
                })
                .sum::<usize>(),
            MemoryOwner::PackageSearchPaths => {
                render_path_buf_vec_memory_proxy_bytes(&self.package_search_paths)
            }
            MemoryOwner::PendingEvalRequest => self
                .pending_eval_request
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<PendingEvalRequest>()),
            MemoryOwner::PendingInputRequest => self
                .pending_input_request
                .as_ref()
                .map_or(0, render_input_request_memory_proxy_bytes),
            MemoryOwner::PendingSuspension => self
                .pending_suspension
                .as_ref()
                .map_or(0, render_suspended_evaluation_memory_proxy_bytes),
            MemoryOwner::PrintSink => self
                .print_sink
                .as_ref()
                .map_or(0, |sink| std::mem::size_of_val(sink.as_ref())),
            MemoryOwner::Prints => render_string_vec_memory_proxy_bytes(&self.prints),
            MemoryOwner::QueuedInputResponses => {
                render_string_vecdeque_memory_proxy_bytes(&self.queued_input_responses)
            }
            MemoryOwner::ReapScopes => self
                .reap_scopes
                .iter()
                .map(render_reap_scope_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.reap_scopes)),
            MemoryOwner::RemovedSymbols => self
                .removed_symbols
                .iter()
                .map(render_symbol_memory_proxy_bytes)
                .sum::<usize>(),
            MemoryOwner::RequestInterruptControl => {
                arc_allocation_memory_proxy_bytes(self.request_interrupt_control.borrow().as_ref())
            }
            MemoryOwner::ResourceLimits => std::mem::size_of::<ResourceLimits>(),
            MemoryOwner::ResumingSuspension => self
                .resuming_suspension
                .as_ref()
                .map_or(0, render_suspended_evaluation_memory_proxy_bytes),
            MemoryOwner::SealedUnits => self
                .sealed_units
                .iter()
                .map(render_source_unit_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.sealed_units)),
            MemoryOwner::StableSymbolCache => {
                render_symbol_keyed_hash_map_memory_proxy_bytes(&self.stable_symbol_cache)
            }
            MemoryOwner::SymbolCallEffectCache => {
                render_symbol_call_effect_cache_memory_proxy_bytes(&self.symbol_call_effect_cache)
            }
            MemoryOwner::SymbolGenerations => self
                .symbol_generations
                .iter()
                .map(render_symbol_generation_memory_proxy_bytes)
                .sum::<usize>(),
            MemoryOwner::SyntaxInformation => self
                .syntax_information
                .iter()
                .map(|(symbol, expr)| symbol.to_string().len() + expr_memory_proxy_bytes(expr))
                .sum::<usize>(),
            MemoryOwner::TraceEventDepths => self
                .trace_event_depths
                .capacity()
                .saturating_mul(std::mem::size_of::<usize>()),
            MemoryOwner::TraceEvents => self
                .trace_events
                .iter()
                .map(render_trace_event_memory_proxy_bytes)
                .sum::<usize>()
                .saturating_add(vec_spare_capacity_bytes(&self.trace_events)),
        }
    }

    pub(super) fn walk_session_memory_bytes(&self) -> usize {
        let total = MemoryOwner::ALL
            .into_iter()
            .fold(std::mem::size_of::<Session>(), |total, owner| {
                total.saturating_add(self.walked_memory_owner_bytes(owner))
            });
        self.peak_memory_proxy_bytes
            .set(self.peak_memory_proxy_bytes.get().max(total));
        total
    }
}
