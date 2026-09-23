use super::*;

fn guest_panic(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    panic!("bridge guest panic control")
}

fn guest_error(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    Err(EvalError::TimeConstraintExceeded)
}

#[test]
fn retention_bridge_releases_prefix_on_guest_error_and_panic() {
    for (name, handler) in [
        ("Panic538", guest_panic as BuiltinFn),
        ("Error538", guest_error),
    ] {
        let mut session = armed();
        session.register_builtin(BuiltinSpec::ordinary(
            name,
            "Test-only exit.",
            handler,
            MemoryGrowthContract::NonAmplifying,
        ));
        let result = catch_unwind(AssertUnwindSafe(|| {
            scoped::<AccountingShadow, _>(&mut session, |mut cx, mut values| {
                cx.push(&mut values, Expr::string("live".repeat(10_000)))?;
                match cx
                    .evaluate(values, Expr::call(system_expr(name), vec![]))
                    .and_then(CallbackValue::adopt)
                {
                    Ok((cx, values)) => cx.finish_list(values),
                    Err(failure) => Ok(failure.finish()),
                }
            })
        }));
        assert_eq!(result.is_err(), name == "Panic538");
        assert_eq!(observe(&session).bytes, Some(0));
        assert_eq!(observe(&session).live, 0);
        assert!(
            session
                .accounting_route
                .as_ref()
                .unwrap()
                .prefixes
                .is_empty()
        );
        println!(
            "guest exit {name}: {:?}, prefix stack empty",
            observe(&session)
        );
    }
}

#[test]
fn retention_nested_unarmed_bridges_keep_every_outer_prefix() {
    for outer in [0, 1] {
        let mut session = probe_session(false);
        let source = format!(
            "LeaseProbe538[{outer}, LeaseProbe538[1, MemoryConstrained[(Print[\"nested538\"]; 7), 2000000, -1]]]"
        );
        let result = session.eval_text(&source).unwrap();
        assert_eq!(result.prints.len(), if outer == 0 { 1 } else { 0 });
        assert!(
            session
                .accounting_route
                .as_ref()
                .unwrap()
                .prefixes
                .is_empty()
        );
        println!(
            "nested outer={outer}, guest entries={}",
            result.prints.len()
        );
    }
}

#[test]
fn retention_overflow_stays_invalid_after_release_and_old_ledger_is_isolated() {
    let mut old_session = armed();
    let mut first = RetentionLease::new(&mut old_session);
    first.resize(Some(usize::MAX));
    let mut second = RetentionLease::new(&mut old_session);
    second.resize(Some(1));
    assert_eq!(observe(&old_session).bytes, None);
    drop(first);
    assert_eq!(observe(&old_session).bytes, None);
    let ledger = old_session
        .accounting_route
        .as_ref()
        .unwrap()
        .retention
        .as_ref()
        .unwrap()
        .clone();
    drop(old_session);
    let mut new_session = armed();
    scoped::<AccountingShadow, _>(&mut new_session, |cx, values| cx.finish_list(values)).unwrap();
    let before = observe(&new_session);
    drop(second);
    assert_eq!(ledger.snapshot().live, 0);
    assert_eq!(ledger.snapshot().bytes, None);
    assert_eq!(
        observe(&new_session),
        before,
        "old drop must not debit a new ledger"
    );
}

#[test]
fn retention_frame_factory_is_lazy_and_refusal_abandons_pending_input() {
    let mut session = armed();
    let mut called = false;
    let result = scoped::<AccountingShadow, _>(&mut session, |cx, values| {
        Ok(
            cx.suspend(values, EvalError::Abort, || -> ProbeContinuation {
                called = true;
                panic!("non-suspension must not build a frame")
            }),
        )
    });
    assert!(matches!(result, Err(EvalError::Abort)));
    assert!(!called);
    assert_eq!(observe(&session).bytes, Some(0));

    let mut session = probe_session(true);
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "TightInput538",
        "Test-only suspension reservation.",
        HoldAttribute::HoldAll,
        tight_input,
        MemoryGrowthContract::Delegated,
    ));
    let expr = session
        .lower_text("TightInput538[InputString[\"bounded frame\"]]")
        .unwrap();
    let result = scoped::<AccountingShadow, _>(&mut session, |mut cx, mut values| {
        cx.reserve(&mut values, 32)?;
        match cx.evaluate(values, expr) {
            Err(failure) => Ok(failure.suspend(|| -> ProbeContinuation {
                called = true;
                panic!("failed reservation must not build a frame")
            })),
            Ok(_) => panic!("real input must suspend"),
        }
    });
    assert!(matches!(
        result,
        Err(EvalError::MemoryConstraintExceeded { .. })
    ));
    assert!(!called);
    assert!(session.pending_suspension.is_none());
    assert_eq!(observe(&session).bytes, Some(0));
}

fn tight_input(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let result = session.evaluate(args[0].clone());
    session.current_memory_limit = Some(0);
    result
}

#[test]
fn retention_suspension_checks_actual_continuation_roots() {
    for payload_bytes in [1, 500_000] {
        let mut session = probe_session(true);
        session.current_memory_limit = Some(200_000);
        session.note_accounting_arm();
        let expr = session.lower_text("InputString[\"frame roots\"]").unwrap();
        let mut built = false;
        let result = scoped::<AccountingShadow, _>(&mut session, |mut cx, mut values| {
            cx.reserve(&mut values, 32)?;
            match cx.evaluate(values, expr) {
                Err(failure) => Ok(failure.suspend(|| {
                    built = true;
                    ProbeContinuation {
                        args: vec![Expr::string("r".repeat(payload_bytes))].into(),
                        next: 1,
                    }
                })),
                Ok(_) => panic!("real input must suspend"),
            }
        });
        assert!(built, "both cases pass the shell reservation");
        if payload_bytes == 1 {
            assert!(matches!(result, Err(EvalError::InputRequired { .. })));
            assert!(session.pending_suspension.is_some());
            session.abandon_pending_suspension(AbandonReason::CancelledRequest);
        } else {
            assert!(matches!(
                result,
                Err(EvalError::MemoryConstraintExceeded { .. })
            ));
            assert!(session.pending_suspension.is_none());
        }
        assert_eq!(observe(&session).bytes, Some(0));
        assert_eq!(observe(&session).live, 0);
        assert_eq!(observe(&session).transfers, 1);
        assert_eq!(observe(&session).releases, 1);
        println!(
            "continuation roots {payload_bytes}: checked before suspension publication, released"
        );
    }
}
