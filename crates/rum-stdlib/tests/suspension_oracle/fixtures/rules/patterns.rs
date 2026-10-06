//! Held pattern preparation and five-candidate matcher callbacks are distinct.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let (target, pattern, accepted) = if role == "patternPreparation" {
        let args = (1..=5)
            .map(|i| step(&mut f, v, i, &i.to_string()))
            .collect::<Vec<_>>()
            .join(",");
        ("{h[0,1,2,3,4,5]}".to_owned(), format!("h[_,{args}]"), true)
    } else {
        // Count/Position accept each candidate; MemberQ/FreeQ accept only the
        // fifth so a short-circuit cannot hide either planned pause.
        let result = if v == Variant::Decline {
            "False"
        } else if matches!(name, "MemberQ" | "FreeQ") {
            "x==5"
        } else {
            "True"
        };
        let body = callback(&f, v, result);
        let pattern = if role == "guard" {
            format!("x_Integer/;{body}")
        } else {
            format!("_Integer?Function[x,{body}]")
        };
        ("{1,2,3,4,5}".into(), pattern, v != Variant::Decline)
    };
    f.source = if name == "FreeQ" {
        format!("FreeQ[{target},{pattern},Heads->False]")
    } else {
        format!("{name}[{target},{pattern},{{1}},Heads->False]")
    };
    let value = match (name, role, accepted) {
        ("Count", "patternPreparation", _) => "1",
        ("Count", _, true) => "5",
        ("Count", _, false) => "0",
        ("Position", "patternPreparation", _) => "List[List[1]]",
        ("Position", _, true) => "List[List[1], List[2], List[3], List[4], List[5]]",
        ("Position", _, false) => "List[]",
        ("MemberQ", _, true) | ("FreeQ", _, false) => "True",
        ("MemberQ", _, false) | ("FreeQ", _, true) => "False",
        _ => unreachable!(),
    };
    finish(f, v, value)
}

pub fn held(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let target = if entry.ends_with("heldTarget") {
        sequence(&mut f, v, "{1,2,3,4,5}")
    } else {
        "{1,2,3,4,5}".into()
    };
    let spec = if entry.ends_with("heldSpec") {
        sequence(&mut f, v, "{1}")
    } else {
        "{1}".into()
    };
    f.source = format!("Position[{target},_Integer,{spec},Heads->False]");
    finish(f, v, "List[List[1], List[2], List[3], List[4], List[5]]")
}
