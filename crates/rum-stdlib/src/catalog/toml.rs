use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use crate::toml;

pub(crate) const TOML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromToml",
        "Parses rum's documented fail-closed TOML subset into exact rum values.",
        AttributeSet::empty().with(Attribute::Protected),
        toml::eval_from_toml,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "depth",
            template: "TOML nesting exceeds the fixed depth limit of 256 at byte offset `1`.",
        },
        BuiltinMessageSpec {
            tag: "toml",
            template: "Reports a TOML decoding or argument failure.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToToml",
        "Renders the TOML-representable subset of rum Associations as TOML text.",
        AttributeSet::empty().with(Attribute::Protected),
        toml::eval_to_toml,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "depth",
            template: "TOML nesting exceeds the fixed depth limit of 256.",
        },
        BuiltinMessageSpec {
            tag: "toml",
            template: "Reports a TOML encoding or argument failure.",
        },
    ]),
];
