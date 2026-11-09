use super::*;

pub(super) fn print_expression_rebuild_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_shapes_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    EXPRESSION_REBUILD_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild rank={} source={} shape={} fingerprint={:#x} rebuilds={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                key.fingerprint,
                stats.rebuilds,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

pub(super) fn print_expression_rebuild_group_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_shapes_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    EXPRESSION_REBUILD_STATS.with(|stats| {
        let mut groups = HashMap::<ExpressionRebuildGroupKey, ExpressionRebuildGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(ExpressionRebuildGroupKey {
                    source: key.source,
                    source_symbol: key.source_symbol,
                    shape: key.shape,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.rebuilds = group.rebuilds.saturating_add(stats.rebuilds);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.max_rebuilds_per_fingerprint =
                group.max_rebuilds_per_fingerprint.max(stats.rebuilds);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild-group rank={} source={} shape={} unique_fingerprints={} rebuilds={} max_rebuilds_per_fingerprint={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                stats.unique_fingerprints,
                stats.rebuilds,
                stats.max_rebuilds_per_fingerprint,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

pub(super) fn print_expression_rebuild_exact_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_shapes_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild-exact rank={} source={} shape={} fingerprint={:#x} rebuilds={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                key.fingerprint,
                stats.rebuilds,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

pub(super) fn print_expression_rebuild_exact_group_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_shapes_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    EXPRESSION_REBUILD_EXACT_STATS.with(|stats| {
        let mut groups = HashMap::<ExpressionRebuildGroupKey, ExpressionRebuildGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(ExpressionRebuildGroupKey {
                    source: key.source,
                    source_symbol: key.source_symbol,
                    shape: key.shape,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.rebuilds = group.rebuilds.saturating_add(stats.rebuilds);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.max_rebuilds_per_fingerprint =
                group.max_rebuilds_per_fingerprint.max(stats.rebuilds);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.rebuilds.cmp(&lhs.rebuilds))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile expression-rebuild-exact-group rank={} source={} shape={} unique_fingerprints={} rebuilds={} max_rebuilds_per_fingerprint={} nodes={} avg_nodes={}",
                rank + 1,
                expression_rebuild_source_name(key.source, key.source_symbol),
                presence_root_shape_name(key.shape),
                stats.unique_fingerprints,
                stats.rebuilds,
                stats.max_rebuilds_per_fingerprint,
                stats.nodes,
                profile_stats_average(stats.nodes, stats.rebuilds),
            );
        }
    });
}

pub(super) fn print_rule_rhs_alpha_rename_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_shapes_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    RULE_RHS_ALPHA_RENAME_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.renames
                .cmp(&lhs.renames)
                .then_with(|| rhs.fresh_with_identity.cmp(&lhs.fresh_with_identity))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile rule-rhs-alpha-rename rank={} source={} local={} renames={} fresh_with_identity={}",
                rank + 1,
                profile_optional_symbol_name(key.source_symbol),
                key.local_text,
                stats.renames,
                stats.fresh_with_identity,
            );
        }
    });
}

pub(super) fn print_rule_rhs_timing_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::rule_rhs_timing_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    RULE_RHS_TIMING_STATS.with(|stats| {
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
                "rum-profile rule-rhs-timing rank={} phase={} source={} calls={} total_ns={} avg_ns={} changed={} unchanged={}",
                rank + 1,
                key.phase,
                profile_optional_symbol_name(key.source_symbol),
                stats.calls,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.calls),
                stats.changed,
                stats.unchanged,
            );
        }
    });
}
