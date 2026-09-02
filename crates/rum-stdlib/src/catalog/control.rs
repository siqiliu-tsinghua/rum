use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, Capability, HoldAttribute,
};

use super::message_templates::*;

use crate::control_flow;

pub(crate) const CONTROL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Evaluates expressions sequentially and returns the last result.",
        HoldAttribute::HoldAll,
        control_flow::eval_compound_expression,
        MemoryGrowthContract::Delegated,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "If",
        "Evaluates one of several branches conditionally.",
        HoldAttribute::HoldRest,
        control_flow::eval_if,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Which",
        "Evaluates conditions in order and returns the first matching branch.",
        HoldAttribute::HoldAll,
        control_flow::eval_which,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected one or more condition/result pairs",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Switch",
        "Matches one evaluated expression against held form/value pairs.",
        HoldAttribute::HoldRest,
        control_flow::eval_switch,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an expression followed by one or more form/value pairs",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Do",
        "Evaluates an expression repeatedly in the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_do,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGFORM_ITERATOR,
        ARGX_AT_LEAST_TWO,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Product",
        "Multiplies expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        control_flow::eval_product,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGFORM_ITERATOR,
        ARGX_AT_LEAST_TWO,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Sum",
        "Adds expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        control_flow::eval_sum,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGFORM_ITERATOR,
        ARGX_AT_LEAST_TWO,
    ]),
    BuiltinSpec::ordinary_with_hold(
        "While",
        "Evaluates a body while its test yields True.",
        HoldAttribute::HoldAll,
        control_flow::eval_while,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Until",
        "Evaluates a body until its test yields True.",
        HoldAttribute::HoldAll,
        control_flow::eval_until,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "For",
        "Evaluates start, test, increment, and body in the current first-pass procedural subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_for,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Table",
        "Generates lists using the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_table,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGFORM_ITERATOR,
        ARGX_AT_LEAST_TWO,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Abort",
        "Generates an abort that terminates the current evaluation unless caught.",
        control_flow::eval_abort,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Exit",
        "Requests host-level termination with an optional integer exit code.",
        control_flow::eval_exit,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Exit called with `1` arguments; 0 or 1 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "intm",
            template: "Machine-sized integer expected at position 1 in `1`.",
        },
    ]).protected()
    .with_capability(Capability::Process),
    BuiltinSpec::ordinary_with_hold(
        "CheckAbort",
        "Evaluates an expression and substitutes a fallback if an Abort occurs.",
        HoldAttribute::HoldAll,
        control_flow::eval_check_abort,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argr",
            template: "expected at least 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argrx",
            template: "expected at least 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position 2 in `2`. An option must be a rule or a list of rules.",
        },
        OPTRS,
        BuiltinMessageSpec {
            tag: "opttfa",
            template: "The value of the option PropagateAborts -> `1` should be True, False or Automatic.",
        },
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "AbortProtect",
        "Evaluates an expression while delaying any abort until the protected region exits in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_abort_protect,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "WithCleanup",
        "Evaluates cleanup code even when the main expression exits abnormally in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_with_cleanup,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Failsafe",
        "Creates a callable wrapper that preserves failure-like inputs in the current first-pass subset.",
        control_flow::eval_failsafe,
        MemoryGrowthContract::Delegated,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Assert",
        "Represents an assertion site and supports the current session flag plus $AssertFunction hook.",
        HoldAttribute::HoldAll,
        control_flow::eval_assert,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "asrte",
            template: "Assertion `1` had no test.",
        },
        BuiltinMessageSpec {
            tag: "asrtf",
            template: "Assertion `1` failed.",
        },
        BuiltinMessageSpec {
            tag: "asrtfe",
            template: "Assertion `1` in `2` failed.",
        },
        BuiltinMessageSpec {
            tag: "asrttf",
            template: "Assertion test `1` evaluated to `2` that is neither True nor False.",
        },
    ]).protected(),
    BuiltinSpec::new(
        "$AssertFunction",
        "Stores the optional assertion failure callback used by Assert in the current first-pass subset.",
        MemoryGrowthContract::NonAmplifying,
    ),
    BuiltinSpec::new(
        "PropagateAborts",
        "Represents the current first-pass CheckAbort option for rethrowing caught aborts.",
        MemoryGrowthContract::Delegated,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Enclose",
        "Evaluates an expression and catches confirmation failures in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_enclose,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Confirm",
        "Confirms that an expression does not evaluate to a failure-like result in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmBy",
        "Confirms that a test function returns True for an expression in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_by,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmMatch",
        "Confirms that an expression matches a held pattern in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_match,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmQuiet",
        "Confirms that an evaluation emits no matching messages in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_quiet,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid message-selector specification.",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmAssert",
        "Confirms that a test evaluates to True in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_assert,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Return",
        "Exits the current function-style evaluation with an optional result.",
        control_flow::eval_return,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Break",
        "Exits the nearest enclosing loop.",
        control_flow::eval_break,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "nofwd",
            template: "No enclosing For, While, Until or Do found for Break[].",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Continue",
        "Skips to the next iteration of the nearest enclosing loop.",
        control_flow::eval_continue,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "nofwd",
            template: "No enclosing For, While, Until or Do found for Continue[].",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Throw",
        "Propagates a nonlocal value, optionally tagged for Catch matching.",
        control_flow::eval_throw,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "nocatch",
            template: "Uncaught `1` returned to top level.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Catch",
        "Evaluates an expression and intercepts matching Throw values.",
        HoldAttribute::HoldAll,
        control_flow::eval_catch,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::new(
        "Exception",
        "Represents a typed exception with an Association payload.",
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "RegisterExceptionType",
        "Registers a string exception type with an optional parent type.",
        control_flow::eval_register_exception_type,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "argform",
            template: "exception type and parent must be strings",
        },
        BuiltinMessageSpec {
            tag: "reregister",
            template: "The exception type is already registered with a different parent.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ExceptionTypes",
        "Returns registered exception types as sorted type-to-parent rules.",
        control_flow::eval_exception_types,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ThrowException",
        "Raises a registered typed exception with an optional Association payload.",
        control_flow::eval_throw_exception,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "argform",
            template: "exception type must be a string and payload must be an Association",
        },
        BuiltinMessageSpec {
            tag: "unreg",
            template: "The exception type is not registered.",
        },
        BuiltinMessageSpec {
            tag: "nocatch",
            template: "Uncaught exception returned to top level.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "CatchExceptions",
        "Catches typed exceptions and dispatches ordered ancestry-aware handler rules.",
        HoldAttribute::HoldAll,
        control_flow::eval_catch_exceptions,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "argform",
            template: "handlers must be a List of rules",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Failure",
        "Represents a failure object with associated details in the current first-pass subset.",
        control_flow::eval_failure,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "FailureQ",
        "Tests whether an expression is a Failure object.",
        control_flow::eval_failure_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Success",
        "Represents a success object with associated details in the current first-pass subset.",
        control_flow::eval_success,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
    BuiltinSpec::ordinary(
        "TerminatedEvaluation",
        "Represents a terminated evaluation object in the current first-pass subset.",
        control_flow::eval_terminated_evaluation,
        MemoryGrowthContract::NonAmplifying,
    ).protected(),
];
