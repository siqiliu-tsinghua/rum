// Delayed option reads must finish the builtin on queued and resumed paths.
// Contract: docs/guide options and rum-runtime Session input lifecycle rustdoc.
#[allow(dead_code)]
#[path = "suspension_oracle/support/mod.rs"]
mod suspension_support;

mod option_reader_continuations {
    mod boolean;
    mod lifecycle;
    mod phases;
    mod rule;
    mod support;
}
