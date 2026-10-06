//! The counters remain zero until the consumer pulls. Source scenarios put the
//! effect in an inner source; the transformer callback is separately counted.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let (name, role) = entry.split_once('.').unwrap();
    let result = match role {
        "predicate" if v == Variant::Decline => "If[x==2,Global`unknown,True]",
        "predicate" => "OddQ[x]",
        _ if name == "LazyFoldList" && role == "callback" => "x+y",
        _ if name == "LazyMap" && role == "callback" => "10 x",
        _ if name == "LazyTable" => "x^2",
        _ => "x+1",
    };
    let body = visits(&f, v, 5, result);
    let parameters = if name == "LazyFoldList" && role == "callback" {
        "{x,y}"
    } else {
        "x"
    };
    f.setup
        .push_str(&format!(";Global`f=Function[{parameters},{body}]"));
    let (descriptor, expected, calls) = match (name, role) {
        ("LazyMap", "callback") => (
            "LazyMap[Global`f,LazyTake[LazyRange[],5]]",
            "List[10, 20, 30, 40, 50]",
            0,
        ),
        ("LazyFoldList", "callback") => (
            "LazyFoldList[Global`f,0,LazyTake[LazyRange[],5]]",
            "List[0, 1, 3, 6, 10, 15]",
            0,
        ),
        ("LazyNestList", "callback") => (
            "LazyTake[LazyNestList[Global`f,1],5]",
            "List[1, 2, 3, 4, 5]",
            0,
        ),
        ("LazySelect", "predicate") => (
            "LazySelect[Global`f,LazyTake[LazyRange[],5]]",
            if v == Variant::Decline {
                "List[1, 3, 4, 5]"
            } else {
                "List[1, 3, 5]"
            },
            0,
        ),
        ("LazyTable", "body") => ("LazyTable[Global`f[x],{x,1,5}]", "List[1, 4, 9, 16, 25]", 0),
        ("LazyMap", "source") => (
            "LazyMap[Function[x,(Global`calls++;10 x)],LazyTake[LazyNestList[Global`f,1],5]]",
            "List[10, 20, 30, 40, 50]",
            5,
        ),
        ("LazySelect", "source") => (
            "LazySelect[Function[x,(Global`calls++;OddQ[x])],LazyTake[LazyNestList[Global`f,1],5]]",
            "List[1, 3, 5]",
            5,
        ),
        ("LazyFoldList", "source") => (
            "LazyFoldList[Function[{x,y},(Global`calls++;x+y)],0,LazyTake[LazyNestList[Global`f,1],5]]",
            "List[0, 1, 3, 6, 10, 15]",
            5,
        ),
        ("LazyDrop", "source") => (
            "LazyDrop[LazyTake[LazyNestList[Global`f,1],5],2]",
            "List[3, 4, 5]",
            0,
        ),
        ("LazyTake", "source") => (
            "LazyTake[LazyNestList[Global`f,1],5]",
            "List[1, 2, 3, 4, 5]",
            0,
        ),
        _ => unreachable!("{entry}"),
    };
    f.source =
        format!("Global`sequence={descriptor};Global`before=Global`state;Normal[Global`sequence]");
    project(&mut f, "Global`before", "0");
    project(
        &mut f,
        "Global`calls",
        &(if v == Variant::Error && calls > 0 {
            calls - 1
        } else {
            calls
        })
        .to_string(),
    );
    finish(f, v, expected)
}

pub fn bounds(entry: &str, v: Variant, slot: usize) -> Fixture {
    let mut f = base(entry, v, 5);
    let mut bounds = ["1".to_string(), "5".to_string(), "1".to_string()];
    bounds[slot] = sequence(&f, v, if slot == 1 { "5" } else { "1" });
    f.source = format!("Normal[LazyTable[x^2,{{x,{}}}]]", bounds.join(","));
    finish(f, v, "List[1, 4, 9, 16, 25]")
}
