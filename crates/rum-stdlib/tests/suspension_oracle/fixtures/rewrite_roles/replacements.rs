//! Five traversal candidates, five root rules, or five ReplaceRepeated passes.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    if matches!(name, "ReplaceList" | "ReplaceRepeated") {
        let rules = (1..=5)
            .map(|i| {
                let lhs = if name == "ReplaceList" { 7 } else { i };
                let rhs = if name == "ReplaceList" { i + 10 } else { i + 1 };
                let declines = v == Variant::Decline && (name == "ReplaceList" || i == 5);
                let result = if role == "delayedRhs" {
                    rhs.to_string()
                } else if declines {
                    "False".into()
                } else {
                    "True".into()
                };
                let body = step(&mut f, v, i, &result);
                match role {
                    "guard" => format!("{lhs}:>({rhs}/;{body})"),
                    "patternTest" => format!("({lhs}?Function[x,{body}]):>{rhs}"),
                    "delayedRhs" => format!("{lhs}:>{body}"),
                    _ => unreachable!(),
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        f.source = if name == "ReplaceList" {
            format!("ReplaceList[7,{{{rules}}}]")
        } else {
            format!("ReplaceRepeated[1,{{{rules}}},MaxIterations->10]")
        };
        let expected = match (name, v == Variant::Decline) {
            ("ReplaceList", false) => "List[11, 12, 13, 14, 15]",
            ("ReplaceList", true) => "List[]",
            (_, false) => "6",
            (_, true) => "5",
        };
        return finish(f, v, expected);
    }
    let result = if role == "delayedRhs" {
        "x+10"
    } else if v == Variant::Decline {
        "False"
    } else {
        "True"
    };
    let body = callback(&f, v, result);
    let rule = match role {
        "guard" => format!("x_Integer:>((x+10)/;{body})"),
        "patternTest" => format!("(x_Integer?Function[x,{body}]):>x+10"),
        "delayedRhs" => format!("x_Integer:>{body}"),
        _ => unreachable!(),
    };
    let target = if name == "ReplaceAllUnheld" {
        "{1,2,3,4,5,HoldComplete[9]}"
    } else {
        "{1,2,3,4,5}"
    };
    f.source = if name == "Replace" {
        format!("Replace[{target},{rule},{{1}},Heads->False]")
    } else {
        format!("{name}[{target},{rule}]")
    };
    let expected = match (name, v == Variant::Decline) {
        ("Cases", true) => "List[]",
        ("ReplaceAllUnheld", true) => "List[1, 2, 3, 4, 5, HoldComplete[9]]",
        ("ReplaceAllUnheld", false) => "List[11, 12, 13, 14, 15, HoldComplete[9]]",
        (_, true) => "List[1, 2, 3, 4, 5]",
        (_, false) => "List[11, 12, 13, 14, 15]",
    };
    finish(f, v, expected)
}
