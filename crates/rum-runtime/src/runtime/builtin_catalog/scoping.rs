use super::message_templates::*;
use super::*;

pub(super) const SCOPING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "With",
        "Lexically substitutes local constants into a body, with each additional declaration list forming a nested stage.",
        HoldAttribute::HoldAll,
        builtin_with,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a list of symbol assignments or rules as the first argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::special_with_hold(
        "Module",
        "Creates lexical local symbols.",
        HoldAttribute::HoldAll,
        builtin_module,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a list of symbols or symbol initializers as the first argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::special_with_attributes(
        "DefinePackage",
        "Registers a held source unit with public, package-scope, and identity-sealed private symbols.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_define_package,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a non-empty valid context string with a trailing context separator as the first argument",
        },
        BuiltinMessageSpec {
            tag: "decl",
            template: "expected at most one Public and PackageScope declaration containing only current-context symbols",
        },
        BuiltinMessageSpec {
            tag: "failed",
            template: "package source-unit registration failed during body evaluation",
        },
    ])
    .protected(),
    BuiltinSpec::special_with_attributes(
        "RestoreSealedReferences",
        "Re-links sealed symbol references while evaluating trusted snapshot restore forms.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_restore_sealed_references,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a unit context or list of unit contexts and HoldComplete[form]",
        },
        BuiltinMessageSpec {
            tag: "mode",
            template: "only valid inside a snapshot restore",
        },
        BuiltinMessageSpec {
            tag: "unit",
            template: "snapshot restore could not find a named current sealed unit",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Public",
        "Declares public symbols inside a held DefinePackage source unit.",
        HoldAttribute::HoldAll,
        builtin_public,
        MemoryGrowthContract::Incremental,
    )
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "PackageScope",
        "Declares conventionally package-scoped symbols inside a held DefinePackage source unit.",
        HoldAttribute::HoldAll,
        builtin_package_scope,
        MemoryGrowthContract::Incremental,
    )
    .protected(),
    BuiltinSpec::ordinary(
        "Unique",
        "Creates fresh symbols using prefix, list, and attribute forms.",
        builtin_unique,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an attribute symbol or list of attribute symbols as the second argument",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "usym",
            template: "`1` is not a symbol or a valid symbol name.",
        },
    ])
    .protected(),
    BuiltinSpec::special_with_hold(
        "Block",
        "Temporarily rebinds symbols dynamically.",
        HoldAttribute::HoldAll,
        builtin_block,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a list of symbols or symbol initializers as the first argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Hold",
        "Prevents evaluation of its arguments.",
        HoldAttribute::HoldAll,
        builtin_hold_literal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "HoldFirst",
        "Prevents evaluation of the first argument.",
        HoldAttribute::HoldFirst,
        builtin_hold_literal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "HoldRest",
        "Prevents evaluation of all but the first argument.",
        HoldAttribute::HoldRest,
        builtin_hold_literal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "HoldAll",
        "Prevents evaluation of all arguments.",
        HoldAttribute::HoldAll,
        builtin_hold_literal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "HoldAllComplete",
        "Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_hold_literal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "HoldComplete",
        "Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_hold_literal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Unevaluated",
        "Temporarily leaves an argument unevaluated as a direct function argument.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_unevaluated,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Evaluate",
        "Forces evaluation in held contexts.",
        HoldAttribute::HoldAll,
        builtin_evaluate,
        MemoryGrowthContract::Delegated,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "HoldPattern",
        "Marks a pattern expression so it is matched without prior evaluation.",
        HoldAttribute::HoldAll,
        builtin_hold_literal,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ReleaseHold",
        "Removes one outer layer of Hold, HoldForm, HoldComplete, HoldPattern, HoldFirst, HoldRest, HoldAll, or HoldAllComplete, then resumes evaluation.",
        HoldAttribute::HoldAll,
        builtin_release_hold,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Sequence",
        "Splices a sequence of arguments into a call.",
        builtin_sequence,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "Thread",
        "Threads an expression over arguments that share a common head.",
        Session::eval_thread,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the third argument to be a standard sequence specification such as All, None, n, -n, {n}, {m, n}, or {m, n, s}",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 1, 2, or 3 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "tdlen",
            template: "Reports incompatible argument lengths while threading an expression.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Dialog",
        "Runs the current first-pass dialog subset with Return-sensitive evaluation and host-input suspension; requires Capability::Interactive.",
        HoldAttribute::HoldAll,
        builtin_dialog,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports invalid Dialog option rules or DialogSymbols bindings.",
        },
    ])
    .protected()
    .with_capability(Capability::Interactive),
    BuiltinSpec::ordinary(
        "InputString",
        "Requests one line of host-provided text and returns it as a string; requires Capability::Interactive.",
        builtin_input_string,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position 1 in `2`. An option must be a rule or a list of rules.",
        },
        OPTRS,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected()
    .with_capability(Capability::Interactive),
    BuiltinSpec::ordinary(
        "Input",
        "Requests one line of host-provided text, parses it as rum source, and evaluates the resulting expression in the current first-pass subset; requires Capability::Interactive.",
        builtin_input,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position 1 in `2`. An option must be a rule or a list of rules.",
        },
        OPTRS,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected()
    .with_capability(Capability::Interactive),
    BuiltinSpec::ordinary(
        "Print",
        "Emits a line of plain print output through the runtime print stream.",
        builtin_print,
        MemoryGrowthContract::Incremental,
    ).protected(),
    BuiltinSpec::ordinary(
        "Pause",
        "Pauses for at least a nonnegative exact number of seconds in the current first-pass subset.",
        builtin_pause,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "numnm",
            template: "Non-negative machine-sized number expected at position 1 in `1`.",
        },
    ])
    .protected()
    .with_capability(Capability::Clock),
];
