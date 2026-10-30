use super::*;

/// Minted only in this module. Tests cannot construct it.
pub(in crate::runtime::profiling) struct ProductionReset {
    _private: (),
}

pub(in crate::runtime) fn reset_lexical_profile_counters() {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    if crate::runtime::profiling::lease::blocks_unregistered_thread() {
        return;
    }
    if !lexical_profile_configured() {
        return;
    }

    logical::epoch::reset_production(ProductionReset { _private: () });

    macro_rules! reset {
        ($($counter:ident),+ $(,)?) => {
            $( $counter.store(0, Ordering::Relaxed); )+
        };
    }

    reset!(
        DEFINITION_RHS_EVAL_COUNT,
        DEFINITION_RHS_EVAL_NANOS,
        EVAL_MODULE_COUNT,
        EVAL_MODULE_NANOS,
        MODULE_RESUME_ENTRY_COUNT,
        MODULE_RESUME_MAX_INDEX,
        MODULE_INIT_SUBSTITUTION_COUNT,
        MODULE_INIT_SUBSTITUTION_NANOS,
        MODULE_BODY_SUBSTITUTION_COUNT,
        MODULE_BODY_SUBSTITUTION_NANOS,
        LEXICAL_SUBSTITUTION_COUNT,
        LEXICAL_SUBSTITUTION_TIME_COUNT,
        LEXICAL_SUBSTITUTION_NANOS,
        LEXICAL_SUBSTITUTION_CHANGED_COUNT,
        LEXICAL_SUBSTITUTION_UNCHANGED_COUNT,
        LEXICAL_BINDING_EMPTY_COUNT,
        LEXICAL_BINDING_SINGLE_COUNT,
        LEXICAL_BINDING_MULTI_COUNT,
        LEXICAL_BINDING_MAX_COUNT,
        LEXICAL_SHADOWED_NONEMPTY_COUNT,
        ARG_LIST_SUBSTITUTION_COUNT,
        ARG_LIST_TOTAL_ARGS,
        ARG_LIST_MAX_LEN,
        ARG_PREFILTER_SKIP_COUNT,
        ARG_PREFILTER_MAYBE_COUNT,
        ARG_SUBSTITUTION_ATTEMPT_COUNT,
        ARG_SUBSTITUTION_ATTEMPT_NANOS,
        ARG_SUBSTITUTION_CHANGED_COUNT,
        ARG_SUBSTITUTION_UNCHANGED_COUNT,
        PRESENCE_SINGLE_SCAN_COUNT,
        PRESENCE_SINGLE_SCAN_NODES,
        PRESENCE_SINGLE_SCAN_NANOS,
        PRESENCE_SINGLE_SCAN_TIME_COUNT,
        PRESENCE_SINGLE_SCAN_TRUE_COUNT,
        PRESENCE_SINGLE_SCAN_FALSE_COUNT,
        PRESENCE_MULTI_SCAN_COUNT,
        PRESENCE_MULTI_SCAN_NODES,
        PRESENCE_MULTI_SCAN_NANOS,
        PRESENCE_MULTI_SCAN_TIME_COUNT,
        PRESENCE_MULTI_SCAN_TRUE_COUNT,
        PRESENCE_MULTI_SCAN_FALSE_COUNT,
        REWRITE_RULE_CANDIDATE_COUNT,
        REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT,
        REWRITE_RULE_WOULD_MATCHER_COUNT,
        REWRITE_RULE_MATCHER_MISS_COUNT,
        REWRITE_RULE_MATCH_COUNT,
        REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT,
    );

    PRESENCE_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_CONTEXT_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_CONTEXT_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_ARG_SITE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_BINDING_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_BINDING_NAME_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_TARGET_SHAPE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_REUSE_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_FINGERPRINT_STATS.with(|stats| stats.borrow_mut().clear());
    PRESENCE_EXACT_FINGERPRINT_STATS.with(|stats| stats.borrow_mut().clear());
    EXPRESSION_REBUILD_STATS.with(|stats| stats.borrow_mut().clear());
    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| stats.borrow_mut().clear());
    RULE_RHS_ALPHA_RENAME_STATS.with(|stats| stats.borrow_mut().clear());
    RULE_RHS_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    DEFINITION_RULE_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    DEFINITION_DISPATCH_STATS.with(|stats| stats.borrow_mut().clear());
    BUILTIN_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    BUILTIN_TIMING_STACK.with(|stack| stack.borrow_mut().clear());
    RUNTIME_PATH_TIMING_STATS.with(|stats| stats.borrow_mut().clear());
    RUNTIME_PATH_TIMING_STACK.with(|stack| stack.borrow_mut().clear());
    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| stats.borrow_mut().clear());
    EXPRESSION_PROVENANCE_BY_EXACT.with(|stats| stats.borrow_mut().clear());
    PRESENCE_PROVENANCE_STATS.with(|stats| stats.borrow_mut().clear());
    LEXICAL_CONTEXT_STACK.with(|stack| stack.borrow_mut().clear());
    LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(0));
    ARG_PREFILTER_SITE_STACK.with(|stack| stack.borrow_mut().clear());
    ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(0));
    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| stack.borrow_mut().clear());
    EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(0));
    LEXICAL_PROFILE_SUMMARY_PRINTED.store(0, Ordering::Relaxed);
    LEXICAL_PROFILE_ACTIVE.store(1, Ordering::Relaxed);
}
