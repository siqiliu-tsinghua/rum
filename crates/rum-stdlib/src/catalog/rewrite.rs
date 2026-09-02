use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::rewrite_builtins;

pub(crate) const REWRITE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::special(
        "MatchQ",
        "Tests whether an expression matches a pattern.",
        rewrite_builtins::eval_match_q,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::special(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
        rewrite_builtins::eval_cases,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid traversal argument or Heads option.",
        },
        ARGTU,
        ARGX_OPERATOR,
        INNF,
        LEVEL,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::special(
        "DeleteCases",
        "Deletes subexpressions matching a pattern on levels specified by a standard levelspec.",
        rewrite_builtins::eval_delete_cases,
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid traversal argument or Heads option.",
        },
        ARGTU,
        ARGX_OPERATOR,
        INNF,
        LEVEL,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::special(
        "Replace",
        "Applies rules at the current first-pass Replace levelspec subset.",
        rewrite_builtins::eval_replace,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid rule list, levelspec, or Heads option.",
        },
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::special(
        "ReplaceList",
        "Returns the current first-pass root replacement results across a rule list, with optional max-result limiting.",
        rewrite_builtins::eval_replace_list,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid rule list or maximum-result limit.",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::special(
        "ReplaceAll",
        "Applies rules everywhere in an expression.",
        rewrite_builtins::eval_replace_all,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a rule or list of rules as the second argument",
        },
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::special(
        "ReplaceAllUnheld",
        "Applies rules throughout the parts of an expression that would evaluate.",
        rewrite_builtins::eval_replace_all_unheld,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a rule or list of rules as the second argument",
        },
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::special(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
        rewrite_builtins::eval_replace_repeated,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid rule list or MaxIterations option.",
        },
        ARGX_EXACT,
    ]).protected(),
];
