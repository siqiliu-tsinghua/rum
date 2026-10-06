// Contract: docs/guide/evaluation.md suspension and the Sow/Reap builtin reference.
// Compare rum queued input with actual suspension, including effects and cleanup.
#[allow(dead_code)]
#[path = "suspension_oracle/support/mod.rs"]
mod suspension_support;

#[path = "sow_trace_continuations/lifecycle.rs"]
mod lifecycle;
#[path = "sow_trace_continuations/sow.rs"]
mod sow;
#[path = "sow_trace_continuations/support.rs"]
mod support;
#[path = "sow_trace_continuations/trace.rs"]
mod trace;
