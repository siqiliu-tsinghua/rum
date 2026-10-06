mod falsifiers;
mod held;
mod lifecycle;
mod options;
mod patterns;
mod replacements;
mod support;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;

pub const ENTRIES: &[&str] = &[
    "Cases.defaultOption",
    "Cases.delayedRhs",
    "Cases.explicitOption",
    "Cases.guard",
    "Cases.heldSpec",
    "Cases.heldTarget",
    "Cases.optionSyntax",
    "Cases.patternPreparation",
    "Cases.patternTest",
    "DeleteCases.defaultOption",
    "DeleteCases.explicitOption",
    "DeleteCases.guard",
    "DeleteCases.heldSpec",
    "DeleteCases.heldTarget",
    "DeleteCases.optionSyntax",
    "DeleteCases.patternPreparation",
    "DeleteCases.patternTest",
    "MatchQ.guard",
    "MatchQ.patternPreparation",
    "MatchQ.patternTest",
    "Replace.defaultOption",
    "Replace.delayedRhs",
    "Replace.explicitOption",
    "Replace.guard",
    "Replace.heldSpec",
    "Replace.heldTarget",
    "Replace.patternPreparation",
    "Replace.patternTest",
    "ReplaceAll.delayedRhs",
    "ReplaceAll.guard",
    "ReplaceAll.patternPreparation",
    "ReplaceAll.patternTest",
    "ReplaceAllUnheld.delayedRhs",
    "ReplaceAllUnheld.guard",
    "ReplaceAllUnheld.patternPreparation",
    "ReplaceAllUnheld.patternTest",
    "ReplaceList.delayedRhs",
    "ReplaceList.guard",
    "ReplaceList.heldSpec",
    "ReplaceList.heldTarget",
    "ReplaceList.patternPreparation",
    "ReplaceList.patternTest",
    "ReplaceRepeated.delayedRhs",
    "ReplaceRepeated.explicitOption",
    "ReplaceRepeated.guard",
    "ReplaceRepeated.heldSpec",
    "ReplaceRepeated.heldTarget",
    "ReplaceRepeated.patternPreparation",
    "ReplaceRepeated.patternTest",
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
        "heldTarget" | "heldSpec" => held::fixture(entry, v),
        "patternPreparation" => patterns::preparation(entry, v),
        "guard" | "patternTest" if matches!(name, "Cases" | "DeleteCases" | "MatchQ") => {
            patterns::matching(entry, v)
        }
        _ => replacements::fixture(entry, v),
    }
}
