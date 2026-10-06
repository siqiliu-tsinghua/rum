//! Map/array callbacks are applied to five distinct elements or neighborhoods.
//! Tail mappings deliberately leave the applications to evaluator sequencing.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let name = entry.split_once('.').unwrap().0;
    let count = if name == "MapAll" { 6 } else { 5 };
    let mut f = base(entry, v, count);
    let result = match name {
        "MapIndexed" => "x+10 First[p]",
        "MapThread" | "Outer" | "ComapApply" => "x+y",
        "BlockMap" | "ArrayFilter" | "ArrayReduce" => "Total[x]",
        "MapAll" => "x",
        _ => "10 x",
    };
    let body = visits(&f, v, count, result);
    let parameters = match name {
        "MapIndexed" => "{x,p}",
        "MapThread" | "Outer" | "ComapApply" => "{x,y}",
        _ => "x",
    };
    f.setup
        .push_str(&format!(";Global`f=Function[{parameters},{body}]"));
    let (source, expected) = match name {
        "Map" => ("Map[Global`f,{1,2,3,4,5}]", "List[10, 20, 30, 40, 50]"),
        "MapIndexed" => (
            "MapIndexed[Global`f,{1,2,3,4,5}]",
            "List[11, 22, 33, 44, 55]",
        ),
        "MapAll" => ("MapAll[Global`f,{1,2,3,4,5}]", "List[1, 2, 3, 4, 5]"),
        "MapAt" => (
            "MapAt[Global`f,{9,1,2,3,4,5,8},{{2},{3},{4},{5},{6}}]",
            "List[9, 10, 20, 30, 40, 50, 8]",
        ),
        "MapThread" => (
            "MapThread[Global`f,{{1,2,3,4,5},{10,20,30,40,50}}]",
            "List[11, 22, 33, 44, 55]",
        ),
        "Array" => ("Array[Global`f,5]", "List[10, 20, 30, 40, 50]"),
        "ArrayFilter" => (
            "ArrayFilter[Global`f,{1,2,3,4,5},1,Padding->0]",
            "List[3, 6, 9, 12, 9]",
        ),
        "ArrayReduce" => (
            "ArrayReduce[Global`f,{{1,10},{2,20},{3,30},{4,40},{5,50}},2]",
            "List[11, 22, 33, 44, 55]",
        ),
        "BlockMap" => (
            "BlockMap[Global`f,{1,10,2,20,3,30,4,40,5,50},2]",
            "List[11, 22, 33, 44, 55]",
        ),
        "Outer" => (
            "Outer[Global`f,{1,2,3,4,5},{10}]",
            "List[List[11], List[12], List[13], List[14], List[15]]",
        ),
        "Comap" => (
            "Comap[{Global`f,Global`f,Global`f,Global`f,Global`f},3]",
            "List[30, 30, 30, 30, 30]",
        ),
        "ComapApply" => (
            "ComapApply[{Global`f,Global`f,Global`f,Global`f,Global`f},{2,3}]",
            "List[5, 5, 5, 5, 5]",
        ),
        "Through" => (
            "Through[{Global`f,Global`f,Global`f,Global`f,Global`f}[3]]",
            "List[30, 30, 30, 30, 30]",
        ),
        _ => unreachable!("{entry}"),
    };
    f.source = source.into();
    finish(f, v, expected)
}

pub fn single(entry: &str, v: Variant) -> Fixture {
    // These builtins invoke one callback. Five ordered effects belong inside
    // that callback; an outer Map would exercise a different continuation.
    let mut f = base(entry, v, 5);
    let (body, source, expected) = if entry == "Operate.callback" {
        (
            sequence(&f, v, "Global`g"),
            "Operate[Global`f,Global`h[1,2,3,4,5]]",
            "g[1, 2, 3, 4, 5]",
        )
    } else {
        (
            sequence(&f, v, "Reverse[x]"),
            "SubsetMap[Global`f,{9,1,2,3,4,5,8},{{2},{3},{4},{5},{6}}]",
            "List[9, 5, 4, 3, 2, 1, 8]",
        )
    };
    f.setup.push_str(&format!(";Global`f=Function[x,{body}]"));
    f.source = source.into();
    finish(f, v, expected)
}
