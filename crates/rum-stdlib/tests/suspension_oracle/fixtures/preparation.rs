use super::super::descriptors::{Fixture, Pause, Variant};
use rum_runtime::RuntimeInputKind;

pub fn fixture(v: Variant) -> Fixture {
    let twice = matches!(
        v,
        Variant::Twice | Variant::AlreadyResumed | Variant::Nested
    );
    let mut f = Fixture::new(
        "Evaluator.callPreparation",
        &format!(
            r#"Global`prepared[ (AppendTo[Global`log,1];1), (AppendTo[Global`log,2];Input["argument:2"]), (AppendTo[Global`log,3];3), (AppendTo[Global`log,4];{}), (AppendTo[Global`log,5];{}) ]"#,
            if twice {
                r#"InputString["argument:4"]"#
            } else {
                "4"
            },
            if v == Variant::Error { "Throw[9]" } else { "5" }
        ),
    );
    f.pauses = vec![Pause {
        prompt: "argument:2".into(),
        kind: RuntimeInputKind::Input,
        answer: "20".into(),
    }];
    if twice {
        f.pauses.push(Pause {
            prompt: "argument:4".into(),
            kind: RuntimeInputKind::InputString,
            answer: "four".into(),
        });
    }
    if v != Variant::Error {
        f.expected_value = Some(
            if twice {
                r#"prepared[1, 20, 3, "four", 5]"#
            } else {
                "prepared[1, 20, 3, 4, 5]"
            }
            .into(),
        );
    }
    f.expected_log = Some("List[1, 2, 3, 4, 5]".into());
    if v == Variant::AlreadyResumed {
        f.source = format!("InputString[\"caller\"];{}", f.source);
        f.pauses.insert(
            0,
            Pause {
                prompt: "caller".into(),
                kind: RuntimeInputKind::InputString,
                answer: "caller".into(),
            },
        );
    }
    if v == Variant::Nested {
        f.source = format!("Block[{{Global`scope=7}},{}]", f.source);
    }
    if v == Variant::Decline {
        f.source = r#"HoldComplete[InputString["held"]]"#.into();
        f.pauses.clear();
        f.expected_value = Some(r#"HoldComplete[InputString["held"]]"#.into());
        f.expected_log = Some("List[]".into());
    }
    f
}
