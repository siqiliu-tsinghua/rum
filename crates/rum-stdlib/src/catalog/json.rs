use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use crate::json;

pub(crate) const JSON_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromJson",
        "Parses an RFC 8259 JSON text string into exact rum values, failing closed on invalid JSON.",
        AttributeSet::empty().with(Attribute::Protected),
        json::eval_from_json,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "depth",
            template: "JSON nesting exceeds the fixed depth limit of 256 at byte offset `1`.",
        },
        BuiltinMessageSpec {
            tag: "json",
            template: "Reports an invalid JSON input or parse failure.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToJson",
        "Renders the JSON-representable subset of rum values as a JSON text string.",
        AttributeSet::empty().with(Attribute::Protected),
        json::eval_to_json,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "json",
            template: "Reports an invalid JSON encoding argument or value.",
        },
    ]),
];
