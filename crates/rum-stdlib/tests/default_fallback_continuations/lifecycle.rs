use crate::oracle::{finish, session};
use rum_runtime::{ClosurePrintSink, RuntimeInputKind, Session, SessionRequestError};
use std::sync::{Arc, Mutex};

const SOURCE: &str = r#"n=0;Default[d]:=(AppendTo[log,40];Print["body"];7)/;(n=n+1;AppendTo[log,n];If[n==1,Print["direct"];Message[probe::tag];InputString["direct: "];AppendTo[log,10]];If[n==2,Print["fallback"];Message[probe::tag];InputString["fallback: "];AppendTo[log,30]];n>=3);Default[d]"#;

fn at_pause(s: &mut Session, stop: usize) {
    let mut out = s.eval_request(SOURCE);
    for i in 0..=stop {
        let Err(failure) = &out else {
            panic!("must suspend at {i}: {out:?}")
        };
        let SessionRequestError::InputRequired { request } = &failure.error else {
            panic!("expected input: {failure:?}")
        };
        assert_eq!(request.kind, RuntimeInputKind::InputString);
        assert_eq!(
            request.prompt,
            if i == 0 { "direct: " } else { "fallback: " }
        );
        assert_eq!(s.pending_input_request(), Some(request));
        assert!(s.reject_overlapping_evaluation_request().is_err());
        if i < stop {
            out = s.resume_eval_request_with_input("answer");
        }
    }
}

fn cancel(stop: usize, without_answer: bool) {
    let mut s = session(None);
    let handle = s.request_cancel_handle();
    at_pause(&mut s, stop);
    handle.cancel();
    let out = if without_answer {
        Ok(s.finish_cancelled_pending_evaluation_request()
            .unwrap()
            .expect("pending work must abandon"))
    } else {
        s.resume_eval_request_with_input("unused")
    };
    let (view, log, _) = finish(&mut s, out, vec![]);
    println!("CANCEL\tstop={stop}\twithout_answer={without_answer}\t{view:?}\t{log}");
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
            "List[1, 10, 2]"
        }
    );
    assert_eq!(
        view.prints,
        if stop == 0 {
            vec!["direct"]
        } else {
            vec!["direct", "fallback"]
        }
    );
    let mut messages = vec![("probe", "tag"); stop + 1];
    messages.push(("General", "cancel"));
    assert_eq!(
        view.messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        messages
    );
    assert!(s.reject_overlapping_evaluation_request().is_ok());
}

fn reset(stop: usize) {
    let mut s = session(None);
    let old_handle = s.request_cancel_handle();
    let prints = Arc::new(Mutex::new(Vec::new()));
    let captured = prints.clone();
    s.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |text: &str| {
            captured.lock().unwrap().push(text.to_owned());
        },
    )));
    at_pause(&mut s, stop);
    let before = prints.lock().unwrap().clone();
    assert_eq!(
        before,
        if stop == 0 {
            vec!["direct"]
        } else {
            vec!["direct", "fallback"]
        }
    );
    // HostSession::reset rebuilds and replaces its inner Session in this way.
    s = session(None);
    old_handle.cancel();
    assert_eq!(
        *prints.lock().unwrap(),
        before,
        "abandon runs no later guard or body"
    );
    assert!(s.pending_input_request().is_none());
    assert!(s.reject_overlapping_evaluation_request().is_ok());
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
    assert_eq!(
        s.eval_request("{n,log}").unwrap().value.to_string(),
        "List[n, List[]]"
    );
    println!("RESET\tstop={stop}\tprints={before:?}\towner=None\tfresh=2");
}

#[test]
fn cancel_direct() {
    cancel(0, false);
}
#[test]
fn cancel_fallback() {
    cancel(1, false);
}
#[test]
fn abandon_direct_without_answer() {
    cancel(0, true);
}
#[test]
fn abandon_fallback_without_answer() {
    cancel(1, true);
}
#[test]
fn reset_direct() {
    reset(0);
}
#[test]
fn reset_fallback() {
    reset(1);
}
