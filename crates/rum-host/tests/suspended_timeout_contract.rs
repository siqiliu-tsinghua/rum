use rum_host::{
    ClosurePrintSink, HostEvalOutcome, HostFailureCode, HostInputKind, HostSession,
    HostStructuredOutputKind, HostTerminationReason, HostTyped,
};
use std::thread;
use std::time::{Duration, Instant};

// These tests pin the Cancel And Timeout contract in docs/guide/embedding.md.
const TIMEOUT: Duration = Duration::from_millis(100);

fn assert_input(outcome: HostEvalOutcome, kind: HostInputKind, prompt: &str) {
    let HostEvalOutcome::InputRequired(suspension) = outcome else {
        panic!("expected input suspension, got {outcome:?}");
    };
    assert_eq!(suspension.request.kind, kind);
    assert_eq!(suspension.request.prompt, prompt);
    assert!(suspension.messages.is_empty());
}

fn assert_value(outcome: HostEvalOutcome, expected: &str) {
    let HostEvalOutcome::Success(success) = outcome else {
        panic!("expected completed evaluation, got {outcome:?}");
    };
    assert_eq!(success.value.text, expected);
    assert_eq!(success.termination, None);
    assert!(success.messages.is_empty());
}

fn assert_termination(outcome: HostEvalOutcome, reason: HostTerminationReason) {
    let HostEvalOutcome::Success(success) = outcome else {
        panic!("expected a synthesized termination value, got {outcome:?}");
    };
    let (expected, tag) = match reason {
        HostTerminationReason::TimeConstraint => {
            ("TerminatedEvaluation[\"TimeConstraint\"]", "timelim")
        }
        HostTerminationReason::Cancelled => ("TerminatedEvaluation[\"Cancelled\"]", "cancel"),
        other => panic!("unexpected test termination reason: {other:?}"),
    };
    assert_eq!(success.termination, Some(reason), "{success:?}");
    assert_eq!(success.value.text, expected);
    assert_eq!(
        success.value.typed,
        HostTyped::Other {
            head: "TerminatedEvaluation".to_string(),
            text: expected.to_string(),
        }
    );
    assert_eq!(success.messages.len(), 1);
    assert_eq!(success.messages[0].symbol, "General");
    assert_eq!(success.messages[0].tag, tag);
    let output = success.outputs.last().unwrap();
    assert_eq!(output.kind, HostStructuredOutputKind::Result);
    assert_eq!(output.text, expected);
}

fn assert_failure(outcome: HostEvalOutcome, code: HostFailureCode) {
    let HostEvalOutcome::Failure(failure) = outcome else {
        panic!("expected host failure, got {outcome:?}");
    };
    assert_eq!(failure.code, code);
}

fn wait_past_timeout() {
    let waiting = Instant::now();
    let duration = TIMEOUT + Duration::from_millis(25);
    while let Some(remaining) = duration.checked_sub(waiting.elapsed()) {
        thread::sleep(remaining);
    }
}

fn assert_timeout_cleared(session: &mut HostSession) {
    assert!(session.pending_input_request().is_none());
    // Pause exceeds TIMEOUT, and the assignments reach cooperative polls;
    // stable values such as Null bypass those polls.
    // This detects an eval_timeout left installed even if a new request would
    // renew its deadline rather than inherit the expired one.
    assert_value(
        session.eval("Pause[1/8]; Do[timeoutPoll511 = 1, 2000]; 9"),
        "9",
    );
}

#[test]
fn expired_input_suspensions_resume_under_the_original_host_deadline() {
    for (name, kind) in [
        ("Input", HostInputKind::Input),
        ("InputString", HostInputKind::InputString),
    ] {
        let mut session = HostSession::new();
        assert_value(session.eval("timeoutTail511 = 0"), "0");
        let source =
            format!("{name}[\"deadline\"]; Do[timeoutPoll511 = 1, 2000]; timeoutTail511 = 1");
        assert_input(session.eval_with_timeout(source, TIMEOUT), kind, "deadline");

        wait_past_timeout();
        assert_termination(
            session.resume_with_input("5"),
            HostTerminationReason::TimeConstraint,
        );
        assert_value(session.eval("timeoutTail511"), "0");
        assert_timeout_cleared(&mut session);
    }
}

#[test]
fn immediate_resume_finishes_with_a_generous_host_timeout() {
    for (name, kind, expected) in [
        ("Input", HostInputKind::Input, "5"),
        ("InputString", HostInputKind::InputString, "\"5\""),
    ] {
        let mut session = HostSession::new();
        assert_input(
            session.eval_with_timeout(format!("{name}[\"control\"]"), Duration::from_secs(10)),
            kind,
            "control",
        );
        assert_value(session.resume_with_input("5"), expected);
        assert_value(session.eval("1 + 1"), "2");
    }
}

#[test]
fn timeout_can_expire_during_a_resumed_segment() {
    let mut session = HostSession::new();
    assert_input(
        session.eval_with_timeout(
            "InputString[\"work\"]; Pause[1/8]; Do[timeoutPoll511 = 1, 2000]; 7",
            TIMEOUT,
        ),
        HostInputKind::InputString,
        "work",
    );
    assert_termination(
        session.resume_with_input("go"),
        HostTerminationReason::TimeConstraint,
    );
    assert_timeout_cleared(&mut session);
}

#[test]
fn two_suspensions_keep_the_same_host_deadline() {
    let mut session = HostSession::new();
    assert_input(
        session.eval_with_timeout(
            "Input[\"first\"]; InputString[\"second\"]; Do[timeoutPoll511 = 1, 2000]; 7",
            TIMEOUT,
        ),
        HostInputKind::Input,
        "first",
    );
    assert_input(
        session.resume_with_input("5"),
        HostInputKind::InputString,
        "second",
    );
    wait_past_timeout();
    assert_termination(
        session.resume_with_input("6"),
        HostTerminationReason::TimeConstraint,
    );
    assert_timeout_cleared(&mut session);
}

#[test]
fn rejected_entrypoints_and_inspection_preserve_a_suspended_deadline() {
    let mut session = HostSession::new();
    assert_input(
        session.eval_with_timeout(
            "InputString[\"original\"]; Do[timeoutPoll511 = 1, 2000]; 7",
            TIMEOUT,
        ),
        HostInputKind::InputString,
        "original",
    );
    assert_failure(session.eval("1"), HostFailureCode::Other);
    assert_failure(session.call("Identity", &[]), HostFailureCode::Other);
    assert_failure(session.eval_script("1\n2"), HostFailureCode::Other);
    assert_failure(
        session.eval_file("missing-timeout-overlap-511.rum"),
        HostFailureCode::Other,
    );
    assert_failure(
        session.eval_with_timeout("1", Duration::from_secs(10)),
        HostFailureCode::Other,
    );
    assert!(session.inspect_symbol("InputString").found);
    assert_eq!(session.pending_input_request().unwrap().prompt, "original");

    wait_past_timeout();
    assert_termination(
        session.resume_with_input("5"),
        HostTerminationReason::TimeConstraint,
    );
    assert_timeout_cleared(&mut session);
}

#[test]
fn rejected_timeout_cannot_arm_or_shorten_a_pending_request() {
    for timeout in [None, Some(Duration::from_secs(10))] {
        let mut session = HostSession::new();
        let source = "InputString[\"original\"]; Do[timeoutPoll511 = 1, 2000]; 7";
        let outcome = match timeout {
            Some(timeout) => session.eval_with_timeout(source, timeout),
            None => session.eval(source),
        };
        assert_input(outcome, HostInputKind::InputString, "original");
        assert_failure(
            session.eval_with_timeout("1", Duration::ZERO),
            HostFailureCode::Other,
        );
        assert_value(session.resume_with_input("5"), "7");
    }
}

#[test]
fn completion_and_syntax_failure_release_the_owned_timeout() {
    for source in ["7", "f["] {
        let mut session = HostSession::new();
        let outcome = session.eval_with_timeout(source, TIMEOUT);
        if source == "7" {
            assert_value(outcome, "7");
        } else {
            assert_failure(outcome, HostFailureCode::Syntax);
        }
        assert_timeout_cleared(&mut session);
    }

    for input in ["5", "f["] {
        let mut session = HostSession::new();
        assert_input(
            session.eval_with_timeout("Input[\"finish\"]", TIMEOUT),
            HostInputKind::Input,
            "finish",
        );
        let outcome = session.resume_with_input(input);
        if input == "5" {
            assert_value(outcome, "5");
        } else {
            assert_failure(outcome, HostFailureCode::Syntax);
        }
        assert_timeout_cleared(&mut session);
    }
}

struct PanickingInput;

impl From<PanickingInput> for String {
    fn from(_: PanickingInput) -> Self {
        panic!("host input conversion must not run after accepted cancellation")
    }
}

#[test]
fn panics_during_timed_eval_or_resume_poison_until_reset() {
    let mut session = HostSession::new();
    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(|_| {
        panic!("timed evaluation print sink panic")
    }));
    assert_failure(
        session.eval_with_timeout("Print[\"panic\"]", TIMEOUT),
        HostFailureCode::Panic,
    );
    assert_failure(session.eval("1"), HostFailureCode::Poisoned);
    session.reset();
    assert_timeout_cleared(&mut session);

    assert_input(
        session.eval_with_timeout("InputString[\"panic\"]", TIMEOUT),
        HostInputKind::InputString,
        "panic",
    );
    assert_failure(
        session.resume_with_input(PanickingInput),
        HostFailureCode::Panic,
    );
    assert_failure(session.eval("1"), HostFailureCode::Poisoned);
    session.reset();
    assert_timeout_cleared(&mut session);
}

#[test]
fn reset_releases_suspended_timeout_and_invalidates_its_cancel_handle() {
    let mut session = HostSession::new();
    let cancel = session.cancel_handle();
    assert_input(
        session.eval_with_timeout("InputString[\"reset\"]", TIMEOUT),
        HostInputKind::InputString,
        "reset",
    );
    session.reset();
    cancel.cancel();
    assert_timeout_cleared(&mut session);
}

#[test]
fn accepted_cancellation_survives_timeout_arming_and_the_request_prologue() {
    let mut session = HostSession::new();
    let stale = session.cancel_handle();
    assert_value(session.eval("1"), "1");
    let current = session.cancel_handle();
    current.cancel();
    stale.cancel();
    assert_termination(
        session.eval_with_timeout("cancelTail511 = 1", Duration::ZERO),
        HostTerminationReason::Cancelled,
    );
    current.cancel();
    assert_value(session.eval("cancelTail511"), "cancelTail511");
    assert_timeout_cleared(&mut session);
}

#[test]
fn cancellation_of_expired_suspensions_wins_before_input_conversion() {
    for acquire_while_suspended in [false, true] {
        let mut session = HostSession::new();
        assert_value(session.eval("cancelTail511 = 0"), "0");
        let before = (!acquire_while_suspended).then(|| session.cancel_handle());
        assert_input(
            session.eval_with_timeout("InputString[\"cancel\"]; cancelTail511 = 1", TIMEOUT),
            HostInputKind::InputString,
            "cancel",
        );
        let cancel = before.unwrap_or_else(|| session.cancel_handle());
        wait_past_timeout();
        cancel.cancel();
        assert_termination(
            session.resume_with_input(PanickingInput),
            HostTerminationReason::Cancelled,
        );
        cancel.cancel();
        assert_value(session.eval("cancelTail511"), "0");
        assert_timeout_cleared(&mut session);
    }
}

#[test]
fn cancellation_during_resumed_work_is_observed_with_timeout_still_armed() {
    let mut session = HostSession::new();
    let cancel = session.cancel_handle();
    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |_| {
        cancel.cancel();
    }));
    assert_input(
        session.eval_with_timeout(
            "InputString[\"cancel\"]; Print[\"resumed\"]; Do[timeoutPoll511 = 1, 2000]; 7",
            TIMEOUT,
        ),
        HostInputKind::InputString,
        "cancel",
    );
    assert_termination(
        session.resume_with_input("5"),
        HostTerminationReason::Cancelled,
    );
    assert_timeout_cleared(&mut session);
}

#[test]
fn cancellation_via_each_evaluation_entrypoint_releases_the_suspended_timeout() {
    let entrypoints: [fn(&mut HostSession) -> HostEvalOutcome; 5] = [
        |session| session.eval("cancelTail511 = 1"),
        |session| session.call("Identity", &[HostTyped::Integer(1)]),
        |session| session.eval_script("cancelTail511 = 1\n2"),
        |session| session.eval_file("missing-cancelled-timeout-511.rum"),
        |session| session.eval_with_timeout("cancelTail511 = 1", Duration::ZERO),
    ];
    for entrypoint in entrypoints {
        let mut session = HostSession::new();
        assert_value(session.eval("cancelTail511 = 0"), "0");
        let cancel = session.cancel_handle();
        assert_input(
            session.eval_with_timeout("InputString[\"cancel\"]; cancelTail511 = 1", TIMEOUT),
            HostInputKind::InputString,
            "cancel",
        );
        cancel.cancel();
        assert_termination(entrypoint(&mut session), HostTerminationReason::Cancelled);
        assert_value(session.eval("cancelTail511"), "0");
        assert_timeout_cleared(&mut session);
    }
}
