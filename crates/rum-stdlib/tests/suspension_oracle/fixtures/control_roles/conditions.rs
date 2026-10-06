use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let expected = match (name, role) {
        ("And", _) => "False",
        ("Nand", _) => "True",
        ("Or", _) => "True",
        ("Nor", _) => "False",
        _ => "73",
    };
    f.source = match (name, role) {
        ("And" | "Or" | "Nand" | "Nor", _) => {
            let identity = if matches!(name, "And" | "Nand") {
                "True"
            } else {
                "False"
            };
            let decisive = if identity == "True" { "False" } else { "True" };
            let operands = (1..=5)
                .map(|i| step(&f, v, i, if i == 5 { decisive } else { identity }))
                .collect::<Vec<_>>();
            format!(
                "{name}[{},(AppendTo[Global`log,99];999)]",
                operands.join(",")
            )
        }
        ("If", "test") => format!("If[{},73,999]", sequence(&f, v, "True")),
        ("Which", "test") => {
            let pairs = (1..=5)
                .map(|i| {
                    format!(
                        "{},{}",
                        step(&f, v, i, if i == 5 { "True" } else { "False" }),
                        if i == 5 { 73 } else { 999 }
                    )
                })
                .collect::<Vec<_>>();
            format!("Which[{}]", pairs.join(","))
        }
        ("Switch", "test") => format!("Switch[{},0,999,7,73]", sequence(&f, v, "7")),
        ("Switch", "pattern") => {
            // False earlier patterns ensure the pauses belong to matching at
            // later candidate positions, including the resumed match owner.
            let pairs = (1..=5)
                .map(|i| {
                    format!(
                        "(_ /; {}),{}",
                        step(&f, v, i, if i == 5 { "True" } else { "False" }),
                        if i == 5 { 73 } else { 999 }
                    )
                })
                .collect::<Vec<_>>();
            format!("Switch[7,{}]", pairs.join(","))
        }
        _ => unreachable!("{entry}"),
    };
    finish(f, v, expected)
}
