use super::*;

pub(in crate::runtime) fn print_lexical_profile_summary_once(session: &crate::runtime::Session) {
    let lexical_enabled = lexical_profile_enabled();
    let repr_enabled = rum_core::repr_profile_enabled();
    if !lexical_enabled && !repr_enabled {
        return;
    }
    if lexical_enabled {
        logical::drop_session(session);
    }
    let printed = {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        LEXICAL_PROFILE_SUMMARY_PRINTED.swap(1, Ordering::Relaxed)
    };
    if printed == 1 {
        return;
    }

    if lexical_enabled {
        print_lexical_profile_summary_body();
    }
    print_repr_profile_summary();
}

pub(in crate::runtime) fn print_lexical_profile_snapshot(label: &str) {
    let lexical_configured = lexical_profile_configured();
    let repr_enabled = rum_core::repr_profile_enabled();
    if !lexical_configured && !repr_enabled {
        return;
    }

    if lexical_configured {
        eprintln!("rum-profile lexical snapshot label={label}");
        print_lexical_profile_summary_body();
        mark_lexical_profile_summary_printed();
    }
    print_repr_profile_summary();
}

fn mark_lexical_profile_summary_printed() {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    LEXICAL_PROFILE_SUMMARY_PRINTED.store(1, Ordering::Relaxed);
}

fn print_repr_profile_summary() {
    if !rum_core::repr_profile_enabled() {
        return;
    }

    let profile = rum_core::repr_profile_snapshot();
    eprintln!(
        "rum-profile repr into_vec_total={} into_vec_clone={} into_owned_total={} into_owned_clone={} shared_args_alloc={} shared_expr_alloc={}",
        profile.into_vec_total,
        profile.into_vec_clone,
        profile.into_owned_total,
        profile.into_owned_clone,
        profile.shared_args_alloc,
        profile.shared_expr_alloc,
    );
}

fn print_lexical_profile_summary_body() {
    logical::summary();
    eprintln!(
        "rum-profile lexical summary definition_rhs_eval_count={} definition_rhs_eval_total_ns={} definition_rhs_eval_avg_ns={}",
        load(&DEFINITION_RHS_EVAL_COUNT),
        load(&DEFINITION_RHS_EVAL_NANOS),
        average(&DEFINITION_RHS_EVAL_NANOS, &DEFINITION_RHS_EVAL_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary eval_module_count={} eval_module_total_ns={} eval_module_avg_ns={}",
        load(&EVAL_MODULE_COUNT),
        load(&EVAL_MODULE_NANOS),
        average(&EVAL_MODULE_NANOS, &EVAL_MODULE_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary module_resume_entries={} module_resume_max_index={}",
        load(&MODULE_RESUME_ENTRY_COUNT),
        load(&MODULE_RESUME_MAX_INDEX),
    );
    eprintln!(
        "rum-profile lexical summary module_init_substitution_count={} module_init_substitution_total_ns={} module_init_substitution_avg_ns={}",
        load(&MODULE_INIT_SUBSTITUTION_COUNT),
        load(&MODULE_INIT_SUBSTITUTION_NANOS),
        average(
            &MODULE_INIT_SUBSTITUTION_NANOS,
            &MODULE_INIT_SUBSTITUTION_COUNT
        ),
    );
    eprintln!(
        "rum-profile lexical summary module_body_substitution_count={} module_body_substitution_total_ns={} module_body_substitution_avg_ns={}",
        load(&MODULE_BODY_SUBSTITUTION_COUNT),
        load(&MODULE_BODY_SUBSTITUTION_NANOS),
        average(
            &MODULE_BODY_SUBSTITUTION_NANOS,
            &MODULE_BODY_SUBSTITUTION_COUNT
        ),
    );
    eprintln!(
        "rum-profile lexical summary lexical_substitutions={} lexical_substitution_timed={} lexical_substitution_total_ns={} lexical_substitution_avg_ns={}",
        load(&LEXICAL_SUBSTITUTION_COUNT),
        load(&LEXICAL_SUBSTITUTION_TIME_COUNT),
        load(&LEXICAL_SUBSTITUTION_NANOS),
        average(
            &LEXICAL_SUBSTITUTION_NANOS,
            &LEXICAL_SUBSTITUTION_TIME_COUNT
        ),
    );
    eprintln!(
        "rum-profile lexical summary lexical_changed={} lexical_unchanged={} binding_empty={} binding_single={} binding_multi={} shadowed_nonempty={} binding_max_count={}",
        load(&LEXICAL_SUBSTITUTION_CHANGED_COUNT),
        load(&LEXICAL_SUBSTITUTION_UNCHANGED_COUNT),
        load(&LEXICAL_BINDING_EMPTY_COUNT),
        load(&LEXICAL_BINDING_SINGLE_COUNT),
        load(&LEXICAL_BINDING_MULTI_COUNT),
        load(&LEXICAL_SHADOWED_NONEMPTY_COUNT),
        load(&LEXICAL_BINDING_MAX_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary arg_list_substitutions={} arg_list_total_args={} arg_list_max_len={} arg_prefilter_skip={} arg_prefilter_maybe={}",
        load(&ARG_LIST_SUBSTITUTION_COUNT),
        load(&ARG_LIST_TOTAL_ARGS),
        load(&ARG_LIST_MAX_LEN),
        load(&ARG_PREFILTER_SKIP_COUNT),
        load(&ARG_PREFILTER_MAYBE_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary arg_substitution_attempts={} arg_substitution_total_ns={} arg_substitution_avg_ns={} arg_substitution_changed={} arg_substitution_unchanged={}",
        load(&ARG_SUBSTITUTION_ATTEMPT_COUNT),
        load(&ARG_SUBSTITUTION_ATTEMPT_NANOS),
        average(
            &ARG_SUBSTITUTION_ATTEMPT_NANOS,
            &ARG_SUBSTITUTION_ATTEMPT_COUNT
        ),
        load(&ARG_SUBSTITUTION_CHANGED_COUNT),
        load(&ARG_SUBSTITUTION_UNCHANGED_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary presence_single_scans={} presence_single_nodes={} presence_single_timed={} presence_single_total_ns={} presence_single_avg_ns={} presence_single_true={} presence_single_false={}",
        load(&PRESENCE_SINGLE_SCAN_COUNT),
        load(&PRESENCE_SINGLE_SCAN_NODES),
        load(&PRESENCE_SINGLE_SCAN_TIME_COUNT),
        load(&PRESENCE_SINGLE_SCAN_NANOS),
        average(
            &PRESENCE_SINGLE_SCAN_NANOS,
            &PRESENCE_SINGLE_SCAN_TIME_COUNT
        ),
        load(&PRESENCE_SINGLE_SCAN_TRUE_COUNT),
        load(&PRESENCE_SINGLE_SCAN_FALSE_COUNT),
    );
    eprintln!(
        "rum-profile lexical summary presence_multi_scans={} presence_multi_nodes={} presence_multi_timed={} presence_multi_total_ns={} presence_multi_avg_ns={} presence_multi_true={} presence_multi_false={}",
        load(&PRESENCE_MULTI_SCAN_COUNT),
        load(&PRESENCE_MULTI_SCAN_NODES),
        load(&PRESENCE_MULTI_SCAN_TIME_COUNT),
        load(&PRESENCE_MULTI_SCAN_NANOS),
        average(&PRESENCE_MULTI_SCAN_NANOS, &PRESENCE_MULTI_SCAN_TIME_COUNT),
        load(&PRESENCE_MULTI_SCAN_TRUE_COUNT),
        load(&PRESENCE_MULTI_SCAN_FALSE_COUNT),
    );
    print_presence_shape_summary();
    print_presence_context_summary();
    print_presence_context_shape_summary();
    print_presence_arg_site_summary();
    print_presence_source_summary();
    print_presence_source_shape_summary();
    print_presence_source_target_shape_summary();
    print_presence_source_fingerprint_summary();
    print_presence_source_fingerprint_group_summary();
    print_presence_provenance_summary();
    print_presence_reuse_summary();
    print_presence_reuse_group_summary();
    print_presence_fingerprint_summary();
    print_presence_fingerprint_group_summary();
    print_presence_exact_fingerprint_group_summary();
    print_expression_rebuild_summary();
    print_expression_rebuild_group_summary();
    print_expression_rebuild_exact_summary();
    print_expression_rebuild_exact_group_summary();
    print_rule_rhs_alpha_rename_summary();
    print_rule_rhs_timing_summary();
    print_definition_rule_timing_summary();
    print_definition_dispatch_summary();
    print_runtime_path_timing_summary();
    print_builtin_timing_summary();
    print_rewrite_rule_prefilter_summary();
    print_presence_binding_summary();
    print_presence_binding_name_summary();
}
