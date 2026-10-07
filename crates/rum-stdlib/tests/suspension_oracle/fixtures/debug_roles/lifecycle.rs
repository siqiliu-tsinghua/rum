use super::super::super::{descriptors::Fixture, support::lifecycle};

pub fn exercise(f: &Fixture) -> usize {
    let plans = f
        .pauses
        .iter()
        .map(|p| {
            (
                rum_runtime::RuntimeInputRequest {
                    kind: p.kind,
                    prompt: p.prompt.clone(),
                    password: false,
                },
                p.answer.clone(),
            )
        })
        .collect::<Vec<_>>();
    lifecycle::exercise_observed_at(
        || f.session(),
        &f.source,
        &plans,
        f.lifecycle_pauses,
        |s, stop, reset| {
            let loading = f.id.ends_with("fileForms") || f.id == "BeginPackage.dependency";
            let expected = if reset || (loading && f.id != "BeginPackage.dependency") {
                "List[]"
            } else if f.id == "BeginPackage.dependency" {
                if stop == 0 {
                    "List[1]"
                } else {
                    "List[1, 2, 3]"
                }
            } else if stop == 0 {
                "List[1, 2]"
            } else {
                "List[1, 2, 3, 4]"
            };
            assert_eq!(
                s.eval_request("Global`log").unwrap().value.to_string(),
                expected,
                "{} lifecycle log",
                f.id
            );
            let n = if reset || (loading && f.id != "BeginPackage.dependency") {
                0
            } else if f.id == "BeginPackage.dependency" {
                if stop == 0 { 1 } else { 3 }
            } else if stop == 0 {
                2
            } else {
                4
            };
            assert_eq!(
                s.eval_request("Global`state").unwrap().value.to_string(),
                n.to_string(),
                "{} lifecycle state",
                f.id
            );
            assert_eq!(
                s.eval_request("{Global`scope,Global`local,Global`other}")
                    .unwrap()
                    .value
                    .to_string(),
                "List[17, 19, 23]",
                "{} scope restoration",
                f.id
            );
            let check = |s: &mut rum_runtime::Session, source: &str, expected: &str| {
                assert_eq!(
                    s.eval_request(source).unwrap().value.to_string(),
                    expected,
                    "{} lifecycle {source}",
                    f.id
                );
            };
            match f.id.as_str() {
                "DefinePackage.body" => check(s, "Length[Names[\"OraclePackage`Private`*\"]]", "0"),
                "SetAttributes.attributes" => check(s, "Attributes[Global`attrTarget]", "List[]"),
                "ClearAttributes.attributes" => {
                    check(s, "Attributes[Global`attrTarget]", "List[Orderless, Flat]")
                }
                "SetOptions.optionSyntax" => {
                    check(s, "OptionValue[Global`optionTarget,Global`answer]", "19")
                }
                "Unset.target" => check(s, "Global`target[73]", "91"),
                "Input.request" | "InputString.request" => check(
                    s,
                    "Global`replies",
                    if reset || stop == 0 {
                        "List[]"
                    } else if f.id == "Input.request" {
                        "List[2]"
                    } else {
                        "List[\"2\"]"
                    },
                ),
                "Check.body" | "Quiet.body" => assert_eq!(
                    s.eval_request("Message[Global`probe::tag]")
                        .unwrap()
                        .messages
                        .len(),
                    1
                ),
                _ => {}
            }
            if loading {
                check(s, "$InputFileName", "\"\"");
                check(s, "$Context", "\"Global`\"");
                check(s, "MemberQ[$Packages,\"OracleOuter`\"]", "False");
                if f.id != "BeginPackage.dependency" {
                    check(s, "Length[OwnValues[OracleLoaded`value]]", "0");
                }
            }
            if f.id.starts_with("Trace") {
                assert!(!s.configuration().tracing_enabled);
            }
        },
    )
}
