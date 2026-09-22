use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum SessionMemoryFieldDecision {
    Accounted(&'static str),
    Exempt(&'static str),
}

macro_rules! field_charge {
    (Accounted, $field:ident, $charge:expr) => {
        $charge
    };
    (Exempt, $field:ident) => {{
        let _ = $field;
        Bytes::Exact(0)
    }};
}

// One row provides both the classification and its independent formula. Neither
// exhaustive Session pattern has `..`: an unclassified new field cannot compile.
macro_rules! session_memory_field_decisions {
    ($($field:ident => $decision:ident($detail:literal) $(= $charge:expr)?,)*) => {
        pub(in crate::runtime) const SESSION_MEMORY_FIELD_DECISIONS:
            &[(&str, SessionMemoryFieldDecision)] = &[
                $((stringify!($field), SessionMemoryFieldDecision::$decision($detail)),)*
            ];

        pub(in crate::runtime) fn assert_session_memory_decisions_are_exhaustive(session: &Session) {
            let Session { $($field: _,)* } = session;
        }

        pub(in crate::runtime) fn session_breakdown(session: &Session) -> MemoryBreakdown {
            let Session { $($field,)* } = session;
            MemoryBreakdown {
                session_inline: Bytes::Exact(size_of::<Session>()),
                fields: vec![$(FieldContribution {
                    field: stringify!($field),
                    bytes: field_charge!($decision, $field $(, $charge)?),
                },)*],
            }
        }
    };
}

// Exemption texts are preserved from the previous session_memory inventory.
session_memory_field_decisions! {
    abandonment_active => Exempt("fixed-size value already covered by size_of::<Session>()"),
    abandonment_worklist => Accounted("suspended owners plus Vec spare capacity") =
        sequence(abandonment_worklist, SuspendedEvaluation::occurrence_bytes),
    abort_protect_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    abort_protect_pending => Exempt("fixed-size value already covered by size_of::<Session>()"),
    active_accepted_request => Exempt("fixed-size wrapper already covered by size_of::<Session>(); Arc contents are already counted through request_interrupt_control"),
    asserts_enabled => Exempt("fixed-size value already covered by size_of::<Session>()"),
    association_lookup_cache => Accounted("AssociationLookupCache::memory_proxy_bytes") =
        association_lookup_cache.occurrence_bytes(),
    attribute_revision => Exempt("fixed-size value already covered by size_of::<Session>()"),
    attribute_set_cache => Accounted("hash-table capacity and live Symbol text") =
        symbol_map(attribute_set_cache),
    attributes => Accounted("live attribute-key Symbol text") =
        sum(attributes.keys().map(|key| Bytes::Exact(key.to_string().len()))),
    builtin_dispatch_spec_cache => Accounted("hash-table capacity and live Symbol text") =
        symbol_map(builtin_dispatch_spec_cache),
    builtin_memory_projection_count => Exempt("fixed-size value already covered by size_of::<Session>()"),
    builtins => Accounted("live BuiltinSpecMap entry storage") =
        slots::<(Symbol, BuiltinSpec)>(builtins.len()),
    call_stability_cache => Accounted("hash-table capacity and retained SharedExpr/SharedArgs values") =
        slots::<(CallStabilityCacheKey, CallStabilityCacheEntry)>(call_stability_cache.capacity()).add(sum(call_stability_cache.values().map(|entry| expression(entry.head.as_ref()).add(shared_args(&entry.args))))),
    call_stability_generation => Exempt("fixed-size value already covered by size_of::<Session>()"),
    capability_policy => Exempt("fixed-size value already covered by size_of::<Session>()"),
    context_path => Accounted("String capacities and Vec capacity") =
        strings(context_path),
    context_stack => Accounted("ContextFrame contents and Vec spare capacity") =
        sequence(context_stack, context_frame),
    current_context => Accounted("String capacity") =
        Bytes::Exact(current_context.capacity()),
    current_memory_limit => Exempt("fixed-size value already covered by size_of::<Session>()"),
    current_message_list_checkpoint => Exempt("fixed-size value already covered by size_of::<Session>()"),
    current_message_list_dirty => Exempt("fixed-size value already covered by size_of::<Session>()"),
    current_time_deadline => Exempt("fixed-size value already covered by size_of::<Session>()"),
    declared_stubs => Accounted("live Symbol values and text") =
        sum(declared_stubs.iter().map(symbol)),
    definitions => Accounted("definition keys, values, rules, guards, and dispatch indexes") =
        sum(definitions.iter().map(|(key, value)| Bytes::Exact(key.to_string().len()).add(definition(value)))),
    disabled_message_selectors => Accounted("selector strings and Vec spare capacity") =
        sequence(disabled_message_selectors, selector),
    dynamic_own_values => Accounted("DynamicOwnValueFrame symbols, expressions, and Vec spare capacity") =
        sequence(dynamic_own_values, dynamic_frame),
    entropy_source => Accounted("Arc allocation header and concrete EntropySource object") =
        entropy_source.as_ref().map_or(Bytes::Exact(0), |provider| arc_object(provider.as_entropy_source())),
    eval_deadline => Exempt("fixed-size value already covered by size_of::<Session>()"),
    eval_timeout => Exempt("fixed-size value already covered by size_of::<Session>()"),
    evaluation_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    evaluation_epoch => Exempt("fixed-size value already covered by size_of::<Session>()"),
    evaluation_stack => Accounted("EvaluationStackEntry expressions and Vec spare capacity") =
        sequence(evaluation_stack, evaluation_entry),
    exception_types => Accounted("BTreeMap entries and owned String capacities") =
        sum(exception_types.iter().map(|(name, parent)| Bytes::Exact(size_of::<(String, Option<String>)>()).add(Bytes::Exact(name.capacity())).add(Bytes::Exact(parent.as_ref().map_or(0, String::capacity))))),
    file_io => Accounted("Arc allocation header and concrete FileIo object") =
        file_io.as_ref().map_or(Bytes::Exact(0), |provider| arc_object(provider.as_file_io())),
    gc_candidate_gensyms => Accounted("hash-table capacity and live Symbol text") =
        symbol_set(gc_candidate_gensyms),
    gensym_gc_sweep_count => Exempt("fixed-size test-only value already covered by size_of::<Session>()"),
    gensym_gc_watermark => Exempt("fixed-size value already covered by size_of::<Session>()"),
    handled_non_loader_eval_errors => Accounted("RuntimeMessage strings and Vec spare capacity") =
        sequence(handled_non_loader_eval_errors, message),
    has_up_value_definitions => Exempt("fixed-size value already covered by size_of::<Session>()"),
    in_gensym_sweep => Exempt("fixed-size value already covered by size_of::<Session>()"),
    installed_system_down_values => Accounted("hash-table storage, Symbol text, rule expressions, and nested Vec spare capacity") =
        slots::<(Symbol, Vec<(Expr, Expr)>)>(installed_system_down_values.capacity()).add(sum(installed_system_down_values.iter().map(|(key, rules)| symbol_text(key).add(sequence(rules, |(lhs, rhs)| expression(lhs).add(expression(rhs))))))),
    installed_system_own_values => Accounted("hash-table capacity and live Symbol text") =
        symbol_set(installed_system_own_values),
    interrupt_armed => Exempt("fixed-size value already covered by size_of::<Session>()"),
    interrupt_poll_counter => Exempt("fixed-size value already covered by size_of::<Session>()"),
    iteration_product_budget => Exempt("fixed-size value already covered by size_of::<Session>()"),
    last_message_list => Accounted("RuntimeMessage strings and Vec spare capacity") =
        sequence(last_message_list, message),
    load_stack => Accounted("PathBuf payloads and Vec capacity") =
        paths(load_stack),
    loaded_packages => Accounted("String capacities and Vec capacity") =
        strings(loaded_packages),
    match_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    message_definitions => Accounted("message keys, tags, and definition expressions") =
        sum(message_definitions.iter().map(|(key, value)| Bytes::Exact(key.symbol.to_string().len()).add(Bytes::Exact(key.tag.len())).add(optional_expr(value.immediate.as_ref())).add(optional_expr(value.delayed.as_ref())))),
    message_symbol_counts => Accounted("hash-table capacity and live Symbol text") =
        symbol_map(message_symbol_counts),
    messages => Accounted("RuntimeMessage strings and Vec spare capacity") =
        sequence(messages, message),
    module_activations => Accounted("ModuleActivation gensym vectors and outer Vec spare capacity") =
        sequence(module_activations, activation),
    monitor_callbacks_enabled => Exempt("fixed-size value already covered by size_of::<Session>()"),
    monitor_scopes => Accounted("MonitorScope expressions, display String, and Vec spare capacity") =
        sequence(monitor_scopes, monitor),
    native_stack_base => Exempt("fixed-size value already covered by size_of::<Session>()"),
    native_stack_next_check_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_generated_symbol_id => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_request_generation => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_sealed_symbol_id => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_source_unit_epoch => Exempt("fixed-size value already covered by size_of::<Session>()"),
    once_values => Accounted("owned String keys and Expr values") =
        sum(once_values.iter().map(|(key, value)| Bytes::Exact(key.capacity()).add(expression(value)))),
    option_value_scopes => Accounted("OptionValueScope symbols, options, and Vec spare capacity") =
        sequence(option_value_scopes, option_scope),
    options => Accounted("Symbol keys and SymbolOptions entries") =
        sum(options.iter().map(|(key, value)| Bytes::Exact(key.to_string().len()).add(payloads::options(value)))),
    package_search_paths => Accounted("PathBuf payloads and Vec capacity") =
        paths(package_search_paths),
    peak_memory_proxy_bytes => Exempt("fixed-size value already covered by size_of::<Session>()"),
    pending_accepted_request => Exempt("fixed-size wrapper already covered by size_of::<Session>(); Arc contents are already counted through request_interrupt_control"),
    pending_eval_request => Accounted("existing conditional PendingEvalRequest conservative term") =
        Bytes::Exact(pending_eval_request.as_ref().map_or(0, |_| size_of::<PendingEvalRequest>())),
    pending_input_request => Accounted("RuntimeInputRequest prompt capacity") =
        pending_input_request.as_ref().map_or(Bytes::Exact(0), |request| Bytes::Exact(size_of::<RuntimeInputRequest>()).add(Bytes::Exact(request.prompt.capacity())).add(Bytes::Exact(1))),
    pending_suspension => Accounted("owned SuspendedEvaluation stack, frames, and roots") =
        pending_suspension.as_ref().map_or(Bytes::Exact(0), SuspendedEvaluation::occurrence_bytes),
    print_sink => Accounted("concrete boxed FnMut object") =
        Bytes::Exact(print_sink.as_ref().map_or(0, |sink| size_of_val(sink.as_ref()))),
    prints => Accounted("String capacities and Vec capacity") =
        strings(prints),
    prng_state => Exempt("fixed-size value already covered by size_of::<Session>()"),
    queued_input_responses => Accounted("String capacities and VecDeque capacity") =
        slots::<String>(queued_input_responses.capacity()).add(sum(queued_input_responses.iter().map(|value| Bytes::Exact(value.capacity())))),
    reap_scopes => Accounted("ReapScope patterns, handlers, buckets, and Vec spare capacity") =
        sequence(reap_scopes, reap),
    removed_symbols => Accounted("live Symbol values and text") =
        sum(removed_symbols.iter().map(symbol)),
    request_interrupt_control => Accounted("Arc allocation header and fixed RequestInterruptState object") =
        arc_object(request_interrupt_control.borrow().as_ref()),
    resource_limits => Accounted("existing ResourceLimits conservative term") =
        Bytes::Exact(size_of_val(resource_limits)),
    restoring_snapshot => Exempt("fixed-size value already covered by size_of::<Session>()"),
    resuming_suspension => Accounted("owned SuspendedEvaluation stack, frames, and roots") =
        resuming_suspension.as_ref().map_or(Bytes::Exact(0), SuspendedEvaluation::occurrence_bytes),
    sealed_units => Accounted("SourceUnit strings, expressions, symbol vectors, private index, and outer Vec spare capacity") =
        sequence(sealed_units, source_unit),
    stability_dirty_bloom => Exempt("fixed-size two-u128 value already covered by size_of::<Session>()"),
    stability_native_stack_poll_counter => Exempt("fixed-size value already covered by size_of::<Session>()"),
    stability_rewritten_dirty_bloom => Exempt("fixed-size two-u128 value already covered by size_of::<Session>()"),
    stable_symbol_cache => Accounted("hash-table capacity and live Symbol text") =
        symbol_map(stable_symbol_cache),
    symbol_cache_generation => Exempt("fixed-size value already covered by size_of::<Session>()"),
    symbol_call_effect_cache => Accounted("hash-table capacity and live key Symbol text") =
        slots::<(SymbolCallEffectKey, CallEffectCacheEntry)>(symbol_call_effect_cache.capacity()).add(sum(symbol_call_effect_cache.keys().map(|key| symbol_text(&key.symbol)))),
    symbol_generations => Accounted("owned context/name String capacities and generation entries") =
        sum(symbol_generations.iter().map(|((context, name), _)| Bytes::Exact(size_of::<((String, String), u64)>()).add(Bytes::Exact(context.capacity())).add(Bytes::Exact(name.capacity())))),
    syntax_information => Accounted("Symbol keys and Expr values") =
        sum(syntax_information.iter().map(|(key, value)| Bytes::Exact(key.to_string().len()).add(expression(value)))),
    trace_event_depths => Accounted("Vec capacity") =
        slots::<usize>(trace_event_depths.capacity()),
    trace_events => Accounted("TraceEvent strings, expressions, and Vec spare capacity") =
        sequence(trace_events, trace),
    tracing_enabled => Exempt("fixed-size value already covered by size_of::<Session>()"),
    value_q_effect_suppression_active => Exempt("fixed-size value already covered by size_of::<Session>()"),
}
