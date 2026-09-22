//! Session-owner identities; no freshness is inferred from counters.
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub(in crate::runtime) enum MemoryOwner {
    AbandonmentWorklist,
    AccountingRoute,
    AssociationLookupCache,
    AttributeSetCache,
    Attributes,
    BuiltinDispatchSpecCache,
    Builtins,
    CallStabilityCache,
    ContextPath,
    ContextStack,
    CurrentContext,
    DeclaredStubs,
    Definitions,
    DisabledMessageSelectors,
    DynamicOwnValues,
    EntropySource,
    EvaluationStack,
    ExceptionTypes,
    FileIo,
    GcCandidateGensyms,
    HandledNonLoaderEvalErrors,
    InstalledSystemDownValues,
    InstalledSystemOwnValues,
    LastMessageList,
    LoadStack,
    LoadedPackages,
    MessageDefinitions,
    MessageSymbolCounts,
    Messages,
    ModuleActivations,
    MonitorScopes,
    OnceValues,
    OptionValueScopes,
    Options,
    PackageSearchPaths,
    PendingEvalRequest,
    PendingInputRequest,
    PendingSuspension,
    PrintSink,
    Prints,
    QueuedInputResponses,
    ReapScopes,
    RemovedSymbols,
    RequestInterruptControl,
    ResourceLimits,
    ResumingSuspension,
    SealedUnits,
    StableSymbolCache,
    SymbolCallEffectCache,
    SymbolGenerations,
    SyntaxInformation,
    TraceEventDepths,
    TraceEvents,
}
impl MemoryOwner {
    pub(in crate::runtime) const ALL: [Self; 53] = [
        Self::AbandonmentWorklist,
        Self::AccountingRoute,
        Self::AssociationLookupCache,
        Self::AttributeSetCache,
        Self::Attributes,
        Self::BuiltinDispatchSpecCache,
        Self::Builtins,
        Self::CallStabilityCache,
        Self::ContextPath,
        Self::ContextStack,
        Self::CurrentContext,
        Self::DeclaredStubs,
        Self::Definitions,
        Self::DisabledMessageSelectors,
        Self::DynamicOwnValues,
        Self::EntropySource,
        Self::EvaluationStack,
        Self::ExceptionTypes,
        Self::FileIo,
        Self::GcCandidateGensyms,
        Self::HandledNonLoaderEvalErrors,
        Self::InstalledSystemDownValues,
        Self::InstalledSystemOwnValues,
        Self::LastMessageList,
        Self::LoadStack,
        Self::LoadedPackages,
        Self::MessageDefinitions,
        Self::MessageSymbolCounts,
        Self::Messages,
        Self::ModuleActivations,
        Self::MonitorScopes,
        Self::OnceValues,
        Self::OptionValueScopes,
        Self::Options,
        Self::PackageSearchPaths,
        Self::PendingEvalRequest,
        Self::PendingInputRequest,
        Self::PendingSuspension,
        Self::PrintSink,
        Self::Prints,
        Self::QueuedInputResponses,
        Self::ReapScopes,
        Self::RemovedSymbols,
        Self::RequestInterruptControl,
        Self::ResourceLimits,
        Self::ResumingSuspension,
        Self::SealedUnits,
        Self::StableSymbolCache,
        Self::SymbolCallEffectCache,
        Self::SymbolGenerations,
        Self::SyntaxInformation,
        Self::TraceEventDepths,
        Self::TraceEvents,
    ];
    pub(in crate::runtime) const fn maintained(self) -> bool {
        matches!(
            self,
            Self::EvaluationStack
                | Self::Builtins
                | Self::RequestInterruptControl
                | Self::ResourceLimits
                | Self::Attributes
                | Self::AttributeSetCache
                | Self::BuiltinDispatchSpecCache
                | Self::Definitions
                | Self::InstalledSystemOwnValues
                | Self::InstalledSystemDownValues
                | Self::StableSymbolCache
                | Self::SymbolCallEffectCache
                | Self::CallStabilityCache
                | Self::AssociationLookupCache
                | Self::Options
                | Self::SyntaxInformation
                | Self::DeclaredStubs
                | Self::RemovedSymbols
                | Self::SymbolGenerations
                | Self::OnceValues
                | Self::CurrentContext
                | Self::ContextPath
                | Self::ContextStack
                | Self::PackageSearchPaths
                | Self::LoadedPackages
                | Self::LoadStack
                | Self::FileIo
                | Self::EntropySource
                | Self::PrintSink
                | Self::ExceptionTypes
                | Self::DynamicOwnValues
                | Self::ModuleActivations
                | Self::GcCandidateGensyms
                | Self::OptionValueScopes
                | Self::SealedUnits
                | Self::ReapScopes
                | Self::MonitorScopes
                | Self::DisabledMessageSelectors
                | Self::MessageDefinitions
                | Self::MessageSymbolCounts
                | Self::Messages
                | Self::HandledNonLoaderEvalErrors
                | Self::LastMessageList
                | Self::Prints
                | Self::TraceEvents
                | Self::TraceEventDepths
                | Self::QueuedInputResponses
                | Self::PendingInputRequest
                | Self::PendingEvalRequest
                | Self::PendingSuspension
                | Self::ResumingSuspension
                | Self::AbandonmentWorklist
                | Self::AccountingRoute
        )
    }
    #[cfg(test)]
    pub(in crate::runtime) const fn name(self) -> &'static str {
        match self {
            Self::AbandonmentWorklist => "abandonment_worklist",
            Self::AccountingRoute => "accounting_route",
            Self::AssociationLookupCache => "association_lookup_cache",
            Self::AttributeSetCache => "attribute_set_cache",
            Self::Attributes => "attributes",
            Self::BuiltinDispatchSpecCache => "builtin_dispatch_spec_cache",
            Self::Builtins => "builtins",
            Self::CallStabilityCache => "call_stability_cache",
            Self::ContextPath => "context_path",
            Self::ContextStack => "context_stack",
            Self::CurrentContext => "current_context",
            Self::DeclaredStubs => "declared_stubs",
            Self::Definitions => "definitions",
            Self::DisabledMessageSelectors => "disabled_message_selectors",
            Self::DynamicOwnValues => "dynamic_own_values",
            Self::EntropySource => "entropy_source",
            Self::EvaluationStack => "evaluation_stack",
            Self::ExceptionTypes => "exception_types",
            Self::FileIo => "file_io",
            Self::GcCandidateGensyms => "gc_candidate_gensyms",
            Self::HandledNonLoaderEvalErrors => "handled_non_loader_eval_errors",
            Self::InstalledSystemDownValues => "installed_system_down_values",
            Self::InstalledSystemOwnValues => "installed_system_own_values",
            Self::LastMessageList => "last_message_list",
            Self::LoadStack => "load_stack",
            Self::LoadedPackages => "loaded_packages",
            Self::MessageDefinitions => "message_definitions",
            Self::MessageSymbolCounts => "message_symbol_counts",
            Self::Messages => "messages",
            Self::ModuleActivations => "module_activations",
            Self::MonitorScopes => "monitor_scopes",
            Self::OnceValues => "once_values",
            Self::OptionValueScopes => "option_value_scopes",
            Self::Options => "options",
            Self::PackageSearchPaths => "package_search_paths",
            Self::PendingEvalRequest => "pending_eval_request",
            Self::PendingInputRequest => "pending_input_request",
            Self::PendingSuspension => "pending_suspension",
            Self::PrintSink => "print_sink",
            Self::Prints => "prints",
            Self::QueuedInputResponses => "queued_input_responses",
            Self::ReapScopes => "reap_scopes",
            Self::RemovedSymbols => "removed_symbols",
            Self::RequestInterruptControl => "request_interrupt_control",
            Self::ResourceLimits => "resource_limits",
            Self::ResumingSuspension => "resuming_suspension",
            Self::SealedUnits => "sealed_units",
            Self::StableSymbolCache => "stable_symbol_cache",
            Self::SymbolCallEffectCache => "symbol_call_effect_cache",
            Self::SymbolGenerations => "symbol_generations",
            Self::SyntaxInformation => "syntax_information",
            Self::TraceEventDepths => "trace_event_depths",
            Self::TraceEvents => "trace_events",
        }
    }
}
