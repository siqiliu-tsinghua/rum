// Delayed option reads must finish the builtin on queued and resumed paths.
// Contract: docs/guide options and rum-runtime Session input lifecycle rustdoc.
mod option_reader_continuations {
    mod boolean;
    mod lifecycle;
    mod support;
}
