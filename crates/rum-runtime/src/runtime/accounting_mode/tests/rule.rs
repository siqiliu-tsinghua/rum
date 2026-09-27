use super::*;
use crate::runtime::debug_tools::resources::occurrence::expression;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[path = "rule_exits.rs"]
mod exits;

fn session(armed: bool) -> Session {
    let mut s = crate::runtime::test_builtins::session_with_test_control_builtins();
    if armed {
        s.current_memory_limit = Some(1_000_000_000);
        s.note_accounting_arm();
    }
    s
}

fn released(s: &Session) {
    if let Some(r) = s.retention_snapshot() {
        assert_eq!(r.bytes, Some(0));
        assert_eq!(r.live, 0);
    }
    assert!(
        s.accounting_route
            .as_ref()
            .is_none_or(|r| r.prefixes.is_empty())
    );
}

#[test]
fn option_rule_clone_and_shell_contract() {
    let mut text = String::with_capacity(8000);
    text.push_str("spare text");
    let mut args = Vec::with_capacity(17);
    args.push(Expr::string("shared child"));
    let call = Expr::call(system_expr("held543"), args);
    let cases = [
        Expr::small_integer(7),
        Expr::integer(BigInt::from(1_u64) << 160),
        Expr::rational(Rational::new(BigInt::from(7), BigInt::from(13)).unwrap()),
        Expr::string(text),
        system_expr("name543"),
        call.clone(),
    ];
    for source in &cases {
        let mut s = session(true);
        scoped::<AccountingShadow, _>(&mut s, |cx, empty| {
            let ledger = cx
                .session
                .accounting_route
                .as_ref()
                .unwrap()
                .retention
                .clone()
                .unwrap();
            let result = clone_owned(cx, empty, source)?;
            let value = result.outcome.as_ref().unwrap();
            assert_eq!(value, source);
            assert_eq!(
                ledger.snapshot().bytes,
                Some(expression(value).ceiling_bytes())
            );
            assert_eq!(ledger.snapshot().live, 1);
            Ok(result)
        })
        .unwrap();
        released(&s);
        // B11 cold/warm literal identity, not a bound on the global interner.
        for _ in 0..2 {
            scoped::<AccountingShadow, _>(&mut s, |cx, empty| {
                let ledger = cx
                    .session
                    .accounting_route
                    .as_ref()
                    .unwrap()
                    .retention
                    .clone()
                    .unwrap();
                let value = match cx.adopt_outcome(empty, Ok(call.clone())) {
                    Ok(v) => v,
                    Err(_) => panic!("ample callback budget"),
                };
                assert!(ledger.snapshot().bytes.unwrap() >= expression(&call).ceiling_bytes());
                let result = value.into_rule(source)?;
                let rule = result.outcome.as_ref().unwrap();
                let Expr::Call { head, args } = rule else {
                    panic!("Rule")
                };
                assert!(is_system_symbol_name(head, "Rule"));
                assert_eq!(args.as_ref(), &[source.clone(), call.clone()]);
                assert_eq!(
                    ledger.snapshot().bytes,
                    Some(expression(rule).ceiling_bytes()),
                    "head, shell, children and actual spare capacity must all be charged"
                );
                assert_eq!(ledger.snapshot().live, 1);
                Ok(result)
            })
            .unwrap();
            released(&s);
        }
    }
    let mut s = session(false);
    assert_eq!(clone_result(&mut s, &call).unwrap(), call);
    assert!(
        s.accounting_route.is_none(),
        "unarmed clone must allocate no route"
    );
}

#[test]
fn option_rule_name_is_adopted_before_growth() {
    let mut s = session(true);
    let large = Expr::string("n".repeat(80_000));
    scoped::<AccountingShadow, _>(&mut s, |mut cx, mut values| {
        // No output-vector allocation is allowed once the cloned occurrence
        // already exceeds the remaining allowance at reserve's checkpoint.
        let before = cx.session.current_memory_proxy_bytes_for_test();
        cx.session.current_memory_limit = Some(before + 1000);
        assert!(matches!(
            clone_child(&mut cx, &mut values, &large),
            Err(EvalError::MemoryConstraintExceeded { .. })
        ));
        assert_eq!(
            values.values.capacity(),
            0,
            "clone receipt must precede reserve"
        );
        Ok(RetainedResult::failure(EvalError::Abort))
    })
    .unwrap_err();
    released(&s);
}

#[test]
fn option_rule_callback_adoption_refuses_before_growth_and_preserves_errors() {
    let mut s = session(true);
    scoped::<AccountingShadow, _>(&mut s, |cx, empty| {
        let before = cx.session.current_memory_proxy_bytes_for_test();
        cx.session.current_memory_limit = Some(before + 1000);
        match cx.adopt_outcome(empty, Ok(Expr::string("v".repeat(80_000)))) {
            Ok(_) => panic!("unadopted callback escaped its checkpoint"),
            Err(f) => Ok(f.finish()),
        }
    })
    .unwrap_err();
    released(&s);
    let payload = Expr::string("error payload".repeat(1000));
    let result = scoped::<AccountingShadow, _>(&mut s, |cx, empty| {
        match cx.adopt_outcome(
            empty,
            Err(EvalError::Return {
                value: payload.clone(),
            }),
        ) {
            Ok(_) => panic!("error became success"),
            Err(f) => Ok(f.finish()),
        }
    });
    assert!(matches!(result, Err(EvalError::Return { value }) if value == payload));
    released(&s);
}

#[test]
fn option_rule_overflow_and_partial_failure_release() {
    let mut s = session(true);
    let mut overflow = RetentionLease::new(&mut s);
    overflow.resize(Some(usize::MAX));
    let result = clone_result(&mut s, &Expr::small_integer(3));
    assert!(matches!(
        result,
        Err(EvalError::MemoryConstraintExceeded { .. })
    ));
    assert_eq!(s.retention_snapshot().unwrap().bytes, None);
    drop(overflow);
    assert_eq!(s.retention_snapshot().unwrap().bytes, None);
    assert_eq!(s.retention_snapshot().unwrap().live, 0);

    let mut s = session(true);
    scoped::<AccountingShadow, _>(&mut s, |mut cx, mut empty| {
        cx.reserve(&mut empty, 8)?;
        let result = match cx.adopt_outcome(empty, Ok(Expr::small_integer(1))) {
            Ok(v) => v.into_rule(&Expr::string("name")),
            Err(_) => panic!("ample budget"),
        }?;
        Ok(result)
    })
    .unwrap();
    released(&s);
}

#[test]
fn option_rule_late_arm_observes_existing_prefix() {
    for large in [false, true] {
        let mut s = session(false);
        let rhs = s
            .lower_text("MemoryConstrained[(Print[\"entered543\"]; 7), 500000, -1]")
            .unwrap();
        let name = Expr::string("n".repeat(if large { 1_000_000 } else { 1 }));
        let result = scoped::<Unarmed, _>(&mut s, |mut cx, mut values| {
            clone_child(&mut cx, &mut values, &name)?;
            match cx.evaluate(values, rhs).and_then(CallbackValue::adopt) {
                Ok((cx, values)) => cx.finish_list(values),
                Err(f) => Ok(f.finish()),
            }
        })
        .unwrap();
        let Expr::Call { args, .. } = result else {
            panic!("List")
        };
        assert_eq!(
            args.last(),
            Some(&Expr::small_integer(if large { -1 } else { 7 }))
        );
        released(&s);
    }
}

fn panic_guest(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    panic!("option rule guest panic");
}

fn error_guest(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    Err(EvalError::Return {
        value: Expr::string("preserved543"),
    })
}

#[test]
fn option_rule_guest_exits_are_lazy_and_release() {
    for armed in [false, true] {
        for (name, handler) in [
            ("RulePanic543", panic_guest as BuiltinFn),
            ("RuleError543", error_guest),
        ] {
            let mut s = session(armed);
            s.register_builtin(BuiltinSpec::ordinary(
                name,
                "exit control",
                handler,
                MemoryGrowthContract::NonAmplifying,
            ));
            if armed {
                s.current_memory_limit = Some(200_000);
            }
            let name_expr = Expr::string("must not clone this on error".repeat(100_000));
            let rhs = Expr::call(system_expr(name), vec![]);
            let result = catch_unwind(AssertUnwindSafe(|| evaluate_rule(&mut s, &name_expr, &rhs)));
            if name == "RulePanic543" {
                assert!(result.is_err());
            } else {
                assert!(
                    matches!(result.unwrap(), Err(EvalError::Return { value }) if value == Expr::string("preserved543"))
                );
            }
            assert!(s.pending_suspension.is_none());
            released(&s);
        }
    }
}
