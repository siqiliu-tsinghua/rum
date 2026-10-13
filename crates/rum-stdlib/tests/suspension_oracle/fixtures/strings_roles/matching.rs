//! Matches and callbacks own the visits; RuleDelayed prevents preparation from
//! running their bodies. Pauses at matches 2 and 4 preserve earlier output.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn apply(entry: &str, v: Variant) -> Fixture {
    apply_shape(entry, v, false)
}
pub fn apply_list(entry: &str, v: Variant) -> Fixture {
    apply_shape(entry, v, true)
}
fn apply_shape(entry: &str, v: Variant, list: bool) -> Fixture {
    let mut f = base(entry, v);
    let body = if list {
        visits(&f, v, "Reverse[Global`points]")
    } else {
        sequence(&f, v, "Reverse[Global`points]")
    };
    let input = if list {
        r#"{"ab","cd","ef","gh","ij"}"#
    } else {
        r#""abcde""#
    };
    f.source = format!("StringApply[Function[Global`points,{body}],{input}]");
    finish(
        f,
        v,
        if list {
            r#"List["ba", "dc", "fe", "hg", "ji"]"#
        } else {
            r#""edcba""#
        },
    )
}
pub fn fixture(entry: &str, v: Variant) -> Fixture {
    shape(entry, v, false, false)
}
pub fn list_fixture(entry: &str, v: Variant) -> Fixture {
    shape(entry, v, true, false)
}
pub fn guarded_rhs(entry: &str, v: Variant) -> Fixture {
    shape(entry, v, false, true)
}
pub fn guarded_rhs_list(entry: &str, v: Variant) -> Fixture {
    shape(entry, v, true, true)
}
fn shape(entry: &str, v: Variant, list: bool, guarded_rhs: bool) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let result = "ToUpperCase[Global`ch]";
    let rule = if role == "guard" {
        let accepted = if v == Variant::Decline {
            "OddQ[Global`state]"
        } else {
            "True"
        };
        format!(
            "Global`ch:LetterCharacter:>({result}/;{})",
            visits(&f, v, accepted)
        )
    } else {
        assert_eq!(role, "delayedRhs");
        let rhs = visits(&f, v, result);
        format!(
            "Global`ch:LetterCharacter:>{}",
            if guarded_rhs {
                format!("({rhs}/;True)")
            } else {
                rhs
            }
        )
    };
    let rules = if v == Variant::Decline {
        format!("{{{rule},LetterCharacter->\"fallback\"}}")
    } else {
        rule
    };
    let input = if list {
        r#"{"a","b","c","d","e"}"#
    } else {
        r#""a-b-c-d-e""#
    };
    f.source = format!("{name}[{input},{rules}]");
    let items = if v == Variant::Decline {
        ["A", "fallback", "C", "fallback", "E"]
    } else {
        ["A", "B", "C", "D", "E"]
    };
    let expected = if name == "StringReplace" {
        if list {
            format!(
                "List[{}]",
                items
                    .iter()
                    .map(|s| format!("\"{s}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            format!("\"{}\"", items.join("-"))
        }
    } else {
        format!(
            "List[{}]",
            items
                .iter()
                .map(|s| if list {
                    format!("List[\"{s}\"]")
                } else {
                    format!("\"{s}\"")
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    finish(f, v, &expected)
}
