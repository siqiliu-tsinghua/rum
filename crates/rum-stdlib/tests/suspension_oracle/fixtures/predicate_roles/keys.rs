use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    phase(entry, v, false)
}

pub fn phase(entry: &str, v: Variant, alternative: bool) -> Fixture {
    let mut f = base(entry, v, 5);
    let name = entry.split_once('.').unwrap().0;
    let callback = format!("Function[x,{}]", visits(&f, v, 5, "Mod[x,2]"));
    let (function, suffix, expected) = match (name, alternative) {
        ("GatherBy", true) => (
            format!("{{{callback},Function[x,x>3]}}"),
            "",
            "List[List[List[1, 3], List[5]], List[List[2], List[4]]]",
        ),
        ("SortBy", true) => (callback, ",Greater", "List[1, 3, 5, 2, 4]"),
        ("DeleteDuplicatesBy", _) => (callback, "", "List[1, 2]"),
        ("GatherBy", _) => (callback, "", "List[List[1, 3, 5], List[2, 4]]"),
        ("SortBy", _) => (callback, "", "List[2, 4, 1, 3, 5]"),
        ("ReverseSortBy", _) => (callback, "", "List[5, 3, 1, 4, 2]"),
        ("MaxBy", _) => (callback, "", "1"),
        ("MinBy", _) => (callback, "", "2"),
        ("MaximalBy", _) => (callback, "", "List[1, 3, 5]"),
        ("MinimalBy", _) => (callback, "", "List[2, 4]"),
        ("TakeLargestBy", _) => (callback, ",3", "List[1, 3, 5]"),
        ("TakeSmallestBy", _) => (callback, ",3", "List[2, 4, 1]"),
        _ => unreachable!("{entry}"),
    };
    f.source = format!("{name}[{{1,2,3,4,5}},{function}{suffix}]");
    finish(f, v, expected)
}
