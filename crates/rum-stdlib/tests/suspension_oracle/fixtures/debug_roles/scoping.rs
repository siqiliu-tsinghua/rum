use super::super::super::descriptors::{Fixture, Pause, Variant};
use super::support::*;
use rum_runtime::RuntimeInputKind;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let mut expected = "73";
    f.source = match (name, role) {
        ("Block" | "Module" | "With", "body") => format!(
            "{name}[{{Global`local=73,Global`other=29}},{}]",
            sequence(&f, v, "Global`local")
        ),
        ("Block" | "Module" | "With", "initializers") => {
            let bindings = (1..=5)
                .map(|i| format!("Global`binding{i}={}", step(&f, v, i, &i.to_string())))
                .collect::<Vec<_>>();
            format!(
                "{name}[{{{}}},{{Global`binding1,Global`binding2,Global`binding3,Global`binding4,Global`binding5}}]",
                bindings.join(",")
            )
        }
        ("DefinePackage", _) => {
            // Use a foreign context for the log so it is outside the unit's binding scan.
            f.setup.push_str(";Begin[\"OracleSource`\"]");
            project(
                &mut f,
                "Length[Names[\"OraclePackage`Private`*\"]]",
                if v == Variant::Error { "0" } else { "1" },
            );
            if v != Variant::Error {
                project(&mut f, "OraclePackage`peek[]", "37");
            }
            expected = "\"OraclePackage`\"";
            format!(
                "DefinePackage[\"OraclePackage`\",Public[{{peek}}],secret=37,peek[]:=secret,{}]",
                (1..=5)
                    .map(|i| step(&f, v, i, "73"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        ("Dialog", _) => format!("Dialog[{}]", sequence(&f, v, "Return[73]")),
        ("Evaluate", _) => format!("Evaluate[{}]", sequence(&f, v, "73")),
        ("ReleaseHold", _) => format!("ReleaseHold[HoldComplete[{}]]", sequence(&f, v, "73")),
        ("Input", "parsedAnswer") => {
            f.lifecycle_pauses = [1, 2];
            let answer = sequence(&f, v, "73");
            f.pauses.insert(
                0,
                Pause {
                    prompt: "parsed body".into(),
                    kind: RuntimeInputKind::Input,
                    answer,
                },
            );
            "Input[\"parsed body\"]".into()
        }
        ("Input" | "InputString", "request") => {
            f.setup.push_str(";Global`replies={}");
            let mut source = sequence(&f, v, "73");
            for p in &f.pauses {
                let input = format!("Input[\"{}\"]", p.prompt);
                source = source.replace(&input, &format!("AppendTo[Global`replies,{input}]"));
            }
            let expected_replies = if name == "InputString" {
                if twice(v) {
                    "List[\"2\", \"4\"]"
                } else {
                    "List[\"2\"]"
                }
            } else if twice(v) {
                "List[2, 4]"
            } else {
                "List[2]"
            };
            project(&mut f, "Global`replies", expected_replies);
            if name == "InputString" {
                for p in &mut f.pauses {
                    p.kind = RuntimeInputKind::InputString;
                }
                source.replace("Input[", "InputString[")
            } else {
                source
            }
        }
        _ => unreachable!("{entry}"),
    };
    if role == "initializers" {
        expected = "List[1, 2, 3, 4, 5]";
    }
    // Return is applied only after the five body visits.
    if name == "Dialog" {
        f.source = format!("Dialog[{};Return[73]]", sequence(&f, v, "Null"));
    }
    let mut f = finish(f, v, expected);
    if name == "DefinePackage" && v == Variant::Error {
        f.expected_error = None;
        f.expected_value = Some("$Failed".into());
    }
    f
}
