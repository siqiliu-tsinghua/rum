use super::*;

pub(super) const INERT_HEAD_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "General",
        "Carries general runtime message templates.",
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "cancel",
            template: "Reports evaluation cancellation requested by the host.",
        },
        BuiltinMessageSpec {
            tag: "iterlim",
            template: "Reports that ReplaceRepeated exceeded its iteration limit.",
        },
        BuiltinMessageSpec {
            tag: "matchreclim",
            template: "Reports that evaluation exceeded the pattern-match recursion limit.",
        },
        BuiltinMessageSpec {
            tag: "memlim",
            template: "Reports that evaluation exceeded a MemoryConstrained limit.",
        },
        BuiltinMessageSpec {
            tag: "noentropy",
            template: "no entropy provider installed",
        },
        BuiltinMessageSpec {
            tag: "noio",
            template: "no file provider installed",
        },
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
        BuiltinMessageSpec {
            tag: "passlim",
            template: "Reports an evaluation cycle or fixed-point pass-limit termination.",
        },
        BuiltinMessageSpec {
            tag: "optreentry",
            template: "The option reentry type is not registered.",
        },
        BuiltinMessageSpec {
            tag: "reclim",
            template: "Reports that evaluation exceeded the recursion limit.",
        },
        BuiltinMessageSpec {
            tag: "reslim",
            template: "Reports a native-stack or configured resource work-limit termination.",
        },
        BuiltinMessageSpec {
            tag: "sandbox",
            template: "`1` is not permitted in this sandbox.",
        },
        BuiltinMessageSpec {
            tag: "timelim",
            template: "Reports that evaluation exceeded a TimeConstrained limit.",
        },
    ])
    .protected(),
    BuiltinSpec::new(
        "UpTo",
        "Represents a non-negative upper bound in supported Span selector forms.",
        MemoryGrowthContract::NonAmplifying,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "UpTo called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "innf",
            template: "Non-negative integer or Infinity expected at position 1 in `1`.",
        },
    ])
    .protected(),
];
