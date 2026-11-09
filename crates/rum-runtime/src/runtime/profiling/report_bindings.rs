use super::*;

pub(super) fn print_presence_binding_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_bindings_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_BINDING_STATS.with(|stats| {
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
                "rum-profile lexical binding rank={} shape={} target={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_root_shape_name(key.shape),
                profile_symbol_name(key.target),
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

pub(super) fn print_presence_binding_name_summary(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    if !config::lexical_profile_bindings_enabled_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
    ) {
        return;
    }

    PRESENCE_BINDING_NAME_STATS.with(|stats| {
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
                "rum-profile lexical binding-name rank={} shape={} target={} scans={} nodes={} timed={} total_ns={} avg_ns={} true={} false={}",
                rank + 1,
                presence_root_shape_name(key.shape),
                key.target_text,
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
