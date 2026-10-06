//! Target evaluation and computed pattern/rule specifications retain the caller.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let mut target = if name == "ReplaceList" {
        "3"
    } else {
        "{1,2,3,4,5}"
    }
    .to_owned();
    let mut spec = if matches!(name, "Cases" | "DeleteCases") {
        "3"
    } else {
        "3->9"
    }
    .to_owned();
    if role == "heldTarget" {
        target = sequence(&mut f, v, &target);
    } else {
        spec = sequence(&mut f, v, &spec);
    }
    f.source = if name == "Replace" {
        format!("Replace[{target},{spec},{{1}}]")
    } else {
        format!("{name}[{target},{spec}]")
    };
    let expected = match name {
        "Cases" => "List[3]",
        "DeleteCases" => "List[1, 2, 4, 5]",
        "ReplaceList" => "List[9]",
        "Replace" | "ReplaceRepeated" => "List[1, 2, 9, 4, 5]",
        _ => unreachable!(),
    };
    finish(f, v, expected)
}
