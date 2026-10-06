//! Held pattern preparation and matcher callbacks use separate factories.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn preparation(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, _) = entry.split_once('.').unwrap();
    let args = (1..=5)
        .map(|i| step(&mut f, v, i, &i.to_string()))
        .collect::<Vec<_>>()
        .join(",");
    let pattern = format!("h[_,{args}]");
    f.source = match name {
        "MatchQ" => format!("MatchQ[h[0,1,2,3,4,5],{pattern}]"),
        "Cases" | "DeleteCases" => format!("{name}[{{h[0,1,2,3,4,5]}},{pattern}]"),
        // RuleDelayed's lhs preparation is entered by this special form's
        // local rule resolver, with the rewrite caller retained underneath.
        "Replace" | "ReplaceAll" | "ReplaceAllUnheld" | "ReplaceList" | "ReplaceRepeated" => {
            format!("{name}[h[0,1,2,3,4,5],{pattern}:>77]")
        }
        _ => unreachable!(),
    };
    let expected = match name {
        "MatchQ" => "True",
        "Cases" => "List[h[0, 1, 2, 3, 4, 5]]",
        "DeleteCases" => "List[]",
        "ReplaceList" => "List[77]",
        _ => "77",
    };
    finish(f, v, expected)
}

pub fn matching(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    if name == "MatchQ" {
        // Five independently matched argument slots. Decline only at slot five
        // so short-circuiting cannot hide either continuation or its prefix.
        let patterns = (1..=5)
            .map(|i| {
                let result = if i == 5 && v == Variant::Decline {
                    "False"
                } else {
                    "True"
                };
                let body = step(&mut f, v, i, result);
                if role == "guard" {
                    format!("(x{i}_Integer/;{body})")
                } else {
                    format!("(_Integer?Function[x,{body}])")
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        f.source = format!("MatchQ[h[1,2,3,4,5],h[{patterns}]]");
        return finish(
            f,
            v,
            if v == Variant::Decline {
                "False"
            } else {
                "True"
            },
        );
    }
    let body = callback(
        &f,
        v,
        if v == Variant::Decline {
            "False"
        } else {
            "True"
        },
    );
    let pattern = if role == "guard" {
        format!("x_Integer/;{body}")
    } else {
        format!("_Integer?Function[x,{body}]")
    };
    f.source = format!("{name}[{{1,2,3,4,5}},{pattern},{{1}},Heads->False]");
    let empty = (name == "Cases") == (v == Variant::Decline);
    finish(
        f,
        v,
        if empty {
            "List[]"
        } else {
            "List[1, 2, 3, 4, 5]"
        },
    )
}
