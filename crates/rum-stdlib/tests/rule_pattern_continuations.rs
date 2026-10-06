// Rule preparation: queued answers and actual suspension must agree.
#[allow(dead_code)]
#[path = "suspension_oracle/support/mod.rs"]
mod suspension_support;

mod rule_pattern_continuations {
    mod lifecycle;
    mod matcher;
    mod policy;
    mod rule;
    mod support;
}
