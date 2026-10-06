//! The generated predicate contracts require exact True where stated; the
//! truth aggregates retain unknown results and short circuit on decisive ones.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let name = entry.split_once('.').unwrap().0;
    if entry == "ArrayQ.rankPattern" {
        let condition = sequence(&f, v, "True");
        f.source = format!(
            "ArrayQ[{{1,2,3,4,5}},(_ /; {condition}),Function[x,(Global`selected++;IntegerQ[x])]]"
        );
        project(
            &mut f,
            "Global`selected",
            if v == Variant::Error { "0" } else { "5" },
        );
        return finish(f, v, "True");
    }
    let decline = v == Variant::Decline;
    let (result, expected) = match name {
        "AllTrue" => (
            if decline { "x<5" } else { "True" },
            if decline { "False" } else { "True" },
        ),
        "AnyTrue" => (
            if decline { "False" } else { "x==5" },
            if decline { "False" } else { "True" },
        ),
        "NoneTrue" => (
            if decline { "x==5" } else { "False" },
            if decline { "False" } else { "True" },
        ),
        "SelectPositions" => (
            if decline {
                "If[x==5,undecided,OddQ[x]]"
            } else {
                "OddQ[x]"
            },
            if decline {
                "List[List[1], List[3]]"
            } else {
                "List[List[1], List[3], List[5]]"
            },
        ),
        _ => (
            if decline {
                "If[x==5,undecided,True]"
            } else {
                "IntegerQ[x]"
            },
            if decline { "False" } else { "True" },
        ),
    };
    let callback = format!("Function[x,{}]", visits(&f, v, 5, result));
    f.source = match name {
        "MatrixQ" => format!("MatrixQ[{{{{1,2,3,4,5}}}},{callback}]"),
        "ArrayQ" => format!("ArrayQ[{{1,2,3,4,5}},1,{callback}]"),
        _ => format!("{name}[{{1,2,3,4,5}},{callback}]"),
    };
    finish(f, v, expected)
}
