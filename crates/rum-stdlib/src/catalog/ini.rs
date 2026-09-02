use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use crate::ini;

pub(crate) const INI_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "FromIni",
        "Parses rum's fail-closed INI dialect into an Association of section Associations.",
        AttributeSet::empty().with(Attribute::Protected),
        ini::eval_from_ini,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[BuiltinMessageSpec {
        tag: "ini",
        template: "Reports an INI decoding or argument failure.",
    }]),
    BuiltinSpec::ordinary_with_attributes(
        "ToIni",
        "Renders an Association of section Associations as rum's INI dialect.",
        AttributeSet::empty().with(Attribute::Protected),
        ini::eval_to_ini,
        MemoryGrowthContract::Preflight,
    )
    .with_messages(&[BuiltinMessageSpec {
        tag: "ini",
        template: "Reports an INI encoding or argument failure.",
    }]),
];
