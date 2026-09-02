use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use crate::yaml;

pub(crate) const YAML_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromYaml",
        "Parses rum's documented fail-closed YAML block subset into exact rum values.",
        AttributeSet::empty().with(Attribute::Protected),
        yaml::eval_from_yaml,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "depth",
            template: "YAML nesting exceeds the fixed depth limit of 256 at byte offset `1`.",
        },
        BuiltinMessageSpec {
            tag: "yaml",
            template: "Reports a YAML decoding or argument failure.",
        },
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToYaml",
        "Renders the YAML-representable subset of rum values as YAML text.",
        AttributeSet::empty().with(Attribute::Protected),
        yaml::eval_to_yaml,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[BuiltinMessageSpec {
        tag: "yaml",
        template: "Reports a YAML encoding or argument failure.",
    }]),
];
