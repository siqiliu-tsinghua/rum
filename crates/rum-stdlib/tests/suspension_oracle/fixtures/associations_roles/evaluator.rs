use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;
pub fn guard(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    // A failed exact guard must enter the ordinary pattern candidate. Both the
    // accepting and declining outcomes cross DirectDefaultFallbackFrame.
    f.setup.push_str(&format!(
        ";Default[x_]:=(Global`suffix++;83);Default[Global`oracleDefault]:=73/;{}",
        sequence(
            &f,
            v,
            if v == Variant::Decline {
                "False"
            } else {
                "True"
            }
        )
    ));
    f.source = "Default[Global`oracleDefault]".into();
    project(
        &mut f,
        "Global`suffix",
        if v == Variant::Decline { "1" } else { "0" },
    );
    finish(f, v, if v == Variant::Decline { "83" } else { "73" })
}
pub fn body(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    if v == Variant::Decline {
        return guard(entry, v);
    }
    f.setup.push_str(&format!(
        ";Default[Global`oracleDefault]:={}",
        sequence(&f, v, "73")
    ));
    f.source = "Default[Global`oracleDefault]".into();
    finish(f, v, "73")
}

pub fn cross(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let before = (1..=3)
        .map(|i| step(&f, v, i, "False"))
        .collect::<Vec<_>>()
        .join(";");
    let after = (4..=5)
        .map(|i| step(&f, v, i, "83"))
        .collect::<Vec<_>>()
        .join(";");
    f.setup.push_str(&format!(
        ";Default[x_]:=({after});Default[Global`oracleDefault]:=73/;({before})"
    ));
    f.source = "Default[Global`oracleDefault]".into();
    finish(f, v, "83")
}
