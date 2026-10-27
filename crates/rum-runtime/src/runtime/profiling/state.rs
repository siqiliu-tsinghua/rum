use super::logical::source_stack::SourceStack;
use super::*;

pub(super) const LEXICAL_PROFILE_ENV: &str = "RUM_PROFILE_LEXICAL_SUBSTITUTION";
pub(super) const LEXICAL_PROFILE_DEFER_ENV: &str = "RUM_PROFILE_LEXICAL_DEFER";
pub(super) const LEXICAL_PROFILE_DETAIL_ENV: &str = "RUM_PROFILE_LEXICAL_DETAIL";
pub(super) const LEXICAL_PROFILE_INCREMENTAL_ENV: &str = "RUM_PROFILE_LEXICAL_INCREMENTAL";
pub(super) const LEXICAL_PROFILE_SHAPES_ENV: &str = "RUM_PROFILE_LEXICAL_SHAPES";
pub(super) const LEXICAL_PROFILE_BINDINGS_ENV: &str = "RUM_PROFILE_LEXICAL_BINDINGS";
pub(super) const LEXICAL_PROFILE_SOURCES_ENV: &str = "RUM_PROFILE_LEXICAL_SOURCES";
pub(super) const LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENV: &str =
    "RUM_PROFILE_LEXICAL_SOURCE_FINGERPRINTS";
pub(super) const LEXICAL_PROFILE_PROVENANCE_ENV: &str = "RUM_PROFILE_LEXICAL_PROVENANCE";
pub(super) const RULE_RHS_TIMING_ENV: &str = "RUM_PROFILE_RULE_RHS_TIMING";
pub(super) const DEFINITION_RULE_TIMING_ENV: &str = "RUM_PROFILE_DEFINITION_RULE_TIMING";
pub(super) const BUILTIN_TIMING_ENV: &str = "RUM_PROFILE_BUILTIN_TIMING";
pub(super) const PRESENCE_REUSE_MIN_NODES: u64 = 1024;
pub(super) const RULE_RHS_ALPHA_ID_MARKER: u64 = 1_u64 << 63;

pub(super) static LEXICAL_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_ACTIVE: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_DETAIL_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_INCREMENTAL_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_SHAPES_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_BINDINGS_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_SOURCES_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_SOURCE_FINGERPRINTS_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_PROVENANCE_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static RULE_RHS_TIMING_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static DEFINITION_RULE_TIMING_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static BUILTIN_TIMING_ENABLED: AtomicU8 = AtomicU8::new(0);
pub(super) static LEXICAL_PROFILE_SUMMARY_PRINTED: AtomicU8 = AtomicU8::new(0);

pub(super) static DEFINITION_RHS_EVAL_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static DEFINITION_RHS_EVAL_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static EVAL_MODULE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static EVAL_MODULE_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static MODULE_RESUME_ENTRY_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static MODULE_RESUME_MAX_INDEX: AtomicU64 = AtomicU64::new(0);
pub(super) static MODULE_INIT_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static MODULE_INIT_SUBSTITUTION_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static MODULE_BODY_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static MODULE_BODY_SUBSTITUTION_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_SUBSTITUTION_TIME_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_SUBSTITUTION_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_SUBSTITUTION_CHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_SUBSTITUTION_UNCHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_BINDING_EMPTY_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_BINDING_SINGLE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_BINDING_MULTI_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_BINDING_MAX_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static LEXICAL_SHADOWED_NONEMPTY_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_LIST_SUBSTITUTION_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_LIST_TOTAL_ARGS: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_LIST_MAX_LEN: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_PREFILTER_SKIP_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_PREFILTER_MAYBE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_SUBSTITUTION_ATTEMPT_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_SUBSTITUTION_ATTEMPT_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_SUBSTITUTION_CHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static ARG_SUBSTITUTION_UNCHANGED_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_SINGLE_SCAN_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_SINGLE_SCAN_NODES: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_SINGLE_SCAN_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_SINGLE_SCAN_TIME_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_SINGLE_SCAN_TRUE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_SINGLE_SCAN_FALSE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_MULTI_SCAN_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_MULTI_SCAN_NODES: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_MULTI_SCAN_NANOS: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_MULTI_SCAN_TIME_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_MULTI_SCAN_TRUE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static PRESENCE_MULTI_SCAN_FALSE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static REWRITE_RULE_CANDIDATE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static REWRITE_RULE_WOULD_PREFILTER_REJECT_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static REWRITE_RULE_WOULD_MATCHER_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static REWRITE_RULE_MATCHER_MISS_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static REWRITE_RULE_MATCH_COUNT: AtomicU64 = AtomicU64::new(0);
pub(super) static REWRITE_RULE_PREFILTER_FALSE_REJECT_COUNT: AtomicU64 = AtomicU64::new(0);

thread_local! {
    pub(super) static PRESENCE_SHAPE_STATS: RefCell<HashMap<PresenceShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_CONTEXT_STATS: RefCell<HashMap<PresenceContextKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_CONTEXT_SHAPE_STATS: RefCell<HashMap<PresenceContextShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_ARG_SITE_STATS: RefCell<HashMap<PresenceArgSiteKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_BINDING_STATS: RefCell<HashMap<PresenceBindingKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_BINDING_NAME_STATS: RefCell<HashMap<PresenceBindingNameKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_SOURCE_STATS: RefCell<HashMap<PresenceSourceKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_SOURCE_SHAPE_STATS: RefCell<HashMap<PresenceSourceShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_SOURCE_TARGET_SHAPE_STATS: RefCell<HashMap<PresenceSourceTargetShapeKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_SOURCE_FINGERPRINT_STATS: RefCell<HashMap<PresenceSourceFingerprintKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_REUSE_STATS: RefCell<HashMap<PresenceReuseKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_FINGERPRINT_STATS: RefCell<HashMap<PresenceFingerprintKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_EXACT_FINGERPRINT_STATS: RefCell<HashMap<PresenceExactFingerprintKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static EXPRESSION_REBUILD_STATS: RefCell<HashMap<ExpressionRebuildKey, ExpressionRebuildStats>> =
        RefCell::new(HashMap::new());
    pub(super) static EXPRESSION_REBUILD_EXACT_STATS: RefCell<HashMap<ExpressionRebuildKey, ExpressionRebuildStats>> =
        RefCell::new(HashMap::new());
    pub(super) static RULE_RHS_ALPHA_RENAME_STATS: RefCell<HashMap<RuleRhsAlphaRenameKey, RuleRhsAlphaRenameStats>> =
        RefCell::new(HashMap::new());
    pub(super) static RULE_RHS_TIMING_STATS: RefCell<HashMap<RuleRhsTimingKey, RuleRhsTimingStats>> =
        RefCell::new(HashMap::new());
    pub(super) static DEFINITION_RULE_TIMING_STATS: RefCell<HashMap<DefinitionRuleTimingKey, DefinitionRuleTimingStats>> =
        RefCell::new(HashMap::new());
    pub(super) static DEFINITION_DISPATCH_STATS: RefCell<HashMap<DefinitionDispatchKey, DefinitionDispatchStats>> =
        RefCell::new(HashMap::new());
    pub(super) static BUILTIN_TIMING_STATS: RefCell<HashMap<BuiltinTimingKey, BuiltinTimingStats>> =
        RefCell::new(HashMap::new());
    pub(super) static BUILTIN_TIMING_STACK: RefCell<Vec<BuiltinTimingFrame>> = const { RefCell::new(Vec::new()) };
    pub(super) static RUNTIME_PATH_TIMING_STATS: RefCell<HashMap<RuntimePathTimingKey, RuntimePathTimingStats>> =
        RefCell::new(HashMap::new());
    pub(super) static RUNTIME_PATH_TIMING_STACK: RefCell<Vec<RuntimePathTimingFrame>> = const { RefCell::new(Vec::new()) };
    pub(super) static REWRITE_RULE_PREFILTER_SOURCE_STATS: RefCell<HashMap<Symbol, RewriteRulePrefilterStats>> =
        RefCell::new(HashMap::new());
    pub(super) static EXPRESSION_PROVENANCE_BY_EXACT: RefCell<HashMap<u64, ExpressionProvenanceValue>> =
        RefCell::new(HashMap::new());
    pub(super) static PRESENCE_PROVENANCE_STATS: RefCell<HashMap<PresenceProvenanceKey, PresenceShapeStats>> =
        RefCell::new(HashMap::new());
    pub(super) static LEXICAL_CONTEXT_STACK: RefCell<SourceStack<&'static str>> = const { RefCell::new(SourceStack::new()) };
    pub(super) static LEXICAL_CONTEXT_DEPTH: Cell<usize> = const { Cell::new(0) };
    pub(super) static ARG_PREFILTER_SITE_STACK: RefCell<SourceStack<PresenceArgSite>> = const { RefCell::new(SourceStack::new()) };
    pub(super) static ARG_PREFILTER_SITE_DEPTH: Cell<usize> = const { Cell::new(0) };
    pub(super) static EXPRESSION_REBUILD_SOURCE_STACK: RefCell<SourceStack<Symbol>> = const { RefCell::new(SourceStack::new()) };
    pub(super) static EXPRESSION_REBUILD_SOURCE_DEPTH: Cell<usize> = const { Cell::new(0) };
}
