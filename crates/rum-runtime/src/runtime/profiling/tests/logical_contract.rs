//! Native LP01 observer. No guest evaluation, ranking, or completion repair.
//! The logical table is comparable; raw maps, durations and flags are evidence
//! only. LP02-LP08 implement the target contract; these are today's counters.
use super::*;
use std::collections::BTreeMap;
use std::fmt::Debug;

#[doc(hidden)]
#[derive(Debug, Default, Clone)]
pub struct LogicalProfileSnapshot {
    pub logical: BTreeMap<String, Vec<u64>>,
    pub durations: BTreeMap<String, Vec<u64>>,
    pub telemetry: BTreeMap<String, String>,
    pub flags: BTreeMap<String, u8>,
    /// One entry per storage map, including empty maps; no top-N truncation.
    pub maps: BTreeMap<String, usize>,
}
impl LogicalProfileSnapshot {
    pub(crate) fn map<K: Debug, V: Debug>(&mut self, name: &str, map: &HashMap<K, V>) {
        assert!(self.maps.insert(name.into(), map.len()).is_none());
        let mut rows = map
            .iter()
            .map(|(k, v)| format!("{k:?} => {v:?}"))
            .collect::<Vec<_>>();
        rows.sort();
        self.telemetry.insert(name.into(), format!("{rows:?}"));
    }
    pub(crate) fn row(&mut self, key: String, logical: Vec<u64>, durations: Vec<u64>) {
        assert!(
            self.logical.insert(key.clone(), logical).is_none(),
            "duplicate profile key {key}"
        );
        self.durations.insert(key, durations);
    }
}

pub(crate) fn snapshot() -> LogicalProfileSnapshot {
    let mut out = LogicalProfileSnapshot::default();
    out.flags.insert(
        "LEXICAL_PROFILE_ENABLED".into(),
        LEXICAL_PROFILE_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_ACTIVE".into(),
        LEXICAL_PROFILE_ACTIVE.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_DETAIL_ENABLED".into(),
        LEXICAL_PROFILE_DETAIL_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_INCREMENTAL_ENABLED".into(),
        LEXICAL_PROFILE_INCREMENTAL_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_SHAPES_ENABLED".into(),
        LEXICAL_PROFILE_SHAPES_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_BINDINGS_ENABLED".into(),
        LEXICAL_PROFILE_BINDINGS_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_SOURCES_ENABLED".into(),
        LEXICAL_PROFILE_SOURCES_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED".into(),
        LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_PROVENANCE_ENABLED".into(),
        LEXICAL_PROFILE_PROVENANCE_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "RULE_RHS_TIMING_ENABLED".into(),
        RULE_RHS_TIMING_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "DEFINITION_RULE_TIMING_ENABLED".into(),
        DEFINITION_RULE_TIMING_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "BUILTIN_TIMING_ENABLED".into(),
        BUILTIN_TIMING_ENABLED.load(Ordering::Relaxed),
    );
    out.flags.insert(
        "LEXICAL_PROFILE_SUMMARY_PRINTED".into(),
        LEXICAL_PROFILE_SUMMARY_PRINTED.load(Ordering::Relaxed),
    );
    out.row(
        "counter|definition_rhs_eval".into(),
        vec![load(&DEFINITION_RHS_EVAL_COUNT)],
        vec![],
    );
    out.telemetry.insert(
        "DEFINITION_RHS_EVAL_NANOS".into(),
        load(&DEFINITION_RHS_EVAL_NANOS).to_string(),
    );
    out.row(
        "counter|eval_module".into(),
        vec![load(&EVAL_MODULE_COUNT)],
        vec![],
    );
    out.telemetry.insert(
        "EVAL_MODULE_NANOS".into(),
        load(&EVAL_MODULE_NANOS).to_string(),
    );
    out.telemetry.insert(
        "MODULE_RESUME_ENTRY_COUNT".into(),
        load(&MODULE_RESUME_ENTRY_COUNT).to_string(),
    );
    out.telemetry.insert(
        "MODULE_RESUME_MAX_INDEX".into(),
        load(&MODULE_RESUME_MAX_INDEX).to_string(),
    );
    out.telemetry.insert(
        "MODULE_INIT_SUBSTITUTION_COUNT".into(),
        load(&MODULE_INIT_SUBSTITUTION_COUNT).to_string(),
    );
    out.telemetry.insert(
        "MODULE_INIT_SUBSTITUTION_NANOS".into(),
        load(&MODULE_INIT_SUBSTITUTION_NANOS).to_string(),
    );
    out.telemetry.insert(
        "MODULE_BODY_SUBSTITUTION_COUNT".into(),
        load(&MODULE_BODY_SUBSTITUTION_COUNT).to_string(),
    );
    out.telemetry.insert(
        "MODULE_BODY_SUBSTITUTION_NANOS".into(),
        load(&MODULE_BODY_SUBSTITUTION_NANOS).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_SUBSTITUTION_COUNT".into(),
        load(&LEXICAL_SUBSTITUTION_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_SUBSTITUTION_TIME_COUNT".into(),
        load(&LEXICAL_SUBSTITUTION_TIME_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_SUBSTITUTION_NANOS".into(),
        load(&LEXICAL_SUBSTITUTION_NANOS).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_SUBSTITUTION_CHANGED_COUNT".into(),
        load(&LEXICAL_SUBSTITUTION_CHANGED_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_SUBSTITUTION_UNCHANGED_COUNT".into(),
        load(&LEXICAL_SUBSTITUTION_UNCHANGED_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_BINDING_EMPTY_COUNT".into(),
        load(&LEXICAL_BINDING_EMPTY_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_BINDING_SINGLE_COUNT".into(),
        load(&LEXICAL_BINDING_SINGLE_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_BINDING_MULTI_COUNT".into(),
        load(&LEXICAL_BINDING_MULTI_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_BINDING_MAX_COUNT".into(),
        load(&LEXICAL_BINDING_MAX_COUNT).to_string(),
    );
    out.telemetry.insert(
        "LEXICAL_SHADOWED_NONEMPTY_COUNT".into(),
        load(&LEXICAL_SHADOWED_NONEMPTY_COUNT).to_string(),
    );
    out.telemetry.insert(
        "ARG_LIST_SUBSTITUTION_COUNT".into(),
        load(&ARG_LIST_SUBSTITUTION_COUNT).to_string(),
    );
    out.telemetry.insert(
        "ARG_LIST_TOTAL_ARGS".into(),
        load(&ARG_LIST_TOTAL_ARGS).to_string(),
    );
    out.telemetry.insert(
        "ARG_LIST_MAX_LEN".into(),
        load(&ARG_LIST_MAX_LEN).to_string(),
    );
    out.telemetry.insert(
        "ARG_PREFILTER_SKIP_COUNT".into(),
        load(&ARG_PREFILTER_SKIP_COUNT).to_string(),
    );
    out.telemetry.insert(
        "ARG_PREFILTER_MAYBE_COUNT".into(),
        load(&ARG_PREFILTER_MAYBE_COUNT).to_string(),
    );
    out.telemetry.insert(
        "ARG_SUBSTITUTION_ATTEMPT_COUNT".into(),
        load(&ARG_SUBSTITUTION_ATTEMPT_COUNT).to_string(),
    );
    out.telemetry.insert(
        "ARG_SUBSTITUTION_ATTEMPT_NANOS".into(),
        load(&ARG_SUBSTITUTION_ATTEMPT_NANOS).to_string(),
    );
    out.telemetry.insert(
        "ARG_SUBSTITUTION_CHANGED_COUNT".into(),
        load(&ARG_SUBSTITUTION_CHANGED_COUNT).to_string(),
    );
    out.telemetry.insert(
        "ARG_SUBSTITUTION_UNCHANGED_COUNT".into(),
        load(&ARG_SUBSTITUTION_UNCHANGED_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_SINGLE_SCAN_COUNT".into(),
        load(&PRESENCE_SINGLE_SCAN_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_SINGLE_SCAN_NODES".into(),
        load(&PRESENCE_SINGLE_SCAN_NODES).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_SINGLE_SCAN_NANOS".into(),
        load(&PRESENCE_SINGLE_SCAN_NANOS).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_SINGLE_SCAN_TIME_COUNT".into(),
        load(&PRESENCE_SINGLE_SCAN_TIME_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_SINGLE_SCAN_TRUE_COUNT".into(),
        load(&PRESENCE_SINGLE_SCAN_TRUE_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_SINGLE_SCAN_FALSE_COUNT".into(),
        load(&PRESENCE_SINGLE_SCAN_FALSE_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_MULTI_SCAN_COUNT".into(),
        load(&PRESENCE_MULTI_SCAN_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_MULTI_SCAN_NODES".into(),
        load(&PRESENCE_MULTI_SCAN_NODES).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_MULTI_SCAN_NANOS".into(),
        load(&PRESENCE_MULTI_SCAN_NANOS).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_MULTI_SCAN_TIME_COUNT".into(),
        load(&PRESENCE_MULTI_SCAN_TIME_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_MULTI_SCAN_TRUE_COUNT".into(),
        load(&PRESENCE_MULTI_SCAN_TRUE_COUNT).to_string(),
    );
    out.telemetry.insert(
        "PRESENCE_MULTI_SCAN_FALSE_COUNT".into(),
        load(&PRESENCE_MULTI_SCAN_FALSE_COUNT).to_string(),
    );
    out.telemetry.insert(
        "REWRITE_RULE_CANDIDATE_COUNT".into(),
        load(&REWRITE_RULE_CANDIDATE_COUNT).to_string(),
    );
    out.telemetry.insert(
        "REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT".into(),
        load(&REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT).to_string(),
    );
    out.telemetry.insert(
        "REWRITE_RULE_WOULD_MATCHER_COUNT".into(),
        load(&REWRITE_RULE_WOULD_MATCHER_COUNT).to_string(),
    );
    out.telemetry.insert(
        "REWRITE_RULE_MATCHER_MISS_COUNT".into(),
        load(&REWRITE_RULE_MATCHER_MISS_COUNT).to_string(),
    );
    out.telemetry.insert(
        "REWRITE_RULE_MATCH_COUNT".into(),
        load(&REWRITE_RULE_MATCH_COUNT).to_string(),
    );
    out.telemetry.insert(
        "REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT".into(),
        load(&REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT).to_string(),
    );
    PRESENCE_SHAPE_STATS.with(|s| out.map("PRESENCE_SHAPE_STATS", &s.borrow()));
    PRESENCE_CONTEXT_STATS.with(|s| out.map("PRESENCE_CONTEXT_STATS", &s.borrow()));
    PRESENCE_CONTEXT_SHAPE_STATS.with(|s| out.map("PRESENCE_CONTEXT_SHAPE_STATS", &s.borrow()));
    PRESENCE_ARG_SITE_STATS.with(|s| out.map("PRESENCE_ARG_SITE_STATS", &s.borrow()));
    PRESENCE_BINDING_STATS.with(|s| out.map("PRESENCE_BINDING_STATS", &s.borrow()));
    PRESENCE_BINDING_NAME_STATS.with(|s| out.map("PRESENCE_BINDING_NAME_STATS", &s.borrow()));
    PRESENCE_SOURCE_STATS.with(|s| out.map("PRESENCE_SOURCE_STATS", &s.borrow()));
    PRESENCE_SOURCE_SHAPE_STATS.with(|s| out.map("PRESENCE_SOURCE_SHAPE_STATS", &s.borrow()));
    PRESENCE_SOURCE_TARGET_SHAPE_STATS
        .with(|s| out.map("PRESENCE_SOURCE_TARGET_SHAPE_STATS", &s.borrow()));
    PRESENCE_SOURCE_FINGERPRINT_STATS
        .with(|s| out.map("PRESENCE_SOURCE_FINGERPRINT_STATS", &s.borrow()));
    PRESENCE_REUSE_STATS.with(|s| out.map("PRESENCE_REUSE_STATS", &s.borrow()));
    PRESENCE_FINGERPRINT_STATS.with(|s| out.map("PRESENCE_FINGERPRINT_STATS", &s.borrow()));
    PRESENCE_EXACT_FINGERPRINT_STATS
        .with(|s| out.map("PRESENCE_EXACT_FINGERPRINT_STATS", &s.borrow()));
    EXPRESSION_REBUILD_STATS.with(|s| out.map("EXPRESSION_REBUILD_STATS", &s.borrow()));
    EXPRESSION_REBUILD_EXACT_STATS.with(|s| out.map("EXPRESSION_REBUILD_EXACT_STATS", &s.borrow()));
    RULE_RHS_ALPHA_RENAME_STATS.with(|s| out.map("RULE_RHS_ALPHA_RENAME_STATS", &s.borrow()));
    RULE_RHS_TIMING_STATS.with(|s| out.map("RULE_RHS_TIMING_STATS", &s.borrow()));
    DEFINITION_RULE_TIMING_STATS.with(|s| out.map("DEFINITION_RULE_TIMING_STATS", &s.borrow()));
    DEFINITION_DISPATCH_STATS.with(|s| out.map("DEFINITION_DISPATCH_STATS", &s.borrow()));
    BUILTIN_TIMING_STATS.with(|s| out.map("BUILTIN_TIMING_STATS", &s.borrow()));
    RUNTIME_PATH_TIMING_STATS.with(|s| out.map("RUNTIME_PATH_TIMING_STATS", &s.borrow()));
    REWRITE_RULE_PREFILTER_SOURCE_STATS
        .with(|s| out.map("REWRITE_RULE_PREFILTER_SOURCE_STATS", &s.borrow()));
    EXPRESSION_PROVENANCE_BY_EXACT.with(|s| out.map("EXPRESSION_PROVENANCE_BY_EXACT", &s.borrow()));
    PRESENCE_PROVENANCE_STATS.with(|s| out.map("PRESENCE_PROVENANCE_STATS", &s.borrow()));
    BUILTIN_TIMING_STACK.with(|s| {
        out.telemetry
            .insert("BUILTIN_TIMING_STACK".into(), format!("{:?}", s.borrow()));
    });
    RUNTIME_PATH_TIMING_STACK.with(|s| {
        out.telemetry.insert(
            "RUNTIME_PATH_TIMING_STACK".into(),
            format!("{:?}", s.borrow()),
        );
    });
    LEXICAL_CONTEXT_STACK.with(|s| {
        out.telemetry
            .insert("LEXICAL_CONTEXT_STACK".into(), format!("{:?}", s.borrow()));
    });
    ARG_PREFILTER_SITE_STACK.with(|s| {
        out.telemetry.insert(
            "ARG_PREFILTER_SITE_STACK".into(),
            format!("{:?}", s.borrow()),
        );
    });
    EXPRESSION_REBUILD_SOURCE_STACK.with(|s| {
        out.telemetry.insert(
            "EXPRESSION_REBUILD_SOURCE_STACK".into(),
            format!("{:?}", s.borrow()),
        );
    });
    LEXICAL_CONTEXT_DEPTH.with(|s| {
        out.telemetry
            .insert("LEXICAL_CONTEXT_DEPTH".into(), s.get().to_string());
    });
    ARG_PREFILTER_SITE_DEPTH.with(|s| {
        out.telemetry
            .insert("ARG_PREFILTER_SITE_DEPTH".into(), s.get().to_string());
    });
    EXPRESSION_REBUILD_SOURCE_DEPTH.with(|s| {
        out.telemetry.insert(
            "EXPRESSION_REBUILD_SOURCE_DEPTH".into(),
            s.get().to_string(),
        );
    });
    RUNTIME_PATH_TIMING_STATS.with(|s| {
        for (k, v) in s.borrow().iter() {
            let key = format!(
                "runtime|{}|{}",
                k.phase,
                profile_optional_symbol_name(k.source_symbol)
            );
            out.telemetry.insert(
                format!("{key}|input_required"),
                v.input_required.to_string(),
            );
            out.row(
                key,
                vec![
                    v.calls,
                    v.ok,
                    v.err,
                    v.some,
                    v.none,
                    v.return_err,
                    v.throw_err,
                ],
                vec![v.nanos, v.self_nanos],
            );
        }
    });
    DEFINITION_RULE_TIMING_STATS.with(|s| {
        for (k, v) in s.borrow().iter() {
            out.row(
                format!(
                    "definition-rule|{}|{}|{}",
                    k.kind,
                    profile_symbol_name(k.symbol),
                    k.index
                ),
                vec![v.calls, v.ok, v.err],
                vec![v.nanos],
            );
        }
    });
    DEFINITION_DISPATCH_STATS.with(|s| {
        for (k, v) in s.borrow().iter() {
            out.row(
                format!(
                    "definition-dispatch|{}|{}|{}",
                    k.kind,
                    profile_symbol_name(k.symbol),
                    profile_optional_symbol_name(k.source_symbol)
                ),
                vec![
                    v.candidates,
                    v.prefilter_skips,
                    v.fast_matches,
                    v.fast_misses,
                    v.matcher_matches,
                    v.matcher_misses,
                ],
                vec![
                    v.prefilter_nanos,
                    v.fast_match_nanos,
                    v.compiled_matcher_nanos,
                    v.rhs_substitution_nanos,
                ],
            );
        }
    });
    BUILTIN_TIMING_STATS.with(|s| {
        for (k, v) in s.borrow().iter() {
            out.row(
                format!(
                    "builtin|{}|{}|{}",
                    profile_symbol_name(k.symbol),
                    profile_optional_symbol_name(k.source_symbol),
                    k.arg_count
                ),
                vec![v.calls, v.ok, v.err],
                vec![v.nanos, v.self_nanos],
            );
        }
    });
    RULE_RHS_TIMING_STATS.with(|s| {
        for (k, v) in s.borrow().iter() {
            out.row(
                format!(
                    "rule-rhs|{}|{}",
                    k.phase,
                    profile_optional_symbol_name(k.source_symbol)
                ),
                vec![v.calls, v.changed, v.unchanged],
                vec![v.nanos],
            );
        }
    });
    crate::runtime::evaluation_cache::observe_logical_profile(&mut out);
    crate::runtime::substitution::observe_sharing_profile(&mut out);
    out.telemetry.insert(
        "repr".into(),
        format!("{:?}", rum_core::repr_profile_snapshot()),
    );
    out.flags.insert(
        "repr_enabled".into(),
        u8::from(rum_core::repr_profile_enabled()),
    );
    out
}

#[cfg(test)]
#[test]
fn profiling_observer_covers_every_map_and_flag_without_ranking() {
    let before = snapshot();
    let source = include_str!("../state.rs");
    for line in source.lines().filter(|l| l.contains("static ")) {
        let name = line
            .split("static ")
            .nth(1)
            .unwrap()
            .split(':')
            .next()
            .unwrap();
        if line.contains("RefCell<HashMap") {
            assert!(before.maps.contains_key(name), "unobserved map {name}");
        }
        if line.contains("AtomicU8") {
            assert!(before.flags.contains_key(name), "unobserved flag {name}");
        }
    }
    let map = (0..96).map(|i| (i, i * 2)).collect::<HashMap<_, _>>();
    let mut probe = LogicalProfileSnapshot::default();
    probe.map("more-than-ranked-display", &map);
    assert_eq!(probe.maps["more-than-ranked-display"], 96);
    assert!(probe.telemetry["more-than-ranked-display"].contains("95 => 190"));
    assert_eq!(
        before.logical,
        snapshot().logical,
        "observer ran guest work or changed counters"
    );
}
