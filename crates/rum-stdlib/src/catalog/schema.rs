use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{BuiltinMessageSpec, BuiltinSpec, HoldAttribute};

use super::message_templates::ARGX_OPERATOR;

use crate::schema;

pub(crate) const SCHEMA_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::special_with_hold(
        "ConformSchema",
        "Validates a value against a symbolic schema and returns the normalized value with defaults filled.",
        HoldAttribute::HoldRest,
        schema::eval_conform_schema,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "schema",
            template: "Reports a malformed schema or conformance failure.",
        },
        ARGX_OPERATOR,
    ])
    .protected(),
    BuiltinSpec::special_with_hold(
        "SchemaErrors",
        "Returns path-keyed validation errors for a value checked against a symbolic schema.",
        HoldAttribute::HoldRest,
        schema::eval_schema_errors,
        MemoryGrowthContract::Incremental,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "schema",
            template: "Reports a malformed schema or conformance failure.",
        },
        ARGX_OPERATOR,
    ])
    .protected(),
];
