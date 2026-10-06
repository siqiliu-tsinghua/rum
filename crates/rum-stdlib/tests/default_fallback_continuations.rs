// Contract: patterns-and-rules.md, right-hand-side guards and direct Default
// fallback; evaluation.md suspension and the Session abandonment lifecycle.
#[allow(dead_code)]
#[path = "rule_pattern_continuations/support.rs"]
mod oracle;
mod default_fallback_continuations {
    mod cases;
    mod lifecycle;
    mod support;
}
