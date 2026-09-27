use super::*;
use crate::runtime::suspension::AbandonReason;

fn rule_guest(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    evaluate_rule(s, &args[0], &args[1])
}

fn rule_session(armed: bool) -> Session {
    let mut s = session(false);
    s.register_builtin(BuiltinSpec::ordinary_with_hold(
        "Rule543",
        "private option rule probe",
        HoldAttribute::HoldAll,
        rule_guest,
        MemoryGrowthContract::Incremental,
    ));
    if armed {
        s.resource_limits.eval_memory_limit = Some(1_000_000_000);
    }
    s
}

#[test]
fn option_rule_repeated_nested_input_transfers_once() {
    for armed in [false, true] {
        let mut s = rule_session(armed);
        s.eval_request("Rule543[name543, (InputString[\"first\"]; InputString[\"second\"])]")
            .unwrap_err();
        if armed {
            assert_eq!(s.retention_snapshot().unwrap().transfers, 1);
            assert_eq!(
                s.retention_snapshot().unwrap().bytes,
                Some(size_of::<std::sync::Mutex<RetentionLease>>() + 2 * size_of::<usize>())
            );
        }
        s.resume_eval_request_with_input("one").unwrap_err();
        if armed {
            assert_eq!(
                s.retention_snapshot().unwrap().transfers,
                1,
                "pending rule transfers only once"
            );
        }
        let result = s.resume_eval_request_with_input("two").unwrap();
        assert_eq!(result.value.to_string(), "Rule[name543, \"two\"]");
        released(&s);
        let before = s.retention_snapshot().map_or(0, |r| r.transfers);
        s.eval_request("Rule543[outer543, Rule543[inner543, InputString[\"nested\"]]]")
            .unwrap_err();
        if armed {
            assert_eq!(s.retention_snapshot().unwrap().transfers, before + 2);
        }
        let result = s.resume_eval_request_with_input("nested value").unwrap();
        assert_eq!(
            result.value.to_string(),
            "Rule[outer543, Rule[inner543, \"nested value\"]]"
        );
        released(&s);
    }
}

#[test]
fn option_rule_abandon_reset_and_late_arm_frame() {
    for armed in [false, true] {
        let mut s = rule_session(armed);
        s.eval_request("Rule543[name543, InputString[\"abandon\"]]")
            .unwrap_err();
        s.current_memory_limit = Some(1_000_000_000);
        s.note_accounting_arm();
        assert!(s.current_memory_proxy_bytes_for_test() > 0);
        if !armed {
            // The name uses one exactly reserved slot, so it is all in S.
            assert_eq!(s.retained_admission_bytes(), Some(0));
            let s_bytes = s.current_memory_proxy_bytes_for_test();
            s.current_memory_limit = Some(s_bytes - 1);
            assert!(
                s.enforce_active_memory_limit_for_projected_bytes(0)
                    .is_err()
            );
        }
        s.abandon_pending_suspension(AbandonReason::CancelledRequest);
        released(&s);
        s.resource_limits.eval_memory_limit = if armed { Some(1_000_000_000) } else { None };
        s.eval_request("Rule543[name543, InputString[\"reset\"]]")
            .unwrap_err();
        let old_ledger = s
            .accounting_route
            .as_ref()
            .and_then(|r| r.retention.clone());
        // Session has no public reset; HostSession::reset replaces it. Pin the
        // same drop boundary here; the stdlib/host suites own the public API.
        s = rule_session(armed);
        if let Some(ledger) = old_ledger {
            assert_eq!(ledger.snapshot().bytes, Some(0));
            assert_eq!(ledger.snapshot().live, 0);
        }
        assert!(s.pending_suspension.is_none());
        released(&s);
    }
}

fn tight_input(s: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let result = s.evaluate(args[0].clone());
    s.current_memory_limit = Some(0);
    result
}

#[test]
fn option_rule_frame_refusal_abandons_without_a_transfer() {
    let mut s = rule_session(true);
    s.register_builtin(BuiltinSpec::ordinary_with_hold(
        "Tight543",
        "frame refusal",
        HoldAttribute::HoldAll,
        tight_input,
        MemoryGrowthContract::Delegated,
    ));
    s.current_memory_limit = Some(1_000_000_000);
    s.note_accounting_arm();
    let expr = s
        .lower_text("Rule543[name543, Tight543[InputString[\"refused\"]]]")
        .unwrap();
    let error = s.evaluate(expr).unwrap_err();
    assert!(matches!(error, EvalError::MemoryConstraintExceeded { .. }));
    assert!(s.pending_suspension.is_none());
    assert_eq!(s.retention_snapshot().unwrap().transfers, 0);
    released(&s);
}

#[test]
fn option_rule_late_arm_after_callback_and_during_input() {
    let mut s = session(false);
    let name = Expr::string("large name".repeat(10_000));
    // Exercise the exact bridge handoff after the limit changed, without
    // mutating evaluator stack mode from an unsupported test builtin.
    let result = scoped::<Unarmed, _>(&mut s, |cx, empty| {
        cx.session.current_memory_limit = Some(1_000_000_000);
        cx.session.note_accounting_arm();
        cx.session.current_memory_limit =
            Some(cx.session.current_memory_proxy_bytes_for_test() + 1000);
        match cx.adopt_outcome(empty, Ok(Expr::small_integer(5))) {
            Ok(value) => value.into_rule(&name),
            Err(failure) => Ok(failure.finish()),
        }
    });
    assert!(matches!(
        result,
        Err(EvalError::MemoryConstraintExceeded { .. })
    ));
    released(&s);

    let mut s = rule_session(false);
    s.eval_request("Rule543[name543, MemoryConstrained[InputString[\"late input\"], 1000000000]]")
        .unwrap_err();
    assert_eq!(s.retention_snapshot().unwrap().transfers, 1);
    assert_eq!(
        s.resume_eval_request_with_input("ok")
            .unwrap()
            .value
            .to_string(),
        "Rule[name543, \"ok\"]"
    );
    released(&s);

    let mut s = session(true);
    let input = s.lower_text("InputString[\"no longer pending\"]").unwrap();
    let error = s.evaluate(input).unwrap_err();
    s.abandon_pending_suspension(AbandonReason::CancelledRequest);
    let large = Expr::string("not captured".repeat(100_000));
    s.current_memory_limit = Some(200_000);
    let result = scoped::<AccountingShadow, _>(&mut s, |cx, empty| {
        match cx.adopt_outcome(empty, Err(error)) {
            Err(failure) => Ok(failure.suspend_rule(&large)),
            Ok(_) => panic!("must remain an error"),
        }
    });
    assert!(matches!(result, Err(EvalError::InputRequired { .. })));
    assert_eq!(s.retention_snapshot().unwrap().transfers, 0);
    released(&s);
}
