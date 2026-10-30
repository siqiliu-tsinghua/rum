use super::*;

pub(in crate::runtime) fn record_profile_rewrite_rule_prefilter_candidate(would_reject: bool) {
    if !lexical_profile_enabled() {
        return;
    }

    {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        REWRITE_RULE_CANDIDATE_COUNT.fetch_add(1, Ordering::Relaxed);
        if would_reject {
            REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT.fetch_add(1, Ordering::Relaxed);
        } else {
            REWRITE_RULE_WOULD_MATCHER_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    let Some(source_symbol) = evaluation_profile_current_source_symbol() else {
        return;
    };
    let source_symbol = Symbol::new(source_symbol.context(), source_symbol.name());
    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_symbol).or_default();
        entry.candidates = entry.candidates.saturating_add(1);
        if would_reject {
            entry.would_reject = entry.would_reject.saturating_add(1);
        } else {
            entry.would_matcher = entry.would_matcher.saturating_add(1);
        }
    });
}

pub(in crate::runtime) fn record_profile_rewrite_rule_match_outcome(
    would_reject: bool,
    matched: bool,
) {
    if !lexical_profile_enabled() {
        return;
    }

    {
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
            return;
        };
        if matched {
            REWRITE_RULE_MATCH_COUNT.fetch_add(1, Ordering::Relaxed);
            if would_reject {
                REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        } else {
            REWRITE_RULE_MATCHER_MISS_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    let Some(source_symbol) = evaluation_profile_current_source_symbol() else {
        return;
    };
    let source_symbol = Symbol::new(source_symbol.context(), source_symbol.name());
    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(source_symbol).or_default();
        if matched {
            entry.matches = entry.matches.saturating_add(1);
            if would_reject {
                entry.false_rejects = entry.false_rejects.saturating_add(1);
            }
        } else {
            entry.matcher_misses = entry.matcher_misses.saturating_add(1);
        }
    });
}

pub(in crate::runtime) fn record_profile_definition_rhs_eval(start: Option<Instant>) {
    record_timing(
        &DEFINITION_RHS_EVAL_COUNT,
        &DEFINITION_RHS_EVAL_NANOS,
        "definition_rhs_eval",
        start,
    );
}

pub(in crate::runtime) fn record_profile_eval_module(start: Option<Instant>) {
    record_timing(&EVAL_MODULE_COUNT, &EVAL_MODULE_NANOS, "eval_module", start);
}

pub(in crate::runtime) fn record_profile_module_resume_entry(index: usize) {
    if !lexical_profile_enabled() {
        return;
    }

    record_counter(&MODULE_RESUME_ENTRY_COUNT, "module_resume_entries");
    record_max(
        &MODULE_RESUME_MAX_INDEX,
        "module_resume_max_index",
        index as u64,
    );
}

pub(in crate::runtime) fn record_profile_module_init_substitution(start: Option<Instant>) {
    record_timing(
        &MODULE_INIT_SUBSTITUTION_COUNT,
        &MODULE_INIT_SUBSTITUTION_NANOS,
        "module_init_substitution",
        start,
    );
}

pub(in crate::runtime) fn record_profile_module_body_substitution(start: Option<Instant>) {
    record_timing(
        &MODULE_BODY_SUBSTITUTION_COUNT,
        &MODULE_BODY_SUBSTITUTION_NANOS,
        "module_body_substitution",
        start,
    );
}

pub(in crate::runtime) fn record_profile_lexical_substitution_start(
    binding_count: usize,
    shadowed_count: usize,
) {
    if !lexical_profile_enabled() {
        return;
    }

    record_counter(&LEXICAL_SUBSTITUTION_COUNT, "lexical_substitutions");
    if shadowed_count > 0 {
        record_counter(
            &LEXICAL_SHADOWED_NONEMPTY_COUNT,
            "lexical_substitutions_shadowed_nonempty",
        );
    }
    match binding_count {
        0 => record_counter(
            &LEXICAL_BINDING_EMPTY_COUNT,
            "lexical_substitutions_binding_empty",
        ),
        1 => record_counter(
            &LEXICAL_BINDING_SINGLE_COUNT,
            "lexical_substitutions_binding_single",
        ),
        _ => record_counter(
            &LEXICAL_BINDING_MULTI_COUNT,
            "lexical_substitutions_binding_multi",
        ),
    }
    record_max(
        &LEXICAL_BINDING_MAX_COUNT,
        "lexical_binding_max_count",
        binding_count as u64,
    );
}

pub(in crate::runtime) fn record_profile_lexical_substitution_finish(
    start: Option<Instant>,
    changed: bool,
) {
    if changed {
        record_counter(
            &LEXICAL_SUBSTITUTION_CHANGED_COUNT,
            "lexical_substitutions_changed",
        );
    } else {
        record_counter(
            &LEXICAL_SUBSTITUTION_UNCHANGED_COUNT,
            "lexical_substitutions_unchanged",
        );
    }
    record_profile_lexical_timing(
        &LEXICAL_SUBSTITUTION_COUNT,
        &LEXICAL_SUBSTITUTION_TIME_COUNT,
        &LEXICAL_SUBSTITUTION_NANOS,
        "lexical_substitution_time",
        start,
    );
}

pub(in crate::runtime) fn record_profile_arg_list_start(arg_count: usize) {
    if !lexical_profile_enabled() {
        return;
    }

    record_counter(&ARG_LIST_SUBSTITUTION_COUNT, "arg_list_substitutions");
    record_add(
        &ARG_LIST_TOTAL_ARGS,
        "arg_list_total_args",
        arg_count as u64,
    );
    record_max(&ARG_LIST_MAX_LEN, "arg_list_max_len", arg_count as u64);
}

pub(in crate::runtime) fn record_profile_arg_prefilter(maybe_contains_binding: bool) {
    if maybe_contains_binding {
        record_counter(&ARG_PREFILTER_MAYBE_COUNT, "arg_prefilter_maybe");
    } else {
        record_counter(&ARG_PREFILTER_SKIP_COUNT, "arg_prefilter_skip");
    }
}

pub(in crate::runtime) fn record_profile_arg_substitution_attempt(
    start: Option<Instant>,
    changed: bool,
) {
    if changed {
        record_counter(&ARG_SUBSTITUTION_CHANGED_COUNT, "arg_substitution_changed");
    } else {
        record_counter(
            &ARG_SUBSTITUTION_UNCHANGED_COUNT,
            "arg_substitution_unchanged",
        );
    }
    record_timing(
        &ARG_SUBSTITUTION_ATTEMPT_COUNT,
        &ARG_SUBSTITUTION_ATTEMPT_NANOS,
        "arg_substitution_attempt",
        start,
    );
}

pub(in crate::runtime) fn record_profile_expression_rebuild(source: &'static str, expr: &Expr) {
    if !lexical_profile_shapes_enabled() && !lexical_profile_provenance_enabled() {
        return;
    }

    let (fingerprint, exact_fingerprint, nodes) =
        structural_text_exact_fingerprints_and_nodes(expr);
    if nodes < PRESENCE_REUSE_MIN_NODES {
        return;
    }
    record_expression_provenance_from_fingerprint(
        exact_fingerprint,
        ExpressionProvenanceKind::for_rebuild_source(source),
        source,
        current_expression_rebuild_source_symbol(),
        max_rule_rhs_alpha_id(expr),
        nodes,
    );

    if !lexical_profile_shapes_enabled() {
        return;
    }

    let key = ExpressionRebuildKey {
        source,
        source_symbol: current_expression_rebuild_source_symbol(),
        shape: presence_root_shape(expr),
        fingerprint,
    };
    let exact_key = ExpressionRebuildKey {
        fingerprint: exact_fingerprint,
        ..key
    };
    EXPRESSION_REBUILD_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.rebuilds = entry.rebuilds.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
    });
    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(exact_key).or_default();
        entry.rebuilds = entry.rebuilds.saturating_add(1);
        entry.nodes = entry.nodes.saturating_add(nodes);
    });
}

pub(in crate::runtime) fn record_profile_rule_rhs_binding_value(expr: &Expr) {
    if !lexical_profile_provenance_enabled() {
        return;
    }

    let (_, exact_fingerprint, nodes) = structural_text_exact_fingerprints_and_nodes(expr);
    record_expression_provenance_from_fingerprint(
        exact_fingerprint,
        ExpressionProvenanceKind::RuleBindingValue,
        "rule_binding_value",
        current_expression_rebuild_source_symbol(),
        max_rule_rhs_alpha_id(expr),
        nodes,
    );
}

pub(in crate::runtime) fn record_profile_rule_rhs_alpha_rename(local: &Symbol, fresh: &Symbol) {
    if !lexical_profile_shapes_enabled() {
        return;
    }

    let key = RuleRhsAlphaRenameKey {
        source_symbol: current_expression_rebuild_source_symbol(),
        local_text: Symbol::new(local.context(), local.name()),
    };
    RULE_RHS_ALPHA_RENAME_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        let entry = stats.entry(key).or_default();
        entry.renames = entry.renames.saturating_add(1);
        if fresh.identity().is_some() {
            entry.fresh_with_identity = entry.fresh_with_identity.saturating_add(1);
        }
    });
}

pub(in crate::runtime) fn record_profile_single_presence_scan(
    start: Option<Instant>,
    expr: &Expr,
    target: &Symbol,
    result: bool,
    nodes: u64,
) {
    let elapsed_nanos = start.map(elapsed_nanos_since);
    record_counter(&PRESENCE_SINGLE_SCAN_COUNT, "presence_single_scans");
    record_profile_lexical_timing_elapsed(
        &PRESENCE_SINGLE_SCAN_COUNT,
        &PRESENCE_SINGLE_SCAN_TIME_COUNT,
        &PRESENCE_SINGLE_SCAN_NANOS,
        "presence_single_scan_time",
        elapsed_nanos,
    );
    record_add(
        &PRESENCE_SINGLE_SCAN_NODES,
        "presence_single_scan_nodes",
        nodes,
    );
    if result {
        record_counter(
            &PRESENCE_SINGLE_SCAN_TRUE_COUNT,
            "presence_single_scan_true",
        );
    } else {
        record_counter(
            &PRESENCE_SINGLE_SCAN_FALSE_COUNT,
            "presence_single_scan_false",
        );
    }
    let target_text = Symbol::new(target.context(), target.name());
    record_presence_shape(PresenceScanKind::Single, expr, result, nodes, elapsed_nanos);
    record_presence_source(
        PresenceScanKind::Single,
        expr,
        Some(target_text),
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_binding(expr, target, result, nodes, elapsed_nanos);
    record_presence_reuse(
        PresenceScanKind::Single,
        expr,
        Some(target_text),
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_provenance(
        PresenceScanKind::Single,
        expr,
        Some(target),
        Some(target_text),
        result,
        nodes,
        elapsed_nanos,
    );
}

pub(in crate::runtime) fn record_profile_multi_presence_scan(
    start: Option<Instant>,
    expr: &Expr,
    result: bool,
    nodes: u64,
) {
    let elapsed_nanos = start.map(elapsed_nanos_since);
    record_counter(&PRESENCE_MULTI_SCAN_COUNT, "presence_multi_scans");
    record_profile_lexical_timing_elapsed(
        &PRESENCE_MULTI_SCAN_COUNT,
        &PRESENCE_MULTI_SCAN_TIME_COUNT,
        &PRESENCE_MULTI_SCAN_NANOS,
        "presence_multi_scan_time",
        elapsed_nanos,
    );
    record_add(
        &PRESENCE_MULTI_SCAN_NODES,
        "presence_multi_scan_nodes",
        nodes,
    );
    if result {
        record_counter(&PRESENCE_MULTI_SCAN_TRUE_COUNT, "presence_multi_scan_true");
    } else {
        record_counter(
            &PRESENCE_MULTI_SCAN_FALSE_COUNT,
            "presence_multi_scan_false",
        );
    }
    record_presence_shape(PresenceScanKind::Multi, expr, result, nodes, elapsed_nanos);
    record_presence_source(
        PresenceScanKind::Multi,
        expr,
        None,
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_reuse(
        PresenceScanKind::Multi,
        expr,
        None,
        result,
        nodes,
        elapsed_nanos,
    );
    record_presence_provenance(
        PresenceScanKind::Multi,
        expr,
        None,
        None,
        result,
        nodes,
        elapsed_nanos,
    );
}
