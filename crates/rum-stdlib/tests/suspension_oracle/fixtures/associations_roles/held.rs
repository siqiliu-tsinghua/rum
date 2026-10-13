use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;
pub fn prepared(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, "prepared")
}
pub fn read(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, "read")
}
pub fn part(entry: &str, v: Variant) -> Fixture {
    fixture(entry, v, "part")
}
fn fixture(entry: &str, v: Variant, phase: &str) -> Fixture {
    let mut f = base(entry, v);
    let data = data();
    let (setup, target, snapshot) = match phase {
        "prepared" => (
            format!("Global`target[1]={data}"),
            format!("Global`target[{}]", sequence(&f, v, "1")),
            "DownValues[Global`target]",
        ),
        "read" => (
            format!("Global`target[1]:={}", sequence(&f, v, &data)),
            "Global`target[1]".into(),
            "DownValues[Global`target]",
        ),
        "part" => (
            format!("Global`target={{{data}}}"),
            format!("Global`target[[{}]]", sequence(&f, v, "1")),
            "OwnValues[Global`target]",
        ),
        _ => unreachable!(),
    };
    f.setup.push_str(&format!(
        ";{setup};Global`before={snapshot};Global`unchanged[]:=SameQ[{snapshot},Global`before]"
    ));
    let name = entry.split_once('.').unwrap().0;
    let expected = if name == "AssociateTo" {
        f.source = format!("AssociateTo[{target},6->60]");
        association((1..=6).map(|i| (i, (i * 10).to_string())))
    } else {
        f.source = format!("KeyDropFrom[{target},{{2,4}}]");
        association([1, 3, 5].map(|i| (i, (i * 10).to_string())))
    };
    project(
        &mut f,
        "Global`unchanged[]",
        if v == Variant::Error { "True" } else { "False" },
    );
    let expected = if phase == "part" && name == "AssociateTo" {
        format!("List[{expected}]")
    } else {
        expected
    };
    finish(f, v, &expected)
}
