//! Contracts: guide/evaluation.md (same request resumption), guide/embedding.md
//! (cancellation/reset), and the corresponding generated builtin references.
mod arithmetic;
mod bodies;
mod conditions;
mod falsifiers;
mod lifecycle;
mod loops;
mod options_properties;
mod phases;
mod support;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;

pub const ENTRIES: &[&str] = &[
    "AbortProtect.body",
    "And.operand",
    "Assert.callback",
    "Assert.test",
    "BlockRandom.body",
    "Catch.body",
    "Catch.pattern",
    "CatchExceptions.body",
    "CatchExceptions.handler",
    "CheckAbort.body",
    "CheckAbort.defaultOption",
    "CheckAbort.explicitOption",
    "CompoundExpression.sequence",
    "Confirm.test",
    "ConfirmAssert.test",
    "ConfirmBy.callback",
    "ConfirmBy.test",
    "ConfirmMatch.pattern",
    "ConfirmMatch.test",
    "ConfirmQuiet.test",
    "Do.body",
    "Do.bounds",
    "Enclose.body",
    "Failure.propertyParameters",
    "Failure.propertyTemplate",
    "For.body",
    "For.increment",
    "For.initialization",
    "For.test",
    "If.test",
    "Mean.arithmetic",
    "Median.arithmetic",
    "Nand.operand",
    "Nor.operand",
    "Or.operand",
    "Product.body",
    "Product.bounds",
    "Sum.body",
    "Sum.bounds",
    "Switch.pattern",
    "Switch.test",
    "Table.body",
    "Table.bounds",
    "Total.arithmetic",
    "Until.body",
    "Until.test",
    "Which.test",
    "While.body",
    "While.test",
    "WithCleanup.body",
    "WithCleanup.cleanup",
];

pub fn descriptors() -> Vec<Descriptor> {
    let mut rows: Vec<_> = ENTRIES
        .iter()
        .map(|&entry| Descriptor {
            id: entry,
            entry,
            variants: support::VARIANTS,
            factory: fixture,
            boundary: false,
            lifecycle: lifecycle::exercise,
        })
        .collect();
    rows.extend(phases::descriptors());
    rows
}

fn fixture(entry: &str, v: Variant) -> Fixture {
    let (name, role) = entry.split_once('.').unwrap();
    match (name, role) {
        ("Mean" | "Median" | "Total", _) => arithmetic::fixture(entry, v),
        ("Do" | "Table" | "Sum" | "Product" | "For" | "While" | "Until", _) => {
            loops::fixture(entry, v)
        }
        ("And" | "Or" | "Nand" | "Nor" | "If" | "Which" | "Switch", _) => {
            conditions::fixture(entry, v)
        }
        ("Failure", _) | ("CheckAbort", "defaultOption" | "explicitOption") => {
            options_properties::fixture(entry, v)
        }
        _ => bodies::fixture(entry, v),
    }
}
