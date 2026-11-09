use super::*;

pub(super) fn print_definition_rule_timing_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::definition_rule_timing_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    DEFINITION_RULE_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nanos
                .cmp(&lhs.nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile definition-rule-timing rank={} kind={} symbol={} index={} calls={} total_ns={} avg_ns={} ok={} err={}",
                rank + 1,
                key.kind,
                key.symbol,
                key.index,
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.ok,
                stats.err,
            );
        }
    });
}

pub(super) fn print_definition_dispatch_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    DEFINITION_DISPATCH_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return;
        }

        let totals =
            entries
                .iter()
                .fold(DefinitionDispatchStats::default(), |mut acc, (_, stats)| {
                    acc.candidates = acc.candidates.saturating_add(stats.candidates);
                    acc.prefilter_skips = acc.prefilter_skips.saturating_add(stats.prefilter_skips);
                    acc.fast_matches = acc.fast_matches.saturating_add(stats.fast_matches);
                    acc.fast_misses = acc.fast_misses.saturating_add(stats.fast_misses);
                    acc.matcher_matches = acc.matcher_matches.saturating_add(stats.matcher_matches);
                    acc.matcher_misses = acc.matcher_misses.saturating_add(stats.matcher_misses);
                    acc.prefilter_nanos = acc.prefilter_nanos.saturating_add(stats.prefilter_nanos);
                    acc.fast_match_nanos =
                        acc.fast_match_nanos.saturating_add(stats.fast_match_nanos);
                    acc.compiled_matcher_nanos = acc
                        .compiled_matcher_nanos
                        .saturating_add(stats.compiled_matcher_nanos);
                    acc.rhs_substitution_nanos = acc
                        .rhs_substitution_nanos
                        .saturating_add(stats.rhs_substitution_nanos);
                    acc
                });
        eprintln!(
            "rum-profile definition-dispatch summary candidates={} prefilter_skips={} fast_matches={} fast_misses={} matcher_matches={} matcher_misses={} prefilter_skip_rate_per_mille={} matcher_miss_rate_per_mille={}",
            totals.candidates,
            totals.prefilter_skips,
            totals.fast_matches,
            totals.fast_misses,
            totals.matcher_matches,
            totals.matcher_misses,
            profile_stats_per_mille(totals.prefilter_skips, totals.candidates),
            profile_stats_per_mille(totals.matcher_misses, totals.candidates),
        );

        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.matcher_misses
                .cmp(&lhs.matcher_misses)
                .then_with(|| rhs.prefilter_skips.cmp(&lhs.prefilter_skips))
                .then_with(|| rhs.candidates.cmp(&lhs.candidates))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile definition-dispatch rank={} kind={} symbol={} source={} candidates={} prefilter_skips={} fast_matches={} fast_misses={} matcher_matches={} matcher_misses={} prefilter_skip_rate_per_mille={} matcher_miss_rate_per_mille={}",
                rank + 1,
                key.kind,
                key.symbol,
                profile_optional_symbol_name(key.source_symbol),
                stats.candidates,
                stats.prefilter_skips,
                stats.fast_matches,
                stats.fast_misses,
                stats.matcher_matches,
                stats.matcher_misses,
                profile_stats_per_mille(stats.prefilter_skips, stats.candidates),
                profile_stats_per_mille(stats.matcher_misses, stats.candidates),
            );
        }

        if config::definition_rule_timing_enabled_admitted(
#[cfg(any(test, feature = "test-builtin-memory-growth", feature = "test-continuation-slots"))]
_access,
) {
            eprintln!(
                "rum-profile definition-dispatch-time summary prefilter_ns={} fast_match_ns={} compiled_matcher_ns={} rhs_substitution_ns={}",
                totals.prefilter_nanos,
                totals.fast_match_nanos,
                totals.compiled_matcher_nanos,
                totals.rhs_substitution_nanos,
            );

            let mut entries = DEFINITION_DISPATCH_STATS.with(|stats| {
                stats
                    .borrow()
                    .iter()
                    .map(|(key, stats)| (*key, *stats))
                    .collect::<Vec<_>>()
            });
            entries.sort_by(|(_, lhs), (_, rhs)| {
                definition_dispatch_timing_total(rhs)
                    .cmp(&definition_dispatch_timing_total(lhs))
                    .then_with(|| rhs.candidates.cmp(&lhs.candidates))
            });

            for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
                let fast_attempts = stats.fast_matches.saturating_add(stats.fast_misses);
                let matcher_attempts = stats
                    .matcher_matches
                    .saturating_add(stats.matcher_misses);
                let rhs_substitutions = stats.fast_matches.saturating_add(stats.matcher_matches);
                eprintln!(
                    "rum-profile definition-dispatch-time rank={} kind={} symbol={} source={} total_ns={} candidates={} prefilter_ns={} prefilter_avg_ns={} fast_attempts={} fast_match_ns={} fast_match_avg_ns={} matcher_attempts={} compiled_matcher_ns={} compiled_matcher_avg_ns={} rhs_substitutions={} rhs_substitution_ns={} rhs_substitution_avg_ns={}",
                    rank + 1,
                    key.kind,
                    key.symbol,
                    profile_optional_symbol_name(key.source_symbol),
                    definition_dispatch_timing_total(&stats),
                    stats.candidates,
                    stats.prefilter_nanos,
                    profile_stats_average(stats.prefilter_nanos, stats.candidates),
                    fast_attempts,
                    stats.fast_match_nanos,
                    profile_stats_average(stats.fast_match_nanos, fast_attempts),
                    matcher_attempts,
                    stats.compiled_matcher_nanos,
                    profile_stats_average(stats.compiled_matcher_nanos, matcher_attempts),
                    rhs_substitutions,
                    stats.rhs_substitution_nanos,
                    profile_stats_average(stats.rhs_substitution_nanos, rhs_substitutions),
                );
            }
        }
    });
}

fn definition_dispatch_timing_total(stats: &DefinitionDispatchStats) -> u64 {
    stats
        .prefilter_nanos
        .saturating_add(stats.fast_match_nanos)
        .saturating_add(stats.compiled_matcher_nanos)
        .saturating_add(stats.rhs_substitution_nanos)
}

pub(super) fn print_runtime_path_timing_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    RUNTIME_PATH_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nanos
                .cmp(&lhs.nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile runtime-path-timing rank={} phase={} source={} calls={} total_ns={} avg_ns={} self_total_ns={} self_avg_ns={} ok={} err={} some={} none={} return_err={} throw_err={} input_required={}",
                rank + 1,
                key.phase,
                profile_optional_symbol_name(key.source_symbol),
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.ok,
                stats.err,
                stats.some,
                stats.none,
                stats.return_err,
                stats.throw_err,
                stats.input_required,
            );
        }
    });

    RUNTIME_PATH_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.self_nanos
                .cmp(&lhs.self_nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile runtime-path-self-timing rank={} phase={} source={} calls={} self_total_ns={} self_avg_ns={} total_ns={} avg_ns={} ok={} err={} some={} none={} return_err={} throw_err={} input_required={}",
                rank + 1,
                key.phase,
                profile_optional_symbol_name(key.source_symbol),
                stats.calls,
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.ok,
                stats.err,
                stats.some,
                stats.none,
                stats.return_err,
                stats.throw_err,
                stats.input_required,
            );
        }
    });
}

pub(super) fn print_builtin_timing_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::builtin_timing_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    BUILTIN_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nanos
                .cmp(&lhs.nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile builtin-timing rank={} symbol={} source={} arg_count={} calls={} total_ns={} avg_ns={} self_total_ns={} self_avg_ns={} ok={} err={}",
                rank + 1,
                key.symbol,
                profile_optional_symbol_name(key.source_symbol),
                key.arg_count,
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.ok,
                stats.err,
            );
        }
    });

    BUILTIN_TIMING_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.self_nanos
                .cmp(&lhs.self_nanos)
                .then_with(|| rhs.calls.cmp(&lhs.calls))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile builtin-self-timing rank={} symbol={} source={} arg_count={} calls={} self_total_ns={} self_avg_ns={} total_ns={} avg_ns={} ok={} err={}",
                rank + 1,
                key.symbol,
                profile_optional_symbol_name(key.source_symbol),
                key.arg_count,
                stats.calls,
                stats.self_nanos,
                profile_stats_average(stats.self_nanos, stats.calls),
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.ok,
                stats.err,
            );
        }
    });
}

pub(super) fn print_rewrite_rule_prefilter_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let candidates = load(&REWRITE_RULE_CANDIDATE_COUNT);
    if candidates == 0 {
        return;
    }

    let would_reject = load(&REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT);
    let would_matcher = load(&REWRITE_RULE_WOULD_MATCHER_COUNT);
    let matcher_misses = load(&REWRITE_RULE_MATCHER_MISS_COUNT);
    let matches = load(&REWRITE_RULE_MATCH_COUNT);
    let false_rejects = load(&REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT);
    eprintln!(
        "rum-profile rewrite-rule prefilter summary candidates={} would_reject={} would_matcher={} matcher_misses={} matches={} false_rejects={} reject_rate_per_mille={} false_reject_rate_per_mille={}",
        candidates,
        would_reject,
        would_matcher,
        matcher_misses,
        matches,
        false_rejects,
        profile_stats_per_mille(would_reject, candidates),
        profile_stats_per_mille(false_rejects, candidates),
    );

    REWRITE_RULE_PREFILTER_SOURCE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(source, stats)| (*source, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.would_reject
                .cmp(&lhs.would_reject)
                .then_with(|| rhs.candidates.cmp(&lhs.candidates))
                .then_with(|| rhs.matcher_misses.cmp(&lhs.matcher_misses))
        });

        for (rank, (source, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile rewrite-rule prefilter-source rank={} source={} candidates={} would_reject={} would_matcher={} matcher_misses={} matches={} false_rejects={} reject_rate_per_mille={} false_reject_rate_per_mille={}",
                rank + 1,
                source,
                stats.candidates,
                stats.would_reject,
                stats.would_matcher,
                stats.matcher_misses,
                stats.matches,
                stats.false_rejects,
                profile_stats_per_mille(stats.would_reject, stats.candidates),
                profile_stats_per_mille(stats.false_rejects, stats.candidates),
            );
        }
    });
}
