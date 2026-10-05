use super::support::{finish, session};
use rum_runtime::SessionRequestError;
fn cancel_case(kind: &str, stop: usize) {
    let pattern = r#"h[x_,(AppendTo[log,1];Print["first"];Input["first: "];AppendTo[log,2];2),(AppendTo[log,3];Print["second"];Input["second: "];AppendTo[log,4];4)]"#;
    let source = match kind {
        "Rule" => format!("Rule[{pattern},9]"),
        _ => format!("RuleDelayed[{pattern},9]"),
    };
    let mut s = session(None);
    let cancel = s.request_cancel_handle();
    let mut out = s.eval_request(&source);
    for index in 0..=stop {
        let Err(f) = &out else {
            panic!("must reach suspension {index}: {out:?}");
        };
        let SessionRequestError::InputRequired { request } = &f.error else {
            panic!("must request input: {f:?}");
        };
        assert_eq!(
            request.prompt,
            if index == 0 { "first: " } else { "second: " }
        );
        assert_eq!(s.pending_input_request(), Some(request));
        if index < stop {
            out = s.resume_eval_request_with_input("2");
        }
    }
    cancel.cancel();
    out = s.resume_eval_request_with_input("unused");
    let result = finish(&mut s, out, Vec::new());
    println!("CANCEL\t{kind}\tstop={stop}\t{result:?}");
    assert_eq!(result.0.termination, "Some(Cancelled)");
    assert_eq!(
        result.0.value.unwrap().to_string(),
        "TerminatedEvaluation[\"Cancelled\"]"
    );
    assert_eq!(
        result.1,
        if stop == 0 {
            "List[1]"
        } else {
            "List[1, 2, 3]"
        }
    );
}
#[test]
fn cancel_rule_first() {
    cancel_case("Rule", 0);
}
#[test]
fn cancel_rule_second() {
    cancel_case("Rule", 1);
}

#[test]
fn cancel_delayed_first() {
    cancel_case("RuleDelayed", 0);
}
#[test]
fn cancel_delayed_second() {
    cancel_case("RuleDelayed", 1);
}
#[test]
fn abandon_without_answer() {
    let mut s = session(None);
    let cancel = s.request_cancel_handle();
    let out=s.eval_request(r#"Rule[h[x_,(AppendTo[log,1];Print["prefix"];Input["first: "];AppendTo[log,2]),(AppendTo[log,3];3)],9]"#);
    let Err(f) = out else {
        panic!("must suspend");
    };
    let SessionRequestError::InputRequired { request } = f.error else {
        panic!("must ask");
    };
    assert_eq!(request.prompt, "first: ");
    assert_eq!(s.pending_input_request(), Some(&request));
    cancel.cancel();
    let out = s
        .finish_cancelled_pending_evaluation_request()
        .unwrap()
        .expect("must abandon");
    let result = finish(&mut s, Ok(out), vec![request.prompt]);
    println!("ABANDON_WITHOUT_ANSWER\t{result:?}");
    assert_eq!(result.0.termination, "Some(Cancelled)");
    assert_eq!(result.1, "List[1]");
    assert_eq!(result.0.prints, vec!["prefix"]);
}
