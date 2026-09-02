use rum_core::{BigInt, Expr, SharedArgs};
use rum_host::{HostConfig, HostEvalOutcome, HostEvalSuccess, HostSession};
use rum_runtime::kernel_api::MemoryGrowthContract;
use rum_runtime::kernel_api::{BuiltinSpec, Capability, EvalError, Session, expr_as_exact_integer};

fn successful_eval(outcome: HostEvalOutcome) -> HostEvalSuccess {
    match outcome {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected success, got {other:?}"),
    }
}

fn assert_success_text(outcome: HostEvalOutcome, expected: &str) -> HostEvalSuccess {
    let success = successful_eval(outcome);
    assert_eq!(success.value.text, expected);
    success
}

fn assert_sandbox_denied(session: &mut HostSession, source: &str) {
    let success = successful_eval(session.eval(source));
    assert_eq!(success.value.text, "$Failed", "source {source}");
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox"),
        "source {source} messages {:?}",
        success.messages
    );
}

fn install_demo_family(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary(
        "DemoDouble",
        "Doubles an exact integer for host installer tests.",
        demo_double,
        MemoryGrowthContract::Preflight,
    ));
    session.register_builtin(
        BuiltinSpec::ordinary(
            "DemoRead",
            "Reports that a host installer FileRead builtin reached dispatch.",
            demo_read,
            MemoryGrowthContract::NonAmplifying,
        )
        .with_capability(Capability::FileRead),
    );
}

fn demo_double(session: &mut Session, _head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DemoDouble", args, 1));
    };
    let Some(value) = expr_as_exact_integer(arg) else {
        return Ok(session.invalid_call_with_arg_form(
            "DemoDouble",
            args,
            "expected an exact integer",
        ));
    };

    Ok(Expr::integer(value * BigInt::from(2_u8)))
}

fn demo_read(session: &mut Session, _head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !args.is_empty() {
        return Ok(session.invalid_call_with_exact_arg_count("DemoRead", args, 0));
    }

    Ok(Expr::string("demo-read"))
}

fn install_order_first(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary(
        "DemoOrder",
        "First installer order probe.",
        demo_order_first,
        MemoryGrowthContract::NonAmplifying,
    ));
}

fn install_order_second(session: &mut Session) {
    session.register_builtin(BuiltinSpec::ordinary(
        "DemoOrder",
        "Second installer order probe.",
        demo_order_second,
        MemoryGrowthContract::NonAmplifying,
    ));
}

fn demo_order_first(
    _session: &mut Session,
    _head: &Expr,
    _args: &SharedArgs,
) -> Result<Expr, EvalError> {
    Ok(Expr::string("first"))
}

fn demo_order_second(
    _session: &mut Session,
    _head: &Expr,
    _args: &SharedArgs,
) -> Result<Expr, EvalError> {
    Ok(Expr::string("second"))
}

#[test]
fn installer_builtins_obey_capabilities_and_survive_reset() {
    let mut trusted = HostSession::builder()
        .with_builtin_installer(install_demo_family)
        .build();

    assert_success_text(trusted.eval("DemoDouble[21]"), "42");
    assert_success_text(trusted.eval("DemoRead[]"), "\"demo-read\"");
    assert_success_text(trusted.eval("x = 7; x"), "7");
    trusted.reset();
    assert_success_text(trusted.eval("x"), "x");
    assert_success_text(trusted.eval("DemoDouble[21]"), "42");
    assert_success_text(trusted.eval("DemoRead[]"), "\"demo-read\"");

    let mut sandboxed = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .with_builtin_installer(install_demo_family)
        .build();

    assert_success_text(sandboxed.eval("DemoDouble[21]"), "42");
    assert_sandbox_denied(&mut sandboxed, "DemoRead[]");
    sandboxed.reset();
    assert_success_text(sandboxed.eval("DemoDouble[22]"), "44");
    assert_sandbox_denied(&mut sandboxed, "DemoRead[]");
}

#[test]
fn installer_config_clones_reapply_downstream_builtins() {
    let config = HostConfig::default().with_builtin_installer(install_demo_family);
    let mut first = HostSession::builder().with_config(config.clone()).build();
    let mut second = HostSession::builder().with_config(config).build();

    assert_success_text(first.eval("DemoDouble[5]"), "10");
    assert_success_text(second.eval("DemoDouble[6]"), "12");
    first.reset();
    second.reset();
    assert_success_text(first.eval("DemoRead[]"), "\"demo-read\"");
    assert_success_text(second.eval("DemoRead[]"), "\"demo-read\"");
}

#[test]
fn installer_order_and_no_core_path_are_preserved() {
    let mut session = HostSession::builder()
        .with_core_builtins(false)
        .with_builtin_installer(install_order_first)
        .with_builtin_installer(install_order_second)
        .with_builtin_installer(install_demo_family)
        .build();

    assert!(!session.configuration().install_core_builtins);
    assert_success_text(session.eval("1 + 2"), "Plus[1, 2]");
    assert_success_text(session.eval("DemoOrder[]"), "\"second\"");
    assert_success_text(session.eval("DemoDouble[8]"), "16");
}

#[test]
fn installer_snapshot_restore_round_trips_definitions_not_builtin_handlers() {
    let mut source = HostSession::builder()
        .with_builtin_installer(install_demo_family)
        .build();
    assert_success_text(source.eval("twice[x_] := DemoDouble[x]"), "Null");
    assert_success_text(source.eval("twice[5]"), "10");
    let bytes = source.snapshot_to_bytes().expect("snapshot should render");

    let mut restored_with_installer = HostSession::builder()
        .with_builtin_installer(install_demo_family)
        .build();
    restored_with_installer
        .restore_from_bytes(&bytes)
        .expect("snapshot restore should succeed");
    assert_success_text(restored_with_installer.eval("twice[6]"), "12");

    let mut restored_without_installer = HostSession::new();
    restored_without_installer
        .restore_from_bytes(&bytes)
        .expect("snapshot restore should succeed");
    assert_success_text(restored_without_installer.eval("twice[6]"), "DemoDouble[6]");
}
