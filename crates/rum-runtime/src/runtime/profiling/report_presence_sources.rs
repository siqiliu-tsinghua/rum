use super::*;

pub(super) fn print_presence_source_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_sources_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_SOURCE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile lexical source rank={} kind={} source={} context={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

pub(super) fn print_presence_source_shape_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_sources_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_SOURCE_SHAPE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(32).enumerate() {
            eprintln!(
                "rum-profile lexical source-shape rank={} kind={} source={} context={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

pub(super) fn print_presence_source_target_shape_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_sources_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_SOURCE_TARGET_SHAPE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            eprintln!(
                "rum-profile lexical source-target-shape rank={} kind={} source={} context={} target={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

pub(super) fn print_presence_source_fingerprint_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_source_fingerprints_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            eprintln!(
                "rum-profile lexical source-fingerprint rank={} fingerprint_kind={} kind={} source={} context={} target={} shape={} fingerprint={:#x} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_source_fingerprint_kind_name(key.fingerprint_kind),
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_root_shape_name(key.shape),
                key.fingerprint,
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

pub(super) fn print_presence_source_fingerprint_group_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_source_fingerprints_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_SOURCE_FINGERPRINT_STATS.with(|stats| {
        let mut groups =
            HashMap::<PresenceSourceFingerprintGroupKey, PresenceSourceFingerprintGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceSourceFingerprintGroupKey {
                    fingerprint_kind: key.fingerprint_kind,
                    kind: key.kind,
                    source: key.source,
                    context: key.context,
                    shape: key.shape,
                    target_text: key.target_text,
                })
                .or_default();
            group.unique_fingerprints = group.unique_fingerprints.saturating_add(1);
            group.scans = group.scans.saturating_add(stats.scans);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.timed = group.timed.saturating_add(stats.timed);
            group.nanos = group.nanos.saturating_add(stats.nanos);
            group.max_scans_per_fingerprint = group.max_scans_per_fingerprint.max(stats.scans);
            group.true_count = group.true_count.saturating_add(stats.true_count);
            group.false_count = group.false_count.saturating_add(stats.false_count);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(48).enumerate() {
            eprintln!(
                "rum-profile lexical source-fingerprint-group rank={} fingerprint_kind={} kind={} source={} context={} target={} shape={} unique_fingerprints={} scans={} max_scans_per_fingerprint={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_source_fingerprint_kind_name(key.fingerprint_kind),
                presence_scan_kind_name(key.kind),
                key.source,
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_root_shape_name(key.shape),
                stats.unique_fingerprints,
                stats.scans,
                stats.max_scans_per_fingerprint,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}

pub(super) fn print_presence_provenance_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_provenance_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_PROVENANCE_STATS.with(|stats| {
        let mut entries = stats
            .borrow()
            .iter()
            .map(|(key, stats)| (*key, *stats))
            .collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(64).enumerate() {
            eprintln!(
                "rum-profile lexical provenance rank={} kind={} source={} context={} target={} target_origin={} expr_origin={} expr_source={} alpha_relation={} shape={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                profile_optional_symbol_name(key.source),
                key.context,
                profile_optional_symbol_name(key.target_text),
                presence_target_origin_name(key.target_origin),
                expression_provenance_kind_name(key.expr_origin),
                expression_rebuild_source_name(key.expr_origin_source, key.expr_origin_symbol),
                presence_alpha_relation_name(key.alpha_relation),
                presence_root_shape_name(key.shape),
                stats.scans,
                stats.nodes,
                stats.timed,
                stats.nanos,
                profile_stats_average(stats.nanos, stats.timed),
                stats.true_count,
                stats.false_count,
            );
        }
    });
}
