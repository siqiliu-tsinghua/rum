#[path = "suspension_oracle/mod.rs"]
mod oracle;

#[test]
fn runner_scan_middle_and_twice() {
    use oracle::descriptors::{Checks, Fixture, Pause, accept};
    use rum_runtime::RuntimeInputKind;
    for twice in [false, true] {
        let mut f = Fixture::new(
            "Scan.callback",
            &format!(
                "Scan[Function[x,(AppendTo[Global`log,x];If[Or[x==2,{}],InputString[\"Scan.callback\"]];x)],{{1,2,3,4,5}}]",
                if twice { "x==4" } else { "False" }
            ),
        );
        f.pauses = (0..if twice { 2 } else { 1 })
            .map(|_| Pause {
                prompt: "Scan.callback".into(),
                kind: RuntimeInputKind::InputString,
                answer: "answer".into(),
            })
            .collect();
        f.expected_value = Some("Null".into());
        f.expected_log = Some("List[1, 2, 3, 4, 5]".into());
        accept(&f, Checks::ALL).unwrap();
    }
}
