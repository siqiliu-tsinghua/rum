use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    if role == "body" {
        // Retain integer leaves, including the returned 73 and the values
        // produced by each operation. Never normalize trace structure.
        f.source = format!("{name}[{},_Integer]", sequence(&f, v, "73"));
        if name == "TracePrint" && v != Variant::Error {
            f.expected_prints
                .as_mut()
                .unwrap()
                .extend(["1", "2", "3", "4", "5", "73"].map(str::to_owned));
        }
        if name == "TraceDetail" {
            return finish(f, v, &detail_body(v));
        }
        finish(
            f,
            v,
            if name == "TracePrint" {
                "73"
            } else {
                "List[Hold[1], Hold[2], Hold[3], Hold[4], Hold[5], Hold[73]]"
            },
        )
    } else {
        // Five ordered alternative patterns run on the single literal trace
        // leaf. Four decline, then the fifth accepts that same leaf.
        let patterns = (1..=5)
            .map(|i| {
                format!(
                    "(_?Function[x,{}])",
                    step(&f, v, i, if i == 5 { "True" } else { "False" })
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        if name == "TraceDetail" {
            f.source = format!("{name}[1+2,{{{}}}]", patterns.replace("(_?", "(_Integer?"));
            return finish(
                f,
                v,
                "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]]]",
            );
        }
        f.source = format!("{name}[73,{{{patterns}}}]");
        if name == "TracePrint" && v != Variant::Error {
            f.expected_prints.as_mut().unwrap().push("73".into());
        }
        finish(
            f,
            v,
            if name == "TracePrint" {
                "73"
            } else {
                "List[Hold[73]]"
            },
        )
    }
}

fn detail_body(v: Variant) -> String {
    let mut events = Vec::new();
    let mut body = Vec::new();
    for i in 1..=5 {
        events.push(format!("Association[Rule[\"Step\", \"DefinitionStored\"], Rule[\"Symbol\", \"Global`state\"], Rule[\"Kind\", \"OwnValue\"], Rule[\"Value\", Hold[{i}]]]"));
        events.push(format!("Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Set[state, {i}]]], Rule[\"After\", Hold[{i}]]]"));
        body.extend([
            format!("AppendTo[log, {i}]"),
            format!("Print[{i}]"),
            format!("Set[state, {i}]"),
        ]);
        if i == 2 || (i == 4 && twice(v)) {
            let input = format!("Input[\"TraceDetail.body:{i}\"]");
            events.push(format!("Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[{input}]], Rule[\"After\", Hold[{i}]]]"));
            body.push(input);
        }
        body.push("73".into());
    }
    events.push(format!("Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[CompoundExpression[{}]]], Rule[\"After\", Hold[73]]]",body.join(", ")));
    format!("List[{}]", events.join(", "))
}
