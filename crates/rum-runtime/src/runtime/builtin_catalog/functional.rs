use super::*;

pub(super) const FUNCTIONAL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "Function",
        "Creates a pure function.",
        HoldAttribute::HoldAll,
        builtin_hold_literal,
        MemoryGrowthContract::Delegated,
    )
    .with_messages(&[BuiltinMessageSpec {
        tag: "argx",
        template: "expected at least `1` argument(s), got `2`",
    }])
    .protected(),
    BuiltinSpec::with_attributes(
        "HoldForm",
        "Holds an expression while rendering transparently in terminal results and output-like forms.",
        AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected),
        MemoryGrowthContract::NonAmplifying,
    ),
];
