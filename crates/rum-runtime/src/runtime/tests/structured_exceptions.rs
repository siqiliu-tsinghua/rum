use super::*;

fn eval_test_register_exception_type(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_register_exception_type(head, args)
}

fn eval_test_throw_exception(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    session.eval_throw_exception(head, args)
}

fn eval_test_catch_exceptions(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_catch_exceptions(args)
}

fn structured_exception_session() -> Session {
    let mut session = session_with_test_control_builtins();
    session.register_builtin(BuiltinSpec::new(
        "Exception",
        "Test-local structured exception value head.",
        MemoryGrowthContract::NonAmplifying,
    ));
    session.register_builtin(BuiltinSpec::ordinary(
        "RegisterExceptionType",
        "Test-local structured exception registration.",
        eval_test_register_exception_type,
        MemoryGrowthContract::Incremental,
    ));
    session.register_builtin(BuiltinSpec::ordinary(
        "ThrowException",
        "Test-local structured exception throw.",
        eval_test_throw_exception,
        MemoryGrowthContract::NonAmplifying,
    ));
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "CatchExceptions",
        "Test-local structured exception catch.",
        HoldAttribute::HoldAll,
        eval_test_catch_exceptions,
        MemoryGrowthContract::Delegated,
    ));
    session
}

#[test]
fn exception_type_registry_round_trips_through_loader_snapshot() {
    let mut session = Session::new();
    session.exception_types.insert("NetError".to_string(), None);
    session
        .exception_types
        .insert("Timeout".to_string(), Some("NetError".to_string()));

    let snapshot = session.capture_loader_snapshot();
    session.exception_types.clear();
    session.exception_types.insert("Other".to_string(), None);
    session.restore_loader_snapshot(snapshot);

    assert_eq!(
        session.exception_types,
        BTreeMap::from([
            ("NetError".to_string(), None),
            ("Timeout".to_string(), Some("NetError".to_string()),),
        ])
    );
}

#[test]
fn declining_exception_handler_guard_continues_ancestor_and_handler_scans() {
    let mut session = structured_exception_session();
    session
        .eval_text(
            "RegisterExceptionType[\"NetError\"]; \
             RegisterExceptionType[\"Timeout\", \"NetError\"]",
        )
        .unwrap();

    assert_eq!(
        session
            .eval_text(
                "{CatchExceptions[ThrowException[\"Timeout\"], \
                    {Exception[t_, _] :> t /; MatchQ[t, \"NetError\"], \
                     Exception[_, _] :> \"fallback\"}], \
                  CatchExceptions[ThrowException[\"Timeout\"], \
                    {Exception[\"Timeout\", _] :> \"exact\" /; False, \
                     Exception[\"NetError\", _] :> \"parent\"}]}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[\"NetError\", \"parent\"]"
    );
}

#[test]
fn suspending_exception_handler_guard_resumes_its_scan_cursor_without_replay() {
    let mut session = structured_exception_session();
    session
        .eval_text(
            "RegisterExceptionType[\"NetError\"]; \
             RegisterExceptionType[\"Timeout\", \"NetError\"]; \
             exceptionGuardResumeCount = 0",
        )
        .unwrap();

    let failure = session
        .eval_request(
            "{CatchExceptions[ThrowException[\"Timeout\"], \
                {Exception[_, _] :> \"handled\" /; \
                  (exceptionGuardResumeCount = exceptionGuardResumeCount + 1; \
                   MatchQ[InputString[\"handler guard: \"], \"yes\"])}], \
              exceptionGuardResumeCount}",
        )
        .expect_err("exception handler guard should suspend on its first candidate");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "handler guard: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let failure = session
        .resume_eval_request_with_input("no")
        .expect_err("decline should continue the same handler at its parent candidate");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "handler guard: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("accepting parent guard should finish the existing handler scan");
    assert_eq!(response.value.to_string(), "List[\"handled\", 2]");
}
