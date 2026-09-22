//! Brief 518's shadow-route contract. Language results follow the guide's
//! evaluation, sequencing, resource-limit and resumable-input contracts.
use crate::install_core;
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{
    AccountingContext, BuiltinSpec, EvalError, HoldAttribute, MemoryGrowthContract, system_expr,
};
use rum_runtime::{Session, SessionRequestError};

fn session(armed: bool) -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    if armed {
        let mut limits = session.resource_limits();
        limits.eval_memory_limit = Some(1_000_000_000);
        session.set_resource_limits(limits);
    }
    session
}

#[test]
fn recursive_evaluation_retains_the_session_shadow_and_balances_stack() {
    let mut session = session(true);
    session.eval_text("f[x_] := x + 1").unwrap();
    let shadow = session.accounting_shadow().unwrap();
    let before = shadow.snapshot();
    assert_eq!(
        session.eval_text("f[f[f[1]]]").unwrap().value.to_string(),
        "4"
    );
    let after = shadow.snapshot();
    assert!(after.pushes > before.pushes + 2, "{after:?}");
    assert!(after.peak >= 3, "{after:?}");
    assert_eq!(after.live, 0);
    assert_eq!(after.pushes, after.pops);
    println!("recursive crossing: {before:?} -> {after:?}");
}

fn legacy_probe(session: &mut Session, _head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    session.evaluate(args[0].clone())
}

fn accounted_probe(
    context: &mut AccountingContext<'_>,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let observer = context.shadow();
    let before = observer.snapshot();
    let value = context.evaluate(args[0].clone())?;
    assert!(observer.snapshot().pushes > before.pushes);
    Ok(value)
}

#[test]
fn builtin_adapter_and_catalog_pilot_reenter_guest_evaluation() {
    let mut session = session(true);
    session.register_accounting_builtin(
        BuiltinSpec::ordinary_with_hold(
            "RouteProbe",
            "Route test.",
            HoldAttribute::HoldAll,
            legacy_probe,
            MemoryGrowthContract::Delegated,
        ),
        accounted_probe,
    );
    let outcome = session.eval_text("RouteProbe[(a = 4; a + 1)]").unwrap();
    assert_eq!(outcome.value.to_string(), "5");
    let observed = session.accounting_shadow().unwrap().snapshot();
    assert!(observed.callbacks >= 2, "{observed:?}");
    assert!(observed.walker_fallbacks > 0, "{observed:?}");
    assert_eq!(observed.pushes, observed.pops);
    println!("callback crossing: {observed:?}");
}

#[test]
fn host_mutation_has_an_explicit_armed_entry() {
    let mut session = session(true);
    session.eval_text("Null").unwrap();
    let shadow = session.accounting_shadow().unwrap();
    session
        .accounting_context()
        .unwrap()
        .install_system_own_value("RouteValue", Expr::small_integer(41));
    let value = session
        .accounting_context()
        .unwrap()
        .evaluate(system_expr("RouteValue"))
        .unwrap();
    assert_eq!(value, Expr::small_integer(41));
    assert_eq!(shadow.snapshot().host_mutations, 1);
    assert_eq!(shadow.snapshot().live, 0);
    println!("host crossing: {:?}", shadow.snapshot());
}

#[test]
fn suspended_host_entry_and_two_resumes_keep_the_explicit_frame_token() {
    let mut session = session(false);
    let failure = session.eval_request(
        "MemoryConstrained[(InputString[\"first\"]; InputString[\"second\"]; System`RouteValue + 1), 1000000000]"
    ).unwrap_err();
    assert!(matches!(
        failure.error,
        SessionRequestError::InputRequired { .. }
    ));
    let shadow = session.accounting_shadow().unwrap();
    let suspended = shadow.snapshot();
    assert_eq!(suspended.suspends, 1);
    assert_eq!(suspended.live, 0);
    session
        .accounting_context()
        .unwrap()
        .install_system_own_value("RouteValue", Expr::small_integer(41));
    let second = session.resume_eval_request_with_input("one").unwrap_err();
    assert!(matches!(
        second.error,
        SessionRequestError::InputRequired { .. }
    ));
    assert_eq!(shadow.snapshot().resumes, 1);
    assert_eq!(shadow.snapshot().suspends, 2);
    let outcome = session.resume_eval_request_with_input("two").unwrap();
    assert_eq!(outcome.value.to_string(), "42");
    let completed = shadow.snapshot();
    assert_eq!(completed.resumes, 2);
    assert_eq!(completed.host_mutations, 1);
    assert_eq!(completed.live, 0);
    assert_eq!(completed.pushes, completed.pops);
    assert!(session.accounting_context().is_none());
    session.eval_text("1 + 2").unwrap();
    assert_eq!(
        shadow.snapshot(),
        completed,
        "unarmed work must not charge the old handle"
    );
    println!("suspension crossing: {suspended:?} -> {completed:?}");
}

#[test]
fn nested_memory_constrained_arms_from_unarmed_and_the_walker_still_refuses() {
    let mut session = session(false);
    assert!(session.accounting_shadow().is_none());
    let value = session
        .eval_text("MemoryConstrained[(MemoryConstrained[(1; 2), 0, 7]; 3), 1000000000]")
        .unwrap()
        .value;
    assert_eq!(value, Expr::small_integer(3));
    let observed = session.accounting_shadow().unwrap().snapshot();
    assert_eq!(observed.arms, 2);
    assert!(observed.callbacks > 0);
    assert_eq!(observed.pushes, observed.pops);
    assert_eq!(session.active_memory_limit(), None);
    assert_eq!(
        session
            .eval_text("MemoryConstrained[Range[100], 0, 123]")
            .unwrap()
            .value,
        Expr::small_integer(123)
    );
    println!("nested limit transition: {observed:?}");
}

fn replacement(
    _session: &mut Session,
    _head: &Expr,
    _args: &SharedArgs,
) -> Result<Expr, EvalError> {
    Ok(Expr::small_integer(99))
}

#[test]
fn old_registration_keeps_its_shape_and_replaces_the_companion_with_fallback() {
    let mut session = session(false);
    let original = *session
        .registered_builtin_specs()
        .find(|spec| spec.name == "CompoundExpression")
        .unwrap();
    // The old public struct-literal and fn-pointer surfaces still compile.
    session.register_builtin(BuiltinSpec {
        handler: Some(replacement),
        ..original
    });
    assert_eq!(
        session
            .eval_text("MemoryConstrained[(1; 2), 1000000000]")
            .unwrap()
            .value,
        Expr::small_integer(99)
    );
    let observed = session.accounting_shadow().unwrap().snapshot();
    assert_eq!(observed.callbacks, 0);
    assert!(observed.walker_fallbacks > 0);
    println!("legacy registration: {observed:?}");
}

#[test]
fn unarmed_evaluation_never_allocates_a_shadow() {
    let mut session = session(false);
    session.eval_text("a = 1; f[x_] := x + a; f[f[1]]").unwrap();
    assert!(session.accounting_shadow().is_none());
    assert!(session.accounting_context().is_none());
}
