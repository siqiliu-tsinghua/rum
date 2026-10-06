// Contract: docs/guide/evaluation.md assignment order and suspension;
// Set::shape and Session's cancellation/abandonment lifecycle.
#[allow(dead_code)]
#[path = "suspension_oracle/support/mod.rs"]
mod suspension_support;

#[allow(dead_code)]
#[path = "rule_pattern_continuations/support.rs"]
mod oracle;
mod destructuring_continuations {
    mod lifecycle;
    mod stores;
    mod support;
}
