use super::*;
use crate::runtime::scoping::packages::SourceUnit;

use std::hash::BuildHasherDefault;
use std::sync::{Arc, atomic::AtomicBool};

use std::time::{Duration, Instant};

pub(in crate::runtime) type SymbolMap<T> =
    HashMap<Symbol, T, BuildHasherDefault<RuntimeCacheHasher>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct ModuleActivation {
    pub(in crate::runtime) gensyms: Vec<Symbol>,
    pub(in crate::runtime) escaped: bool,
    pub(in crate::runtime) side_table_stamp: SideTableStamp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct SideTableStamp {
    pub(in crate::runtime) once_values: usize,
    pub(in crate::runtime) monitor_scopes: usize,
    pub(in crate::runtime) option_value_scopes: usize,
    pub(in crate::runtime) dynamic_own_values: usize,
    pub(in crate::runtime) reap_bucket_values: usize,
}

pub struct Session {
    pub(in crate::runtime) builtins: BuiltinSpecMap,
    pub(in crate::runtime) attributes: AttributeMap,
    pub(in crate::runtime) attribute_revision: u64,
    pub(in crate::runtime) attribute_set_cache: AttributeSetCache,
    pub(in crate::runtime) builtin_dispatch_spec_cache: BuiltinDispatchSpecCache,
    pub(in crate::runtime) evaluation_epoch: u64,
    pub(in crate::runtime) symbol_cache_generation: u64,
    pub(in crate::runtime) call_stability_generation: u64,
    pub(in crate::runtime) stability_dirty_bloom: StabilityBloom,
    pub(in crate::runtime) stability_rewritten_dirty_bloom: StabilityBloom,
    pub(in crate::runtime) stable_symbol_cache: SymbolStabilityCache,
    pub(in crate::runtime) symbol_call_effect_cache: SymbolCallEffectCache,
    pub(in crate::runtime) call_stability_cache: CallStabilityCache,
    pub(in crate::runtime) association_lookup_cache: AssociationLookupCache,
    pub(in crate::runtime) options: BTreeMap<Symbol, SymbolOptions>,
    pub(in crate::runtime) syntax_information: BTreeMap<Symbol, Expr>,
    pub(in crate::runtime) declared_stubs: SymbolSet,
    pub(in crate::runtime) removed_symbols: SymbolSet,
    pub(in crate::runtime) symbol_generations: BTreeMap<(String, String), u64>,
    pub(in crate::runtime) once_values: BTreeMap<String, Expr>,
    pub(in crate::runtime) option_value_scopes: Vec<OptionValueScope>,
    pub(in crate::runtime) dynamic_own_values: Vec<DynamicOwnValueFrame>,
    pub(in crate::runtime) module_activations: Vec<ModuleActivation>,
    pub(in crate::runtime) gc_candidate_gensyms: std::collections::HashSet<Symbol>,
    pub(in crate::runtime) gensym_gc_watermark: usize,
    #[cfg(test)]
    pub(in crate::runtime) gensym_gc_sweep_count: usize,
    pub(in crate::runtime) in_gensym_sweep: bool,
    pub(in crate::runtime) current_context: String,
    pub(in crate::runtime) context_path: Vec<String>,
    pub(in crate::runtime) context_stack: Vec<ContextFrame>,
    pub(in crate::runtime) package_search_paths: Vec<PathBuf>,
    pub(in crate::runtime) loaded_packages: Vec<String>,
    pub(in crate::runtime) load_stack: Vec<PathBuf>,
    pub(in crate::runtime) file_io: Option<FileIoProvider>,
    pub(in crate::runtime) entropy_source: Option<EntropyProvider>,
    pub(in crate::runtime) prng_state: PrngState,
    pub(in crate::runtime) definitions: DefinitionMap,
    pub(in crate::runtime) installed_system_own_values: SymbolSet,
    pub(in crate::runtime) installed_system_down_values: SymbolMap<Vec<(Expr, Expr)>>,
    pub(in crate::runtime) has_up_value_definitions: bool,
    pub(in crate::runtime) message_definitions: BTreeMap<MessageNameKey, MessageDefinition>,
    pub(in crate::runtime) exception_types: BTreeMap<String, Option<String>>,
    pub(in crate::runtime) message_symbol_counts: SymbolCountMap,
    pub(in crate::runtime) next_generated_symbol_id: u64,
    pub(in crate::runtime) sealed_units: Vec<SourceUnit>,
    pub(in crate::runtime) next_sealed_symbol_id: u64,
    pub(in crate::runtime) next_source_unit_epoch: u64,
    // Transient execution state; intentionally absent from loader/rollback snapshots.
    pub(in crate::runtime) restoring_snapshot: bool,
    pub(in crate::runtime) resource_limits: ResourceLimits,
    pub(in crate::runtime) evaluation_depth: usize,
    pub(in crate::runtime) match_depth: usize,
    pub(in crate::runtime) native_stack_base: usize,
    pub(in crate::runtime) native_stack_next_check_depth: usize,
    pub(in crate::runtime) interrupt_armed: bool,
    pub(in crate::runtime) evaluation_stack: Vec<EvaluationStackEntry>,
    pub(in crate::runtime) tracing_enabled: bool,
    pub(in crate::runtime) trace_events: Vec<TraceEvent>,
    pub(in crate::runtime) trace_event_depths: Vec<usize>,
    pub(in crate::runtime) disabled_message_selectors: Vec<MessageSelector>,
    pub(in crate::runtime) messages: Vec<RuntimeMessage>,
    pub(in crate::runtime) prints: Vec<String>,
    pub(in crate::runtime) print_sink: Option<Box<dyn FnMut(&str) + Send>>,
    pub(in crate::runtime) peak_memory_proxy_bytes: std::cell::Cell<usize>,
    pub(in crate::runtime) last_message_list: Vec<RuntimeMessage>,
    pub(in crate::runtime) current_message_list_checkpoint: Option<usize>,
    pub(in crate::runtime) current_message_list_dirty: bool,
    pub(in crate::runtime) handled_non_loader_eval_errors: Vec<RuntimeMessage>,
    pub(in crate::runtime) reap_scopes: Vec<ReapScope>,
    pub(in crate::runtime) monitor_scopes: Vec<MonitorScope>,
    pub(in crate::runtime) monitor_callbacks_enabled: bool,
    pub(in crate::runtime) current_time_deadline: Option<Instant>,
    pub(in crate::runtime) current_memory_limit: Option<usize>,
    pub(in crate::runtime) memory_limit_check_counter: std::cell::Cell<u32>,
    pub(in crate::runtime) asserts_enabled: bool,
    pub(in crate::runtime) abort_protect_depth: usize,
    pub(in crate::runtime) abort_protect_pending: usize,
    pub(in crate::runtime) queued_input_responses: VecDeque<String>,
    pub(in crate::runtime) pending_input_request: Option<RuntimeInputRequest>,
    pub(in crate::runtime) pending_eval_request: Option<PendingEvalRequest>,
    pub(in crate::runtime) pending_suspension: Option<SuspendedEvaluation>,
    pub(in crate::runtime) resuming_suspension: Option<SuspendedEvaluation>,
    pub(in crate::runtime) next_suspension_generation: u64,
    // The protocol intentionally lands before its production callers (brief-403).
    #[allow(dead_code)]
    pub(in crate::runtime) abandonment_active: bool,
    #[allow(dead_code)]
    pub(in crate::runtime) abandonment_worklist: Vec<SuspendedEvaluation>,
    pub(in crate::runtime) interrupt_flag: Arc<AtomicBool>,
    pub(in crate::runtime) eval_timeout: Option<Duration>,
    pub(in crate::runtime) eval_deadline: Option<Instant>,
    pub(in crate::runtime) interrupt_poll_counter: u64,
    // Running iteration product paired with its `IterationConsumer` tag.
    pub(in crate::runtime) iteration_product_budget: std::cell::Cell<(usize, u8)>,
    pub(in crate::runtime) capability_policy: CapabilityPolicy,
    pub(in crate::runtime) value_q_effect_suppression_active: bool,
}

impl Session {
    /// Installs a host-selected destination for `Print` output.
    ///
    /// Installing a sink is the host's explicit choice about where print output
    /// goes. Without a sink, output is buffered into the request response. A host
    /// that owns stdout as a protocol channel must not install a sink that writes
    /// there.
    pub fn set_print_sink(&mut self, sink: Box<dyn FnMut(&str) + Send>) {
        self.print_sink = Some(sink);
    }

    pub fn clear_print_sink(&mut self) {
        self.print_sink = None;
    }

    pub(in crate::runtime) fn current_side_table_stamp(&self) -> SideTableStamp {
        SideTableStamp {
            once_values: self.once_values.len(),
            monitor_scopes: self.monitor_scopes.len(),
            option_value_scopes: self.option_value_scopes.len(),
            dynamic_own_values: self.dynamic_own_values.len(),
            reap_bucket_values: self
                .reap_scopes
                .iter()
                .flat_map(|scope| scope.buckets_by_pattern.iter())
                .flat_map(|buckets| buckets.iter())
                .map(|bucket| bucket.values.len())
                .sum(),
        }
    }

    pub(in crate::runtime) fn note_active_modules_escaped(&mut self) {
        if self.module_activations.is_empty() {
            return;
        }

        for activation in &mut self.module_activations {
            activation.escaped = true;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct EvaluationStackEntry {
    pub(in crate::runtime) expr: Expr,
    pub(in crate::runtime) epoch: u64,
}
