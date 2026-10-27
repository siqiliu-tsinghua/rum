use super::{Atom, Expr, Symbol, evaluation_profile_current_source_symbol};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

mod config;
mod counters;
mod fingerprints;
mod formatting;
#[allow(dead_code)] // LP02 supplies infrastructure; later briefs adopt phases.
pub(in crate::runtime) mod logical;
mod presence;
mod records;
mod report_bindings;
mod report_presence_reuse;
mod report_presence_shapes;
mod report_presence_sources;
mod report_rebuild;
mod report_timing;
mod reset;
mod scopes;
mod state;
mod summary;
mod timing;

#[cfg(test)]
pub(in crate::runtime) mod tests;

use config::{
    builtin_timing_enabled, definition_rule_timing_enabled, lexical_profile_bindings_enabled,
    lexical_profile_configured, lexical_profile_incremental_enabled,
    lexical_profile_provenance_enabled, lexical_profile_shapes_enabled,
    lexical_profile_source_fingerprints_enabled, lexical_profile_sources_enabled,
    rule_rhs_timing_enabled,
};
use counters::{
    average, elapsed_nanos_since, load, record_add, record_counter, record_max,
    record_profile_lexical_timing, record_profile_lexical_timing_elapsed, record_timing,
};
use fingerprints::{
    max_rule_rhs_alpha_id, presence_alpha_relation, presence_target_origin,
    structural_text_and_exact_fingerprints, structural_text_exact_fingerprints_and_nodes,
};
use formatting::{
    expression_provenance_kind_name, expression_rebuild_source_name, presence_alpha_relation_name,
    presence_root_shape_name, presence_scan_kind_name, presence_source_fingerprint_kind_name,
    presence_target_origin_name, profile_optional_symbol_name, profile_stats_average,
    profile_stats_per_mille, profile_symbol_name,
};
use presence::{
    current_expression_rebuild_source_symbol, presence_root_shape,
    record_expression_provenance_from_fingerprint, record_presence_binding,
    record_presence_provenance, record_presence_reuse, record_presence_shape,
    record_presence_source,
};
use report_bindings::{print_presence_binding_name_summary, print_presence_binding_summary};
use report_presence_reuse::{
    print_presence_exact_fingerprint_group_summary, print_presence_fingerprint_group_summary,
    print_presence_fingerprint_summary, print_presence_reuse_group_summary,
    print_presence_reuse_summary,
};
use report_presence_shapes::{
    print_presence_arg_site_summary, print_presence_context_shape_summary,
    print_presence_context_summary, print_presence_shape_summary,
};
use report_presence_sources::{
    print_presence_provenance_summary, print_presence_source_fingerprint_group_summary,
    print_presence_source_fingerprint_summary, print_presence_source_shape_summary,
    print_presence_source_summary, print_presence_source_target_shape_summary,
};
use report_rebuild::{
    print_expression_rebuild_exact_group_summary, print_expression_rebuild_exact_summary,
    print_expression_rebuild_group_summary, print_expression_rebuild_summary,
    print_rule_rhs_alpha_rename_summary, print_rule_rhs_timing_summary,
};
use report_timing::{
    print_builtin_timing_summary, print_definition_dispatch_summary,
    print_definition_rule_timing_summary, print_rewrite_rule_prefilter_summary,
    print_runtime_path_timing_summary,
};
use state::*;

pub(super) use config::{lexical_profile_detail_enabled, lexical_profile_enabled};
pub(super) use records::{
    record_profile_arg_list_start, record_profile_arg_prefilter,
    record_profile_arg_substitution_attempt, record_profile_definition_rhs_eval,
    record_profile_eval_module, record_profile_expression_rebuild,
    record_profile_lexical_substitution_finish, record_profile_lexical_substitution_start,
    record_profile_module_body_substitution, record_profile_module_init_substitution,
    record_profile_module_resume_entry, record_profile_multi_presence_scan,
    record_profile_rewrite_rule_match_outcome, record_profile_rewrite_rule_prefilter_candidate,
    record_profile_rule_rhs_alpha_rename, record_profile_rule_rhs_binding_value,
    record_profile_single_presence_scan,
};
pub(super) use reset::reset_lexical_profile_counters;
pub(super) use scopes::{
    expression_rebuild_source_symbol, lexical_profile_arg_prefilter_site, lexical_profile_context,
};
pub(super) use summary::{print_lexical_profile_snapshot, print_lexical_profile_summary_once};
pub(super) use timing::{
    builtin_timing_start, definition_dispatch_timing_start, definition_rule_timing_start,
    lexical_profile_start, record_profile_builtin_timing,
    record_profile_definition_dispatch_candidate, record_profile_definition_dispatch_fast_outcome,
    record_profile_definition_dispatch_matcher_outcome,
    record_profile_definition_dispatch_phase_timing,
    record_profile_definition_dispatch_prefilter_skip, record_profile_definition_rule_timing,
    record_profile_rule_rhs_timing, record_profile_runtime_path_timing, rule_rhs_timing_start,
    runtime_path_timing_start,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceScanKind {
    Single,
    Multi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceRootShape {
    AtomSymbol(Symbol),
    AtomOther,
    CallHeadSymbol(Symbol),
    CallHeadAtomOther,
    CallHeadCallSymbol(Symbol),
    CallHeadCallOther,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceShapeKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceContextKey {
    kind: PresenceScanKind,
    context: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceContextShapeKey {
    kind: PresenceScanKind,
    context: &'static str,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceArgSite {
    parent: PresenceRootShape,
    index: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceArgSiteKey {
    kind: PresenceScanKind,
    site: PresenceArgSite,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceShapeStats {
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceBindingKey {
    shape: PresenceRootShape,
    target: Symbol,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceBindingNameKey {
    shape: PresenceRootShape,
    target_text: Symbol,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceKey {
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceShapeKey {
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceTargetShapeKey {
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceSourceFingerprintKind {
    Text,
    Exact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceFingerprintKey {
    fingerprint_kind: PresenceSourceFingerprintKind,
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceSourceFingerprintGroupKey {
    fingerprint_kind: PresenceSourceFingerprintKind,
    kind: PresenceScanKind,
    source: Symbol,
    context: &'static str,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceSourceFingerprintGroupStats {
    unique_fingerprints: u64,
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    max_scans_per_fingerprint: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceReuseKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    head_ptr: usize,
    args_ptr: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceReuseGroupKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceReuseGroupStats {
    unique_ptrs: u64,
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    max_scans_per_ptr: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceFingerprintKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceFingerprintGroupKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct PresenceFingerprintGroupStats {
    unique_fingerprints: u64,
    scans: u64,
    nodes: u64,
    timed: u64,
    nanos: u64,
    max_scans_per_fingerprint: u64,
    true_count: u64,
    false_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceExactFingerprintKey {
    kind: PresenceScanKind,
    shape: PresenceRootShape,
    target_text: Option<Symbol>,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExpressionRebuildKey {
    source: &'static str,
    source_symbol: Option<Symbol>,
    shape: PresenceRootShape,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExpressionRebuildGroupKey {
    source: &'static str,
    source_symbol: Option<Symbol>,
    shape: PresenceRootShape,
}

#[derive(Clone, Copy, Debug, Default)]
struct ExpressionRebuildStats {
    rebuilds: u64,
    nodes: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct ExpressionRebuildGroupStats {
    unique_fingerprints: u64,
    rebuilds: u64,
    nodes: u64,
    max_rebuilds_per_fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RuleRhsAlphaRenameKey {
    source_symbol: Option<Symbol>,
    local_text: Symbol,
}

#[derive(Clone, Copy, Debug, Default)]
struct RuleRhsAlphaRenameStats {
    renames: u64,
    fresh_with_identity: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RuleRhsTimingKey {
    phase: &'static str,
    source_symbol: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct RuleRhsTimingStats {
    calls: u64,
    nanos: u64,
    changed: u64,
    unchanged: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DefinitionRuleTimingKey {
    kind: &'static str,
    symbol: Symbol,
    index: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct DefinitionRuleTimingStats {
    calls: u64,
    nanos: u64,
    ok: u64,
    err: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DefinitionDispatchKey {
    kind: &'static str,
    symbol: Symbol,
    source_symbol: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct DefinitionDispatchStats {
    candidates: u64,
    prefilter_skips: u64,
    fast_matches: u64,
    fast_misses: u64,
    matcher_matches: u64,
    matcher_misses: u64,
    prefilter_nanos: u64,
    fast_match_nanos: u64,
    compiled_matcher_nanos: u64,
    rhs_substitution_nanos: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BuiltinTimingKey {
    symbol: Symbol,
    source_symbol: Option<Symbol>,
    arg_count: u16,
}

#[derive(Clone, Copy, Debug, Default)]
struct BuiltinTimingStats {
    calls: u64,
    nanos: u64,
    self_nanos: u64,
    ok: u64,
    err: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RuntimePathTimingKey {
    phase: &'static str,
    source_symbol: Option<Symbol>,
}

#[derive(Clone, Copy, Debug, Default)]
struct RuntimePathTimingStats {
    calls: u64,
    nanos: u64,
    self_nanos: u64,
    ok: u64,
    err: u64,
    some: u64,
    none: u64,
    return_err: u64,
    throw_err: u64,
    input_required: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeProfileOutcome {
    Ok,
    Some,
    None,
    Return,
    Throw,
    InputRequired,
    Error,
}

#[derive(Clone, Copy, Debug, Default)]
struct RewriteRulePrefilterStats {
    candidates: u64,
    would_reject: u64,
    would_matcher: u64,
    matcher_misses: u64,
    matches: u64,
    false_rejects: u64,
}

#[derive(Clone, Copy, Debug)]
struct BuiltinTimingFrame {
    key: BuiltinTimingKey,
    child_nanos: u64,
}

#[derive(Clone, Copy, Debug)]
struct RuntimePathTimingFrame {
    key: RuntimePathTimingKey,
    child_nanos: u64,
}

pub(super) struct BuiltinTimingToken {
    start: Instant,
    key: BuiltinTimingKey,
}

pub(super) struct RuntimePathTimingToken {
    start: Instant,
    key: RuntimePathTimingKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceTargetOrigin {
    None,
    PlainSymbol,
    GeneratedDollarName,
    RuleRhsAlphaIdentity,
    OtherIdentity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ExpressionProvenanceKind {
    RuleBindingValue,
    RuleRhsRebuild,
    OtherRebuild,
    Unknown,
}

impl ExpressionProvenanceKind {
    fn for_rebuild_source(source: &'static str) -> Self {
        if source.starts_with("rule_rhs_") {
            Self::RuleRhsRebuild
        } else {
            Self::OtherRebuild
        }
    }

    fn priority(self) -> u8 {
        match self {
            Self::RuleBindingValue => 3,
            Self::RuleRhsRebuild => 2,
            Self::OtherRebuild => 1,
            Self::Unknown => 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ExpressionProvenanceValue {
    kind: ExpressionProvenanceKind,
    source: &'static str,
    source_symbol: Option<Symbol>,
    max_rule_rhs_alpha_id: Option<u64>,
    records: u64,
    nodes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PresenceAlphaRelation {
    NoTarget,
    TargetNotRuleRhsAlpha,
    ExprHasNoRuleRhsAlpha,
    ExprOlderThanTarget,
    ExprSameOrNewerThanTarget,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PresenceProvenanceKey {
    kind: PresenceScanKind,
    source: Option<Symbol>,
    context: &'static str,
    target_text: Option<Symbol>,
    target_origin: PresenceTargetOrigin,
    expr_origin: ExpressionProvenanceKind,
    expr_origin_source: &'static str,
    expr_origin_symbol: Option<Symbol>,
    alpha_relation: PresenceAlphaRelation,
    shape: PresenceRootShape,
}

pub(super) struct LexicalProfileContextGuard {
    active: bool,
}

pub(super) struct LexicalProfileArgSiteGuard {
    active: bool,
}

pub(super) struct ExpressionRebuildSourceGuard {
    active: bool,
}

#[cfg(any(test, feature = "test-continuation-slots"))]
#[path = "tests/logical_contract.rs"]
mod logical_contract;
#[cfg(any(test, feature = "test-continuation-slots"))]
pub use logical_contract::LogicalProfileSnapshot;
#[cfg(any(test, feature = "test-continuation-slots"))]
pub(crate) use logical_contract::snapshot as logical_profile_snapshot;
