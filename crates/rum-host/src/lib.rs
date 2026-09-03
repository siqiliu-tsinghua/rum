//! Narrow Rust embedding facade for `rum`.
//!
//! `rum-host` wraps the broader `rum-runtime` request API in transport-friendly
//! host types. Rust hosts should use this crate for session construction,
//! evaluation, structured outcomes, resumable input, cancellation, timeouts,
//! reset, snapshot/restore, downstream builtin-family installation, and
//! capability policy configuration instead of depending directly on evaluator
//! internals.
//!
//! # Quick start
//!
//! ```
//! use rum_host::{HostEvalOutcome, HostSession};
//!
//! let mut session = HostSession::new();
//!
//! match session.eval("rule[x_Integer] := x + 1; rule[41]") {
//!     HostEvalOutcome::Success(success) => {
//!         assert_eq!(success.value.text, "42");
//!     }
//!     HostEvalOutcome::InputRequired(suspension) => {
//!         panic!("unexpected input request: {}", suspension.request.prompt);
//!     }
//!     HostEvalOutcome::Failure(failure) => {
//!         panic!("evaluation failed: {}", failure.message);
//!     }
//! }
//! ```
//!
//! # Running untrusted code
//!
//! Use [`HostConfig::sandboxed`] when evaluating untrusted source. It installs a
//! deterministic `{Pure}` capability policy that blocks clock, file, random,
//! and diagnostic builtins at dispatch, including runtime-constructed calls,
//! and omits the default file I/O and entropy providers.
//! Fine-grained policies are expressible through [`CapabilityPolicy`] and
//! [`Capability`]; role and identity mapping is the caller's responsibility.
//!
//! See `docs/guide/embedding.md` for the embedding narrative and `DISCLAIMER.md`
//! for the project boundary.

#![warn(missing_docs)]

mod config;
mod cursor;
mod entropy;
mod file_io;
mod model;
mod session;

pub use config::{BuiltinInstaller, HostConfig, HostResourceLimits};
pub use entropy::StdEntropySource;
pub use file_io::StdFileIo;
pub use model::{
    HostCompletenessStatus, HostCompletionRequest, HostCompletionResponse, HostDefinition,
    HostDefinitionSummary, HostEvalOutcome, HostEvalRequest, HostEvalSuccess, HostFailure,
    HostFailureCode, HostInputKind, HostInputRequest, HostInputSuspension, HostInspectRequest,
    HostInspectResponse, HostIsCompleteRequest, HostIsCompleteResponse, HostMessage,
    HostMessageTemplate, HostParseRequest, HostParseResponse, HostStructuredOutput,
    HostStructuredOutputKind, HostSymbolInfo, HostTerminationReason, HostTraceEvent, HostTyped,
    HostValue,
};
/// Capability label used by host capability policies.
pub use rum_runtime::Capability;
/// Per-session policy that authorizes builtin capability groups.
pub use rum_runtime::CapabilityPolicy;
/// Builder for constructing a [`CapabilityPolicy`].
pub use rum_runtime::CapabilityPolicyBuilder;
pub use rum_runtime::{
    DEFAULT_NATIVE_STACK_LIMIT, MIN_SUPPORTED_NATIVE_STACK_BYTES, NATIVE_STACK_DEPTH_POLL_STRIDE,
    NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES, NATIVE_STACK_POLL_HEADROOM_BYTES,
    NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES, native_stack_limit_for_declared_stack,
};
pub use session::{HostCancelHandle, HostSession, HostSessionBuilder};

#[cfg(test)]
mod lifecycle_contract_tests;

#[cfg(test)]
mod tests;
