mod inspection;
mod outcome;
mod requests;

pub use inspection::{
    HostDefinition, HostDefinitionSummary, HostInspectResponse, HostMessageTemplate, HostSymbolInfo,
};
pub use outcome::{
    HostEvalOutcome, HostEvalSuccess, HostFailure, HostFailureCode, HostInputKind,
    HostInputRequest, HostInputSuspension, HostMessage, HostStructuredOutput,
    HostStructuredOutputKind, HostTerminationReason, HostTraceEvent, HostTyped, HostValue,
};
pub use requests::{
    HostCompletenessStatus, HostCompletionRequest, HostCompletionResponse, HostEvalRequest,
    HostInspectRequest, HostIsCompleteRequest, HostIsCompleteResponse, HostParseRequest,
    HostParseResponse,
};

pub(crate) use inspection::inspect_response_from_runtime;
pub(crate) use outcome::{
    eval_failure_from_runtime, eval_success_from_runtime, host_failure_from_runtime,
    synthesized_host_failure,
};
pub(crate) use requests::completeness_status;

pub(crate) use outcome::cm740_output_scope_cm740_driven as cm740_output_scope;
pub(crate) use outcome::eval_success_from_runtime_cm740_driven;
