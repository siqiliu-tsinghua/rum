use crate::runtime::profiling::logical::test_support;
use crate::runtime::{EvalError, Session};

fn row(rows: &[(String, [u64; 10])], phase: &str) -> [u64; 10] {
    rows.iter()
        .find(|(key, _)| key.contains(&format!("phase: \"{phase}\"")))
        .map(|(_, counts)| *counts)
        .unwrap_or_else(|| panic!("missing {phase}: {rows:?}"))
}

fn assert_leaf_rows(rows: &[(String, [u64; 10])]) {
    for phase in [
        "evaluate_inner_unstable.total",
        "evaluate_inner_unstable.call",
        "evaluate_atom_fast_path.total",
        "evaluate_call.head_eval",
        "call.stage.prepare_arguments",
        "prepare_arguments.total",
        "prepare_arguments.evaluate_arg",
    ] {
        let counts = row(rows, phase);
        assert_eq!(
            counts[0],
            counts[1] + counts[2],
            "{phase}: {counts:?}; rows={rows:?}"
        );
        assert!(counts[1] > 0, "{phase}: {counts:?}; rows={rows:?}");
    }
    for (key, counts) in rows {
        assert_eq!(counts[8], 0, "{key}: {counts:?}");
        assert_eq!(counts[9], 0, "{key}: {counts:?}");
        if key.contains("phase: \"call.stage.") {
            assert_eq!(counts[3] + counts[4], counts[1], "{key}: {counts:?}");
        }
    }
    assert!(row(rows, "call.stage.prepare_arguments")[4] > 0);
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(test_support::active(), 0);
}

fn defined_session(definition: &str) -> Session {
    let mut session = Session::new();
    session.eval_text(definition).unwrap();
    session
}

#[test]
fn lp03_leaf_phases_close_on_completed_evaluation() {
    let mut session = defined_session("f[x_] := x");
    let _enabled = test_support::enabled();
    assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "1");
    assert_leaf_rows(&Session::logical_profile_test_rows());
}

#[test]
fn lp03_inner_symbol_and_atom_close() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    assert!(
        session
            .evaluate_inner_unstable(crate::runtime::Expr::small_integer(1))
            .is_ok()
    );
    assert!(
        session
            .evaluate_inner_unstable(crate::runtime::Expr::contextual_symbol("Global`", "x"))
            .is_ok()
    );
    let rows = Session::logical_profile_test_rows();
    assert!(row(&rows, "evaluate_inner_unstable.symbol")[1] > 0);
    assert!(row(&rows, "evaluate_inner_unstable.atom")[1] > 0);
    Session::logical_profile_test_assert_quiescent();
}

#[test]
fn lp03_leaf_phases_close_after_resume() {
    let mut session = defined_session("f[x_] := x");
    let _enabled = test_support::enabled();
    assert_eq!(session.eval_text("f[0]").unwrap().value.to_string(), "0");
    assert!(matches!(
        session.eval_text("f[InputString[\"prompt\"]]"),
        Err(EvalError::InputRequired { .. })
    ));
    assert_eq!(
        session
            .resume_suspended_evaluation("reply")
            .unwrap()
            .to_string(),
        "\"reply\""
    );
    assert!(Session::logical_profile_test_pauses() > 0);
    assert_leaf_rows(&Session::logical_profile_test_rows());
}

#[test]
fn lp03_argument_total_closes_each_pause() {
    let mut session = defined_session("f[x_, y_] := x");
    let _enabled = test_support::enabled();
    assert_eq!(session.eval_text("f[0, 1]").unwrap().value.to_string(), "0");
    assert!(matches!(
        session.eval_text("f[InputString[\"first\"], InputString[\"second\"]]"),
        Err(EvalError::InputRequired { .. })
    ));
    assert!(matches!(
        session.resume_suspended_evaluation("one"),
        Err(EvalError::InputRequired { .. })
    ));
    assert_eq!(
        session
            .resume_suspended_evaluation("two")
            .unwrap()
            .to_string(),
        "\"one\""
    );
    let rows = Session::logical_profile_test_rows();
    assert_leaf_rows(&rows);
    assert_eq!(row(&rows, "prepare_arguments.total")[0], 4);
    assert_eq!(row(&rows, "prepare_arguments.evaluate_arg")[0], 6);
    assert_eq!(row(&rows, "call.stage.prepare_arguments")[0], 4);
}

#[test]
fn lp03_adopted_site_table_is_exact() {
    let expected = std::collections::BTreeSet::from([
        "evaluate.total",
        "evaluate.once_unstable",
        "evaluate_inner_unstable.total",
        "evaluate_inner_unstable.symbol",
        "evaluate_inner_unstable.call",
        "evaluate_inner_unstable.atom",
        "evaluate_atom_fast_path.total",
        "evaluate_atom_fast_path.stable_symbol",
        "evaluate_atom_fast_path.unstable_symbol",
        "evaluate_atom_fast_path.atom",
        "evaluate_atom_fast_path.call",
        "evaluate_call.total",
        "evaluate_call.head_eval",
        "evaluate_call.resume_after_head",
        "evaluate_call.dispatch",
        "call.stage.special_form",
        "call.stage.prepare_arguments",
        "call.stage.listable",
        "call.stage.up_values",
        "call.stage.down_values",
        "call.stage.sub_values",
        "call.stage.ordinary_builtin",
        "call.stage.function",
        "prepare_arguments.total",
        "prepare_arguments.evaluate_arg",
        "definition.rhs.total",
        "definition.rhs.evaluate",
        "down_values.direct_default_lookup",
        "down_values.compiled_matcher",
        "sub_values.compiled_matcher",
        "set.rhs.evaluate",
        "set.rhs.suspend",
        "set.finish",
        "set.try_list_destructuring",
        "set.try_association_lvalue",
        "set.prepare_lhs",
        "while.test.evaluate",
        "while.test.suspend",
        "while.body.evaluate",
        "while.body.suspend",
        "compound.item.evaluate",
        "compound.item.suspend",
        "function.apply.evaluate",
        "catch.eval.total",
        "catch.body.evaluate",
        "catch.resume_after_body",
        "module.eval.total",
        "module.resume_initializers",
        "module.initializer.evaluate",
        "module.body.evaluate",
    ]);
    let actual = super::PhaseKey::ALL
        .iter()
        .map(|key| key.name())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actual, expected);
    for key in super::PhaseKey::ALL {
        assert_eq!(super::PhaseKey::from_name(key.name()), Some(key));
    }
}

// Disposable brief-657 context equivalence probes, identical on both arms.
use crate::runtime::profiling::logical::{AggregateKey, Call, Outcome, Policy, segments};
use crate::runtime::{BuiltinSpec, Expr, MemoryGrowthContract, SharedArgs};
use std::sync::{Arc, atomic::Ordering};

fn lp657_match(session: &Session, label: &str) {
    let control = Arc::clone(&session.request_interrupt_control.borrow());
    let inherited = segments::owner(&control);
    let key = AggregateKey::Runtime {
        phase: "function.apply.evaluate",
        source: None,
    };
    let expected = Call::start(session, key, Policy::Expr).expect("sessionful start");
    let reference = expected.inner.epoch.inspect(expected.inner.record).unwrap();
    assert!(Arc::ptr_eq(&reference.0.control, &control));
    eprintln!(
        "LP657 RECORD {label} reference: key={key:?} control=session generation={} root={} parent={:?}",
        reference.0.generation, reference.0.root, reference.1
    );
    expected.finish(Outcome::Ok);
    let actual = Call::start_from_active(key, Policy::Expr)
        .expect("session-free start must reproduce the sessionful owner-or-root rule");
    let observed = actual.inner.epoch.inspect(actual.inner.record).unwrap();
    assert!(
        Arc::ptr_eq(&observed.0.control, &control),
        "{label}: wrong session control"
    );
    assert_eq!(
        observed.0.generation, reference.0.generation,
        "{label}: generation"
    );
    assert_eq!(observed.1, reference.1, "{label}: parent");
    if let Some((_, parent_owner, parent)) = inherited {
        assert!(observed.0.same(&parent_owner));
        assert_eq!(observed.1, Some(parent));
    } else {
        assert_eq!(observed.0.generation, control.logical_generation());
        assert_eq!(observed.1, None);
        assert!(
            observed.0.root > reference.0.root,
            "each root mints a fresh id"
        );
    }
    eprintln!(
        "LP657 RECORD {label} candidate: key={key:?} control=session generation={} root={} parent={:?}",
        observed.0.generation, observed.0.root, observed.1
    );
    actual.finish(Outcome::Ok);
    let epoch = test_support::epoch();
    assert_eq!(epoch.faults.load(Ordering::Relaxed), 0);
    let rows: Vec<_> = epoch
        .rows()
        .into_iter()
        .filter(|(k, _)| *k == key)
        .collect();
    assert_eq!(rows.len(), 1);
    let c = rows[0].1;
    assert_eq!((c.calls, c.ok, c.err, c.open, c.abandoned), (2, 2, 0, 0, 0));
    eprintln!(
        "LP657 MATCH {label}: PASS key={key:?} calls=2 ok=2 err=0 abandoned=0 open=0 faults=0"
    );
}

fn lp657_reset_root(session: &mut Session, label: &str) {
    session.eval_rum_profile_reset(&[]).unwrap();
    assert!(segments::owner(&session.request_interrupt_control.borrow()).is_none());
    lp657_match(session, label);
}

fn lp657_handler(session: &mut Session, head: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    match head.to_string().as_str() {
        "LpProbeRoot" => lp657_reset_root(session, "reset-root"),
        "LpProbeChild" => lp657_match(session, "host-root-child-control"),
        "LpProbeReenter" => {
            let mut nested = lp657_session();
            assert!(!Arc::ptr_eq(
                &session.request_interrupt_control.borrow(),
                &nested.request_interrupt_control.borrow()
            ));
            nested.eval_text("LpProbeRoot[]")?;
            lp657_reset_root(session, "reentrancy-restored-parent");
        }
        "LpProbePanic" => {
            session.eval_rum_profile_reset(&[])?;
            panic!("brief-657 deliberate host callback unwind");
        }
        "LpProbeUnwind" => {
            let mut nested = lp657_session();
            let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                nested.eval_text("LpProbePanic[]")
            }));
            assert!(panic.is_err());
            drop(nested);
            lp657_reset_root(session, "unwind-restored-parent");
        }
        "LpProbeGate" => {
            use crate::runtime::profiling::{
                runtime_path_timing_start, state::LEXICAL_PROFILE_ACTIVE,
            };
            {
                let _publication = crate::runtime::profiling::lease::publish()
                    .expect("STOP: lease holder must publish profiling flags");
                LEXICAL_PROFILE_ACTIVE.store(2, Ordering::Relaxed)
            };
            assert!(runtime_path_timing_start("function.apply.evaluate").is_none());
            lp657_reset_root(session, "on-off-on-root");
        }
        "LpProbeGeneration" => {
            let prior = segments::owner(&session.request_interrupt_control.borrow())
                .unwrap()
                .1;
            let accepted = session.accept_cancellable_request().unwrap();
            assert_ne!(
                prior.generation,
                session
                    .request_interrupt_control
                    .borrow()
                    .logical_generation()
            );
            lp657_match(session, "live-owner-inherits-generation");
            drop(accepted);
        }
        "LpProbeRollover" => {
            // Reach the same exhaustion branch as MAX_REQUEST_GENERATION requests.
            session.next_request_generation.set(0);
            let before = Arc::clone(&session.request_interrupt_control.borrow());
            let accepted = session.accept_cancellable_request().unwrap();
            assert!(!Arc::ptr_eq(
                &before,
                &session.request_interrupt_control.borrow()
            ));
            lp657_reset_root(session, "request-control-rollover");
            drop(accepted);
        }
        other => panic!("unknown probe {other}"),
    }
    Ok(Expr::small_integer(1))
}

fn lp657_session() -> Session {
    let mut s = crate::runtime::test_builtins::session_with_test_control_builtins();
    for name in [
        "LpProbeRoot",
        "LpProbeChild",
        "LpProbeReenter",
        "LpProbePanic",
        "LpProbeUnwind",
        "LpProbeGate",
        "LpProbeGeneration",
        "LpProbeRollover",
    ] {
        s.register_builtin(BuiltinSpec::ordinary(
            name,
            "Disposable execution-context falsifier.",
            lp657_handler,
            MemoryGrowthContract::NonAmplifying,
        ));
    }
    s
}

fn lp657_quiescent() {
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(test_support::epoch().faults.load(Ordering::Relaxed), 0);
    eprintln!("LP657 QUIESCENT: PASS open=0 faults=0");
}

#[test]
fn lp657_host_root_control() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeChild[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_reset_mid_evaluation() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeRoot[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_reentrancy() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeReenter[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_two_sessions_one_thread() {
    let _enabled = test_support::enabled();
    let mut a = lp657_session();
    let mut b = lp657_session();
    assert!(!Arc::ptr_eq(
        &a.request_interrupt_control.borrow(),
        &b.request_interrupt_control.borrow()
    ));
    for index in 0..3 {
        let s = if index % 2 == 0 { &mut a } else { &mut b };
        assert_eq!(
            s.eval_text("LpProbeRoot[]").unwrap().value,
            Expr::small_integer(1)
        );
        lp657_quiescent();
    }
}

#[test]
fn lp657_thread_handoff_resume() {
    let enabled = test_support::enabled();
    let mut s = lp657_session();
    assert!(matches!(
        s.eval_request("InputString[\"handoff\"]; LpProbeRoot[]"),
        Err(_)
    ));
    let control = Arc::clone(&s.request_interrupt_control.borrow());
    let lease = enabled.lease();
    std::thread::scope(|scope| {
        scope
            .spawn(move || {
                lease.allow_current_thread();
                assert!(Arc::ptr_eq(&control, &s.request_interrupt_control.borrow()));
                assert_eq!(
                    s.resume_eval_request_with_input("reply").unwrap().value,
                    Expr::small_integer(1)
                );
                lp657_quiescent();
            })
            .join()
            .unwrap();
    });
}

#[test]
fn lp657_off_to_on() {
    let _enabled = test_support::enabled();
    {
        let _publication = crate::runtime::profiling::lease::publish()
            .expect("STOP: lease holder must publish profiling flags");
        crate::runtime::profiling::state::LEXICAL_PROFILE_ACTIVE.store(2, Ordering::Relaxed)
    };
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeRoot[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_on_to_off_to_on() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeGate[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_panic_unwind() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeUnwind[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_generation_change_inherits_open_owner() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeGeneration[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_request_control_rollover() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert_eq!(
        s.eval_text("LpProbeRollover[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp657_abandonment_cleanup() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    assert!(
        s.eval_request("WithCleanup[InputString[\"blocked\"],LpProbeRoot[]]")
            .is_err()
    );
    s.abandon_pending_suspension(
        crate::runtime::suspension::AbandonReason::HelperBestEffortDiscard,
    );
    lp657_quiescent();
}

// Disposable route and nested-rollover probes; append identically to both
// evaluator test modules after the calibrated baseline has been consumed.
fn lp658_assert_probe_record() {
    let key = AggregateKey::Runtime {
        phase: "function.apply.evaluate",
        source: None,
    };
    let rows = test_support::epoch().rows();
    let counts = rows.iter().find(|(actual, _)| *actual == key).unwrap().1;
    assert_eq!(
        (
            counts.calls,
            counts.ok,
            counts.err,
            counts.abandoned,
            counts.open
        ),
        (2, 2, 0, 0, 0)
    );
    lp657_quiescent();
}

fn lp658_route(route: &str) {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    let expr = s.lower_request("LpProbeRoot[]").unwrap().expr;
    let file = std::env::temp_dir().join(format!("brief658-{route}.rum"));
    if route.contains("file") {
        crate::runtime::tests::install_test_file_io(&mut s);
        std::fs::write(&file, "LpProbeRoot[]").unwrap();
    }
    match route {
        "evaluate" => {
            assert_eq!(s.evaluate(expr).unwrap(), Expr::small_integer(1));
        }
        "text" => {
            assert_eq!(
                s.eval_text("LpProbeRoot[]").unwrap().value,
                Expr::small_integer(1)
            );
        }
        "script" => {
            assert_eq!(
                s.eval_script("LpProbeRoot[]").unwrap().value,
                Expr::small_integer(1)
            );
        }
        "file" => {
            assert_eq!(s.eval_file(&file).unwrap().value, Expr::small_integer(1));
        }
        "request" => {
            assert_eq!(
                s.eval_request("LpProbeRoot[]").unwrap().value,
                Expr::small_integer(1)
            );
        }
        "expr_request" => {
            assert_eq!(
                s.eval_expr_request(expr).unwrap().value,
                Expr::small_integer(1)
            );
        }
        "accepted_request" => {
            let accepted = s.accept_cancellable_request().unwrap();
            assert_eq!(
                s.eval_accepted_request(accepted, "LpProbeRoot[]")
                    .unwrap()
                    .value,
                Expr::small_integer(1)
            );
        }
        "accepted_expr_request" => {
            let accepted = s.accept_cancellable_request().unwrap();
            assert_eq!(
                s.eval_accepted_expr_request(accepted, expr).unwrap().value,
                Expr::small_integer(1)
            );
        }
        "script_request" => {
            assert_eq!(
                s.eval_script_request("LpProbeRoot[]").unwrap().value,
                Expr::small_integer(1)
            );
        }
        "file_request" => {
            assert_eq!(
                s.eval_file_request(&file).unwrap().value,
                Expr::small_integer(1)
            );
        }
        "accepted_script_request" => {
            let accepted = s.accept_cancellable_request().unwrap();
            assert_eq!(
                s.eval_accepted_script_request(accepted, "LpProbeRoot[]")
                    .unwrap()
                    .value,
                Expr::small_integer(1)
            );
        }
        "accepted_file_request" => {
            let accepted = s.accept_cancellable_request().unwrap();
            assert_eq!(
                s.eval_accepted_file_request(accepted, &file).unwrap().value,
                Expr::small_integer(1)
            );
        }
        "request_resume" => {
            assert!(
                s.eval_request("InputString[\"route\"];LpProbeRoot[]")
                    .is_err()
            );
            assert_eq!(
                s.resume_eval_request_with_input("reply").unwrap().value,
                Expr::small_integer(1)
            );
        }
        "inspect" => {
            assert!(s.inspect_rewrite_rules(expr).unwrap().is_none());
        }
        "inspect_resume" => {
            let expr = s
                .lower_request("InputString[\"inspect\"];LpProbeRoot[]")
                .unwrap()
                .expr;
            assert!(s.inspect_rewrite_rules(expr).is_err());
            assert!(
                s.resume_inspect_rewrite_rules_with_input("reply")
                    .unwrap()
                    .is_none()
            );
        }
        "abandon" => {
            assert!(
                s.eval_request("WithCleanup[InputString[\"route\"],LpProbeRoot[]]")
                    .is_err()
            );
            s.abandon_pending_suspension(
                crate::runtime::suspension::AbandonReason::HelperBestEffortDiscard,
            );
        }
        "cancel" => {
            let accepted = s.accept_cancellable_request().unwrap();
            let cancel = accepted.cancel_handle();
            assert!(
                s.eval_accepted_request(
                    accepted,
                    "WithCleanup[InputString[\"route\"],LpProbeRoot[]]"
                )
                .is_err()
            );
            cancel.cancel();
            assert!(
                s.finish_cancelled_pending_evaluation_request()
                    .unwrap()
                    .is_some()
            );
        }
        "snapshot" => {
            assert_eq!(
                s.restore_snapshot_request("LpProbeRoot[]").unwrap().value,
                Expr::small_integer(1)
            );
        }
        "snapshot_abandon" => {
            assert!(
                s.restore_snapshot_request("WithCleanup[InputString[\"restore\"],LpProbeRoot[]]")
                    .is_err()
            );
        }
        "cache" => {
            // A cached stable value cannot invoke a phase start. A subsequent
            // guest evaluation must still establish the right execution owner.
            for _ in 0..3 {
                assert_eq!(
                    s.evaluate(Expr::small_integer(7)).unwrap(),
                    Expr::small_integer(7)
                );
            }
            assert_eq!(s.evaluate(expr).unwrap(), Expr::small_integer(1));
        }
        "random_state" => {
            assert_eq!(
                s.evaluate_with_random_state_restored(expr).unwrap(),
                Expr::small_integer(1)
            );
        }
        other => panic!("unknown entry route {other}"),
    }
    lp658_assert_probe_record();
    eprintln!(
        "LP658 ENTRY {route}: PASS key=function.apply.evaluate calls=2 ok=2 err=0 abandoned=0 open=0 faults=0 owner=session fresh_root=true"
    );
}

#[test]
fn lp658_entry_evaluate() {
    lp658_route("evaluate");
}
#[test]
fn lp658_entry_text() {
    lp658_route("text");
}
#[test]
fn lp658_entry_script() {
    lp658_route("script");
}
#[test]
fn lp658_entry_file() {
    lp658_route("file");
}
#[test]
fn lp658_entry_request() {
    lp658_route("request");
}
#[test]
fn lp658_entry_expr_request() {
    lp658_route("expr_request");
}
#[test]
fn lp658_entry_accepted_request() {
    lp658_route("accepted_request");
}
#[test]
fn lp658_entry_accepted_expr_request() {
    lp658_route("accepted_expr_request");
}
#[test]
fn lp658_entry_script_request() {
    lp658_route("script_request");
}
#[test]
fn lp658_entry_file_request() {
    lp658_route("file_request");
}
#[test]
fn lp658_entry_accepted_script_request() {
    lp658_route("accepted_script_request");
}
#[test]
fn lp658_entry_accepted_file_request() {
    lp658_route("accepted_file_request");
}
#[test]
fn lp658_entry_request_resume() {
    lp658_route("request_resume");
}
#[test]
fn lp658_entry_inspect() {
    lp658_route("inspect");
}
#[test]
fn lp658_entry_inspect_resume() {
    lp658_route("inspect_resume");
}
#[test]
fn lp658_entry_abandon() {
    lp658_route("abandon");
}
#[test]
fn lp658_entry_cancel() {
    lp658_route("cancel");
}
#[test]
fn lp658_entry_snapshot() {
    lp658_route("snapshot");
}
#[test]
fn lp658_entry_snapshot_abandon() {
    lp658_route("snapshot_abandon");
}
#[test]
fn lp658_entry_cache() {
    lp658_route("cache");
}
#[test]
fn lp658_entry_random_state() {
    lp658_route("random_state");
}

fn lp658_nested_rollover_handler(
    s: &mut Session,
    _: &Expr,
    _: &SharedArgs,
) -> Result<Expr, EvalError> {
    let before = Arc::clone(&s.request_interrupt_control.borrow());
    let expr = s
        .lower_request("WithCleanup[InputString[\"nested-rollover\"],LpProbeRollover[]]")
        .unwrap()
        .expr;
    assert!(matches!(
        s.evaluate(expr),
        Err(EvalError::InputRequired { .. })
    ));
    s.abandon_pending_suspension(
        crate::runtime::suspension::AbandonReason::HelperBestEffortDiscard,
    );
    assert!(!Arc::ptr_eq(&before, &s.request_interrupt_control.borrow()));
    lp657_reset_root(s, "nested-rollover-propagated-to-outer");
    Ok(Expr::small_integer(1))
}

#[test]
fn lp658_nested_same_session_rollover() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    s.register_builtin(BuiltinSpec::ordinary(
        "LpNestedRollover",
        "Disposable nested execution guard rollover.",
        lp658_nested_rollover_handler,
        MemoryGrowthContract::NonAmplifying,
    ));
    assert_eq!(
        s.eval_text("LpNestedRollover[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp658_assert_probe_record();
}

fn lp658_observe_deferred_compound(
    s: &mut Session,
    _: &Expr,
    _: &SharedArgs,
) -> Result<Expr, EvalError> {
    super::PHASE_LIFO.with_borrow(|stack| {
        let records: Vec<_> = stack.iter()
            .filter(|entry| entry.key == super::PhaseKey::CompoundItemEvaluate)
            .filter_map(|entry| entry.call.as_ref())
            .filter_map(|call| call.inner.epoch.inspect(call.inner.record).map(|record| (call, record)))
            .collect();
        assert_eq!(records.len(), 1, "exactly the post-reset compound item is live");
        let (call, (owner, parent, policy)) = &records[0];
        let key = call.inner.epoch.path(call.inner.record).unwrap().key;
        assert_eq!(key, AggregateKey::Runtime { phase: "compound.item.evaluate", source: None });
        assert!(Arc::ptr_eq(&owner.control, &s.request_interrupt_control.borrow()));
        assert_eq!(owner.generation, s.request_interrupt_control.borrow().logical_generation());
        assert_eq!(*parent, None);
        assert_eq!(*policy, Policy::Expr);
        assert!(owner.root > 0);
        eprintln!("LP658 DEFERRED RECORD key={key:?} calls=1 owner=session generation={} root={} parent=None policy=Expr", owner.generation, owner.root);
    });
    Ok(Expr::small_integer(1))
}

#[test]
fn lp658_deferred_exact_record() {
    let _enabled = test_support::enabled();
    let mut s = lp657_session();
    s.register_builtin(BuiltinSpec::ordinary(
        "RumProfileReset",
        "Actual reset.",
        |s, _, args| s.eval_rum_profile_reset(args.as_ref()),
        MemoryGrowthContract::NonAmplifying,
    ));
    s.register_builtin(BuiltinSpec::ordinary(
        "LpObserveDeferred",
        "Observe the real compound phase.",
        lp658_observe_deferred_compound,
        MemoryGrowthContract::NonAmplifying,
    ));
    {
        let _publication = crate::runtime::profiling::lease::publish()
            .expect("STOP: lease holder must publish profiling flags");
        crate::runtime::profiling::state::LEXICAL_PROFILE_ACTIVE.store(2, Ordering::Relaxed)
    };
    assert_eq!(
        s.eval_text("RumProfileReset[];LpObserveDeferred[]")
            .unwrap()
            .value,
        Expr::small_integer(1)
    );
    let key = AggregateKey::Runtime {
        phase: "compound.item.evaluate",
        source: None,
    };
    let rows = test_support::epoch().rows();
    let c = rows.iter().find(|(actual, _)| *actual == key).unwrap().1;
    assert_eq!((c.calls, c.ok, c.err, c.abandoned, c.open), (1, 1, 0, 0, 0));
    lp657_quiescent();
    eprintln!(
        "LP658 DEFERRED TERMINAL: PASS key=compound.item.evaluate source=None calls=1 ok=1 err=0 abandoned=0 open=0 faults=0"
    );
}

// Brief 660: registered host callbacks combine the previously separate routes.
fn lp660_reset_suspend(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    s.eval_rum_profile_reset(&[])?;
    assert!(segments::owner(&s.request_interrupt_control.borrow()).is_none());
    eprintln!("LP660 SUSPEND: reset=true session_owner=None direct_input=true");
    s.eval_input_string(&[Expr::string("combined")])
}

fn lp660_observe(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    super::PHASE_LIFO.with_borrow(|stack| {
        let records: Vec<_> = stack.iter()
            .filter(|entry| entry.key == super::PhaseKey::CompoundItemEvaluate)
            .filter_map(|entry| entry.call.as_ref())
            .filter_map(|call| call.inner.epoch.inspect(call.inner.record).map(|record| (call, record)))
            .collect();
        assert_eq!(records.len(), 1, "exactly the resumed compound item must exist");
        let (call, (owner, parent, policy)) = &records[0];
        let key = call.inner.epoch.path(call.inner.record).unwrap().key;
        let own = Arc::ptr_eq(&owner.control, &s.request_interrupt_control.borrow());
        eprintln!("LP660 RESUMED RECORD key={key:?} owner=session:{own} generation={} root={} parent={parent:?} policy={policy:?}", owner.generation, owner.root);
        assert_eq!(key, AggregateKey::Runtime { phase: "compound.item.evaluate", source: None });
        assert!(own, "resumed free phase inherited another session's owner");
        assert_eq!(owner.generation, s.request_interrupt_control.borrow().logical_generation());
        assert_eq!(*parent, None, "no captured open segment at this resume");
        assert_eq!(*policy, Policy::Expr);
        assert!(owner.root > 0);
    });
    Ok(Expr::small_integer(1))
}

fn lp660_session() -> Session {
    let mut s = lp657_session();
    for (name, handler) in [
        (
            "Lp660ResetSuspend",
            lp660_reset_suspend as fn(&mut Session, &Expr, &SharedArgs) -> Result<Expr, EvalError>,
        ),
        ("Lp660Observe", lp660_observe),
        ("Lp660DeferredNested", lp660_deferred_nested),
        ("Lp660NestedResume", lp660_nested_resume),
    ] {
        s.register_builtin(BuiltinSpec::ordinary(
            name,
            "Disposable combined context falsifier.",
            handler,
            MemoryGrowthContract::NonAmplifying,
        ));
    }
    s
}

fn lp660_deferred_nested(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    s.eval_rum_profile_reset(&[])?;
    let mut nested = lp660_session();
    assert!(!Arc::ptr_eq(
        &s.request_interrupt_control.borrow(),
        &nested.request_interrupt_control.borrow()
    ));
    assert_eq!(nested.eval_text("7").unwrap().value, Expr::small_integer(7));
    assert!(segments::owner(&s.request_interrupt_control.borrow()).is_none());
    lp657_match(s, "combined-a-deferred-reset-nested-free");
    Ok(Expr::small_integer(1))
}

#[test]
fn lp660_combined_a_deferred_reset_nested_free() {
    let _enabled = test_support::enabled();
    let mut s = lp660_session();
    {
        let _publication = crate::runtime::profiling::lease::publish()
            .expect("STOP: lease holder must publish profiling flags");
        crate::runtime::profiling::state::LEXICAL_PROFILE_ACTIVE.store(2, Ordering::Relaxed)
    };
    assert_eq!(
        s.eval_text("Lp660DeferredNested[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
    eprintln!(
        "LP660 COMBINED a: PASS deferred/reset/nested-other/free owner=outer calls=2 ok=2 err=0 abandoned=0 open=0 faults=0"
    );
}

fn lp660_resume_terminal(label: &str) {
    let key = AggregateKey::Runtime {
        phase: "compound.item.evaluate",
        source: None,
    };
    let rows = test_support::epoch().rows();
    let c = rows.iter().find(|(actual, _)| *actual == key).unwrap().1;
    assert_eq!((c.calls, c.ok, c.err, c.abandoned, c.open), (1, 1, 0, 0, 0));
    lp657_quiescent();
    eprintln!(
        "LP660 COMBINED {label}: PASS key=compound.item.evaluate source=None calls=1 ok=1 err=0 abandoned=0 open=0 faults=0"
    );
}

fn lp660_handoff(deferred: bool) {
    let enabled = test_support::enabled();
    let mut s = lp660_session();
    if deferred {
        {
            let _publication = crate::runtime::profiling::lease::publish()
                .expect("STOP: lease holder must publish profiling flags");
            crate::runtime::profiling::state::LEXICAL_PROFILE_ACTIVE.store(2, Ordering::Relaxed)
        };
    }
    assert!(
        s.eval_request("Lp660ResetSuspend[];Lp660Observe[]")
            .is_err()
    );
    let control = Arc::clone(&s.request_interrupt_control.borrow());
    assert!(segments::owner(&control).is_none());
    let lease = enabled.lease();
    std::thread::scope(|threads| {
        threads
            .spawn(move || {
                lease.allow_current_thread();
                assert!(segments::owner(&control).is_none());
                assert!(Arc::ptr_eq(&control, &s.request_interrupt_control.borrow()));
                assert_eq!(
                    s.resume_eval_request_with_input("reply").unwrap().value,
                    Expr::small_integer(1)
                );
                lp660_resume_terminal(if deferred {
                    "c-deferred-reset-suspend-handoff"
                } else {
                    "b-reset-suspend-handoff"
                });
                // A subsequent, different session observes only its own entry control.
                let mut next = lp657_session();
                assert_eq!(
                    next.eval_text("LpProbeRoot[]").unwrap().value,
                    Expr::small_integer(1)
                );
                lp657_quiescent();
            })
            .join()
            .unwrap();
    });
}

#[test]
fn lp660_combined_b_reset_suspend_handoff() {
    lp660_handoff(false);
}

#[test]
fn lp660_combined_c_deferred_reset_suspend_handoff() {
    lp660_handoff(true);
}

thread_local! {
    static LP660_NESTED: std::cell::RefCell<Option<Session>> = const { std::cell::RefCell::new(None) };
}

fn lp660_nested_resume(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    let mut nested = LP660_NESTED.with_borrow_mut(|cell| cell.take().unwrap());
    assert!(segments::owner(&s.request_interrupt_control.borrow()).is_some());
    assert!(segments::owner(&nested.request_interrupt_control.borrow()).is_none());
    assert_eq!(
        nested
            .resume_eval_request_with_input("reply")
            .unwrap()
            .value,
        Expr::small_integer(1)
    );
    drop(nested);
    lp657_reset_root(s, "combined-d-nested-resume-restored-outer");
    Ok(Expr::small_integer(1))
}

#[test]
fn lp660_combined_d_nested_other_session_resume() {
    let _enabled = test_support::enabled();
    let mut nested = lp660_session();
    {
        let _publication = crate::runtime::profiling::lease::publish()
            .expect("STOP: lease holder must publish profiling flags");
        crate::runtime::profiling::state::LEXICAL_PROFILE_ACTIVE.store(2, Ordering::Relaxed)
    };
    assert!(
        nested
            .eval_request("Lp660ResetSuspend[];Lp660Observe[]")
            .is_err()
    );
    LP660_NESTED.with_borrow_mut(|cell| *cell = Some(nested));
    let mut outer = lp660_session();
    assert_eq!(
        outer.eval_text("Lp660NestedResume[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
    eprintln!(
        "LP660 COMBINED d: PASS nested-other-resume owner=inner restored=outer function.calls=2 function.ok=2 err=0 abandoned=0 open=0 faults=0"
    );
}

// Brief 664: a pending context must not retain its control across rollover.
fn lp664_carrier_observe(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    lp657_reset_root(s, "carrier-rollover-handoff-reset-private-resume");
    eprintln!(
        "LP664 CARRIER RECORD: PASS free-start=current-session after rollover, handoff, profile reset and private resume"
    );
    Ok(Expr::small_integer(1))
}

#[test]
fn lp664_carrier_rollover_handoff_reset_private_resume() {
    let enabled = test_support::enabled();
    let mut s = lp657_session();
    s.register_builtin(BuiltinSpec::ordinary(
        "Lp664CarrierObserve",
        "Disposable carrier falsifier.",
        lp664_carrier_observe,
        MemoryGrowthContract::NonAmplifying,
    ));
    assert!(
        s.eval_request("InputString[];Lp664CarrierObserve[]")
            .is_err()
    );
    let old = Arc::clone(&s.request_interrupt_control.borrow());
    s.next_request_generation.set(0);
    let accepted = s.accept_cancellable_request().unwrap();
    let current = Arc::clone(&s.request_interrupt_control.borrow());
    assert!(!Arc::ptr_eq(&old, &current));
    drop(accepted);
    let lease = enabled.lease();
    std::thread::scope(|threads| {
        threads.spawn(move || {
            lease.allow_current_thread();
            assert!(Arc::ptr_eq(&current, &s.request_interrupt_control.borrow()));
            assert_eq!(s.resume_suspended_evaluation("reply").unwrap(), Expr::small_integer(1));
            lp657_quiescent();
            let mut next = lp657_session();
            assert_eq!(next.eval_text("LpProbeRoot[]").unwrap().value, Expr::small_integer(1));
            lp657_quiescent();
            eprintln!("LP664 CARRIER LIFETIME: PASS subsequent session receives its own control; open=0 faults=0");
        }).join().unwrap();
    });
}

thread_local! {
    static LP664_PANIC_RESUME: std::cell::RefCell<Option<Session>> = const { std::cell::RefCell::new(None) };
}

fn lp664_catch_resume(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    let mut nested = LP664_PANIC_RESUME.with_borrow_mut(|slot| slot.take().unwrap());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        nested.resume_suspended_evaluation("reply")
    }));
    assert!(panic.is_err());
    drop(nested);
    lp657_reset_root(s, "carrier-unwind-restores-outer");
    eprintln!(
        "LP664 CARRIER UNWIND: PASS panic crosses private resume; next free start belongs to outer session"
    );
    Ok(Expr::small_integer(1))
}

#[test]
fn lp664_carrier_unwind_restores_outer() {
    let _enabled = test_support::enabled();
    let mut nested = lp657_session();
    assert!(nested.eval_request("InputString[];LpProbePanic[]").is_err());
    LP664_PANIC_RESUME.with_borrow_mut(|slot| *slot = Some(nested));
    let mut outer = lp657_session();
    outer.register_builtin(BuiltinSpec::ordinary(
        "Lp664CatchResume",
        "Disposable resume unwind falsifier.",
        lp664_catch_resume,
        MemoryGrowthContract::NonAmplifying,
    ));
    assert_eq!(
        outer.eval_text("Lp664CatchResume[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

fn lp664_rollover_loop(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    let n: usize = std::env::var("LP664_ROLLOVERS")
        .unwrap_or_else(|_| "1".into())
        .parse()
        .unwrap();
    for _ in 0..n {
        s.next_request_generation.set(0);
        let before = Arc::clone(&s.request_interrupt_control.borrow());
        let request = s.accept_cancellable_request().unwrap();
        assert!(!Arc::ptr_eq(&before, &s.request_interrupt_control.borrow()));
        drop(request);
    }
    eprintln!("LP664 ROLLOVER COST: PASS replacements={n}");
    Ok(Expr::small_integer(1))
}

#[test]
fn lp664_rare_rollover_cost() {
    let enabled = std::env::var_os("LP664_ROLLOVER_ENABLED").is_some();
    let _enabled = enabled.then(test_support::enabled);
    let mut session = lp657_session();
    session.register_builtin(BuiltinSpec::ordinary(
        "Lp664RolloverLoop",
        "Disposable cold-path cost probe.",
        lp664_rollover_loop,
        MemoryGrowthContract::NonAmplifying,
    ));
    assert_eq!(
        session.eval_text("Lp664RolloverLoop[]").unwrap().value,
        Expr::small_integer(1)
    );
    if enabled {
        lp657_quiescent();
    }
}

fn lp664_reset_between_pause_and_resume(deferred: bool) {
    let _enabled = test_support::enabled();
    let mut s = lp660_session();
    if deferred {
        {
            let _publication = crate::runtime::profiling::lease::publish()
                .expect("STOP: lease holder must publish profiling flags");
            crate::runtime::profiling::state::LEXICAL_PROFILE_ACTIVE.store(2, Ordering::Relaxed)
        };
    }
    assert!(s.eval_request("InputString[];Lp660Observe[]").is_err());
    let mut other = lp657_session();
    other.eval_rum_profile_reset(&[]).unwrap();
    assert_eq!(
        s.resume_suspended_evaluation("reply").unwrap(),
        Expr::small_integer(1)
    );
    lp660_resume_terminal(if deferred {
        "carrier-inactive-pause-external-enable"
    } else {
        "carrier-external-reset-invalidates-context"
    });
    eprintln!(
        "LP664 EXTERNAL RESET: PASS deferred={deferred} resumed compound phase owner=current session"
    );
}

#[test]
fn lp664_carrier_external_reset_invalidates_context() {
    lp664_reset_between_pause_and_resume(false);
}

#[test]
fn lp664_carrier_inactive_pause_external_enable() {
    lp664_reset_between_pause_and_resume(true);
}

#[test]
fn lp664_carrier_cap_pause_external_reset() {
    let _enabled = test_support::enabled();
    // Force the actual production cap-retirement branch on an existing epoch.
    // A zero test_limit alone leaves no epoch and permits lazy default creation.
    let retired = test_support::epoch();
    assert!(retired.storage(32 * 1024 * 1024).is_none());
    assert!(!retired.is_valid());
    let mut s = lp660_session();
    assert!(s.eval_request("InputString[];Lp660Observe[]").is_err());
    assert!(!retired.is_valid());
    let mut other = lp657_session();
    other.eval_rum_profile_reset(&[]).unwrap();
    assert_eq!(
        s.resume_suspended_evaluation("reply").unwrap(),
        Expr::small_integer(1)
    );
    lp660_resume_terminal("carrier-cap-pause-external-reset");
    eprintln!(
        "LP664 CAP RESET: PASS resumed phase owner=current session after confirmed diagnostic-cap retirement"
    );
}

// Brief 665: preserve the missing selection event as an exact owner assertion.
thread_local! {
    static LP665_CAP_NESTED: std::cell::RefCell<Option<Session>> = const { std::cell::RefCell::new(None) };
}

fn lp665_nested_cap_resume(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    let mut nested = LP665_CAP_NESTED.with_borrow_mut(|cell| cell.take().unwrap());
    let foreign = Arc::clone(&s.request_interrupt_control.borrow());
    assert!(!Arc::ptr_eq(
        &foreign,
        &nested.request_interrupt_control.borrow()
    ));
    assert_eq!(
        nested.resume_suspended_evaluation("reply").unwrap(),
        Expr::small_integer(1)
    );
    drop(nested);
    lp657_reset_root(s, "665-cap-nested-private-restores-outer");
    eprintln!(
        "LP665 NESTED CAP: PASS direct private resume owner=inner restored=outer after cap, rollover, external reset"
    );
    Ok(Expr::small_integer(1))
}

#[test]
fn lp665_carrier_cap_nested_rollover_external_reset() {
    let _enabled = test_support::enabled();
    let retired = test_support::epoch();
    assert!(retired.storage(32 * 1024 * 1024).is_none());
    assert!(!retired.is_valid());
    let mut nested = lp660_session();
    assert!(nested.eval_request("InputString[];Lp660Observe[]").is_err());
    let previous = Arc::clone(&nested.request_interrupt_control.borrow());
    nested.next_request_generation.set(0);
    drop(nested.accept_cancellable_request().unwrap());
    assert!(!Arc::ptr_eq(
        &previous,
        &nested.request_interrupt_control.borrow()
    ));
    LP665_CAP_NESTED.with_borrow_mut(|cell| *cell = Some(nested));
    let mut outer = lp660_session();
    outer.eval_rum_profile_reset(&[]).unwrap();
    outer.register_builtin(BuiltinSpec::ordinary(
        "Lp665NestedCapResume",
        "Disposable exact resumed owner falsifier.",
        lp665_nested_cap_resume,
        MemoryGrowthContract::NonAmplifying,
    ));
    assert_eq!(
        outer.eval_text("Lp665NestedCapResume[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp665_carrier_cap_handoff_external_reset() {
    let enabled = test_support::enabled();
    let retired = test_support::epoch();
    assert!(retired.storage(32 * 1024 * 1024).is_none());
    assert!(!retired.is_valid());
    let mut s = lp660_session();
    assert!(s.eval_request("InputString[];Lp660Observe[]").is_err());
    let old = Arc::clone(&s.request_interrupt_control.borrow());
    s.next_request_generation.set(0);
    drop(s.accept_cancellable_request().unwrap());
    assert!(!Arc::ptr_eq(&old, &s.request_interrupt_control.borrow()));
    let mut other = lp657_session();
    other.eval_rum_profile_reset(&[]).unwrap();
    assert_eq!(other.eval_text("7").unwrap().value, Expr::small_integer(7));
    let lease = enabled.lease();
    std::thread::scope(|threads| {
        threads.spawn(move || {
            lease.allow_current_thread();
            assert_eq!(s.resume_suspended_evaluation("reply").unwrap(), Expr::small_integer(1));
            lp660_resume_terminal("665-cap-rollover-handoff-external-reset");
            eprintln!("LP665 HANDOFF CAP: PASS owner=current session after cap, rollover, handoff and external reset; open=0 faults=0");
        }).join().unwrap();
    });
}

fn lp665_cap_unwind_observe(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    let mut nested = LP664_PANIC_RESUME.with_borrow_mut(|cell| cell.take().unwrap());
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        nested.resume_suspended_evaluation("reply")
    }));
    assert!(unwind.is_err());
    drop(nested);
    let actual = Call::start_from_active(
        AggregateKey::Runtime {
            phase: "function.apply.evaluate",
            source: None,
        },
        Policy::Expr,
    )
    .expect("outer execution control survives inner unwind");
    let (owner, _, _) = actual.inner.epoch.inspect(actual.inner.record).unwrap();
    assert!(Arc::ptr_eq(
        &owner.control,
        &s.request_interrupt_control.borrow()
    ));
    actual.finish(Outcome::Ok);
    lp657_reset_root(s, "665-cap-private-unwind-restores-outer");
    eprintln!(
        "LP665 UNWIND CAP: PASS panic across cap-retired private resume restores exact outer control before any new sessionful start"
    );
    Ok(Expr::small_integer(1))
}

#[test]
fn lp665_carrier_cap_unwind_restores_outer() {
    let _enabled = test_support::enabled();
    let retired = test_support::epoch();
    assert!(retired.storage(32 * 1024 * 1024).is_none());
    assert!(!retired.is_valid());
    let mut nested = lp657_session();
    assert!(nested.eval_request("InputString[];LpProbePanic[]").is_err());
    LP664_PANIC_RESUME.with_borrow_mut(|cell| *cell = Some(nested));
    let mut outer = lp657_session();
    outer.eval_rum_profile_reset(&[]).unwrap();
    outer.register_builtin(BuiltinSpec::ordinary(
        "Lp665CapUnwindObserve",
        "Disposable cap and unwind carrier falsifier.",
        lp665_cap_unwind_observe,
        MemoryGrowthContract::NonAmplifying,
    ));
    assert_eq!(
        outer.eval_text("Lp665CapUnwindObserve[]").unwrap().value,
        Expr::small_integer(1)
    );
    lp657_quiescent();
}

#[test]
fn lp665_carrier_cap_external_reset_second_pause() {
    let _enabled = test_support::enabled();
    let retired = test_support::epoch();
    assert!(retired.storage(32 * 1024 * 1024).is_none());
    assert!(!retired.is_valid());
    let mut s = lp660_session();
    assert!(s.eval_request("InputString[];InputString[];7").is_err());
    assert!(!retired.is_valid());
    let mut other = lp657_session();
    other.eval_rum_profile_reset(&[]).unwrap();
    assert!(s.resume_suspended_evaluation("first").is_err());
    let key = AggregateKey::Runtime {
        phase: "compound.item.evaluate",
        source: None,
    };
    let rows = test_support::epoch().rows();
    let c = rows.iter().find(|(actual, _)| *actual == key).unwrap().1;
    assert_eq!(
        (c.calls, c.ok, c.err, c.abandoned, c.open),
        (0, 0, 0, 0, 1),
        "fresh post-reset phase must remain open at the second pause"
    );
    assert_eq!(
        s.resume_suspended_evaluation("second").unwrap(),
        Expr::small_integer(7)
    );
    let rows = test_support::epoch().rows();
    let c = rows.iter().find(|(actual, _)| *actual == key).unwrap().1;
    assert_eq!((c.calls, c.ok, c.err, c.abandoned, c.open), (2, 2, 0, 0, 0));
    lp657_quiescent();
    eprintln!(
        "LP665 CAP SECOND PAUSE: PASS fresh epoch phase remains open across second pause and completes exactly; calls=2 ok=2 err=0 abandoned=0 open=0 faults=0"
    );
}
