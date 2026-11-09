use super::super::*;
use super::source::CALL_STABILITY_SOURCE_HEAD_STATS;
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub(in crate::runtime) fn print_evaluation_cache_profile_snapshot(label: &str) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    if !evaluation_cache_profile_configured_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    ) {
        return;
    }

    eprintln!("rum-profile evaluation-cache snapshot label={label}");
    eprintln!(
        "rum-profile evaluation-cache summary epoch_bumps={} cache_clears={} symbol_invalidations={} call_generation_bumps={} dirty_bloom_saturations={} call_stability_size_cap_clears={}",
        load(&EPOCH_BUMP_COUNT),
        load(&CACHE_CLEAR_COUNT),
        load(&SYMBOL_INVALIDATION_COUNT),
        load(&CALL_GENERATION_BUMP_COUNT),
        load(&DIRTY_BLOOM_SATURATION_COUNT),
        load(&CALL_STABILITY_SIZE_CAP_CLEAR_COUNT),
    );
    eprintln!(
        "rum-profile evaluation-cache summary stability_checks={} call_stability_hits={} call_stability_misses={} miss_absent={} miss_generation={} miss_dirty={} miss_pointer={} insertions={} rewritten_dirty_skips={}",
        load(&STABILITY_CHECK_COUNT),
        load(&CALL_STABILITY_HIT_COUNT),
        load(&CALL_STABILITY_MISS_COUNT),
        load(&CALL_STABILITY_MISS_ABSENT_COUNT),
        load(&CALL_STABILITY_MISS_GENERATION_COUNT),
        load(&CALL_STABILITY_MISS_DIRTY_COUNT),
        load(&CALL_STABILITY_MISS_POINTER_COUNT),
        load(&CALL_STABILITY_INSERT_COUNT),
        load(&CALL_STABILITY_REWRITTEN_DIRTY_SKIP_COUNT),
    );
    eprintln!(
        "rum-profile evaluation-cache summary evaluate_entries={} down_value_scans={} down_value_applications={} sub_value_applications={} builtin_applications={} eval_module_entries={}",
        load(&EVALUATE_ENTRY_COUNT),
        load(&DOWN_VALUE_SCAN_COUNT),
        load(&DOWN_VALUE_APPLICATION_COUNT),
        load(&SUB_VALUE_APPLICATION_COUNT),
        load(&BUILTIN_APPLICATION_COUNT),
        load(&EVAL_MODULE_ENTRY_COUNT),
    );
    print_symbol_profile_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_symbol_invalidation_profile_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_invalidation_site_profile_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_invalidation_family_profile_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_invalidation_critical_profile_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_symbol_timing_profile_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_call_stability_source_head_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_call_stability_source_head_invalidation_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_call_stability_source_head_timing_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
    print_call_stability_source_head_invalidation_timing_summary(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    );
}

fn print_symbol_profile_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let counters = symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|((name, symbol), count)| (*name, *symbol, *count))
        .collect::<Vec<_>>();

    entries.sort_by(
        |(left_name, left_symbol, left_count), (right_name, right_symbol, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, count)) in entries.into_iter().take(32).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache symbol rank={} name={} symbol={} count={}",
            rank + 1,
            name,
            symbol,
            count,
        );
    }
}

fn print_symbol_invalidation_profile_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let counters = symbol_profile_counters()
        .lock()
        .expect("profile counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .filter(|((name, _), _)| {
            matches!(
                *name,
                "symbol_invalidations_by_source"
                    | "stability_dirty_writes_by_source"
                    | "stability_rewritten_dirty_writes_by_source"
                    | "dirty_bloom_saturations_by_source"
                    | "call_generation_bumps_by_source"
                    | "cache_clears_by_source"
            )
        })
        .map(|((name, symbol), count)| (*name, *symbol, *count))
        .collect::<Vec<_>>();

    entries.sort_by(
        |(left_name, left_symbol, left_count), (right_name, right_symbol, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, count)) in entries.into_iter().take(64).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-source rank={} name={} source={} count={}",
            rank + 1,
            name,
            symbol,
            count,
        );
    }
}

fn print_invalidation_site_profile_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let counters = invalidation_site_profile_counters()
        .lock()
        .expect("profile invalidation-site counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|(key, stats)| (*key, *stats))
        .collect::<Vec<_>>();

    entries.sort_by(|(left_key, left_stats), (right_key, right_stats)| {
        right_stats
            .count
            .cmp(&left_stats.count)
            .then_with(|| left_key.event.cmp(right_key.event))
            .then_with(|| left_key.file.cmp(right_key.file))
            .then_with(|| left_key.line.cmp(&right_key.line))
            .then_with(|| {
                optional_symbol_sort_name(left_key.source)
                    .cmp(&optional_symbol_sort_name(right_key.source))
            })
            .then_with(|| {
                optional_symbol_sort_name(left_key.target_text)
                    .cmp(&optional_symbol_sort_name(right_key.target_text))
            })
    });

    for (rank, (key, stats)) in entries.into_iter().take(80).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-site rank={} event={} file={} line={} source={} target={} count={} target_with_identity={} target_generated_dollar={}",
            rank + 1,
            key.event,
            key.file,
            key.line,
            optional_symbol_profile_name(key.source),
            optional_symbol_profile_name(key.target_text),
            stats.count,
            stats.target_with_identity,
            stats.target_generated_dollar,
        );
    }
}

fn print_invalidation_family_profile_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let counters = invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|(key, stats)| (key.clone(), *stats))
        .collect::<Vec<_>>();

    entries.sort_by(|(left_key, left_stats), (right_key, right_stats)| {
        right_stats
            .count
            .cmp(&left_stats.count)
            .then_with(|| left_key.event.cmp(right_key.event))
            .then_with(|| left_key.file.cmp(right_key.file))
            .then_with(|| left_key.line.cmp(&right_key.line))
            .then_with(|| {
                optional_symbol_sort_name(left_key.source)
                    .cmp(&optional_symbol_sort_name(right_key.source))
            })
            .then_with(|| {
                optional_string_sort_name(&left_key.target_family)
                    .cmp(&optional_string_sort_name(&right_key.target_family))
            })
    });

    for (rank, (key, stats)) in entries.into_iter().take(80).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-family rank={} event={} file={} line={} source={} target_family={} count={} target_with_identity={} target_generated_dollar={}",
            rank + 1,
            key.event,
            key.file,
            key.line,
            optional_symbol_profile_name(key.source),
            optional_string_profile_name(&key.target_family),
            stats.count,
            stats.target_with_identity,
            stats.target_generated_dollar,
        );
    }
}

fn print_invalidation_critical_profile_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let counters = invalidation_family_profile_counters()
        .lock()
        .expect("profile invalidation-family counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .filter(|(key, _)| matches!(key.event, "dirty_bloom_saturation" | "call_generation_bump"))
        .map(|(key, stats)| (key.clone(), *stats))
        .collect::<Vec<_>>();

    entries.sort_by(|(left_key, left_stats), (right_key, right_stats)| {
        right_stats
            .count
            .cmp(&left_stats.count)
            .then_with(|| left_key.event.cmp(right_key.event))
            .then_with(|| left_key.file.cmp(right_key.file))
            .then_with(|| left_key.line.cmp(&right_key.line))
            .then_with(|| {
                optional_symbol_sort_name(left_key.source)
                    .cmp(&optional_symbol_sort_name(right_key.source))
            })
            .then_with(|| {
                optional_string_sort_name(&left_key.target_family)
                    .cmp(&optional_string_sort_name(&right_key.target_family))
            })
    });

    for (rank, (key, stats)) in entries.into_iter().take(80).enumerate() {
        eprintln!(
            "rum-profile evaluation-cache invalidation-critical rank={} event={} file={} line={} source={} target_family={} count={} target_with_identity={} target_generated_dollar={}",
            rank + 1,
            key.event,
            key.file,
            key.line,
            optional_symbol_profile_name(key.source),
            optional_string_profile_name(&key.target_family),
            stats.count,
            stats.target_with_identity,
            stats.target_generated_dollar,
        );
    }
}

fn print_symbol_timing_profile_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let counters = symbol_timing_profile_counters()
        .lock()
        .expect("profile timing counter lock should not be poisoned");
    let mut entries = counters
        .iter()
        .map(|((name, symbol), stats)| (*name, *symbol, *stats))
        .collect::<Vec<_>>();

    entries.sort_by(
        |(left_name, left_symbol, left_stats), (right_name, right_symbol, right_stats)| {
            right_stats
                .nanos
                .cmp(&left_stats.nanos)
                .then_with(|| right_stats.count.cmp(&left_stats.count))
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, stats)) in entries.into_iter().take(32).enumerate() {
        let average = if stats.count == 0 {
            0
        } else {
            stats.nanos / stats.count
        };
        let self_average = if stats.count == 0 {
            0
        } else {
            stats.self_nanos / stats.count
        };
        eprintln!(
            "rum-profile evaluation-cache symbol-timing rank={} name={} symbol={} count={} total_ns={} avg_ns={} self_total_ns={} self_avg_ns={}",
            rank + 1,
            name,
            symbol,
            stats.count,
            stats.nanos,
            average,
            stats.self_nanos,
            self_average,
        );
    }

    let mut self_entries = counters
        .iter()
        .map(|((name, symbol), stats)| (*name, *symbol, *stats))
        .collect::<Vec<_>>();
    self_entries.sort_by(
        |(left_name, left_symbol, left_stats), (right_name, right_symbol, right_stats)| {
            right_stats
                .self_nanos
                .cmp(&left_stats.self_nanos)
                .then_with(|| right_stats.count.cmp(&left_stats.count))
                .then_with(|| left_name.cmp(right_name))
                .then_with(|| left_symbol.to_string().cmp(&right_symbol.to_string()))
        },
    );

    for (rank, (name, symbol, stats)) in self_entries.into_iter().take(32).enumerate() {
        let self_average = if stats.count == 0 {
            0
        } else {
            stats.self_nanos / stats.count
        };
        eprintln!(
            "rum-profile evaluation-cache symbol-self-timing rank={} name={} symbol={} count={} self_total_ns={} self_avg_ns={} total_ns={}",
            rank + 1,
            name,
            symbol,
            stats.count,
            stats.self_nanos,
            self_average,
            stats.nanos,
        );
    }
}

fn print_call_stability_source_head_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.checks
                .cmp(&lhs.checks)
                .then_with(|| rhs.misses.cmp(&lhs.misses))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile evaluation-cache call-stability-source-head rank={} source={} head={} checks={} hits={} misses={} miss_absent={} miss_generation={} miss_dirty={} miss_pointer={} insertions={} rewritten_dirty_skips={}",
                rank + 1,
                key.source,
                call_stability_head_shape_name(key.head),
                stats.checks,
                stats.hits,
                stats.misses,
                stats.miss_absent,
                stats.miss_generation,
                stats.miss_dirty,
                stats.miss_pointer,
                stats.insertions,
                stats.rewritten_dirty_skips,
            );
        }
    });
}

fn print_call_stability_source_head_invalidation_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.miss_dirty
                .saturating_add(rhs.miss_generation)
                .cmp(&lhs.miss_dirty.saturating_add(lhs.miss_generation))
                .then_with(|| rhs.misses.cmp(&lhs.misses))
                .then_with(|| rhs.checks.cmp(&lhs.checks))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            let invalidation_misses = stats.miss_dirty.saturating_add(stats.miss_generation);
            if invalidation_misses == 0 {
                continue;
            }
            eprintln!(
                "rum-profile evaluation-cache call-stability-source-head-invalidated rank={} source={} head={} invalidation_misses={} miss_generation={} miss_dirty={} misses={} checks={} hits={} insertions={} rewritten_dirty_skips={}",
                rank + 1,
                key.source,
                call_stability_head_shape_name(key.head),
                invalidation_misses,
                stats.miss_generation,
                stats.miss_dirty,
                stats.misses,
                stats.checks,
                stats.hits,
                stats.insertions,
                stats.rewritten_dirty_skips,
            );
        }
    });
}

fn print_call_stability_source_head_timing_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .filter(|(_, stats)| stats.total_nanos > 0)
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.self_nanos
                .cmp(&lhs.self_nanos)
                .then_with(|| rhs.total_nanos.cmp(&lhs.total_nanos))
                .then_with(|| rhs.checks.cmp(&lhs.checks))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            eprintln!(
                "rum-profile evaluation-cache call-stability-source-head-time rank={} source={} head={} checks={} hits={} misses={} total_ns={} avg_ns={} self_total_ns={} self_avg_ns={} hit_ns={} miss_absent_ns={} miss_generation_ns={} miss_dirty_ns={} miss_pointer_ns={}",
                rank + 1,
                key.source,
                call_stability_head_shape_name(key.head),
                stats.checks,
                stats.hits,
                stats.misses,
                stats.total_nanos,
                profile_average(stats.total_nanos, stats.checks),
                stats.self_nanos,
                profile_average(stats.self_nanos, stats.checks),
                stats.hit_nanos,
                stats.miss_absent_nanos,
                stats.miss_generation_nanos,
                stats.miss_dirty_nanos,
                stats.miss_pointer_nanos,
            );
        }
    });
}

fn print_call_stability_source_head_invalidation_timing_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    CALL_STABILITY_SOURCE_HEAD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .filter(|(_, stats)| stats.miss_generation_nanos.saturating_add(stats.miss_dirty_nanos) > 0)
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.miss_generation_nanos
                .saturating_add(rhs.miss_dirty_nanos)
                .cmp(&lhs.miss_generation_nanos.saturating_add(lhs.miss_dirty_nanos))
                .then_with(|| rhs.self_nanos.cmp(&lhs.self_nanos))
                .then_with(|| rhs.checks.cmp(&lhs.checks))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            let invalidation_misses = stats.miss_dirty.saturating_add(stats.miss_generation);
            let invalidation_nanos = stats
                .miss_dirty_nanos
                .saturating_add(stats.miss_generation_nanos);
            eprintln!(
                "rum-profile evaluation-cache call-stability-source-head-invalidation-time rank={} source={} head={} invalidation_misses={} invalidation_total_ns={} invalidation_avg_ns={} miss_generation={} miss_generation_ns={} miss_dirty={} miss_dirty_ns={} misses={} checks={} hits={} self_total_ns={}",
                rank + 1,
                key.source,
                call_stability_head_shape_name(key.head),
                invalidation_misses,
                invalidation_nanos,
                profile_average(invalidation_nanos, invalidation_misses),
                stats.miss_generation,
                stats.miss_generation_nanos,
                stats.miss_dirty,
                stats.miss_dirty_nanos,
                stats.misses,
                stats.checks,
                stats.hits,
                stats.self_nanos,
            );
        }
    });
}

fn call_stability_head_shape_name(shape: CallStabilityHeadShape) -> String {
    match shape {
        CallStabilityHeadShape::AtomSymbol(symbol) => symbol.to_string(),
        CallStabilityHeadShape::AtomOther => "atom-other".to_string(),
        CallStabilityHeadShape::CallHeadSymbol(symbol) => {
            format!("call-head:{symbol}")
        }
        CallStabilityHeadShape::CallOther => "call-head-other".to_string(),
    }
}

fn optional_symbol_profile_name(symbol: Option<Symbol>) -> String {
    symbol.map_or_else(|| "-".to_string(), |symbol| symbol.to_string())
}

fn optional_symbol_sort_name(symbol: Option<Symbol>) -> String {
    optional_symbol_profile_name(symbol)
}

fn optional_string_profile_name(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("-")
}

fn optional_string_sort_name(value: &Option<String>) -> String {
    optional_string_profile_name(value).to_string()
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

fn profile_average(total: u64, count: u64) -> u64 {
    if count == 0 { 0 } else { total / count }
}
