use super::fault_support::*;
use super::*;
use crate::runtime::SessionRequestError;

#[test]
fn wrong_arms_and_exact_state_unwind_captures_and_scopes() {
    let cases = [
        (
            "EvaluationData",
            "EvaluationData[(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1)]",
            true,
            true,
        ),
        (
            "TestReport",
            "TestReport[{TestCreate[(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1),7]}]",
            true,
            true,
        ),
        (
            "Catch",
            "Catch[(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1)]",
            false,
            false,
        ),
        (
            "Check",
            "Check[(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1),fallback=1]",
            false,
            true,
        ),
        (
            "Quiet",
            "Quiet[(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1)]",
            false,
            true,
        ),
        (
            "Trace",
            "Trace[(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1)]",
            false,
            false,
        ),
        (
            "Block",
            "Block[{x=inside},(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1)]",
            false,
            false,
        ),
        (
            "iterator",
            "Table[(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1),{i,{7}}]",
            false,
            false,
        ),
        ("options", "f[Rule[a,9]]", false, false),
    ];
    for kind in [
        FaultKind::ValueForState,
        FaultKind::StateForValue,
        FaultKind::WrongState,
    ] {
        for (label, source, capture_print, capture_message) in cases {
            let (mut s, events) = session(kind, false);
            s.eval_request("x=outside;i=outsideI;later=0;fallback=0;probe::tag=\"inside message\";Options[f]={a->1};f[OptionsPattern[]]:=(Print[\"inside\"];Message[probe::tag];SlotTrip[];later=1;OptionValue[a])").unwrap();
            let probe = s.continuation_fault_probe();
            s.enable_tracing(true);
            let first = s
                .eval_request(&format!("Print[\"outside\"];{source}"))
                .unwrap_err();
            assert!(
                matches!(first.error, SessionRequestError::InputRequired { .. }),
                "{label}: {first:?}"
            );
            let boundary = s.pending_output_boundary();
            let kept_trace = boundary.trace.map(|end| s.trace_values_at_or_before(end));
            super::accounting::reconcile(&s);
            let failure = s.resume_eval_request_with_input("answer").unwrap_err();
            kind.assert_request(&failure.error);
            assert_eq!(probe.occurrences(), 1, "{label}");
            assert_eq!(
                *events.lock().unwrap(),
                ["emit", "consumer", "suffix-one", "suffix-two"],
                "{label}"
            );
            assert_eq!(
                failure.prints,
                if capture_print {
                    vec!["outside"]
                } else {
                    vec!["outside", "inside"]
                },
                "{label}"
            );
            assert_eq!(
                failure.messages.len(),
                usize::from(!capture_message),
                "{label}"
            );
            if let Some(kept) = kept_trace {
                assert_eq!(s.trace_values(), kept, "{label}");
            }
            assert!(s.tracing_enabled);
            restored(&mut s);
            assert_eq!(
                s.eval_request("{x,i,later,fallback}")
                    .unwrap()
                    .value
                    .to_string(),
                "List[outside, outsideI, 0, 0]",
                "{label}"
            );
            assert_eq!(probe.occurrences(), 1);
        }
    }
}

#[test]
fn cleanup_failure_or_nested_input_never_replaces_the_first_fault() {
    for cleanup in [
        "cleaned=1;Abort[]",
        "cleaned=1;InputString[\"cleanup input\"];cleanupLater=1",
        "cleaned=1;SlotDirect[]",
    ] {
        let kind = FaultKind::StateForValue;
        let (mut s, events) = session(kind, true);
        s.eval_request("cleaned=0;cleanupLater=0;later=0;x=outside")
            .unwrap();
        let probe = s.continuation_fault_probe();
        let first = s
            .eval_request(&format!(
                "WithCleanup[Block[{{x=inside}},SlotTrip[];later=1],({cleanup})]"
            ))
            .unwrap_err();
        assert!(matches!(
            first.error,
            SessionRequestError::InputRequired { .. }
        ));
        super::accounting::reconcile(&s);
        let failure = s.resume_eval_request_with_input("answer").unwrap_err();
        kind.assert_request(&failure.error);
        assert_eq!(probe.occurrences(), 1);
        assert_eq!(
            *events.lock().unwrap(),
            ["emit", "nested", "consumer", "suffix-one", "suffix-two"]
        );
        restored(&mut s);
        assert_eq!(
            s.eval_request("{x,cleaned,cleanupLater,later}")
                .unwrap()
                .value
                .to_string(),
            "List[outside, 1, 0, 0]"
        );
    }
}

#[test]
fn b3_cleanup_throw_reports_nocatch_without_replacing_the_first_fault() {
    let cleanup = "cleaned=1;Throw[630]";
    let kind = FaultKind::StateForValue;
    let (mut s, events) = session(kind, true);
    s.eval_request("cleaned=0;cleanupLater=0;later=0;x=outside")
        .unwrap();
    let probe = s.continuation_fault_probe();
    let first = s
        .eval_request(&format!(
            "WithCleanup[Block[{{x=inside}},SlotTrip[];later=1],({cleanup})]"
        ))
        .unwrap_err();
    assert!(matches!(
        first.error,
        SessionRequestError::InputRequired { .. }
    ));
    super::accounting::reconcile(&s);
    let failure = s.resume_eval_request_with_input("answer").unwrap_err();
    kind.assert_request(&failure.error);
    assert_eq!(probe.occurrences(), 1);
    assert_eq!(
        *events.lock().unwrap(),
        ["emit", "nested", "consumer", "suffix-one", "suffix-two"]
    );
    restored(&mut s);
    assert_eq!(
        s.eval_request("{x,cleaned,cleanupLater,later}")
            .unwrap()
            .value
            .to_string(),
        "List[outside, 1, 0, 0]"
    );
    let tags: Vec<_> = failure
        .messages
        .iter()
        .map(|message| format!("{}::{}", message.symbol, message.tag))
        .collect();
    eprintln!("B3SLOTVERDICT tags={tags:?}");
    assert_eq!(tags, ["Throw::nocatch"]);
}

#[test]
fn completion_exact_decode_and_probe_survive_rollback_and_replacement() {
    for kind in [
        FaultKind::ValueForState,
        FaultKind::StateForValue,
        FaultKind::WrongState,
    ] {
        let (mut s, events) = session(kind, false);
        let probe = s.continuation_fault_probe();
        let snapshot = s.capture_evaluation_rollback_snapshot();
        completion(&mut s, kind, &events);
        let error = match kind {
            FaultKind::StateForValue => s.resume_suspended_evaluation("answer").unwrap_err(),
            _ => s.resume_suspended_slot::<StateBool>("answer").unwrap_err(),
        };
        kind.assert_eval(&error);
        let error = s.continuation_fault_outcome::<Expr>(Err(error));
        kind.assert_eval(&error);
        assert_eq!(*events.lock().unwrap(), ["emit"]);
        assert_eq!(probe.occurrences(), 1);
        s.restore_evaluation_rollback_snapshot(snapshot);
        assert_eq!(probe.occurrences(), 1);
        restored(&mut s);
        s = Session::new();
        assert_eq!(
            probe.occurrences(),
            1,
            "an observer survives reset/replacement"
        );
        assert_eq!(s.continuation_fault_probe().occurrences(), 0);
        restored(&mut s);
    }
}

#[test]
fn direct_public_fault_is_fatal_at_evaluation_and_test_capture() {
    for source in [
        "EvaluationData[SlotDirect[]]",
        "TestReport[{TestCreate[SlotDirect[],7]}]",
    ] {
        let (mut s, _) = session(FaultKind::StateForValue, false);
        let probe = s.continuation_fault_probe();
        let failure = s.eval_request(source).unwrap_err();
        assert!(matches!(
            failure.error,
            SessionRequestError::ContinuationSlotMismatch {
                expected: "direct expected",
                got: "direct got"
            }
        ));
        assert_eq!(
            probe.occurrences(),
            0,
            "only private fault creation increments"
        );
        restored(&mut s);
    }
}
