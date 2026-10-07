use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    option(entry, v, "SameTest")
}
pub fn option(entry: &str, v: Variant, phase: &str) -> Fixture {
    let mut f = base(entry, v);
    let role = entry.split_once('.').unwrap().1;
    let expected = "5";
    let mut options = String::new();
    if role.ends_with("Option") {
        let value = sequence(
            &f,
            v,
            if phase == "SameTest" {
                "SameQ"
            } else {
                "Infinity"
            },
        );
        if role == "defaultOption" {
            f.setup.push_str(&format!(";Unprotect[TestReport];SetOptions[TestReport,{phase}->Global`reportOption];Global`reportOption:={value}"));
            options = if phase == "SameTest" {
                ",TimeConstraint->Infinity"
            } else {
                ",SameTest->SameQ"
            }
            .into();
        } else {
            options = format!(",Unevaluated[{phase}->{value}]");
        }
    }
    let tests = (1..=5)
        .map(|i| format!("TestCreate[{i},{i}]"))
        .collect::<Vec<_>>()
        .join(",");
    f.source = format!("TestReport[{{{tests}}},\"TestsSucceededCount\"{options}]");
    finish(f, v, expected)
}
