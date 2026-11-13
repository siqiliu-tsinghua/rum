use rum_core::{Expr, SharedArgs};
use rum_runtime::{BuiltinSpec, EvalError, MemoryGrowthContract, Session, SessionRequestError};
use std::sync::atomic::{AtomicU8, Ordering};

static OBSERVED: AtomicU8 = AtomicU8::new(0);

fn free(s: &Session) -> bool {
    s.accept_cancellable_request().is_ok()
}

fn nested_consume(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    let retirement_handle = s.request_cancel_handle();
    // The caller's queued response is still present while its cleanup runs.
    // Consume it through the public input path so the nested request really pauses.
    let consumed = s.eval_text("InputString[\"cleanup-consumes-queued\"]")?;
    assert_eq!(consumed.value.to_string(), "\"outer-answer\"");
    let nested = s.eval_request("InputString[\"nested-owner\"]").unwrap_err();
    assert!(matches!(
        nested.error,
        SessionRequestError::InputRequired { .. }
    ));
    assert_eq!(s.pending_input_request().unwrap().prompt, "nested-owner");
    assert!(
        !free(s),
        "the outer retirement ticket is still active before the raw takeover"
    );
    // Consume the nested, request-created suspension through the public raw
    // inspection resume: change 2's finish authority on a borrowed ticket.
    let resumed = s.resume_inspect_rewrite_rules_with_input("nested-answer")?;
    assert!(resumed.is_some());
    assert!(s.pending_input_request().is_none());
    let free_after_inner = free(s);
    retirement_handle.cancel();
    let cancellation_seen = s.cancellation_observer().is_cancelled();
    OBSERVED.store(
        1 | u8::from(free_after_inner) * 2 | u8::from(cancellation_seen) * 4,
        Ordering::SeqCst,
    );
    println!(
        "B734 C4 REENTRANT_CONSUME inside=(ticket_free_after_raw_takeover={free_after_inner},retirement_cancel_observed={cancellation_seen},pending={:?})",
        s.pending_input_request()
    );
    // Replenish the outer input response before its checked pop resumes.
    s.queue_input_response("outer-answer");
    Ok(Expr::system_symbol("Null"))
}

#[test]
fn raw_takeover_during_cleanup_must_not_finish_outer_retirement_ticket() {
    OBSERVED.store(0, Ordering::SeqCst);
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s.register_builtin(BuiltinSpec::ordinary(
        "NestedConsume734",
        "Public reentrant raw-takeover fixture.",
        nested_consume,
        MemoryGrowthContract::Incremental,
    ));
    let accepted = s.accept_cancellable_request().unwrap();
    let first = s.eval_accepted_request(
        accepted,
        "Print[\"old-prefix\"];WithCleanup[WithCleanup[InputString[\"old-owner\"],NestedConsume734[]],Print[\"outer-cleanup\"]]",
    ).unwrap_err();
    assert!(matches!(
        first.error,
        SessionRequestError::InputRequired { .. }
    ));
    assert_eq!(first.prints, ["old-prefix"]);
    assert_eq!(s.pending_input_request().unwrap().prompt, "old-owner");
    s.queue_input_response("outer-answer");
    let result = s.eval_text("InputString[\"consume-outer\"]");
    let flags = OBSERVED.load(Ordering::SeqCst);
    let summary = result.as_ref().map(|r| {
        (
            r.value.to_string(),
            r.termination,
            r.prints.clone(),
            r.messages
                .iter()
                .map(|m| format!("{}::{}", m.symbol, m.tag))
                .collect::<Vec<_>>(),
        )
    });
    println!(
        "B734 C4 REENTRANT_CONSUME outer=(result={summary:?},observations={flags},pending={:?},ticket_free={},observer={})",
        s.pending_input_request(),
        free(&s),
        s.cancellation_observer().is_cancelled()
    );
    assert_ne!(
        flags & 1,
        0,
        "the real nested request/raw-takeover sequence must execute"
    );
    assert_eq!(
        flags & 2,
        0,
        "a raw takeover of a nested suspension must keep the outer retirement ticket installed until its drain completes"
    );
    assert_ne!(
        flags & 4,
        0,
        "a later cancel of the retirement generation must remain observable during cleanup"
    );
    let result = result.unwrap();
    assert!(
        !result.prints.iter().any(|p| p == "outer-cleanup"),
        "the later cancel must skip the remaining outer cleanup"
    );
}
