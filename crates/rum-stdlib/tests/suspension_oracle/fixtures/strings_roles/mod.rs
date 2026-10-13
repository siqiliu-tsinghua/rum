//! Genuine guest roles of the three string families. Contracts are the generated
//! reference and patterns-and-rules.md; options are delayed local reads.
mod falsifiers;
mod lifecycle;
mod matching;
mod options;
mod parsed_source;
mod support;
mod text;
use super::super::descriptors::{Fixture, Variant};
use super::primary::Descriptor;

pub const ENTRIES: &[&str] = &[
    "AlignColumns.defaultOption",
    "AlignColumns.explicitOption",
    "HtmlUnescape.defaultOption",
    "HtmlUnescape.explicitOption",
    "ParseArgs.defaultOption",
    "ParseArgs.explicitOption",
    "StringApply.callback",
    "StringCases.defaultOption",
    "StringCases.delayedRhs",
    "StringCases.explicitOption",
    "StringCases.guard",
    "StringContainsQ.defaultOption",
    "StringContainsQ.explicitOption",
    "StringCount.defaultOption",
    "StringCount.explicitOption",
    "StringDelete.defaultOption",
    "StringDelete.explicitOption",
    "StringEndsQ.defaultOption",
    "StringEndsQ.explicitOption",
    "StringFreeQ.defaultOption",
    "StringFreeQ.explicitOption",
    "StringIndent.defaultOption",
    "StringIndent.explicitOption",
    "StringMatchQ.defaultOption",
    "StringMatchQ.explicitOption",
    "StringPosition.defaultOption",
    "StringPosition.explicitOption",
    "StringReplace.defaultOption",
    "StringReplace.delayedRhs",
    "StringReplace.explicitOption",
    "StringReplace.guard",
    "StringSplit.defaultOption",
    "StringSplit.explicitOption",
    "StringStartsQ.defaultOption",
    "StringStartsQ.explicitOption",
    "StringWrap.defaultOption",
    "StringWrap.explicitOption",
    "TemplateApply.defaultOption",
    "TemplateApply.explicitOption",
    "ToExpression.parsedSource",
    "ToExpression.resultHead",
    "Uncompress.defaultOption",
    "Uncompress.explicitOption",
    "Uncompress.parsedSource",
    "UrlDecode.defaultOption",
    "UrlDecode.explicitOption",
    "UrlEncode.defaultOption",
    "UrlEncode.explicitOption",
    "UrlParse.defaultOption",
    "UrlParse.explicitOption",
];
pub fn descriptors() -> Vec<Descriptor> {
    let mut rows: Vec<_> = ENTRIES
        .iter()
        .map(|&entry| Descriptor {
            id: entry,
            entry,
            variants: if entry.ends_with(".guard") {
                support::GUARD_VARIANTS
            } else {
                support::VARIANTS
            },
            factory: fixture,
            boundary: false,
            lifecycle: lifecycle::exercise,
        })
        .collect();
    let extra: &[(&str, &str, fn(&str, Variant) -> Fixture)] = &[
        (
            "StringApply.callback.list",
            "StringApply.callback",
            matching::apply_list,
        ),
        (
            "StringCases.delayedRhs.list",
            "StringCases.delayedRhs",
            matching::list_fixture,
        ),
        (
            "StringCases.delayedRhs.guarded",
            "StringCases.delayedRhs",
            matching::guarded_rhs,
        ),
        (
            "StringCases.delayedRhs.guardedList",
            "StringCases.delayedRhs",
            matching::guarded_rhs_list,
        ),
        (
            "StringReplace.delayedRhs.list",
            "StringReplace.delayedRhs",
            matching::list_fixture,
        ),
        (
            "StringReplace.delayedRhs.guarded",
            "StringReplace.delayedRhs",
            matching::guarded_rhs,
        ),
        (
            "StringReplace.delayedRhs.guardedList",
            "StringReplace.delayedRhs",
            matching::guarded_rhs_list,
        ),
        (
            "StringCases.guard.list",
            "StringCases.guard",
            matching::list_fixture,
        ),
        (
            "StringReplace.guard.list",
            "StringReplace.guard",
            matching::list_fixture,
        ),
        (
            "Uncompress.parsedSource.wrapped",
            "Uncompress.parsedSource",
            text::wrapped_uncompress,
        ),
    ];
    rows.extend(extra.iter().map(|&(id, entry, factory)| Descriptor {
        id,
        entry,
        factory,
        variants: if entry.ends_with(".guard") {
            support::GUARD_VARIANTS
        } else {
            support::VARIANTS
        },
        boundary: false,
        lifecycle: lifecycle::exercise,
    }));
    rows
}
fn fixture(entry: &str, v: Variant) -> Fixture {
    match entry.split_once('.').unwrap().1 {
        "defaultOption" | "explicitOption" => options::fixture(entry, v),
        "callback" => matching::apply(entry, v),
        "guard" | "delayedRhs" => matching::fixture(entry, v),
        "parsedSource" | "resultHead" => text::fixture(entry, v),
        _ => unreachable!("{entry}"),
    }
}
