use super::*;

const PROBE_PACKAGE: &str =
    "DefinePackage[\"Pkg`\", Public[{f}], f[x_] := helper[x] + 1; helper[x_] := x * 2]";

fn test_symbol(session: &mut Session, head: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [Expr::Atom(Atom::String(name))] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(session
        .resolve_symbol_name(name, true)
        .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
        .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
}

fn test_to_expression(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [Expr::Atom(Atom::String(source))] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let syntax = parse_expr(source)?;
    let lowered = session.lower_syntax_expr(&syntax)?;
    session.evaluate(lowered)
}

fn test_string_join(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let Some(parts) = args.iter().map(expr_as_string).collect::<Option<Vec<_>>>() else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(parts.concat()))
}

fn sealed_package_session() -> Session {
    let mut session = session_with_test_control_builtins();
    session.register_builtin(
        BuiltinSpec::ordinary(
            "Symbol",
            "Test-only resolver adapter.",
            test_symbol,
            MemoryGrowthContract::NonAmplifying,
        )
        .protected(),
    );
    session.register_builtin(
        BuiltinSpec::ordinary(
            "ToExpression",
            "Test-only lowering adapter.",
            test_to_expression,
            MemoryGrowthContract::Incremental,
        )
        .protected(),
    );
    session.register_builtin(
        BuiltinSpec::ordinary(
            "StringJoin",
            "Test-only string adapter.",
            test_string_join,
            MemoryGrowthContract::Preflight,
        )
        .protected(),
    );
    session.register_builtin(
        BuiltinSpec::new(
            "Map",
            "Test-only known System symbol.",
            MemoryGrowthContract::Incremental,
        )
        .protected(),
    );
    session
}

fn register_probe_package(session: &mut Session) -> Symbol {
    let outcome = session.eval_text(PROBE_PACKAGE).unwrap();
    assert_eq!(outcome.value.to_string(), "\"Pkg`\"");
    assert!(outcome.messages.is_empty());
    let unit = session
        .sealed_units
        .iter()
        .find(|unit| unit.package_context == "Pkg`")
        .expect("probe package should be registered");
    unit.private_index["helper"]
}

fn evaluated_symbol(session: &mut Session, source: &str) -> Symbol {
    let outcome = session.eval_text(source).unwrap();
    expr_as_symbol(&outcome.value).copied().unwrap_or_else(|| {
        panic!(
            "{source} should evaluate to a symbol, got {}",
            outcome.value
        )
    })
}

fn assert_failed_message(outcome: &EvalOutcome, tag: &str) {
    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "DefinePackage" && message.tag == tag),
        "expected DefinePackage::{tag}, got {:?}",
        outcome.messages
    );
}

#[test]
fn public_reaches_sealed_private() {
    let mut session = sealed_package_session();
    let sealed = register_probe_package(&mut session);

    assert_eq!(
        session.eval_text("Pkg`f[3]").unwrap().value.to_string(),
        "7"
    );
    assert_ne!(sealed.identity(), None);
    assert_ne!(
        sealed.identity().unwrap() & crate::runtime::scoping::packages::SEALED_IDENTITY_MARKER,
        0
    );
    assert!(
        !session
            .symbol_generations
            .contains_key(&("Pkg`Private`".to_string(), "helper".to_string()))
    );
}

#[test]
fn spelled_private_name_is_ordinary() {
    let mut session = sealed_package_session();
    let sealed = register_probe_package(&mut session);
    let spellings = [
        "Pkg`Private`helper",
        "Symbol[\"Pkg`Private`helper\"]",
        "ToExpression[\"Pkg`Private`helper\"]",
        "ToExpression[\"Pkg`Priv\" <> \"ate`helper\"]",
    ];

    for spelling in spellings {
        let ordinary = evaluated_symbol(&mut session, spelling);
        assert_eq!(ordinary.identity(), None, "{spelling}");
        assert_ne!(ordinary, sealed, "{spelling}");

        let applied = Expr::call(
            Expr::Atom(Atom::Symbol(ordinary)),
            vec![Expr::small_integer(3)],
        );
        assert_eq!(
            session.evaluate(applied).unwrap().to_string(),
            "Pkg`Private`helper[3]",
            "{spelling}"
        );
    }
}

#[test]
fn decoy_assignment_is_distinct() {
    let mut session = sealed_package_session();
    let sealed = register_probe_package(&mut session);

    session.eval_text("Pkg`Private`helper = 99").unwrap();

    assert_eq!(
        session.eval_text("Pkg`f[3]").unwrap().value.to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("Pkg`Private`helper")
            .unwrap()
            .value
            .to_string(),
        "99"
    );
    let ordinary = session
        .resolve_symbol_name("Pkg`Private`helper", true)
        .unwrap();
    assert_eq!(ordinary.identity(), None);
    assert_ne!(ordinary, sealed);
    assert!(session.definitions.contains_key(&ordinary));
    assert!(session.definitions.contains_key(&sealed));
}

#[test]
fn private_state_inside_package() {
    let mut session = sealed_package_session();
    let outcome = session
        .eval_text(
            "DefinePackage[\"State`\", Public[{setValue, getValue}], state = 0; setValue[x_] := state = x; getValue[] := state]",
        )
        .unwrap();
    assert_eq!(outcome.value.to_string(), "\"State`\"");

    assert_eq!(
        session
            .eval_text("State`setValue[42]")
            .unwrap()
            .value
            .to_string(),
        "42"
    );
    assert_eq!(
        session
            .eval_text("State`getValue[]")
            .unwrap()
            .value
            .to_string(),
        "42"
    );
    let spelled = evaluated_symbol(&mut session, "State`Private`state");
    let sealed = session.sealed_units[0].private_index["state"];
    assert_eq!(spelled.identity(), None);
    assert_ne!(spelled, sealed);
}

#[test]
fn package_scope_is_fqn_reachable() {
    let mut session = sealed_package_session();
    let outcome = session
        .eval_text("DefinePackage[\"Tier`\", PackageScope[{h}], h[x_] := x + 2]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "\"Tier`\"");
    assert_eq!(
        session
            .eval_text("Tier`PackageScope`h[3]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(session.sealed_units[0].package_scope[0].identity(), None);
}

#[test]
fn remove_sealed_writes_no_generation() {
    let mut session = sealed_package_session();
    let sealed = register_probe_package(&mut session);
    let key = ("Pkg`Private`".to_string(), "helper".to_string());

    session.remove_symbol_state(&sealed);

    assert!(!session.symbol_generations.contains_key(&key));
    let spelled = session
        .resolve_symbol_name("Pkg`Private`helper", true)
        .unwrap();
    assert_eq!(spelled.identity(), None);
    assert_ne!(spelled, sealed);
}

#[test]
fn registration_failure_cleans_sealed_state() {
    let mut session = sealed_package_session();
    let outcome = session
        .eval_text(
            "DefinePackage[\"Broken`\", Public[{f}], f[] := helper; helper = 41; Throw[\"boom\"]]",
        )
        .unwrap();

    assert_failed_message(&outcome, "failed");
    assert!(session.sealed_units.is_empty());
    let minted = Symbol::with_identity(
        "Broken`Private`",
        "helper",
        crate::runtime::scoping::packages::SEALED_IDENTITY_MARKER | session.next_sealed_symbol_id,
    );
    assert!(!session.definitions.contains_key(&minted));
    assert!(!session.attributes.contains_key(&minted));
    assert!(!session.options.contains_key(&minted));
    assert!(!session.syntax_information.contains_key(&minted));
    assert!(
        !session
            .message_definitions
            .keys()
            .any(|key| key.symbol == minted)
    );
    let spelled = session
        .resolve_symbol_name("Broken`Private`helper", true)
        .unwrap();
    assert_eq!(spelled.identity(), None);
    assert_ne!(spelled, minted);
}

#[test]
fn re_registration_replaces_in_place() {
    let mut session = sealed_package_session();
    session
        .eval_text("DefinePackage[\"Reload`\", Public[{f}], f[] := helper[]; helper[] := 1]")
        .unwrap();
    let old_symbol = session.sealed_units[0].private_index["helper"];
    let old_epoch = session.sealed_units[0].epoch;

    session
        .eval_text("DefinePackage[\"Other`\", Public[{g}], g = 10]")
        .unwrap();
    session
        .eval_text("DefinePackage[\"Reload`\", Public[{f}], f[] := helper[]; helper[] := 2]")
        .unwrap();

    assert_eq!(
        session.eval_text("Reload`f[]").unwrap().value.to_string(),
        "2"
    );
    assert!(!session.definitions.contains_key(&old_symbol));
    assert_eq!(session.sealed_units.len(), 2);
    assert_eq!(session.sealed_units[0].package_context, "Reload`");
    assert_eq!(session.sealed_units[1].package_context, "Other`");
    assert!(session.sealed_units[0].epoch > old_epoch);
    assert_ne!(session.sealed_units[0].private_index["helper"], old_symbol);
    assert_eq!(
        session
            .sealed_units
            .iter()
            .filter(|unit| unit.package_context == "Reload`")
            .count(),
        1
    );
}

#[test]
fn declaration_validation() {
    let mut session = sealed_package_session();
    let duplicate = session
        .eval_text("DefinePackage[\"Dup`\", Public[{f}], Public[{g}], f = 1]")
        .unwrap();
    assert_failed_message(&duplicate, "decl");

    let foreign = session
        .eval_text("DefinePackage[\"Foreign`\", Public[{Map}], x = 1]")
        .unwrap();
    assert_failed_message(&foreign, "decl");

    assert!(session.sealed_units.is_empty());
    assert_eq!(session.next_sealed_symbol_id, 0);
    assert_eq!(session.next_source_unit_epoch, 0);
}

#[test]
fn context_validation_returns_the_call_inert() {
    let mut session = sealed_package_session();
    let outcome = session
        .eval_text("DefinePackage[\"MissingTerminator\", Public[{f}], f = 1]")
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "DefinePackage[\"MissingTerminator\", Public[List[f]], Set[f, 1]]"
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "DefinePackage" && message.tag == "argform")
    );
    assert!(session.sealed_units.is_empty());
}

#[test]
fn internal_snapshot_round_trips_registry() {
    let mut session = sealed_package_session();
    let before_units = session.sealed_units.clone();
    let before_sealed_id = session.next_sealed_symbol_id;
    let before_epoch = session.next_source_unit_epoch;
    let snapshot = session.capture_loader_snapshot();

    register_probe_package(&mut session);
    assert_ne!(session.sealed_units, before_units);
    assert!(session.next_sealed_symbol_id > before_sealed_id);
    assert!(session.next_source_unit_epoch > before_epoch);

    session.restore_loader_snapshot(snapshot);
    assert_eq!(session.sealed_units, before_units);
    assert_eq!(session.next_sealed_symbol_id, before_sealed_id);
    assert_eq!(session.next_source_unit_epoch, before_epoch);
}

#[test]
fn package_install_resumes_at_the_next_form() {
    let mut session = sealed_package_session();
    session.eval_text("External`installCount = 0").unwrap();

    let failure = session
        .eval_request(
            "DefinePackage[\"Resume`\", Public[{f}], External`installCount = External`installCount + 1; helper = InputString[\"value: \"]; f[] := helper]",
        )
        .expect_err("package body should suspend for host input");
    assert!(matches!(
        failure.error,
        SessionRequestError::InputRequired { .. }
    ));

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("package registration should resume and finish");
    assert_eq!(response.value.to_string(), "\"Resume`\"");
    assert_eq!(
        session
            .eval_text("External`installCount")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Resume`f[]").unwrap().value.to_string(),
        "\"ok\""
    );
    assert_eq!(session.sealed_units.len(), 1);
}
