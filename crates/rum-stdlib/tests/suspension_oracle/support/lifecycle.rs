use super::session;
use rum_runtime::{
    ClosurePrintSink, EvalTerminationReason, RuntimeInputRequest, Session, SessionRequestError,
};
use std::sync::{Arc, Mutex};

// Checks a continuation owner at either of its two planned pauses. The caller
// supplies the same session configuration as its completion descriptor.
pub fn exercise(
    make: impl Fn() -> Session,
    source: &str,
    plans: &[(RuntimeInputRequest, String)],
) -> usize {
    assert!(
        plans.len() >= 2,
        "lifecycle requires first and second pauses"
    );
    let source =
        format!("WithCleanup[({source});Print[\"oracle suffix\"],Print[\"oracle cleanup\"]]");
    for stop in 0..2 {
        for reset in [false, true] {
            let mut s = make();
            let delivered = Arc::new(Mutex::new(Vec::<String>::new()));
            let sink = delivered.clone();
            s.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
                move |text: &str| sink.lock().unwrap().push(text.to_owned()),
            )));
            let handle = s.request_cancel_handle();
            let mut out = s.eval_request(&source);
            for (i, (planned, answer)) in plans.iter().enumerate().take(stop + 1) {
                let Err(f) = &out else {
                    panic!("lifecycle did not pause at {i}: {out:?}")
                };
                let SessionRequestError::InputRequired { request } = &f.error else {
                    panic!("lifecycle failure: {f:?}")
                };
                assert_eq!(request, planned);
                assert_eq!(s.pending_input_request(), Some(planned));
                assert!(s.eval_request("Global`oracleOverlap=1").is_err());
                assert_eq!(s.pending_input_request(), Some(planned));
                if i < stop {
                    out = s.resume_eval_request_with_input(answer);
                }
            }
            let prefix = delivered.lock().unwrap().clone();
            assert!(
                !prefix
                    .iter()
                    .any(|p| p == "oracle suffix" || p == "oracle cleanup")
            );
            if reset {
                // The Session counterpart of HostSession::reset (whose host
                // lifecycle is tested in rum-host): replace from configuration.
                s.queue_input_response("must be discarded by reset");
                s = make();
                handle.cancel();
                assert_eq!(
                    *delivered.lock().unwrap(),
                    prefix,
                    "reset must run no guest cleanup"
                );
            } else {
                handle.cancel();
                let cancelled = s
                    .finish_cancelled_pending_evaluation_request()
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    cancelled.termination,
                    Some(EvalTerminationReason::Cancelled)
                );
                assert_eq!(
                    cancelled.value.to_string(),
                    "TerminatedEvaluation[\"Cancelled\"]"
                );
                let after = delivered.lock().unwrap();
                assert!(after.starts_with(&prefix));
                assert!(!after.iter().any(|p| p == "oracle suffix"));
                assert_eq!(
                    after
                        .iter()
                        .filter(|p| p.as_str() == "oracle cleanup")
                        .count(),
                    1
                );
            }
            assert!(s.pending_input_request().is_none());
            assert!(s.reject_overlapping_evaluation_request().is_ok());
            session::fresh(&mut s).unwrap();
            session::empty_queue(&mut s).unwrap();
        }
    }
    // Cancellation, reset, pending rejection at both pauses; completion is
    // separately driven through exactly the same descriptor by acceptance.
    6
}
