use super::super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::scoping::packages::SourceUnit;
use std::hash::BuildHasherDefault;
use thiserror::Error;

pub(in crate::runtime) type SymbolMap<T> =
    HashMap<Symbol, T, BuildHasherDefault<RuntimeCacheHasher>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum ContextFrameKind {
    Begin,
    BeginPackage,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct ContextFrame {
    pub(in crate::runtime) kind: ContextFrameKind,
    pub(in crate::runtime) previous_context: String,
    pub(in crate::runtime) previous_context_path: Vec<String>,
    pub(in crate::runtime) opened_context: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum CallDispatchStage {
    SpecialForm,
    PrepareArguments,
    Listable,
    UpValues,
    DownValues,
    SubValues,
    OrdinaryBuiltin,
    Function,
}

const MAX_CALL_DISPATCH_STAGES: usize = 7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct CallDispatchPlan {
    pub(in crate::runtime) attributes: AttributeSet,
    pub(in crate::runtime) builtin_spec: Option<BuiltinSpec>,
    stages: [CallDispatchStage; MAX_CALL_DISPATCH_STAGES],
    stage_count: usize,
}

impl CallDispatchPlan {
    pub(in crate::runtime) fn new(
        attributes: AttributeSet,
        builtin_spec: Option<BuiltinSpec>,
    ) -> Self {
        Self {
            attributes,
            builtin_spec,
            stages: [CallDispatchStage::SpecialForm; MAX_CALL_DISPATCH_STAGES],
            stage_count: 0,
        }
    }

    pub(in crate::runtime) fn special_form(
        attributes: AttributeSet,
        builtin_spec: Option<BuiltinSpec>,
    ) -> Self {
        let mut plan = Self::new(attributes, builtin_spec);
        plan.push(CallDispatchStage::SpecialForm);
        plan
    }

    pub(in crate::runtime) fn push(&mut self, stage: CallDispatchStage) {
        debug_assert!(self.stage_count < MAX_CALL_DISPATCH_STAGES);
        self.stages[self.stage_count] = stage;
        self.stage_count += 1;
    }

    pub(in crate::runtime) fn len(&self) -> usize {
        self.stage_count
    }

    pub(in crate::runtime) fn stage(&self, index: usize) -> CallDispatchStage {
        self.stages[index]
    }

    pub(in crate::runtime) fn into_suspended(self) -> SuspendedCallDispatchPlan {
        SuspendedCallDispatchPlan {
            attributes: self.attributes,
            builtin_spec: self.builtin_spec.map(SuspendedBuiltinSpec::from),
            stages: self.stages,
            stage_count: self.stage_count,
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) fn stages(&self) -> &[CallDispatchStage] {
        &self.stages[..self.stage_count]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum SuspendedCapability {
    Pure,
    FileRead,
    FileWrite,
    Clock,
    Random,
    Diagnostic,
    Process,
    Interactive,
}

impl From<Capability> for SuspendedCapability {
    fn from(capability: Capability) -> Self {
        match capability {
            Capability::Pure => Self::Pure,
            Capability::FileRead => Self::FileRead,
            Capability::FileWrite => Self::FileWrite,
            Capability::Clock => Self::Clock,
            Capability::Random => Self::Random,
            Capability::Diagnostic => Self::Diagnostic,
            Capability::Process => Self::Process,
            Capability::Interactive => Self::Interactive,
        }
    }
}

impl SuspendedCapability {
    pub(in crate::runtime) fn into_capability(self) -> Capability {
        match self {
            Self::Pure => Capability::Pure,
            Self::FileRead => Capability::FileRead,
            Self::FileWrite => Capability::FileWrite,
            Self::Clock => Capability::Clock,
            Self::Random => Capability::Random,
            Self::Diagnostic => Capability::Diagnostic,
            Self::Process => Capability::Process,
            Self::Interactive => Capability::Interactive,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct SuspendedBuiltinSpec {
    pub(in crate::runtime) name: &'static str,
    pub(in crate::runtime) handler: Option<BuiltinFn>,
    pub(in crate::runtime) capability: SuspendedCapability,
    pub(in crate::runtime) memory_growth_code: u8,
}

impl From<BuiltinSpec> for SuspendedBuiltinSpec {
    fn from(spec: BuiltinSpec) -> Self {
        Self {
            name: spec.name,
            handler: spec.handler,
            capability: spec.capability.into(),
            memory_growth_code: spec.memory_growth.suspended_code(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct SuspendedCallDispatchPlan {
    pub(in crate::runtime) attributes: AttributeSet,
    pub(in crate::runtime) builtin_spec: Option<SuspendedBuiltinSpec>,
    stages: [CallDispatchStage; MAX_CALL_DISPATCH_STAGES],
    stage_count: usize,
}

impl SuspendedCallDispatchPlan {
    pub(in crate::runtime) fn len(&self) -> usize {
        self.stage_count
    }

    pub(in crate::runtime) fn stage(&self, index: usize) -> CallDispatchStage {
        self.stages[index]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct CallDispatchState {
    raw_args: SharedArgs,
    prepared_identity: Option<SharedArgs>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct PreparedDefinitionLookup<'a> {
    head: &'a Expr,
    args: &'a SharedArgs,
    attributes: AttributeSet,
}

impl CallDispatchState {
    pub(in crate::runtime) fn new(raw_args: SharedArgs) -> Self {
        Self {
            raw_args,
            prepared_identity: None,
        }
    }

    pub(in crate::runtime) fn from_prepared(prepared_args: SharedArgs) -> Self {
        Self {
            raw_args: SharedArgs::default(),
            prepared_identity: Some(prepared_args),
        }
    }

    pub(in crate::runtime) fn is_prepared(&self) -> bool {
        self.prepared_identity.is_some()
    }

    pub(in crate::runtime) fn raw_args(&self) -> &SharedArgs {
        &self.raw_args
    }

    #[inline]
    pub(in crate::runtime) fn take_raw_args(&mut self) -> SharedArgs {
        debug_assert!(self.prepared_identity.is_none());
        std::mem::take(&mut self.raw_args)
    }

    #[inline]
    pub(in crate::runtime) fn set_prepared_args(&mut self, prepared_args: SharedArgs) {
        debug_assert!(self.prepared_identity.is_none());
        self.prepared_identity = Some(prepared_args);
    }

    #[inline]
    pub(in crate::runtime) fn prepared_args(&self) -> &[Expr] {
        self.prepared_shared_args().as_ref()
    }

    #[inline]
    pub(in crate::runtime) fn prepared_shared_args(&self) -> &SharedArgs {
        self.prepared_identity
            .as_ref()
            .expect("call arguments should be prepared before this stage")
    }

    pub(in crate::runtime) fn prepared_definition_lookup<'a>(
        &'a self,
        head: &'a Expr,
        attributes: AttributeSet,
    ) -> PreparedDefinitionLookup<'a> {
        PreparedDefinitionLookup {
            head,
            args: self.prepared_shared_args(),
            attributes,
        }
    }

    pub(in crate::runtime) fn into_final_shared_args(self) -> SharedArgs {
        self.prepared_identity.unwrap_or(self.raw_args)
    }
}

impl PreparedDefinitionLookup<'_> {
    pub(in crate::runtime) fn head(&self) -> &Expr {
        self.head
    }

    pub(in crate::runtime) fn args(&self) -> &SharedArgs {
        self.args
    }

    pub(in crate::runtime) fn attributes(&self) -> AttributeSet {
        self.attributes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreadingDispatch {
    NotApplicable,
    Threaded(Expr),
    Mismatch,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct BlockSymbolState {
    pub(in crate::runtime) definition: Option<SymbolDefinition>,
    pub(in crate::runtime) attributes: Option<AttributeSet>,
    pub(in crate::runtime) options: Option<SymbolOptions>,
    pub(in crate::runtime) syntax_information: Option<Expr>,
    pub(in crate::runtime) messages: Vec<(MessageNameKey, MessageDefinition)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct DynamicOwnValueFrame {
    pub(in crate::runtime) symbol: Symbol,
    pub(in crate::runtime) value: Option<Expr>,
}

#[derive(Clone, Debug, FrameRootTrace)]
pub(in crate::runtime) struct LoaderSnapshot {
    pub(in crate::runtime) attributes: AttributeMap,
    pub(in crate::runtime) options: BTreeMap<Symbol, SymbolOptions>,
    pub(in crate::runtime) syntax_information: BTreeMap<Symbol, Expr>,
    pub(in crate::runtime) declared_stubs: SymbolSet,
    pub(in crate::runtime) removed_symbols: SymbolSet,
    pub(in crate::runtime) symbol_generations: BTreeMap<(String, String), u64>,
    pub(in crate::runtime) once_values: BTreeMap<String, Expr>,
    pub(in crate::runtime) gc_candidate_gensyms: std::collections::HashSet<Symbol>,
    pub(in crate::runtime) current_context: String,
    pub(in crate::runtime) context_path: Vec<String>,
    pub(in crate::runtime) context_stack: Vec<ContextFrame>,
    pub(in crate::runtime) package_search_paths: Vec<PathBuf>,
    pub(in crate::runtime) loaded_packages: Vec<String>,
    pub(in crate::runtime) definitions: DefinitionMap,
    pub(in crate::runtime) installed_system_own_values: SymbolSet,
    pub(in crate::runtime) installed_system_down_values: SymbolMap<Vec<(Expr, Expr)>>,
    pub(in crate::runtime) has_up_value_definitions: bool,
    pub(in crate::runtime) message_definitions: BTreeMap<MessageNameKey, MessageDefinition>,
    pub(in crate::runtime) exception_types: BTreeMap<String, Option<String>>,
    pub(in crate::runtime) next_generated_symbol_id: u64,
    pub(in crate::runtime) sealed_units: Vec<SourceUnit>,
    pub(in crate::runtime) next_sealed_symbol_id: u64,
    pub(in crate::runtime) next_source_unit_epoch: u64,
    pub(in crate::runtime) trace_events: Vec<TraceEvent>,
    pub(in crate::runtime) trace_event_depths: Vec<usize>,
    pub(in crate::runtime) disabled_message_selectors: Vec<MessageSelector>,
    pub(in crate::runtime) messages: Vec<RuntimeMessage>,
    pub(in crate::runtime) last_message_list: Vec<RuntimeMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct ReapBucket {
    pub(in crate::runtime) tag: Expr,
    pub(in crate::runtime) values: Vec<Expr>,
}

#[derive(Default, FrameRootTrace)]
pub(in crate::runtime) struct ReapBucketGroup {
    pub(in crate::runtime) buckets: Vec<ReapBucket>,
    /// Lookup mirror for `buckets`; logical state remains entirely in the
    /// ordered bucket vector.
    pub(in crate::runtime) tag_positions: Option<HashMap<Expr, usize>>,
}

impl Clone for ReapBucketGroup {
    fn clone(&self) -> Self {
        Self {
            buckets: self.buckets.clone(),
            tag_positions: None,
        }
    }
}

impl std::fmt::Debug for ReapBucketGroup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.buckets, formatter)
    }
}

impl PartialEq for ReapBucketGroup {
    fn eq(&self, other: &Self) -> bool {
        self.buckets == other.buckets
    }
}

impl Eq for ReapBucketGroup {}

impl ReapBucketGroup {
    pub(in crate::runtime) fn iter(&self) -> std::slice::Iter<'_, ReapBucket> {
        self.buckets.iter()
    }
}

impl<'a> IntoIterator for &'a ReapBucketGroup {
    type Item = &'a ReapBucket;
    type IntoIter = std::slice::Iter<'a, ReapBucket>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct ReapScope {
    pub(in crate::runtime) patterns: Vec<Expr>,
    pub(in crate::runtime) pattern_list_form: bool,
    pub(in crate::runtime) handler: Option<Expr>,
    pub(in crate::runtime) buckets_by_pattern: Vec<ReapBucketGroup>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct MonitorScope {
    pub(in crate::runtime) display_expr: Expr,
    pub(in crate::runtime) evaluation_monitor: Option<Expr>,
    pub(in crate::runtime) step_monitor: Option<Expr>,
    pub(in crate::runtime) last_display: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum MatcherNode {
    Exact(Expr),
    Blank {
        head_constraint: Option<Expr>,
    },
    KeyValuePattern(Vec<(MatcherNode, MatcherNode, bool)>),
    Named {
        name: Symbol,
        inner: Box<MatcherNode>,
    },
    Condition {
        inner: Box<MatcherNode>,
        condition: Expr,
    },
    DefinitionCondition(Box<DefinitionConditionMatcher>),
    PatternTest {
        inner: Box<MatcherNode>,
        test: Expr,
    },
    Except {
        excluded: Box<MatcherNode>,
        inner: Box<MatcherNode>,
    },
    Alternatives(Vec<MatcherNode>),
    Call {
        head: Box<MatcherNode>,
        args: Vec<MatcherArg>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct DefinitionConditionMatcher {
    pub(in crate::runtime) inner: MatcherNode,
    pub(in crate::runtime) lhs: Expr,
    pub(in crate::runtime) guards: Box<[Expr]>,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum MatcherArg {
    Single(MatcherNode),
    Sequence(CompiledSequencePattern),
    Optional(CompiledOptionalPattern),
    PatternSequenceGroup(Vec<MatcherArg>),
    OrderlessPatternSequenceGroup(Vec<MatcherArg>),
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct CompiledSequencePattern {
    pub(in crate::runtime) min_len: usize,
    pub(in crate::runtime) max_len: Option<usize>,
    pub(in crate::runtime) matcher: MatcherNode,
    pub(in crate::runtime) name: Option<Symbol>,
    pub(in crate::runtime) checks: Vec<SequencePatternCheck>,
    pub(in crate::runtime) greediness: SequenceGreediness,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum SequencePatternCheck {
    Condition(Expr),
    PatternTest(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum SequenceGreediness {
    Shortest,
    Longest,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct CompiledOptionalPattern {
    pub(in crate::runtime) inner: Box<MatcherArg>,
    pub(in crate::runtime) default: CompiledOptionalDefault,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum CompiledOptionalDefault {
    Explicit(Expr),
    Implicit { head: Expr, position: usize },
    UnavailableImplicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum MessageCatalogQuery {
    All,
    Symbol(Symbol),
    Key(MessageNameKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) enum AttributeUpdate {
    Insert,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreadArgSpec {
    All,
    None,
    Prefix(usize),
    Suffix(usize),
    Single(isize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

#[derive(Debug, Error, FrameRootTrace)]
pub enum EvalError {
    #[error("continuation slot mismatch: expected {}, got {}", .0.expected(), .0.got())]
    ContinuationSlotMismatch(SlotMismatch),
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error("invalid integer literal `{literal}`")]
    InvalidInteger { literal: String },
    #[error("could not read `{path}`: {source}")]
    Io {
        path: String,
        #[source]
        source: FileIoError,
    },
    #[error("no file provider installed")]
    NoFileIoProvider,
    #[error("could not resolve package or file `{target}`")]
    LoadPathNotFound {
        target: String,
        searched: Vec<String>,
    },
    #[error("evaluation cycle detected while re-entering `{expr}`")]
    EvaluationCycleDetected { expr: String },
    #[error("evaluation exceeded recursion limit ({limit})")]
    RecursionLimitExceeded { limit: usize },
    #[error("evaluation exceeded native stack limit ({limit} bytes)")]
    StackLimitExceeded { limit: usize },
    #[error("evaluation exceeded pattern-match recursion limit ({limit})")]
    MatchRecursionLimitExceeded { limit: usize },
    #[error("evaluation cancelled by host")]
    Cancelled,
    #[error("evaluation exceeded TimeConstrained limit")]
    TimeConstraintExceeded,
    #[error("evaluation exceeded MemoryConstrained limit ({limit})")]
    MemoryConstraintExceeded { limit: usize },
    #[error("evaluation exceeded generated element limit ({limit})")]
    GeneratedElementLimitExceeded { limit: usize },
    #[error("evaluation exceeded integer work limit ({limit})")]
    IntegerWorkLimitExceeded { limit: usize },
    #[error("evaluation exceeded iteration work limit ({limit})")]
    IterationWorkLimitExceeded { limit: usize },
    #[error("ReplaceRepeated exceeded iteration limit ({limit})")]
    IterationLimitExceeded { limit: usize },
    #[error("evaluation exceeded fixed-point pass limit ({limit})")]
    EvaluationPassLimitExceeded { limit: usize },
    #[error("nonlocal return: {value}")]
    Return { value: Expr },
    #[error("uncaught throw: {value} with tag {tag}")]
    Thrown {
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    },
    #[error(
        "uncaught throw: {cleanup_value} with tag {cleanup_tag}; uncaught throw: {value} with tag {tag}"
    )]
    ThrownBypassingCatch {
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    },
    #[error("uncaught Abort[]")]
    Abort,
    #[error("uncaught protected Abort[]")]
    ProtectedAbort,
    #[error("uncaught Break[]")]
    Break,
    #[error("uncaught Continue[]")]
    Continue,
    #[error("Exit[{code}]")]
    Exit { code: i64 },
    #[error("{request}")]
    InputRequired { request: RuntimeInputRequest },
    #[error("uncaught exception: {exception}")]
    ExceptionThrown { exception: Expr },
}

impl Session {
    pub(in crate::runtime) fn empty() -> Self {
        Self {
            accounting_route: None,
            builtins: BuiltinSpecMap::default(),
            attributes: AttributeMap::default(),
            attribute_revision: 0,
            attribute_set_cache: AttributeSetCache::default(),
            builtin_dispatch_spec_cache: BuiltinDispatchSpecCache::default(),
            builtin_memory_projection_count: std::cell::Cell::new(0),
            evaluation_epoch: 0,
            symbol_cache_generation: 0,
            call_stability_generation: 0,
            stability_dirty_bloom: StabilityBloom::default(),
            stability_rewritten_dirty_bloom: StabilityBloom::default(),
            stable_symbol_cache: SymbolStabilityCache::default(),
            symbol_call_effect_cache: SymbolCallEffectCache::default(),
            call_stability_cache: CallStabilityCache::default(),
            association_lookup_cache: AssociationLookupCache::default(),
            options: BTreeMap::new(),
            syntax_information: BTreeMap::new(),
            declared_stubs: SymbolSet::default(),
            removed_symbols: SymbolSet::default(),
            symbol_generations: BTreeMap::new(),
            once_values: BTreeMap::new(),
            option_value_scopes: Vec::new(),
            dynamic_own_values: Vec::new(),
            module_activations: Vec::new(),
            gc_candidate_gensyms: std::collections::HashSet::new(),
            gensym_gc_watermark: gensym_gc_threshold(),
            #[cfg(test)]
            gensym_gc_sweep_count: 0,
            in_gensym_sweep: false,
            current_context: GLOBAL_CONTEXT.to_string(),
            context_path: vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()],
            context_stack: Vec::new(),
            package_search_paths: vec![PathBuf::from(".")],
            loaded_packages: Vec::new(),
            load_stack: Vec::new(),
            file_io: None,
            entropy_source: None,
            prng_state: PrngState::default(),
            definitions: DefinitionMap::default(),
            installed_system_own_values: SymbolSet::default(),
            installed_system_down_values: SymbolMap::default(),
            has_up_value_definitions: false,
            message_definitions: BTreeMap::new(),
            exception_types: BTreeMap::new(),
            message_symbol_counts: SymbolCountMap::default(),
            next_generated_symbol_id: 0,
            sealed_units: Vec::new(),
            next_sealed_symbol_id: 0,
            next_source_unit_epoch: 0,
            restoring_snapshot: false,
            resource_limits: ResourceLimits::default(),
            iteration_product_budget: std::cell::Cell::new((1, 0)),
            evaluation_depth: 0,
            match_depth: 0,
            native_stack_base: 0,
            native_stack_next_check_depth: NATIVE_STACK_DEPTH_POLL_STRIDE,
            stability_native_stack_poll_counter: 0,
            interrupt_armed: false,
            evaluation_stack: Vec::new(),
            tracing_enabled: false,
            trace_events: Vec::new(),
            trace_event_depths: Vec::new(),
            disabled_message_selectors: Vec::new(),
            messages: Vec::new(),
            prints: Vec::new(),
            print_sink: None,
            peak_memory_proxy_bytes: std::cell::Cell::new(0),
            last_message_list: Vec::new(),
            current_message_list_checkpoint: None,
            current_message_list_dirty: false,
            handled_non_loader_eval_errors: Vec::new(),
            reap_scopes: Vec::new(),
            monitor_scopes: Vec::new(),
            monitor_callbacks_enabled: true,
            current_time_deadline: None,
            current_memory_limit: None,
            asserts_enabled: false,
            abort_protect_depth: 0,
            abort_protect_pending: 0,
            queued_input_responses: VecDeque::new(),
            pending_input_request: None,
            pending_eval_request: None,
            guest_form: None,
            pending_suspension: SuspensionSlot::empty(),
            resuming_suspension: None,
            abandonment_active: false,
            abandonment_worklist: Vec::new(),
            request_interrupt_control: std::cell::RefCell::new(std::sync::Arc::new(
                RequestInterruptState::new(),
            )),
            next_request_generation: std::cell::Cell::new(1),
            pending_accepted_request: std::cell::RefCell::new(None),
            active_accepted_request: None,
            eval_timeout: None,
            eval_deadline: None,
            interrupt_poll_counter: 0,
            capability_policy: CapabilityPolicy::unrestricted(),
            value_q_effect_suppression_active: false,
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        let mut session = Self::empty();
        for spec in core_builtin_specs() {
            session.register_builtin(*spec);
        }
        #[cfg(test)]
        test_builtins::register_test_arithmetic_builtins(&mut session);
        #[cfg(test)]
        crate::runtime::tests::install_test_file_io(&mut session);

        session
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        print_lexical_profile_summary_once();
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<BlockSymbolState>();
    assert_traces::<CallDispatchState>();
    assert_traces::<CompiledOptionalDefault>();
    assert_traces::<CompiledOptionalPattern>();
    assert_traces::<CompiledSequencePattern>();
    assert_traces::<DefinitionConditionMatcher>();
    assert_traces::<EvalError>();
    assert_traces::<MatcherArg>();
    assert_traces::<MatcherNode>();
    assert_traces::<PreparedDefinitionLookup<'_>>();
    assert_traces::<ReapBucket>();
    assert_traces::<ReapBucketGroup>();
    assert_traces::<SequenceGreediness>();
    assert_traces::<SequencePatternCheck>();
};
