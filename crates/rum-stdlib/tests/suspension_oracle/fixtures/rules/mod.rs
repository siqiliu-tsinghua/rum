mod assignments;
mod construction;
mod falsifiers;
mod lifecycle;
mod options;
mod patterns;
mod replacements;
mod support;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;

pub const ENTRIES: &[&str] = &[
    "AddTo.target",
    "AddTo.update",
    "AppendTo.target",
    "AppendTo.update",
    "ApplyTo.target",
    "ApplyTo.update",
    "Count.defaultOption",
    "Count.explicitOption",
    "Count.guard",
    "Count.optionSyntax",
    "Count.patternPreparation",
    "Count.patternTest",
    "Decrement.target",
    "Decrement.update",
    "DivideBy.target",
    "DivideBy.update",
    "FreeQ.defaultOption",
    "FreeQ.explicitOption",
    "FreeQ.guard",
    "FreeQ.optionSyntax",
    "FreeQ.patternPreparation",
    "FreeQ.patternTest",
    "Increment.target",
    "Increment.update",
    "Level.defaultOption",
    "MemberQ.defaultOption",
    "MemberQ.explicitOption",
    "MemberQ.guard",
    "MemberQ.optionSyntax",
    "MemberQ.patternPreparation",
    "MemberQ.patternTest",
    "NormalizeRules.lhsPreparation",
    "Position.defaultOption",
    "Position.explicitOption",
    "Position.guard",
    "Position.heldSpec",
    "Position.heldTarget",
    "Position.optionSyntax",
    "Position.patternPreparation",
    "Position.patternTest",
    "PreDecrement.target",
    "PreDecrement.update",
    "PreIncrement.target",
    "PreIncrement.update",
    "PrependTo.target",
    "PrependTo.update",
    "ReplaceAt.delayedRhs",
    "ReplaceAt.guard",
    "ReplaceAt.heldSpec",
    "ReplaceAt.heldTarget",
    "ReplaceAt.patternPreparation",
    "ReplaceAt.patternTest",
    "ReplacePart.delayedRhs",
    "ReplacePart.guard",
    "ReplacePart.patternPreparation",
    "ReplacePart.patternTest",
    "Rule.lhsPreparation",
    "RuleDelayed.lhsPreparation",
    "Set.destructuring",
    "Set.target",
    "SetDelayed.target",
    "SubtractFrom.target",
    "SubtractFrom.update",
    "TagSet.target",
    "TagSetDelayed.target",
    "TagUnset.target",
    "TimesBy.target",
    "TimesBy.update",
    "UpSet.target",
    "UpSetDelayed.target",
];

pub fn descriptors() -> Vec<Descriptor> {
    ENTRIES
        .iter()
        .map(|&entry| Descriptor {
            id: entry,
            entry,
            variants: if entry.ends_with(".guard") || entry.ends_with(".patternTest") {
                support::MATCH_VARIANTS
            } else {
                support::VARIANTS
            },
            factory: fixture,
            boundary: false,
            lifecycle: lifecycle::exercise,
        })
        .collect()
}

fn fixture(entry: &str, v: Variant) -> Fixture {
    let (name, role) = entry.split_once('.').unwrap();
    match role {
        "defaultOption" | "explicitOption" | "optionSyntax" => options::fixture(entry, v),
        "lhsPreparation" => construction::fixture(entry, v),
        "destructuring" => assignments::destructuring(entry, v),
        "target"
            if matches!(
                name,
                "Set"
                    | "SetDelayed"
                    | "UpSet"
                    | "UpSetDelayed"
                    | "TagSet"
                    | "TagSetDelayed"
                    | "TagUnset"
            ) =>
        {
            assignments::target(entry, v)
        }
        "target" => assignments::update_target(entry, v),
        "update" => assignments::update(entry, v),
        _ if matches!(name, "ReplaceAt" | "ReplacePart") => replacements::fixture(entry, v),
        "heldTarget" | "heldSpec" => patterns::held(entry, v),
        _ => patterns::fixture(entry, v),
    }
}
