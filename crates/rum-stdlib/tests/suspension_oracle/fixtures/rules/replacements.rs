//! Contracts: pattern preparation, rhs guards and delayed rule materialization.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    if name == "ReplacePart" && role == "guard" {
        // At a contested position rum tries rules last-to-first. Four guards
        // decline before the fifth accepts (or also declines). This tests the
        // documented candidate order without assuming a position-walk order.
        let guards = (1..=5)
            .map(|i| {
                let result = if i == 5 && v != Variant::Decline {
                    "True"
                } else {
                    "False"
                };
                let guard = step(&mut f, v, i, result);
                format!("1:>(99/;{guard})")
            })
            .collect::<Vec<_>>();
        f.source = format!(
            "ReplacePart[{{1,2,3,4,5}},{{{}}}]",
            guards.into_iter().rev().collect::<Vec<_>>().join(",")
        );
        return finish(
            f,
            v,
            if v == Variant::Decline {
                "List[1, 2, 3, 4, 5]"
            } else {
                "List[99, 2, 3, 4, 5]"
            },
        );
    }
    let mut target = "{1,2,3,4,5}".to_owned();
    let mut positions = "{{1},{2},{3},{4},{5}}".to_owned();
    let mut expected = "List[11, 12, 13, 14, 15]";
    let rule = match role {
        "heldTarget" => {
            target = sequence(&mut f, v, &target);
            "x_Integer:>x+10".into()
        }
        "heldSpec" => {
            positions = sequence(&mut f, v, &positions);
            "x_Integer:>x+10".into()
        }
        "delayedRhs" => {
            let body = callback(&f, v, "x+10");
            if name == "ReplaceAt" {
                format!("x_Integer:>{body}")
            } else {
                format!("{{x_Integer}}:>{body}")
            }
        }
        "guard" => {
            let body = callback(
                &f,
                v,
                if v == Variant::Decline {
                    "False"
                } else {
                    "True"
                },
            );
            if v == Variant::Decline {
                expected = "List[1, 2, 3, 4, 5]";
            }
            if name == "ReplaceAt" {
                format!("x_Integer:>((x+10)/;{body})")
            } else {
                format!("{{x_Integer}}:>((x+10)/;{body})")
            }
        }
        "patternTest" => {
            let body = callback(
                &f,
                v,
                if v == Variant::Decline {
                    "False"
                } else {
                    "True"
                },
            );
            if v == Variant::Decline {
                expected = "List[1, 2, 3, 4, 5]";
            }
            if name == "ReplaceAt" {
                format!("(x_Integer?Function[x,{body}]):>x+10")
            } else {
                format!("{{x_Integer?Function[x,{body}]}}:>x+10")
            }
        }
        "patternPreparation" => {
            let args = (1..=5)
                .map(|i| step(&mut f, v, i, &i.to_string()))
                .collect::<Vec<_>>()
                .join(",");
            if name == "ReplaceAt" {
                target = "{h[0,1,2,3,4,5]}".into();
                positions = "{1}".into();
                expected = "List[77]";
                format!("h[_,{args}]:>77")
            } else {
                // HoldPattern prevents RuleDelayed's own lhs preparation.
                // ReplacePart's local position-pattern resolver owns the work.
                target = "{{{{{{0,0,0,0,7}}}}}}".into();
                expected = "List[List[List[List[List[List[0, 0, 0, 0, 77]]]]]]";
                let args = (1..=5)
                    .map(|i| step(&mut f, v, i, if i == 5 { "5" } else { "1" }))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("HoldPattern[{{_,{args}}}]:>77")
            }
        }
        _ => unreachable!(),
    };
    f.source = if name == "ReplaceAt" {
        format!("ReplaceAt[{target},{rule},{positions}]")
    } else {
        format!("ReplacePart[{target},{rule},Heads->False]")
    };
    finish(f, v, expected)
}
