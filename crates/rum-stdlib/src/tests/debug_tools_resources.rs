use crate::install_core;
use rum_runtime::{RuntimeInputKind, Session, SessionRequestError, SessionRequestFailure};
use std::thread;
use std::time::{Duration, Instant};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn timing_and_absolute_timing_return_elapsed_and_value() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Part[Timing[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[Timing[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("Part[AbsoluteTiming[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[AbsoluteTiming[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("Part[RepeatedTiming[1 + 1], 2]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("NumberQ[Part[RepeatedTiming[1 + 1], 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn time_and_memory_constraints_cover_current_first_pass_resource_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("TimeConstrained[Pause[1/100]; 7, 1/1000, timeout]")
            .unwrap()
            .value
            .to_string(),
        "timeout"
    );
    assert_eq!(
        session
            .eval_text("MemoryConstrained[Range[100], 20, memfail]")
            .unwrap()
            .value
            .to_string(),
        "memfail"
    );
    assert_eq!(
        session
            .eval_text("IntegerQ[MemoryInUse[]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

fn assert_constraint_input(
    session: &Session,
    failure: SessionRequestFailure,
    kind: RuntimeInputKind,
    prompt: &str,
) {
    let SessionRequestError::InputRequired { request } = failure.error else {
        panic!("expected an active TimeConstrained body to suspend, got {failure:?}");
    };
    assert_eq!(request.kind, kind);
    assert_eq!(request.prompt, prompt);
    assert_eq!(session.pending_input_request(), Some(&request));
}

fn wait_past_constraint_limit() {
    // The body has already suspended under a 100 ms limit. Waiting another
    // 125 ms must cross its original deadline, using only monotonic time.
    let waiting = Instant::now();
    let duration = Duration::from_millis(125);
    while let Some(remaining) = duration.checked_sub(waiting.elapsed()) {
        thread::sleep(remaining);
    }
}

fn assert_constraint_restored(session: &mut Session) {
    let response = session.eval_request("Pause[1/100]; 9").unwrap();
    assert_eq!(response.value.to_string(), "9");
    assert_eq!(response.termination, None);
    assert!(response.messages.is_empty());
    assert!(session.pending_input_request().is_none());
}

#[test]
fn time_constrained_charges_input_suspension_to_original_deadline() {
    for (name, kind) in [
        ("Input", RuntimeInputKind::Input),
        ("InputString", RuntimeInputKind::InputString),
    ] {
        let mut session = core_session();
        let source = format!("TimeConstrained[{name}[\"deadline\"]; 7, 1/10, timedOut511]");
        let failure = session.eval_request(&source).unwrap_err();
        assert_constraint_input(&session, failure, kind, "deadline");

        wait_past_constraint_limit();
        let resumed = session.resume_eval_request_with_input("5");
        eprintln!("TimeConstrained {name}, expired suspension: {resumed:?}");
        let response = resumed.unwrap();
        assert_eq!(response.value.to_string(), "timedOut511");
        assert_eq!(response.termination, None);
        assert!(response.messages.is_empty());
        assert_constraint_restored(&mut session);
    }
}

#[test]
fn time_constrained_immediate_input_resume_finishes_before_deadline() {
    for (name, kind, expected) in [
        ("Input", RuntimeInputKind::Input, "5"),
        ("InputString", RuntimeInputKind::InputString, "\"5\""),
    ] {
        let mut session = core_session();
        let source = format!("TimeConstrained[{name}[\"control\"], 10, timedOut511]");
        let failure = session.eval_request(&source).unwrap_err();
        assert_constraint_input(&session, failure, kind, "control");

        let resumed = session.resume_eval_request_with_input("5");
        eprintln!("TimeConstrained {name}, immediate resume: {resumed:?}");
        let response = resumed.unwrap();
        assert_eq!(response.value.to_string(), expected);
        assert_eq!(response.termination, None);
        assert!(response.messages.is_empty());
        assert_constraint_restored(&mut session);
    }
}

#[test]
fn time_constrained_preserves_deadline_through_two_input_suspensions() {
    for (name, kind) in [
        ("Input", RuntimeInputKind::Input),
        ("InputString", RuntimeInputKind::InputString),
    ] {
        let mut session = core_session();
        let source =
            format!("TimeConstrained[{name}[\"first\"]; {name}[\"second\"]; 7, 1/10, timedOut511]");
        let first = session.eval_request(&source).unwrap_err();
        assert_constraint_input(&session, first, kind, "first");
        let second = session.resume_eval_request_with_input("5").unwrap_err();
        assert_constraint_input(&session, second, kind, "second");

        wait_past_constraint_limit();
        let resumed = session.resume_eval_request_with_input("6");
        eprintln!("TimeConstrained {name}, expired second suspension: {resumed:?}");
        let response = resumed.unwrap();
        assert_eq!(response.value.to_string(), "timedOut511");
        assert_eq!(response.termination, None);
        assert!(response.messages.is_empty());
        assert_constraint_restored(&mut session);
    }
}

#[test]
fn monitor_runs_current_display_and_callback_subset() {
    let mut session = core_session();

    let outcome = session
        .eval_text(
            "Reap[Monitor[1 + 1, progress, {StepMonitor :> Sow[step], EvaluationMonitor :> Sow[eval]}]]",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[2, List[List[step, eval]]]");
    assert_eq!(outcome.prints, vec!["progress"]);

    let changing = session.eval_text("x = 0; Monitor[(x = 1; x), x]").unwrap();
    assert_eq!(changing.value.to_string(), "1");
    assert_eq!(changing.prints, vec!["0", "1"]);

    session.eval_text("Unprotect[Monitor]").unwrap();
    session
        .eval_text("SetOptions[Monitor, StepMonitor :> Sow[stepDefault]]")
        .unwrap();
    let defaulted = session.eval_text("Reap[Monitor[1 + 1, progress]]").unwrap();
    assert_eq!(
        defaulted.value.to_string(),
        "List[2, List[List[stepDefault]]]"
    );

    session
        .eval_text("SetOptions[Monitor, EvaluationMonitor :> Sow[evalDefault]]")
        .unwrap();
    let default_eval = session.eval_text("Reap[Monitor[1 + 1, progress]]").unwrap();
    assert_eq!(
        default_eval.value.to_string(),
        "List[2, List[List[stepDefault, evalDefault]]]"
    );

    let disabled_defaults = session
        .eval_text(
            "Reap[Monitor[1 + 1, progress, {StepMonitor :> None, EvaluationMonitor :> None}]]",
        )
        .unwrap();
    assert_eq!(disabled_defaults.value.to_string(), "List[2, List[]]");
}

#[test]
fn sow_and_reap_collect_by_tag_and_optional_handler() {
    let mut session = core_session();

    assert_eq!(
        session.eval_text("Reap[Sow[1]]").unwrap().value.to_string(),
        "List[1, List[List[1]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], a]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[1]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], b]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], _, List]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[a, List[1]]]]"
    );
    assert_eq!(
        session
            .eval_text("Reap[Sow[1, a], {_Symbol}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, List[List[List[1]]]]"
    );
}
