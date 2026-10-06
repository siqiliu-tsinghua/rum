use crate::oracle::{finish, session};
use rum_runtime::{RuntimeInputKind, SessionRequestError};

fn cancel(stop: usize, without_answer: bool) {
    let source = r#"x=0;y=0;a={0,0};b=0;{x,a[[(AppendTo[log,1];Print["first"];Message[probe::tag];InputString["p: "];AppendTo[log,11];1)]],y,a[[(AppendTo[log,2];Print["second"];Message[probe::tag];InputString["q: "];AppendTo[log,22];2)]],b}={5,7,6,8,9}"#;
    let mut s = session(None);
    let handle = s.request_cancel_handle();
    let mut out = s.eval_request(source);
    let mut prompts = Vec::new();
    for i in 0..=stop {
        let Err(failure) = &out else {
            panic!("must suspend at {i}: {out:?}")
        };
        let SessionRequestError::InputRequired { request } = &failure.error else {
            panic!("expected input: {failure:?}")
        };
        assert_eq!(request.kind, RuntimeInputKind::InputString);
        assert_eq!(request.prompt, if i == 0 { "p: " } else { "q: " });
        assert_eq!(s.pending_input_request(), Some(request));
        assert!(s.reject_overlapping_evaluation_request().is_err());
        prompts.push(request.prompt.clone());
        if i < stop {
            out = s.resume_eval_request_with_input("answer");
        }
    }
    handle.cancel();
    out = if without_answer {
        Ok(s.finish_cancelled_pending_evaluation_request()
            .unwrap()
            .expect("pending work must abandon"))
    } else {
        s.resume_eval_request_with_input("unused")
    };
    let (view, log, prompts) = finish(&mut s, out, prompts);
    assert!(s.reject_overlapping_evaluation_request().is_ok());
    let stored = s.eval_request("{x,y,a,b}").unwrap().value.to_string();
    println!(
        "LIFECYCLE\tstop={stop}\twithout_answer={without_answer}\t{view:?}\t{log}\t{prompts:?}\t{stored}"
    );
    assert_eq!(view.termination, "Some(Cancelled)");
    assert_eq!(
        view.value.unwrap().to_string(),
        "TerminatedEvaluation[\"Cancelled\"]"
    );
    assert_eq!(
        log,
        if stop == 0 {
            "List[1]"
        } else {
            "List[1, 11, 2]"
        }
    );
    assert_eq!(
        stored,
        if stop == 0 {
            "List[5, 0, List[0, 0], 0]"
        } else {
            "List[5, 6, List[7, 0], 0]"
        }
    );
    let expected_prints = if stop == 0 {
        vec!["first"]
    } else {
        vec!["first", "second"]
    };
    assert_eq!(view.prints, expected_prints);
    let mut expected_messages = vec![("probe", "tag"); stop + 1];
    expected_messages.push(("General", "cancel"));
    assert_eq!(
        view.messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        expected_messages
    );
}
#[test]
fn cancel_at_first_target() {
    cancel(0, false);
}
#[test]
fn cancel_at_second_target() {
    cancel(1, false);
}
#[test]
fn abandon_first_target_without_answer() {
    cancel(0, true);
}
#[test]
fn abandon_second_target_without_answer() {
    cancel(1, true);
}
