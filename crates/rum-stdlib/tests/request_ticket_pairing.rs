use rum_runtime::{
    EvalError, EvalTerminationReason, RequestCancelHandle, Session, SessionRequestError,
};

fn session() -> Session {
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s
}

fn accepted_pause(s: &mut Session, source: &str) -> RequestCancelHandle {
    let accepted = s.accept_cancellable_request().unwrap();
    let handle = accepted.cancel_handle();
    let first = s.eval_accepted_request(accepted, source).unwrap_err();
    assert!(matches!(
        first.error,
        SessionRequestError::InputRequired { .. }
    ));
    assert_eq!(first.prints, ["old-prefix"]);
    handle
}

fn raw_resume(s: &mut Session, answer: &str) -> Result<Option<Vec<String>>, EvalError> {
    s.resume_inspect_rewrite_rules_with_input(answer)
        .map(|rules| rules.map(|items| items.into_iter().map(|r| r.lhs.to_string()).collect()))
}

fn ticket_free(s: &Session) -> bool {
    s.accept_cancellable_request().is_ok()
}

fn completed_request() -> (Session, RequestCancelHandle) {
    let mut s = session();
    let handle = accepted_pause(&mut s, "Print[\"old-prefix\"];InputString[\"old-owner\"]");
    assert_eq!(
        raw_resume(&mut s, "old-answer").unwrap(),
        Some(vec!["\"old-answer\"".into()])
    );
    assert!(s.pending_input_request().is_none());
    let late = s.resume_eval_request_with_input("late").unwrap_err();
    assert!(
        matches!(late.error, SessionRequestError::Other { ref message } if message == "no suspended evaluation is waiting for host input")
    );
    assert!(late.messages.is_empty() && late.prints.is_empty() && late.trace.is_empty());
    (s, handle)
}

#[test]
fn completed_ticket_cannot_cancel_a_later_raw_owner() {
    let (mut s, old) = completed_request();
    let raw = s
        .eval_text("WithCleanup[InputString[\"new-owner\"],Print[\"new-cleanup\"]]")
        .unwrap_err();
    assert!(matches!(raw, EvalError::InputRequired { .. }));
    old.cancel();
    let observed = s.cancellation_observer().is_cancelled();
    let cancelled = s.finish_cancelled_pending_evaluation_request().unwrap();
    let intact = s
        .pending_input_request()
        .is_some_and(|p| p.prompt == "new-owner");
    let resumed = if intact {
        Some(raw_resume(&mut s, "new-answer"))
    } else {
        None
    };
    let fresh = s.eval_request("6*7").unwrap();
    let free = ticket_free(&s);
    println!(
        "B734 C2 TICKET completed=(observer={observed},cancel={:?},raw_intact={intact},raw_resume={resumed:?},fresh={},fresh_termination={:?},ticket_free={free})",
        cancelled
            .as_ref()
            .map(|r| (r.value.to_string(), r.termination, r.prints.clone())),
        fresh.value,
        fresh.termination
    );
    assert!(!observed, "completed request handle must be inert");
    assert!(
        cancelled.is_none(),
        "old ticket must not abandon the later raw owner"
    );
    assert!(intact);
    assert_eq!(
        resumed.unwrap().unwrap(),
        Some(vec!["\"new-answer\"".into()])
    );
    assert_eq!(fresh.value.to_string(), "42");
    assert!(fresh.termination.is_none() && free);
}

#[test]
fn completed_ticket_is_free_when_no_owner_remains() {
    let (mut s, old) = completed_request();
    old.cancel();
    let cancelled = s.finish_cancelled_pending_evaluation_request().unwrap();
    let observer = s.cancellation_observer().is_cancelled();
    let free = ticket_free(&s);
    println!(
        "B734 C2 TICKET vacant=(cancel_none={},observer={observer},ticket_free={free},pending={:?})",
        cancelled.is_none(),
        s.pending_input_request()
    );
    assert!(cancelled.is_none());
    assert!(!observer && free && s.pending_input_request().is_none());
}

#[test]
fn raw_takeover_closes_an_unhonoured_cancel_before_evaluation() {
    let mut s = session();
    let old = accepted_pause(
        &mut s,
        "Print[\"old-prefix\"];InputString[\"old-owner\"];6*7",
    );
    old.cancel();
    assert!(s.cancellation_observer().is_cancelled());
    let raw = raw_resume(&mut s, "old-answer");
    let observer = s.cancellation_observer().is_cancelled();
    let fresh = s.eval_request("6*7").unwrap();
    let free = ticket_free(&s);
    println!(
        "B734 C2 TICKET cancelled_own=(raw={raw:?},observer={observer},fresh={},fresh_termination={:?},ticket_free={free})",
        fresh.value, fresh.termination
    );
    assert_eq!(raw.unwrap(), Some(vec!["42".into()]));
    assert!(!observer && free);
    assert_eq!(fresh.value.to_string(), "42");
    assert!(fresh.termination.is_none());
    old.cancel();
    assert!(!s.cancellation_observer().is_cancelled());
}

#[test]
fn deferred_reservation_cannot_abandon_a_reportless_raw_owner() {
    let mut s = session();
    let raw = s
        .eval_text("WithCleanup[InputString[\"raw-owner\"],Print[\"raw-cleanup\"]]")
        .unwrap_err();
    assert!(matches!(raw, EvalError::InputRequired { .. }));
    let reserved = s.request_cancel_handle();
    reserved.cancel();
    let cancelled = s.finish_cancelled_pending_evaluation_request().unwrap();
    let intact = s
        .pending_input_request()
        .is_some_and(|p| p.prompt == "raw-owner");
    let resumed = if intact {
        Some(raw_resume(&mut s, "raw-answer"))
    } else {
        None
    };
    let next = s.eval_request("6*7").unwrap();
    let fresh = s.eval_request("6*7").unwrap();
    println!(
        "B734 C2 TICKET deferred_raw=(cancel={:?},raw_intact={intact},raw_resume={resumed:?},next={},next_termination={:?},fresh={},ticket_free={})",
        cancelled
            .as_ref()
            .map(|r| (r.value.to_string(), r.termination, r.prints.clone())),
        next.value,
        next.termination,
        fresh.value,
        ticket_free(&s)
    );
    assert!(
        cancelled.is_none(),
        "a deferred reservation must not abandon a raw owner"
    );
    assert!(intact);
    assert_eq!(
        resumed.unwrap().unwrap(),
        Some(vec!["\"raw-answer\"".into()])
    );
    assert_eq!(next.termination, Some(EvalTerminationReason::Cancelled));
    assert_eq!(fresh.value.to_string(), "42");
    assert!(ticket_free(&s));
}

#[test]
fn deferred_reservation_cancels_a_request_owned_suspension() {
    let mut s = session();
    let first = s
        .eval_request("Print[\"published-prefix\"];InputString[\"request-owner\"]")
        .unwrap_err();
    assert_eq!(first.prints, ["published-prefix"]);
    let reserved = s.request_cancel_handle();
    reserved.cancel();
    let cancelled = s
        .finish_cancelled_pending_evaluation_request()
        .unwrap()
        .unwrap();
    println!(
        "B734 C2 TICKET deferred_request=(value={},termination={:?},prints={:?},messages={:?},pending={:?},observer={},ticket_free={})",
        cancelled.value,
        cancelled.termination,
        cancelled.prints,
        cancelled.messages,
        s.pending_input_request(),
        s.cancellation_observer().is_cancelled(),
        ticket_free(&s)
    );
    assert_eq!(
        cancelled.termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(cancelled.prints, ["published-prefix"]);
    assert!(s.pending_input_request().is_none());
    assert!(!s.cancellation_observer().is_cancelled() && ticket_free(&s));
}

#[test]
fn installed_cancelled_entry_still_retires_reportless_work() {
    let mut s = session();
    assert!(matches!(
        s.eval_text("WithCleanup[InputString[\"raw-owner\"],Print[\"raw-cleanup\"]]"),
        Err(EvalError::InputRequired { .. })
    ));
    let accepted = s.accept_cancellable_request().unwrap();
    accepted.cancel_handle().cancel();
    let cancelled = s.eval_accepted_request(accepted, "").unwrap();
    println!(
        "B734 C2 TICKET explicit_entry=(value={},termination={:?},prints={:?},pending={:?},ticket_free={})",
        cancelled.value,
        cancelled.termination,
        cancelled.prints,
        s.pending_input_request(),
        ticket_free(&s)
    );
    assert_eq!(
        cancelled.termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(cancelled.prints, ["raw-cleanup"]);
    assert!(s.pending_input_request().is_none() && ticket_free(&s));
}

#[test]
fn normal_request_resume_keeps_its_ticket_and_published_prefix() {
    let mut s = session();
    let handle = accepted_pause(&mut s, "Print[\"old-prefix\"];InputString[\"old-owner\"]");
    assert!(!ticket_free(&s));
    let resumed = s.resume_eval_request_with_input("reply").unwrap();
    handle.cancel();
    println!(
        "B734 C2 TICKET normal_resume=(value={},termination={:?},prints={:?},observer={},ticket_free={})",
        resumed.value,
        resumed.termination,
        resumed.prints,
        s.cancellation_observer().is_cancelled(),
        ticket_free(&s)
    );
    assert_eq!(resumed.value.to_string(), "\"reply\"");
    assert_eq!(resumed.prints, ["old-prefix"]);
    assert!(resumed.termination.is_none());
    assert!(!s.cancellation_observer().is_cancelled() && ticket_free(&s));
}

#[test]
fn normal_request_cancellation_keeps_its_published_prefix() {
    let mut s = session();
    let handle = accepted_pause(
        &mut s,
        "Print[\"old-prefix\"];WithCleanup[InputString[\"old-owner\"],Print[\"own-cleanup\"]]",
    );
    handle.cancel();
    let cancelled = s.resume_eval_request_with_input("unused").unwrap();
    println!(
        "B734 C2 TICKET normal_cancel=(value={},termination={:?},prints={:?},messages={:?},pending={:?},observer={},ticket_free={})",
        cancelled.value,
        cancelled.termination,
        cancelled.prints,
        cancelled.messages,
        s.pending_input_request(),
        s.cancellation_observer().is_cancelled(),
        ticket_free(&s)
    );
    assert_eq!(
        cancelled.termination,
        Some(EvalTerminationReason::Cancelled)
    );
    assert_eq!(cancelled.prints, ["old-prefix", "own-cleanup"]);
    assert!(s.pending_input_request().is_none());
    assert!(!s.cancellation_observer().is_cancelled() && ticket_free(&s));
}

#[test]
fn idle_reservation_waits_for_its_next_admitted_request() {
    let mut s = session();
    let reserved = s.request_cancel_handle();
    let idle = s.finish_cancelled_pending_evaluation_request().unwrap();
    println!(
        "B734 C3 NINTH idle=(handoff_none={},ticket_free={},pending={:?},observer={})",
        idle.is_none(),
        ticket_free(&s),
        s.pending_input_request(),
        s.cancellation_observer().is_cancelled()
    );
    assert!(idle.is_none());
    assert!(!ticket_free(&s));
    assert!(s.pending_input_request().is_none());
    assert!(!s.cancellation_observer().is_cancelled());

    let raw = s
        .eval_text("WithCleanup[InputString[\"later-raw-owner\"],Print[\"later-raw-cleanup\"]]")
        .unwrap_err();
    println!(
        "B734 C3 NINTH raw=(error={raw:?},pending={:?})",
        s.pending_input_request()
    );
    assert!(matches!(raw, EvalError::InputRequired { .. }));
    assert_eq!(s.pending_input_request().unwrap().prompt, "later-raw-owner");
    reserved.cancel();
    assert!(s.cancellation_observer().is_cancelled());
    let handoff = s.finish_cancelled_pending_evaluation_request().unwrap();
    println!(
        "B734 C3 NINTH handoff=(value={:?},messages={:?},prints={:?},pending={:?},observer={})",
        handoff.as_ref().map(|r| r.value.to_string()),
        handoff.as_ref().map(|r| &r.messages),
        handoff.as_ref().map(|r| &r.prints),
        s.pending_input_request(),
        s.cancellation_observer().is_cancelled()
    );
    assert!(
        handoff.is_none(),
        "an idle reservation must not acquire a later report-less owner"
    );
    assert_eq!(s.pending_input_request().unwrap().prompt, "later-raw-owner");
    assert!(s.cancellation_observer().is_cancelled());
    let raw = raw_resume(&mut s, "raw-answer").unwrap();
    assert_eq!(raw, Some(vec!["\"raw-answer\"".into()]));
    assert!(s.pending_input_request().is_none());

    let admitted = s.eval_request("6*7").unwrap();
    println!(
        "B734 C3 NINTH admitted=(value={},termination={:?},messages={:?},prints={:?})",
        admitted.value, admitted.termination, admitted.messages, admitted.prints
    );
    assert_eq!(admitted.termination, Some(EvalTerminationReason::Cancelled));
    assert_eq!(
        admitted
            .messages
            .iter()
            .map(|m| format!("{}::{}", m.symbol, m.tag))
            .collect::<Vec<_>>(),
        ["General::cancel"]
    );
    reserved.cancel();
    assert!(!s.cancellation_observer().is_cancelled());
    let fresh = s.eval_request("6*7").unwrap();
    println!(
        "B734 C3 NINTH fresh=(value={},termination={:?},ticket_free={},pending={:?},observer={})",
        fresh.value,
        fresh.termination,
        ticket_free(&s),
        s.pending_input_request(),
        s.cancellation_observer().is_cancelled()
    );
    assert_eq!(fresh.value.to_string(), "42");
    assert!(fresh.termination.is_none());
    assert!(ticket_free(&s));
    assert!(!s.cancellation_observer().is_cancelled());
    assert!(s.pending_input_request().is_none());
}

#[test]
fn queued_input_retirement_ends_request_ticket() {
    let mut s = session();
    let old = accepted_pause(
        &mut s,
        "Print[\"old-prefix\"];WithCleanup[InputString[\"old-owner\"],Print[\"old-cleanup\"]]",
    );
    assert_eq!(s.pending_input_request().unwrap().prompt, "old-owner");
    s.queue_input_response("queued-answer");
    assert!(s.pending_input_request().is_none());
    let drained = s.eval_text("InputString[\"consume-queued\"]").unwrap();
    let free = ticket_free(&s);
    let observed = s.cancellation_observer().is_cancelled();
    println!(
        "B734 C4 TENTH discarded=(value={},termination={:?},prints={:?},pending={:?},ticket_free={free},observer={observed})",
        drained.value,
        drained.termination,
        drained.prints,
        s.pending_input_request()
    );
    assert_eq!(drained.value.to_string(), "\"queued-answer\"");
    assert_eq!(drained.prints, ["old-cleanup"]);
    assert!(drained.termination.is_none() && s.pending_input_request().is_none());
    assert!(
        free,
        "a retired request report and owner must leave a free ticket"
    );
    assert!(!observed);

    let late = s.resume_eval_request_with_input("late").unwrap_err();
    assert!(
        matches!(late.error, SessionRequestError::Other { ref message } if message == "no suspended evaluation is waiting for host input")
    );
    assert!(late.messages.is_empty() && late.prints.is_empty() && late.trace.is_empty());
    old.cancel();
    assert!(!s.cancellation_observer().is_cancelled());
    assert!(
        s.finish_cancelled_pending_evaluation_request()
            .unwrap()
            .is_none()
    );

    let raw = s
        .eval_text("WithCleanup[InputString[\"new-owner\"],Print[\"new-cleanup\"]]")
        .unwrap_err();
    assert!(matches!(raw, EvalError::InputRequired { .. }));
    assert_eq!(s.pending_input_request().unwrap().prompt, "new-owner");
    old.cancel();
    assert!(!s.cancellation_observer().is_cancelled());
    assert!(
        s.finish_cancelled_pending_evaluation_request()
            .unwrap()
            .is_none()
    );
    assert_eq!(s.pending_input_request().unwrap().prompt, "new-owner");
    let resumed = raw_resume(&mut s, "new-answer").unwrap();
    assert_eq!(resumed, Some(vec!["\"new-answer\"".into()]));
    assert!(s.pending_input_request().is_none());
    let fresh = s.eval_request("6*7").unwrap();
    let free = ticket_free(&s);
    println!(
        "B734 C4 TENTH completed=(raw_resume={resumed:?},fresh={},termination={:?},ticket_free={free},observer={})",
        fresh.value,
        fresh.termination,
        s.cancellation_observer().is_cancelled()
    );
    assert_eq!(fresh.value.to_string(), "42");
    assert!(fresh.termination.is_none() && free);
    assert!(!s.cancellation_observer().is_cancelled());
}

#[test]
fn queued_input_discard_of_raw_owner_stays_reportless() {
    let mut s = session();
    let raw = s
        .eval_text("WithCleanup[InputString[\"raw-owner\"],Print[\"raw-cleanup\"]]")
        .unwrap_err();
    assert!(matches!(raw, EvalError::InputRequired { .. }));
    s.queue_input_response("queued-answer");
    let drained = s.eval_text("InputString[\"consume-queued\"]").unwrap();
    let free = ticket_free(&s);
    println!(
        "B734 C4 CONTROL raw_discard=(value={},prints={:?},pending={:?},ticket_free={free},observer={})",
        drained.value,
        drained.prints,
        s.pending_input_request(),
        s.cancellation_observer().is_cancelled()
    );
    assert_eq!(drained.value.to_string(), "\"queued-answer\"");
    assert_eq!(drained.prints, ["raw-cleanup"]);
    assert!(drained.termination.is_none() && s.pending_input_request().is_none() && free);
    assert!(!s.cancellation_observer().is_cancelled());
    assert!(
        s.finish_cancelled_pending_evaluation_request()
            .unwrap()
            .is_none()
    );
    let fresh = s.eval_request("6*7").unwrap();
    assert_eq!(fresh.value.to_string(), "42");
    assert!(fresh.termination.is_none() && ticket_free(&s));
}

#[test]
fn request_owned_cancellation_still_drains_cleanup_before_ticket_finish() {
    let mut s = session();
    let accepted = s.accept_cancellable_request().unwrap();
    let cancel = accepted.cancel_handle();
    let first = s
        .eval_accepted_request(
            accepted,
            "WithCleanup[InputString[\"cancel-owner\"],Print[\"cancel-cleanup\"]]",
        )
        .unwrap_err();
    assert!(matches!(
        first.error,
        SessionRequestError::InputRequired { .. }
    ));
    assert!(first.prints.is_empty());
    cancel.cancel();
    assert!(s.cancellation_observer().is_cancelled());
    let retired = s
        .finish_cancelled_pending_evaluation_request()
        .unwrap()
        .expect("request-owned cancellation must retire its suspension");
    let tags = retired
        .messages
        .iter()
        .map(|m| format!("{}::{}", m.symbol, m.tag))
        .collect::<Vec<_>>();
    let free = ticket_free(&s);
    println!(
        "B734 C4 CONTROL request_cancel=(value={},termination={:?},prints={:?},tags={tags:?},pending={:?},ticket_free={free},observer={})",
        retired.value,
        retired.termination,
        retired.prints,
        s.pending_input_request(),
        s.cancellation_observer().is_cancelled()
    );
    assert_eq!(
        retired.value.to_string(),
        "TerminatedEvaluation[\"Cancelled\"]"
    );
    assert_eq!(retired.termination, Some(EvalTerminationReason::Cancelled));
    assert_eq!(retired.prints, ["cancel-cleanup"]);
    assert_eq!(tags, ["General::cancel"]);
    assert!(free && s.pending_input_request().is_none());
    assert!(!s.cancellation_observer().is_cancelled());
    cancel.cancel();
    assert!(!s.cancellation_observer().is_cancelled());
    let fresh = s.eval_request("6*7").unwrap();
    assert_eq!(fresh.value.to_string(), "42");
    assert!(fresh.termination.is_none() && ticket_free(&s));
}
