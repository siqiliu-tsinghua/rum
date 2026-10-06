use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let name = entry.split_once('.').unwrap().0;
    let decline = v == Variant::Decline;
    let (count, arguments, expected) = match name {
        "DuplicateFreeQ" => (
            if decline { 10 } else { 7 },
            "{1,2,3,4,2}",
            if decline { "True" } else { "False" },
        ),
        "DeleteDuplicates" => (
            if decline { 10 } else { 8 },
            "{1,2,3,4,2}",
            if decline {
                "List[1, 2, 3, 4, 2]"
            } else {
                "List[1, 2, 3, 4]"
            },
        ),
        "Tally" => (
            if decline { 10 } else { 8 },
            "{1,2,3,4,2}",
            if decline {
                "List[List[1, 1], List[2, 1], List[3, 1], List[4, 1], List[2, 1]]"
            } else {
                "List[List[1, 1], List[2, 2], List[3, 1], List[4, 1]]"
            },
        ),
        "ContainsNone" | "DisjointQ" => (
            25,
            "{1,2,3,4,5},{5,4,3,2,1}",
            if decline { "True" } else { "False" },
        ),
        _ => (
            25,
            "{1,2,3,4,5},{5,4,3,2,1}",
            if decline { "False" } else { "True" },
        ),
    };
    let mut f = base(entry, v, count);
    let body = visits(&f, v, count, if decline { "undecided" } else { "x==y" });
    let callback = format!("Function[{{x,y}},{body}]");
    // Tally and DeleteDuplicates take a positional test, not SameTest syntax.
    f.source = if matches!(name, "Tally" | "DeleteDuplicates") {
        format!("{name}[{arguments},{callback}]")
    } else {
        format!("{name}[{arguments},SameTest->{callback}]")
    };
    finish(f, v, expected)
}

pub fn set_fixture(entry: &str, v: Variant, deduplicate: bool) -> Fixture {
    let name = entry.split_once('.').unwrap().0;
    let decline = v == Variant::Decline;
    let (count, arguments, expected, active, comparison) = match (name, deduplicate) {
        ("Union", _) => (
            15,
            "{1,2,3,4,5,5}",
            if decline {
                "List[1, 2, 3, 4, 5, 5]"
            } else {
                "List[1, 2, 3, 4, 5]"
            },
            "True",
            "x==y",
        ),
        ("Intersection", false) => (
            if decline { 25 } else { 15 },
            "{1,2,3,4,5},{11,12,13,14,15}",
            if decline {
                "List[]"
            } else {
                "List[1, 2, 3, 4, 5]"
            },
            "y>10",
            "If[y>10,x+10==y,x==y]",
        ),
        ("Intersection", true) => (
            10,
            "{1,2,3,4,5},{11,12,13,14,15}",
            "List[1, 2, 3, 4, 5]",
            "y<10",
            "If[y>10,x+10==y,x==y]",
        ),
        ("Complement", false) => (
            if decline { 15 } else { 12 },
            "{1,2,3,4,5},{12,14,99}",
            if decline {
                "List[1, 2, 3, 4, 5]"
            } else {
                "List[1, 3, 5]"
            },
            "y>10",
            "If[y>10,x+10==y,x==y]",
        ),
        ("Complement", true) => (
            6,
            "{1,2,3,4,5},{12,14,99}",
            "List[1, 3, 5]",
            "y<10",
            "If[y>10,x+10==y,x==y]",
        ),
        _ => unreachable!(),
    };
    let mut f = base(entry, v, count);
    let body = visits(&f, v, count, if decline { "undecided" } else { comparison });
    f.source =
        format!("{name}[{arguments},SameTest->Function[{{x,y}},If[{active},{body},{comparison}]]]");
    finish(f, v, expected)
}
