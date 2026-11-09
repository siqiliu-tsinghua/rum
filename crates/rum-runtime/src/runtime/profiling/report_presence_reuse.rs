use super::*;

pub(super) fn print_presence_reuse_summary(
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

    PRESENCE_REUSE_STATS.with(|stats| {
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

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical reuse rank={} kind={} shape={} target={} head_ptr={:#x} args_ptr={:#x} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
                key.head_ptr,
                key.args_ptr,
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

pub(super) fn print_presence_reuse_group_summary(
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

    PRESENCE_REUSE_STATS.with(|stats| {
        let mut groups = HashMap::<PresenceReuseGroupKey, PresenceReuseGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceReuseGroupKey {
                    kind: key.kind,
                    shape: key.shape,
                    target_text: key.target_text,
                })
                .or_default();
            group.unique_ptrs = group.unique_ptrs.saturating_add(1);
            group.scans = group.scans.saturating_add(stats.scans);
            group.nodes = group.nodes.saturating_add(stats.nodes);
            group.timed = group.timed.saturating_add(stats.timed);
            group.nanos = group.nanos.saturating_add(stats.nanos);
            group.max_scans_per_ptr = group.max_scans_per_ptr.max(stats.scans);
            group.true_count = group.true_count.saturating_add(stats.true_count);
            group.false_count = group.false_count.saturating_add(stats.false_count);
        }

        let mut entries = groups.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(_, lhs), (_, rhs)| {
            rhs.nodes
                .cmp(&lhs.nodes)
                .then_with(|| rhs.scans.cmp(&lhs.scans))
        });

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical reuse-group rank={} kind={} shape={} target={} unique_ptrs={} scans={} max_scans_per_ptr={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
                stats.unique_ptrs,
                stats.scans,
                stats.max_scans_per_ptr,
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

pub(super) fn print_presence_fingerprint_summary(
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

    PRESENCE_FINGERPRINT_STATS.with(|stats| {
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

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical fingerprint rank={} kind={} shape={} target={} fingerprint={:#x} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
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

pub(super) fn print_presence_fingerprint_group_summary(
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

    PRESENCE_FINGERPRINT_STATS.with(|stats| {
        let mut groups =
            HashMap::<PresenceFingerprintGroupKey, PresenceFingerprintGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceFingerprintGroupKey {
                    kind: key.kind,
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

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical fingerprint-group rank={} kind={} shape={} target={} unique_fingerprints={} scans={} max_scans_per_fingerprint={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
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

pub(super) fn print_presence_exact_fingerprint_group_summary(
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

    PRESENCE_EXACT_FINGERPRINT_STATS.with(|stats| {
        let mut groups =
            HashMap::<PresenceFingerprintGroupKey, PresenceFingerprintGroupStats>::new();
        for (key, stats) in stats.borrow().iter() {
            let group = groups
                .entry(PresenceFingerprintGroupKey {
                    kind: key.kind,
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

        for (rank, (key, stats)) in entries.into_iter().take(24).enumerate() {
            eprintln!(
                "rum-profile lexical exact-fingerprint-group rank={} kind={} shape={} target={} unique_fingerprints={} scans={} max_scans_per_fingerprint={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_scan_kind_name(key.kind),
                presence_root_shape_name(key.shape),
                profile_optional_symbol_name(key.target_text),
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
