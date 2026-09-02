use super::message_templates::*;
use super::*;

pub(super) const DEBUG_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "Trace",
        "Evaluates an expression and returns a first-pass structured trace of current evaluation events.",
        HoldAttribute::HoldAll,
        builtin_trace,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid level specification or Heads option for Trace.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "TraceDetail",
        "Evaluates an expression and returns a lossless structured trace whose event associations preserve rule and definition details.",
        HoldAttribute::HoldAll,
        builtin_trace_detail,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid level specification or Heads option for TraceDetail.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "TracePrint",
        "Evaluates an expression, emits its current first-pass structured trace through the print stream, and returns the result.",
        HoldAttribute::HoldAll,
        builtin_trace_print,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid level specification or Heads option for TracePrint.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Timing",
        "Evaluates an expression and returns {elapsed, value}, with elapsed exact rational seconds measured by a monotonic clock.",
        HoldAttribute::HoldAll,
        builtin_timing,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary(
        "AbsoluteTime",
        "Returns the current exact rational Unix timestamp (seconds since 1970-01-01T00:00:00Z); requires Capability::Clock.",
        Session::eval_absolute_time,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_hold(
        "AbsoluteTiming",
        "Evaluates an expression and returns {elapsed, value}, with elapsed exact rational seconds measured by a monotonic clock.",
        HoldAttribute::HoldAll,
        builtin_absolute_timing,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_attributes(
        "RepeatedTiming",
        "Evaluates an expression over a fixed sample set and returns {elapsed, value}, using the minimum monotonic elapsed time.",
        AttributeSet::empty()
            .with(Attribute::HoldFirst)
            .with(Attribute::SequenceHold),
        builtin_repeated_timing,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary(
        "Short",
        "Returns the current canonical-text shortened display subset for an expression.",
        Session::eval_short,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the optional second argument to be a positive integer",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Echo",
        "Prints a value through the runtime print stream and returns it.",
        Session::eval_echo,
        MemoryGrowthContract::Incremental,
    ).protected(),
    BuiltinSpec::ordinary(
        "EchoFunction",
        "Returns a pure function that echoes its argument through the runtime print stream.",
        Session::eval_echo_function,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "EchoLabel",
        "Returns a pure function that echoes its argument with a label through the runtime print stream.",
        Session::eval_echo_label,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "ByteCount",
        "Returns the current canonical-text byte count for an expression in the first-pass resource subset.",
        builtin_byte_count,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "On",
        "Enables session-level assertion checking or emission of matching messages in the current selector subset.",
        HoldAttribute::HoldAll,
        builtin_on,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, MessageName expressions, \"symbol::tag\" strings, or lists of those",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected at least 1 argument",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Off",
        "Disables session-level assertion checking or emission of matching messages in the current selector subset.",
        HoldAttribute::HoldAll,
        builtin_off,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, MessageName expressions, \"symbol::tag\" strings, or lists of those",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected at least 1 argument",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Sow",
        "Collects a value for enclosing Reap in the current first-pass tag-and-group subset.",
        Session::eval_sow,
        MemoryGrowthContract::Incremental,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Reap",
        "Evaluates an expression and collects values emitted by Sow in the current first-pass grouping subset.",
        HoldAttribute::HoldFirst,
        builtin_reap,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "TimeConstrained",
        "Evaluates an expression under the current cooperative exact-numeric timeout subset.",
        HoldAttribute::HoldAll,
        builtin_time_constrained,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the timeout argument to be a non-negative exact number in the current first-pass subset",
        },
        ARGTU,
    ])
    .protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_hold(
        "MemoryConstrained",
        "Evaluates an expression under the current cooperative memory-proxy limit subset.",
        HoldAttribute::HoldAll,
        builtin_memory_constrained,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the memory limit argument to be a non-negative exact integer",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "MemoryInUse",
        "Returns the current session memory-proxy byte count subset.",
        builtin_memory_in_use,
        MemoryGrowthContract::NonAmplifying,
    ).protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary(
        "MaxMemoryUsed",
        "Returns the peak session memory-proxy byte count high-water mark subset.",
        builtin_max_memory_used,
        MemoryGrowthContract::NonAmplifying,
    ).protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary_with_hold(
        "Monitor",
        "Evaluates an expression while exposing the current first-pass monitor display and callback subset.",
        HoldAttribute::HoldAll,
        builtin_monitor,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid monitor option rule.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "EvaluationData",
        "Evaluates an expression and returns a first-pass association of result and diagnostics.",
        HoldAttribute::HoldFirst,
        builtin_evaluation_data,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols",
        },
        ARGTU,
    ])
    .protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary(
        "RumProfileReset",
        "Internal diagnostic hook: resets rum runtime profile counters when profiling is enabled.",
        builtin_rum_profile_reset,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary(
        "RumProfileSnapshot",
        "Internal diagnostic hook: prints rum runtime profile counters when profiling is enabled.",
        builtin_rum_profile_snapshot,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ])
    .protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary_with_hold(
        "TestCreate",
        "Builds a first-pass symbolic test specification.",
        HoldAttribute::HoldAll,
        Session::eval_test_create,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "TestReport",
        "Runs one or more first-pass symbolic tests and returns an association summary.",
        Session::eval_test_report,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the optional property argument to be All, Properties, a property string or symbol, or a list of property strings or symbols",
        },
    ])
    .protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_hold(
        "Stack",
        "Returns the current evaluation stack in the first-pass held-expression subset.",
        HoldAttribute::HoldAll,
        builtin_stack,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected(),
];
