use super::super::super::descriptors::{Checks, Fixture, accept};
use super::roles::pause;

#[test]
fn evaluation_data_selectors_and_preparation_run_once() {
    let properties = r#"List["Input", "Result", "Success", "FailureType", "Messages", "MessagesExpressions", "MessagesText", "OutputLog", "Trace", "Timing", "AbsoluteTiming", "InputString"]"#;
    for (selector, projection, expected) in [
        ("", "Global`data[\"Result\"]", "7"),
        (",All", "Global`data[\"Result\"]", "7"),
        (",Properties", "Global`data", properties),
        (",\"Result\"", "Global`data", "7"),
        (
            ",{\"Result\",\"Success\"}",
            "Global`data",
            r#"Association[Rule["Result", 7], Rule["Success", True]]"#,
        ),
        (
            ",\"absent\"",
            "Global`data",
            r#"Missing["KeyAbsent", "absent"]"#,
        ),
        (",17", "Head[Global`data]", "EvaluationData"),
        (",{17}", "Head[Global`data]", "EvaluationData"),
    ] {
        let mut f = Fixture::new(
            "selectors",
            &format!(
                "Global`data=EvaluationData[(AppendTo[Global`log,1];InputString[\"selector body\"];AppendTo[Global`log,2];7){selector}];{projection}"
            ),
        );
        f.pauses = vec![pause("selector body")];
        f.expected_value = Some(expected.into());
        f.expected_log = Some("List[1, 2]".into());
        f.pause_prints = Some(vec![vec![]]);
        f.pause_messages = Some(vec![vec![]]);
        f.pause_trace = Some(vec![vec![]]);
        accept(&f, Checks::ALL).unwrap_or_else(|e| panic!("selector {selector}: {e:?}"));
    }
    let mut f = Fixture::new(
        "selector preparation",
        r#"EvaluationData[(AppendTo[Global`log,2];InputString["body"];7),(AppendTo[Global`log,1];InputString["property"];"Result")]"#,
    );
    f.pauses = vec![pause("property"), pause("body")];
    f.expected_value = Some("7".into());
    f.expected_log = Some("List[1, 2]".into());
    f.pause_prints = Some(vec![vec![]; 2]);
    f.pause_messages = Some(vec![vec![]; 2]);
    f.pause_trace = Some(vec![vec![]; 2]);
    accept(&f, Checks::ALL).unwrap();
}

#[test]
fn report_failed_input_copy_and_release_wrong_output_and_empty_report() {
    for (input, expected, options, succeeded, prints) in [
        (
            r#"(Print["inside"];InputString["report"];Throw[19])"#,
            "$Failed",
            "",
            "0",
            vec!["inside".into()],
        ),
        (
            r#"(Print["inside"];InputString["report"];7)"#,
            "8",
            "",
            "0",
            vec![],
        ),
        (
            r#"(Print["inside"];InputString["report"];7)"#,
            "7",
            ",SameTest->Function[False]",
            "0",
            vec![],
        ),
        (
            r#"(Print["inside"];InputString["report"];Quiet[Check[1 //. {1->2,2->1},7,"General::iterlim"]])"#,
            "7",
            "",
            "1",
            vec![],
        ),
    ] {
        let mut f = Fixture::new(
            "report failures",
            &format!(
                r#"Global`report=TestReport[{{TestCreate[{input},{expected}]}}{options}];{{Global`report["TestsSucceededCount"],Global`report["TestsFailedCount"],Map[Function[t,{{t["ActualOutput"],t["OutputLog"],t["ActualMessages"],t["Success"]}}],Global`report["TestResults"]]}}"#
            ),
        );
        f.configure = |s| s.set_replace_repeated_limit(8);
        f.pauses = vec![pause("report")];
        f.expected_prints = Some(prints);
        f.pause_prints = Some(vec![vec![]]);
        f.pause_messages = Some(vec![vec![]]);
        f.pause_trace = Some(vec![vec![]]);
        f.projections = vec![
            "Global`report[\"TestsSucceededCount\"]".into(),
            "Global`report[\"TestsFailedCount\"]".into(),
        ];
        f.expected_projections = vec![
            succeeded.into(),
            if succeeded == "1" { "0" } else { "1" }.into(),
        ];
        accept(&f, Checks::ALL).unwrap_or_else(|e| panic!("{input}: {e:?}"));
    }
    let mut f = Fixture::new(
        "empty report",
        r#"Global`r=TestReport[{}];{Global`r["TestsSucceededCount"],Global`r["TestsFailedCount"],Global`r["TestResults"]}"#,
    );
    f.expected_value = Some("List[0, 0, List[]]".into());
    accept(&f, Checks::ALL).unwrap();
}
