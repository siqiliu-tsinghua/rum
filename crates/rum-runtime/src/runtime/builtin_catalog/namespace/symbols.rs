use super::*;

pub(in super::super) const NAMESPACE_SYMBOL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "Message",
        "Emits a message described by a MessageName expression.",
        HoldAttribute::HoldFirst,
        builtin_message,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the first argument to be a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected at least 1 argument",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "MessageName",
        "Represents a symbolic message name such as f::tag.",
        HoldAttribute::HoldAll,
        builtin_message_name,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string, plus a message tag string or symbol",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Messages",
        "Lists message names defined for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        builtin_messages,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "MessageDefinitions",
        "Lists stored message template definitions for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        builtin_message_definitions,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "MessageTags",
        "Lists message tags defined globally, for a symbol, or for one message query.",
        HoldAttribute::HoldAll,
        builtin_message_tags,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 0 or 1 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Information",
        "Returns a first-pass InformationData summary for a symbol or symbol-name string.",
        HoldAttribute::HoldFirst,
        Session::eval_information,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Definition",
        "Returns a held first-pass expression containing visible definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_definition,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ValueQ",
        "Tests whether an expression has a value by trial evaluation without committing side effects.",
        HoldAttribute::HoldFirst,
        builtin_value_q,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "DefaultValues",
        "Lists stored Default[...] definitions for a symbol in the current optional-pattern subset.",
        HoldAttribute::HoldFirst,
        builtin_default_values,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "OwnValues",
        "Lists stored immediate or delayed own-value definitions for a symbol, reconstructing right-hand-side guards.",
        HoldAttribute::HoldFirst,
        builtin_own_values,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid OwnValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "UpValues",
        "Lists stored tagged upvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_up_values,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid UpValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "DownValues",
        "Lists stored downvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_down_values,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid DownValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SubValues",
        "Lists stored subvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_sub_values,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "setval",
            template: "Reports an invalid SubValues assignment.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Attributes",
        "Lists the currently active attributes for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_attributes,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SyntaxInformation",
        "Lists the current syntax-information metadata rules for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_syntax_information,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Options",
        "Lists the current default option rules for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_options,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string argument",
        },
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "OptionValue",
        "Looks up an option value from explicit rules or a symbol's defaults.",
        HoldAttribute::HoldAll,
        builtin_option_value,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid option target, option name, or explicit option rules.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "nodef",
            template: "Unknown option `1` for `2`.",
        },
        BuiltinMessageSpec {
            tag: "optnf",
            template: "Option name `1` not found in defaults for `2`.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ArgumentsOptions",
        "Partitions a held call into positional arguments and trailing option rules.",
        HoldAttribute::HoldFirst,
        builtin_arguments_options,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid held call, argument-count specification, or result association.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "CheckArguments",
        "Checks whether a held call has an allowed positional argument count and valid trailing options.",
        HoldAttribute::HoldFirst,
        builtin_check_arguments,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid held call, argument-count specification, or result association.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SetAttributes",
        "Adds attributes to one or more symbols.",
        HoldAttribute::HoldFirst,
        builtin_set_attributes,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid target or attribute specification.",
        },
        ARGX_EXACT,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ClearAttributes",
        "Removes attributes from one or more symbols.",
        HoldAttribute::HoldFirst,
        builtin_clear_attributes,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid target or attribute specification.",
        },
        ARGX_EXACT,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SetOptions",
        "Updates known default options for a symbol.",
        HoldAttribute::HoldFirst,
        builtin_set_options,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid target or option-rule specification.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "optnf",
            template: "`1` is not a known option for `2`.",
        },
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Protect",
        "Adds the Protected attribute to one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_protect,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Unprotect",
        "Removes the Protected attribute from one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_unprotect,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Unset",
        "Removes a specific stored definition or message template.",
        HoldAttribute::HoldFirst,
        builtin_unset,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol, definition lhs, message name, or tagged definition target",
        },
        ARGX_EXACT,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Clear",
        "Clears values and rules from one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_clear,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ClearAll",
        "Clears values, rules, messages, and user-defined attributes from one or more symbols.",
        HoldAttribute::HoldAll,
        builtin_clear_all,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Remove",
        "Removes the current stored state for one or more symbols in the first-pass namespace subset.",
        HoldAttribute::HoldAll,
        builtin_remove,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more symbols, symbol-name strings, or lists of them",
        },
        BuiltinMessageSpec {
            tag: "rmptc",
            template: "Symbol `1` is Protected and cannot be removed.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Check",
        "Evaluates an expression and substitutes a fallback if matching messages are emitted.",
        HoldAttribute::HoldAll,
        builtin_check,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the optional message selector to be a symbol, a \"symbol::tag\" string, or a list of those",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 2 or 3 argument(s), got `1`",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Quiet",
        "Evaluates an expression while suppressing matching runtime messages it emits.",
        HoldAttribute::HoldAll,
        builtin_quiet,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected the optional message selector to be a symbol, a \"symbol::tag\" string, or a list of those",
        },
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 1 or 2 argument(s), got `1`",
        },
    ])
    .protected(),
];
