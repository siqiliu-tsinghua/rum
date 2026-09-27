//! Host reset/cancel dispose of pending input without running abandoned guest
//! code. Exercise that contract with the delayed-option continuation.
use super::super::*;
use super::common::successful_eval;

#[test]
fn option_rule_host_cancel_and_reset_dispose_pending_input() {
    for armed in [false, true] {
        let mut limits = HostResourceLimits::default();
        limits.eval_memory_limit = if armed { Some(1_000_000_000) } else { None };
        let mut session = HostSession::builder()
            .with_config(HostConfig::default().with_resource_limits(limits))
            .build();
        let source = "seen543 = 0; StringReplace[\"A\", \"a\" -> \"b\", IgnoreCase :> (InputString[\"pause\"]; seen543 = 1; True)]";
        assert!(matches!(
            session.eval(source),
            HostEvalOutcome::InputRequired(_)
        ));
        session.cancel_handle().cancel();
        let cancelled = successful_eval(session.resume_with_input("yes"));
        assert!(cancelled.termination.is_some());
        assert!(session.pending_input_request().is_none());
        assert_eq!(successful_eval(session.eval("seen543")).value.text, "0");
        assert!(matches!(
            session.eval(source),
            HostEvalOutcome::InputRequired(_)
        ));
        session.reset();
        assert!(session.pending_input_request().is_none());
        assert_eq!(
            successful_eval(session.eval("seen543")).value.text,
            "seen543"
        );
        session.queue_input_response("yes");
        assert_eq!(successful_eval(session.eval(source)).value.text, "\"b\"");
        assert_eq!(successful_eval(session.eval("seen543")).value.text, "1");
    }
}
