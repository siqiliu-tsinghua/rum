//! Pure `rum` kernel.
//!
//! This crate owns lowering, evaluation sessions, definitions, scoping,
//! messages, matching, tracing, resource limits, capability policy enforcement,
//! and the host-facing request APIs used by embedding layers.

mod runtime;

pub use runtime::*;

#[cfg(any(test, feature = "test-continuation-slots"))]
#[doc(hidden)]
pub fn test_logical_profile_snapshot() -> LogicalProfileSnapshot {
    runtime::logical_profile_snapshot()
}
#[cfg(any(test, feature = "test-continuation-slots"))]
#[doc(hidden)]
pub fn test_logical_profile_reset() {
    runtime::logical_profile_reset();
}
