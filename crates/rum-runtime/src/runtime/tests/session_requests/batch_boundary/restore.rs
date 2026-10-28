use super::*;
use std::sync::{Arc, Mutex};

#[test]
fn terminal_restore_retires_identity_and_all_owner_state() {
    for source in [
        "x=1\nInputString[\"p: \"]\ny=2",
        "x=1\nGet[\"Outer`\"]\ny=2",
    ] {
        let mut s = session(&[
            ("Outer.rum", "x=2\nGet[\"Inner`\"]\nx=3"),
            ("Inner.rum", "x=4\nInputString[\"p: \"]\nx=5"),
        ]);
        let old = s.request_cancel_handle();
        let error = s.restore_snapshot_request(source).unwrap_err().error;
        assert!(matches!(error, SessionRequestError::Other { message }
            if message == "snapshot restore unexpectedly requested input"));
        idle(&mut s);
        assert!(!s.restoring_snapshot);
        assert_eq!(
            s.eval_request("{x,y}").unwrap().value.to_string(),
            "List[1, y]"
        );
        let pending = s.eval_request("InputString[\"fresh: \"]").unwrap_err();
        assert!(matches!(
            pending.error,
            SessionRequestError::InputRequired { .. }
        ));
        old.cancel();
        assert_eq!(
            s.resume_eval_request_with_input("fresh").unwrap().value,
            Expr::string("fresh")
        );
        idle(&mut s);
    }
}

#[test]
fn restore_admission_rejection_preserves_unrelated_request() {
    let mut s = session(&[]);
    let old = s.request_cancel_handle();
    let pending = s.eval_request("InputString[\"original: \"]").unwrap_err();
    assert!(matches!(
        pending.error,
        SessionRequestError::InputRequired { .. }
    ));
    let generation = s.pending_suspension_generation();
    let failure = s
        .restore_snapshot_request("x=1\nInputString[\"restore: \"]")
        .unwrap_err();
    assert!(
        matches!(failure.error, SessionRequestError::Other { message }
        if message.contains("cannot start a new evaluation"))
    );
    assert_eq!(s.pending_suspension_generation(), generation);
    assert!(matches!(
        s.pending_request,
        Some(PendingRequest::Resumable(_))
    ));
    assert_eq!(s.pending_input_request().unwrap().prompt, "original: ");
    assert!(s.active_accepted_request.is_some());
    assert!(!s.restoring_snapshot);
    old.cancel();
    assert_eq!(
        s.resume_eval_request_with_input("unused")
            .unwrap()
            .termination,
        Some(EvalTerminationReason::Cancelled)
    );
    idle(&mut s);
    assert_eq!(s.eval_request("x").unwrap().value.to_string(), "x");
}

#[test]
fn restore_abandons_nested_cleanup_and_file_rollbacks_once() {
    let mut s = session(&[
        (
            "Outer.rum",
            "x=2\nWithCleanup[Get[\"Inner`\"], Print[x]]\nx=3",
        ),
        (
            "Inner.rum",
            "x=4\nWithCleanup[InputString[\"p: \"], Print[x]]\nx=5",
        ),
    ]);
    let prints = Arc::new(Mutex::new(Vec::new()));
    let sink = prints.clone();
    s.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |line| {
            sink.lock().unwrap().push(line.to_string());
        },
    )));
    let failure = s
        .restore_snapshot_request("x=1\nGet[\"Outer`\"]\ny=2")
        .unwrap_err();
    assert!(
        matches!(failure.error, SessionRequestError::Other { message }
        if message == "snapshot restore unexpectedly requested input")
    );
    assert_eq!(*prints.lock().unwrap(), vec!["4", "2"]);
    idle(&mut s);
    assert_eq!(
        s.eval_request("{x,y,$InputFileName}")
            .unwrap()
            .value
            .to_string(),
        "List[1, y, \"\"]"
    );
    assert_eq!(*prints.lock().unwrap(), vec!["4", "2"]);
}
