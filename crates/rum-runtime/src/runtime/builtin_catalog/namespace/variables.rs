use super::*;

pub(in super::super) const NAMESPACE_VARIABLE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "$Context",
        "Evaluates to the current default context name as a string.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "$ContextPath",
        "Evaluates to the active context search path as a list of strings.",
        MemoryGrowthContract::Incremental,
    ),
    BuiltinSpec::new(
        "$Aborted",
        "Represents a computation aborted by Abort or an external interrupt.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new(
        "$Canceled",
        "Represents a canceled operation for confirmation-flow compatibility.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new("$Failed", "Represents a failed operation.", MemoryGrowthContract::NonAmplifying).protected(),
    BuiltinSpec::new(
        "$MessageList",
        "Lists the message names emitted by the most recent top-level evaluation.",
        MemoryGrowthContract::Incremental,
    ).protected(),
    BuiltinSpec::new(
        "$Packages",
        "Lists package contexts loaded into the current session.",
        MemoryGrowthContract::Incremental,
    ),
    BuiltinSpec::new(
        "$InputFileName",
        "Evaluates to the path of the file currently being loaded, or the empty string at top level.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::new("$Path", "Lists search roots used by the package loader.", MemoryGrowthContract::Incremental),
    BuiltinSpec::new(
        "Nothing",
        "Represents a list element that is removed from evaluated Lists.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
];
