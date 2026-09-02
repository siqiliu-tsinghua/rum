use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionMemoryFieldDecision {
    Accounted(&'static str),
    Exempt(&'static str),
}

macro_rules! session_memory_field_decisions {
    ($($field:ident => $decision:ident($detail:literal),)*) => {
        const SESSION_MEMORY_FIELD_DECISIONS: &[(&str, SessionMemoryFieldDecision)] = &[
            $(
                (
                    stringify!($field),
                    SessionMemoryFieldDecision::$decision($detail),
                ),
            )*
        ];

        fn assert_session_memory_decisions_are_exhaustive(session: &Session) {
            let Session {
                $($field: _,)*
            } = session;
        }
    };
}

// This sorted inventory is also a compile-time completeness gate: the macro's
// exhaustive Session pattern stops compiling when a field is added without an
// ACCOUNTED or EXEMPT row here.
session_memory_field_decisions! {
    abandonment_active => Exempt("fixed-size value already covered by size_of::<Session>()"),
    abandonment_worklist => Accounted("suspended owners plus Vec spare capacity"),
    abort_protect_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    abort_protect_pending => Exempt("fixed-size value already covered by size_of::<Session>()"),
    active_accepted_request => Exempt("fixed-size wrapper already covered by size_of::<Session>(); Arc contents are already counted through request_interrupt_control"),
    asserts_enabled => Exempt("fixed-size value already covered by size_of::<Session>()"),
    association_lookup_cache => Accounted("AssociationLookupCache::memory_proxy_bytes"),
    attribute_revision => Exempt("fixed-size value already covered by size_of::<Session>()"),
    attribute_set_cache => Accounted("hash-table capacity and live Symbol text"),
    attributes => Accounted("live attribute-key Symbol text"),
    builtin_dispatch_spec_cache => Accounted("hash-table capacity and live Symbol text"),
    builtins => Accounted("live BuiltinSpecMap entry storage"),
    call_stability_cache => Accounted("hash-table capacity and retained SharedExpr/SharedArgs values"),
    call_stability_generation => Exempt("fixed-size value already covered by size_of::<Session>()"),
    capability_policy => Exempt("fixed-size value already covered by size_of::<Session>()"),
    context_path => Accounted("String capacities and Vec capacity"),
    context_stack => Accounted("ContextFrame contents and Vec spare capacity"),
    current_context => Accounted("String capacity"),
    current_memory_limit => Exempt("fixed-size value already covered by size_of::<Session>()"),
    current_message_list_checkpoint => Exempt("fixed-size value already covered by size_of::<Session>()"),
    current_message_list_dirty => Exempt("fixed-size value already covered by size_of::<Session>()"),
    current_time_deadline => Exempt("fixed-size value already covered by size_of::<Session>()"),
    declared_stubs => Accounted("live Symbol values and text"),
    definitions => Accounted("definition keys, values, rules, guards, and dispatch indexes"),
    disabled_message_selectors => Accounted("selector strings and Vec spare capacity"),
    dynamic_own_values => Accounted("DynamicOwnValueFrame symbols, expressions, and Vec spare capacity"),
    entropy_source => Accounted("Arc allocation header and concrete EntropySource object"),
    eval_deadline => Exempt("fixed-size value already covered by size_of::<Session>()"),
    eval_timeout => Exempt("fixed-size value already covered by size_of::<Session>()"),
    evaluation_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    evaluation_epoch => Exempt("fixed-size value already covered by size_of::<Session>()"),
    evaluation_stack => Accounted("EvaluationStackEntry expressions and Vec spare capacity"),
    exception_types => Accounted("BTreeMap entries and owned String capacities"),
    file_io => Accounted("Arc allocation header and concrete FileIo object"),
    gc_candidate_gensyms => Accounted("hash-table capacity and live Symbol text"),
    gensym_gc_sweep_count => Exempt("fixed-size test-only value already covered by size_of::<Session>()"),
    gensym_gc_watermark => Exempt("fixed-size value already covered by size_of::<Session>()"),
    handled_non_loader_eval_errors => Accounted("RuntimeMessage strings and Vec spare capacity"),
    has_up_value_definitions => Exempt("fixed-size value already covered by size_of::<Session>()"),
    in_gensym_sweep => Exempt("fixed-size value already covered by size_of::<Session>()"),
    installed_system_down_values => Accounted("hash-table storage, Symbol text, rule expressions, and nested Vec spare capacity"),
    installed_system_own_values => Accounted("hash-table capacity and live Symbol text"),
    interrupt_armed => Exempt("fixed-size value already covered by size_of::<Session>()"),
    interrupt_poll_counter => Exempt("fixed-size value already covered by size_of::<Session>()"),
    iteration_product_budget => Exempt("fixed-size value already covered by size_of::<Session>()"),
    last_message_list => Accounted("RuntimeMessage strings and Vec spare capacity"),
    load_stack => Accounted("PathBuf payloads and Vec capacity"),
    loaded_packages => Accounted("String capacities and Vec capacity"),
    match_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    memory_limit_check_counter => Exempt("fixed-size value already covered by size_of::<Session>()"),
    message_definitions => Accounted("message keys, tags, and definition expressions"),
    message_symbol_counts => Accounted("hash-table capacity and live Symbol text"),
    messages => Accounted("RuntimeMessage strings and Vec spare capacity"),
    module_activations => Accounted("ModuleActivation gensym vectors and outer Vec spare capacity"),
    monitor_callbacks_enabled => Exempt("fixed-size value already covered by size_of::<Session>()"),
    monitor_scopes => Accounted("MonitorScope expressions, display String, and Vec spare capacity"),
    native_stack_base => Exempt("fixed-size value already covered by size_of::<Session>()"),
    native_stack_next_check_depth => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_generated_symbol_id => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_request_generation => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_sealed_symbol_id => Exempt("fixed-size value already covered by size_of::<Session>()"),
    next_source_unit_epoch => Exempt("fixed-size value already covered by size_of::<Session>()"),
    once_values => Accounted("owned String keys and Expr values"),
    option_value_scopes => Accounted("OptionValueScope symbols, options, and Vec spare capacity"),
    options => Accounted("Symbol keys and SymbolOptions entries"),
    package_search_paths => Accounted("PathBuf payloads and Vec capacity"),
    peak_memory_proxy_bytes => Exempt("fixed-size value already covered by size_of::<Session>()"),
    pending_accepted_request => Exempt("fixed-size wrapper already covered by size_of::<Session>(); Arc contents are already counted through request_interrupt_control"),
    pending_eval_request => Accounted("existing conditional PendingEvalRequest conservative term"),
    pending_input_request => Accounted("RuntimeInputRequest prompt capacity"),
    pending_suspension => Accounted("owned SuspendedEvaluation stack, frames, and roots"),
    print_sink => Accounted("concrete boxed FnMut object"),
    prints => Accounted("String capacities and Vec capacity"),
    prng_state => Exempt("fixed-size value already covered by size_of::<Session>()"),
    queued_input_responses => Accounted("String capacities and VecDeque capacity"),
    reap_scopes => Accounted("ReapScope patterns, handlers, buckets, and Vec spare capacity"),
    removed_symbols => Accounted("live Symbol values and text"),
    request_interrupt_control => Accounted("Arc allocation header and fixed RequestInterruptState object"),
    resource_limits => Accounted("existing ResourceLimits conservative term"),
    restoring_snapshot => Exempt("fixed-size value already covered by size_of::<Session>()"),
    resuming_suspension => Accounted("owned SuspendedEvaluation stack, frames, and roots"),
    sealed_units => Accounted("SourceUnit strings, expressions, symbol vectors, private index, and outer Vec spare capacity"),
    stability_dirty_bloom => Exempt("fixed-size two-u128 value already covered by size_of::<Session>()"),
    stability_rewritten_dirty_bloom => Exempt("fixed-size two-u128 value already covered by size_of::<Session>()"),
    stable_symbol_cache => Accounted("hash-table capacity and live Symbol text"),
    symbol_cache_generation => Exempt("fixed-size value already covered by size_of::<Session>()"),
    symbol_call_effect_cache => Accounted("hash-table capacity and live key Symbol text"),
    symbol_generations => Accounted("owned context/name String capacities and generation entries"),
    syntax_information => Accounted("Symbol keys and Expr values"),
    trace_event_depths => Accounted("Vec capacity"),
    trace_events => Accounted("TraceEvent strings, expressions, and Vec spare capacity"),
    tracing_enabled => Exempt("fixed-size value already covered by size_of::<Session>()"),
    value_q_effect_suppression_active => Exempt("fixed-size value already covered by size_of::<Session>()"),
}

#[test]
fn session_memory_field_decision_inventory_is_sorted_and_exhaustive() {
    let session = Session::empty();
    assert_session_memory_decisions_are_exhaustive(&session);

    assert_eq!(SESSION_MEMORY_FIELD_DECISIONS.len(), 93);
    assert!(
        SESSION_MEMORY_FIELD_DECISIONS
            .windows(2)
            .all(|entries| entries[0].0 < entries[1].0),
        "the Session memory-field decision inventory must stay sorted"
    );

    for (field, decision) in SESSION_MEMORY_FIELD_DECISIONS {
        let detail = match decision {
            SessionMemoryFieldDecision::Accounted(term) => term,
            SessionMemoryFieldDecision::Exempt(reason) => {
                assert!(
                    reason.starts_with("fixed-size"),
                    "{field} uses an unsupported exemption reason: {reason}"
                );
                reason
            }
        };
        assert!(!detail.is_empty(), "{field} must record a term or reason");
    }
}

#[test]
fn stability_blooms_are_fixed_size_inline_values() {
    assert_eq!(
        std::mem::size_of::<StabilityBloom>(),
        2 * std::mem::size_of::<u128>()
    );
}

#[test]
fn session_memory_walk_counts_declared_exception_storage() {
    let mut session = Session::empty();
    let before = session.current_memory_proxy_bytes_for_test();

    let mut name = String::with_capacity(97);
    name.push_str("WalkChild");
    let name_capacity = name.capacity();
    let mut parent = String::with_capacity(131);
    parent.push_str("WalkParent");
    let parent_capacity = parent.capacity();
    session.exception_types.insert(name, Some(parent));

    let after = session.current_memory_proxy_bytes_for_test();
    let expected_delta = std::mem::size_of::<(String, Option<String>)>()
        .saturating_add(name_capacity)
        .saturating_add(parent_capacity);

    assert_eq!(after.saturating_sub(before), expected_delta);
}

#[test]
fn byte_array_proxy_tracks_encoded_string_capacity_and_preserves_invalid_fallback() {
    let mut session = Session::empty();
    let byte_array_head = system_expr("ByteArray");
    let Expr::Atom(Atom::Symbol(head_symbol)) = &byte_array_head else {
        unreachable!("ByteArray head should be a symbol")
    };
    let head_bytes = std::mem::size_of::<Expr>()
        .saturating_add(head_symbol.context().len())
        .saturating_add(head_symbol.name().len());

    for (payload, decoded_len) in [("AQID", Some(3usize)), ("not-base64", None)] {
        let mut encoded = String::with_capacity(64);
        encoded.push_str(payload);
        let encoded_capacity = encoded.capacity();
        let mut args = Vec::with_capacity(1);
        args.push(Expr::string(encoded));
        assert_eq!(args.capacity(), 1);
        let byte_array = Expr::call(byte_array_head.clone(), args);

        let actual = session
            .eval_byte_count(&system_expr("ByteCount"), std::slice::from_ref(&byte_array))
            .expect("ByteCount should accept one expression");
        let expected = std::mem::size_of::<Expr>()
            .saturating_add(head_bytes)
            .saturating_add(std::mem::size_of::<Expr>())
            .saturating_add(encoded_capacity);

        assert_eq!(actual, Expr::integer(BigInt::from(expected as u64)));
        if let Some(decoded_len) = decoded_len {
            let decoded_substitution = std::mem::size_of::<Expr>()
                .saturating_add(head_bytes)
                .saturating_add(decoded_len);
            assert_ne!(
                actual,
                Expr::integer(BigInt::from(decoded_substitution as u64)),
                "valid ByteArray text must not be replaced by its decoded length"
            );
        }
    }
}
