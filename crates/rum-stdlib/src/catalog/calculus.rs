use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{BuiltinMessageSpec, BuiltinSpec, HoldAttribute};

use super::message_templates::*;

use crate::calculus;

pub(crate) const CALCULUS_BUILTINS: &[BuiltinSpec] = &[BuiltinSpec::ordinary_with_hold(
    "D",
    "Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.",
    HoldAttribute::HoldRest,
    calculus::eval_d,
    MemoryGrowthContract::Incremental,
)
    .with_messages(&[
        ARGX_AT_LEAST_TWO,
        BuiltinMessageSpec {
            tag: "dvar",
            template: "`1` is not a valid variable.",
        },
        BuiltinMessageSpec {
            tag: "dvord",
            template: "Derivative order `1` is not a non-negative integer.",
        },
    ]).protected()];
