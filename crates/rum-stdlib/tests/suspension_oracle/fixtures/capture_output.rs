pub mod messages;
pub mod roles;
mod selectors;
// Provisional capture output must never appear in a cumulative host response.
use super::super::descriptors::{Checks, Fixture, Pause, accept};
use rum_runtime::RuntimeInputKind;

fn witness(source: &str, value: &str) -> Fixture {
    let mut f = Fixture::new("capture-output", source);
    f.pauses = vec![Pause {
        prompt: "capture".into(),
        kind: RuntimeInputKind::InputString,
        answer: "answer".into(),
    }];
    f.expected_value = Some(value.into());
    f.expected_log = Some("List[]".into());
    f.expected_prints = Some(vec![]);
    f
}

#[test]
fn evaluation_data_scalar_selector() {
    accept(
        &witness(
            r#"EvaluationData[InputString["capture"],"Result"]"#,
            r#""answer""#,
        ),
        Checks::ALL,
    )
    .unwrap();
}

#[test]
fn evaluation_data_list_selector() {
    accept(
        &witness(
            r#"EvaluationData[(InputString["capture"];7),{"Result","Success"}]"#,
            r#"Association[Rule["Result", 7], Rule["Success", True]]"#,
        ),
        Checks::ALL,
    )
    .unwrap();
}

#[test]
fn evaluation_data_print_prefix() {
    accept(&witness(r#"EvaluationData[(Print["before"];InputString["capture"];Print["after"];7),{"Result","OutputLog"}]"#, r#"Association[Rule["Result", 7], Rule["OutputLog", List["before", "after"]]]"#), Checks::ALL).unwrap();
}

#[test]
fn test_report_input_print_prefix() {
    accept(&witness(r#"TestReport[{TestCreate[(Print["before"];InputString["capture"];Print["after"];7),7]},"TestsSucceededCount"]"#, "1"), Checks::ALL).unwrap();
}

#[test]
fn quiet_message_prefix() {
    accept(&witness(r#"Quiet[(Message[Global`probe::tag];InputString["capture"];Message[Global`probe::tag];7)]"#, "7"), Checks::ALL).unwrap();
}

#[test]
fn adjacent_trace_capture_prefix() {
    let mut f = witness(r#"Trace[(1+2;InputString["capture"];4+5)]"#, "unused");
    f.expected_value = None;
    let mut s = f.session();
    let paused = s.eval_request(&f.source).unwrap_err();
    let completed = s.resume_eval_request_with_input("answer").unwrap();
    println!(
        "TRACE_CONTROL paused_trace={:?} completed_trace={:?}",
        paused.trace, completed.trace
    );
    accept(&f, Checks::ALL).unwrap();
}
