use super::lease::Mode;
use super::retained::scoped;
use super::*;
use crate::runtime::suspension::AbandonReason;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::panic::{AssertUnwindSafe, catch_unwind};

mod compile;
mod exits;

#[test]
fn retention_unarmed_suspended_capacity_is_visible_to_later_arm() {
    let mut session = probe_session(false);
    session
        .eval_request("LeaseProbe538[0, InputString[\"cold\"]]")
        .unwrap_err();
    assert!(session.retention_snapshot().is_none());
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let r = session.retained_admission_bytes().unwrap();
    assert_eq!(
        r,
        32 * size_of::<Expr>(),
        "unarmed frozen spare capacity escaped later arming"
    );
    let s = session.current_memory_proxy_bytes_for_test();
    session.current_memory_limit = Some(s + r / 2);
    assert!(
        session
            .enforce_active_memory_limit_for_projected_bytes(0)
            .is_err()
    );
    session.abandon_pending_suspension(AbandonReason::CancelledRequest);
    assert_eq!(session.retained_admission_bytes(), Some(0));
    println!("unarmed suspension: S={s}, R={r}, rejected while pending, released on abandon");
}

fn armed() -> Session {
    let mut session = Session::new();
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    session
}

fn observe(session: &Session) -> RetentionSnapshot {
    session.retention_snapshot().unwrap()
}

#[test]
fn retention_lifetime_normal_early_error_drop_and_unwind() {
    let mut session = armed();
    for exit in 0..5 {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            scoped::<AccountingShadow, _>(&mut session, |mut cx, mut values| {
                cx.push(&mut values, Expr::string("x".repeat(20_000)))?;
                assert!(observe(cx.session).bytes.unwrap() >= 20_000);
                match exit {
                    0 => cx.finish_list(values),
                    1 => Err(EvalError::Abort), // early return, live local
                    2 => {
                        Err::<(), _>(EvalError::TimeConstraintExceeded)?;
                        unreachable!()
                    }
                    3 => {
                        drop(values);
                        assert_eq!(observe(cx.session).bytes, Some(0));
                        Err(EvalError::Abort)
                    }
                    _ => panic!("retention unwind control"),
                }
            })
        }));
        assert_eq!(outcome.is_err(), exit == 4);
        let snapshot = observe(&session);
        assert_eq!(snapshot.bytes, Some(0), "exit {exit}: {snapshot:?}");
        assert_eq!(snapshot.live, 0, "exit {exit}: {snapshot:?}");
        assert_eq!(snapshot.releases, exit + 1);
        println!("lifetime exit {exit}: {snapshot:?}");
    }
}

#[test]
fn retention_r_is_separate_from_s_and_both_admission_entries_see_it() {
    let mut session = armed();
    scoped::<AccountingShadow, _>(&mut session, |mut cx, mut values| {
        // All accounting infrastructure has been allocated before the baseline.
        let before = cx.session.current_memory_proxy_bytes_for_test();
        let peak = cx.session.peak_memory_proxy_bytes.get();
        let memory_in_use = cx
            .session
            .eval_memory_in_use(&system_expr("MemoryInUse"), &[])?;
        let max_memory = cx
            .session
            .eval_max_memory_used(&system_expr("MaxMemoryUsed"), &[])?;
        let byte_count = cx
            .session
            .eval_byte_count(&system_expr("ByteCount"), &[Expr::small_integer(9)])?;
        cx.reserve(&mut values, 1000)?;
        let child = Expr::call(
            system_expr("shared538"),
            vec![Expr::string("y".repeat(40_000))],
        );
        cx.push(&mut values, child.clone())?;
        cx.push(&mut values, child)?;
        let r = observe(cx.session).bytes.unwrap();
        assert_eq!(
            cx.session
                .eval_memory_in_use(&system_expr("MemoryInUse"), &[])?,
            memory_in_use
        );
        assert_eq!(
            cx.session
                .eval_max_memory_used(&system_expr("MaxMemoryUsed"), &[])?,
            max_memory
        );
        assert!(r > 80_000, "shared children count twice: {r}");
        assert_eq!(
            cx.session.current_memory_proxy_bytes_for_test(),
            before,
            "R entered S"
        );
        assert_eq!(
            cx.session.peak_memory_proxy_bytes.get(),
            peak,
            "R entered S peak"
        );
        assert_eq!(
            cx.session
                .eval_byte_count(&system_expr("ByteCount"), &[Expr::small_integer(9)])?,
            byte_count
        );
        let independent = values.values.iter().fold(
            crate::runtime::debug_tools::resources::occurrence::Bytes::Exact(
                (values.values.capacity() - values.values.len()) * size_of::<Expr>(),
            ),
            |n, e| n.add(crate::runtime::debug_tools::resources::occurrence::expression(e)),
        );
        assert_eq!(
            independent,
            crate::runtime::debug_tools::resources::occurrence::Bytes::Exact(r)
        );
        cx.session.current_memory_limit = Some(before + 1000);
        assert!(matches!(
            cx.session
                .enforce_active_memory_limit_for_projected_bytes(0),
            Err(EvalError::MemoryConstraintExceeded { .. })
        ));
        assert!(matches!(
            cx.session
                .enforce_active_resource_limits_for_expr(&Expr::small_integer(9)),
            Err(EvalError::MemoryConstraintExceeded { .. })
        ));
        cx.session.current_memory_limit = Some(1_000_000_000);
        println!("R/S separation: S={before}, peak={peak}, R={r}, both admissions rejected");
        cx.clear(&mut values)?;
        assert_eq!(
            observe(cx.session).bytes,
            Some(values.values.capacity() * size_of::<Expr>())
        );
        cx.finish_list(values)
    })
    .unwrap();
    assert_eq!(observe(&session).bytes, Some(0));
}

fn probe(session: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    if session.current_memory_limit.is_some() {
        scoped::<AccountingShadow, _>(session, |cx, values| start(cx, values, args))
    } else {
        scoped::<Unarmed, _>(session, |cx, values| start(cx, values, args))
    }
}

fn start<'id, M: Mode>(
    mut cx: RetentionContext<'_, 'id, M>,
    mut values: RetainedValues<'id, M>,
    args: &SharedArgs,
) -> Result<RetainedResult<'id, M>, EvalError> {
    cx.reserve(&mut values, 32)?;
    if args[0] == Expr::small_integer(1) {
        cx.push(&mut values, Expr::string("prefix538".repeat(150_000)))?;
    }
    drive(cx, values, args, 1)
}

fn drive<'id, M: Mode>(
    mut cx: RetentionContext<'_, 'id, M>,
    mut values: RetainedValues<'id, M>,
    args: &SharedArgs,
    start: usize,
) -> Result<RetainedResult<'id, M>, EvalError> {
    for index in start..args.len() {
        match cx
            .evaluate(values, args[index].clone())
            .and_then(CallbackValue::adopt)
        {
            Ok((context, prefix)) => {
                cx = context;
                values = prefix;
            }
            Err(failure) => {
                return Ok(failure.suspend(|| ProbeContinuation {
                    args: args.clone(),
                    next: index + 1,
                }));
            }
        }
    }
    cx.finish_list(values)
}

#[derive(rum_derive::FrameRootTrace)]
struct ProbeContinuation {
    args: SharedArgs,
    next: usize,
}

fn resume<'id, M: Mode>(
    mut cx: RetentionContext<'_, 'id, M>,
    empty: RetainedValues<'id, M>,
    frame: &ProbeContinuation,
    prefix: &FrozenValues,
    outcome: Result<Expr, EvalError>,
) -> Result<RetainedResult<'id, M>, EvalError> {
    let mut values = cx.resume_values(empty, prefix)?;
    cx.push(&mut values, outcome?)?;
    drive(cx, values, &frame.args, frame.next)
}

impl RetentionContinuation for ProbeContinuation {
    fn resume(
        &self,
        session: &mut Session,
        prefix: &FrozenValues,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let result = if session.current_memory_limit.is_some() {
            scoped::<AccountingShadow, _>(session, |cx, empty| {
                resume(cx, empty, self, prefix, outcome)
            })
        } else {
            scoped::<Unarmed, _>(session, |cx, empty| {
                resume(cx, empty, self, prefix, outcome)
            })
        };
        FrameAction::complete(result)
    }
}

fn probe_session(armed: bool) -> Session {
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "LeaseProbe538",
        "Test-only lease protocol.",
        HoldAttribute::HoldAll,
        probe,
        MemoryGrowthContract::Incremental,
    ));
    if armed {
        session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    }
    session
}

#[test]
fn retention_activation_rejects_live_unarmed_prefix_before_guest_body() {
    for prefix in [0, 1] {
        let mut session = probe_session(false);
        let source = format!(
            "LeaseProbe538[{prefix}, MemoryConstrained[(Print[\"entered538\"]; 7), 500000, -1]]"
        );
        let outcome = session.eval_text(&source).unwrap();
        let Expr::Call { args, .. } = outcome.value else {
            panic!("probe result")
        };
        assert_eq!(
            args.last(),
            Some(&Expr::small_integer(if prefix == 0 { 7 } else { -1 }))
        );
        assert_eq!(
            outcome.prints.len(),
            if prefix == 0 { 1 } else { 0 },
            "prefix must reject before guest body, not final result"
        );
        assert!(
            session
                .accounting_route
                .as_ref()
                .unwrap()
                .prefixes
                .is_empty()
        );
        println!(
            "activation prefix={prefix}: guest-body entries={}, callback={}",
            outcome.prints.len(),
            args.last().unwrap()
        );
    }
}

#[test]
fn retention_suspension_transfers_once_resumes_twice_and_abandons() {
    let mut session = probe_session(true);
    let failure = session
        .eval_request("LeaseProbe538[1, InputString[\"one\"], InputString[\"two\"]]")
        .unwrap_err();
    assert!(matches!(
        failure.error,
        SessionRequestError::InputRequired { .. }
    ));
    let first = observe(&session);
    assert_eq!(first.transfers, 1);
    assert_eq!(first.live, 1);
    assert!(
        first.bytes.unwrap() > 0 && first.bytes.unwrap() < 10_000,
        "only frozen complement in R: {first:?}"
    );
    assert!(
        session.current_memory_proxy_bytes_for_test() > 1_000_000,
        "frame roots remain in S"
    );
    assert!(session.resume_eval_request_with_input("first").is_err());
    let second = observe(&session);
    assert_eq!(second.transfers, 2);
    assert_eq!(second.live, 1);
    session.resume_eval_request_with_input("second").unwrap();
    assert_eq!(observe(&session).bytes, Some(0));
    assert_eq!(observe(&session).live, 0);
    assert_eq!(observe(&session).transfers, 2);
    println!(
        "suspension first={first:?}, second={second:?}, complete={:?}",
        observe(&session)
    );
    session
        .eval_request("LeaseProbe538[1, InputString[\"abandon\"]]")
        .unwrap_err();
    assert!(observe(&session).bytes.unwrap() > 0);
    session.abandon_pending_suspension(AbandonReason::CancelledRequest);
    assert_eq!(observe(&session).bytes, Some(0));
    assert_eq!(observe(&session).live, 0);
    let released = observe(&session);
    session.abandon_pending_suspension(AbandonReason::CancelledRequest);
    assert_eq!(observe(&session), released, "no double release");
    println!("abandoned: {released:?}");
}

#[test]
fn retention_unarmed_operations_do_not_construct_receipts_or_walk_weights() {
    assert_eq!(size_of::<<Unarmed as RetentionMode>::Receipt>(), 0);
    assert_eq!(
        size_of::<RetainedValues<'_, Unarmed>>(),
        size_of::<Vec<Expr>>()
    );
    let mut session = Session::new();
    session
        .with_unarmed_retention(|mut cx, mut values| {
            cx.push(&mut values, Expr::string("unarmed"))?;
            cx.clear(&mut values)?;
            assert!(cx.session.retention_snapshot().is_none());
            assert!(cx.session.accounting_shadow().is_none());
            cx.finish_list(values)
        })
        .unwrap();
    assert!(session.accounting_route.is_none());
    let mut called = false;
    <Unarmed as Mode>::resize(&mut (), || {
        called = true;
        Some(123)
    });
    assert!(
        !called,
        "the unarmed mode must erase the weight computation"
    );
}
