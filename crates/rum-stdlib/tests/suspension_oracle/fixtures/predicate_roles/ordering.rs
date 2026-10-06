//! Sort comparison schedules are deliberately not a contract (guide/evaluation
//! Ordering Callbacks). Log the first five visits, compare the complete pair
//! transcript between queued/resumed runs, and assert the documented result.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    if entry == "LexicographicOrder.comparator" {
        return lexicographic(entry, v, "forward");
    }
    let mut f = base(entry, v, 5);
    let name = entry.split_once('.').unwrap().0;
    let visit = visits(&f, v, 5, "x<y");
    let callback = if name == "OrderedQ" {
        format!("Function[{{x,y}},{visit}]")
    } else {
        f.setup.push_str(";Global`pairs={}");
        // No fixed comparison count/order expectation. Equality still observes
        // every pair, including comparisons after the two selected pauses.
        f.projections.push("Global`pairs".into());
        format!("Function[{{x,y}},(AppendTo[Global`pairs,{{x,y}}];If[Global`state<5,{visit},x<y])]")
    };
    let items = if name == "OrderedQ" {
        "{1,2,3,4,5,6}"
    } else {
        "{6,2,5,1,4,3}"
    };
    f.source = format!("{name}[{items},{callback}]");
    finish(
        f,
        v,
        match name {
            "OrderedQ" => "True",
            "Sort" => "List[1, 2, 3, 4, 5, 6]",
            "ReverseSort" => "List[6, 5, 4, 3, 2, 1]",
            _ => unreachable!(),
        },
    )
}

pub fn lexicographic(entry: &str, v: Variant, phase: &str) -> Fixture {
    let mut f = base(entry, v, 5);
    let (arguments, callback) = match phase {
        "forward" | "reverse" => {
            let active = if phase == "forward" {
                "OddQ[Global`calls]"
            } else {
                "EvenQ[Global`calls]"
            };
            (
                "{1,2,3,4,5},{1,2,3,4,5}",
                format!(
                    "(Global`calls++;If[{active},{},False])",
                    visits(&f, v, 5, "False")
                ),
            )
        }
        "pair-forward" | "pair-reverse" => {
            let active = if phase == "pair-forward" {
                "x==1"
            } else {
                "x==2"
            };
            (
                "1,2",
                format!("If[{active},{},False]", sequence(&f, v, "False")),
            )
        }
        "cross" => {
            // Two completed comparisons precede the forward pause at the
            // second pair. Its reverse comparison pauses again on reentry.
            let after = (3..=5)
                .map(|i| step(&f, v, i, "False"))
                .collect::<Vec<_>>()
                .join(";");
            (
                "{1,2,3,4,5},{1,2,3,4,5}",
                format!(
                    "(Global`calls++;Switch[Global`calls,2,{},3,{},4,({after}),_,False])",
                    step(&f, v, 1, "False"),
                    step(&f, v, 2, "False")
                ),
            )
        }
        "pair-cross" => {
            // Pause once in the forward callback and once in its reverse
            // callback, covering the owner entered directly from resume.
            let before = (1..=3)
                .map(|i| step(&f, v, i, "False"))
                .collect::<Vec<_>>()
                .join(";");
            let after = (4..=5)
                .map(|i| step(&f, v, i, "False"))
                .collect::<Vec<_>>()
                .join(";");
            ("1,2", format!("If[x==1,({before}),({after})]"))
        }
        _ => unreachable!(),
    };
    f.source = format!("LexicographicOrder[{arguments},Function[{{x,y}},{callback}]]");
    finish(f, v, "0")
}
