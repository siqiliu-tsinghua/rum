use super::*;
use crate::runtime::suspension::AbandonReason;

#[test]
fn maintained_requests_queues_nested_resume_and_cleanup() {
    let mut session = session_with_test_control_builtins();
    session.resource_limits.eval_memory_limit = Some(1_000_000_000);
    session.eval_text("Null").unwrap();
    session.queue_input_response("first queued input");
    session.queue_input_response("second queued input");
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.eval_text("{InputString[], InputString[]}").unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    assert!(
        session
            .eval_request("{InputString[\"first\"], InputString[\"second\"]}")
            .is_err()
    );
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.take_pending_input_request().unwrap();
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    assert!(session.resume_eval_request_with_input("one").is_err());
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    let response = session.resume_eval_request_with_input("two").unwrap();
    assert_eq!(response.value.to_string(), "List[\"one\", \"two\"]");
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    assert!(session.eval_request(
        "WithCleanup[InputString[\"body\"], WithCleanup[InputString[\"cleanup\"], cleanup523 = 7]]"
    ).is_err());
    session.abandon_pending_suspension(AbandonReason::CancelledRequest);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    assert_eq!(
        session.eval_text("cleanup523").unwrap().value,
        Expr::small_integer(7)
    );
    assert!(session.pending_suspension.is_none());
    assert!(session.abandonment_worklist.is_empty());
}

#[test]
fn maintained_accounting_route_adapter_replacement() {
    fn ordinary(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
        Ok(system_expr("Null"))
    }
    fn accounted(
        context: &mut crate::runtime::accounting_mode::AccountingContext<'_>,
        _: &Expr,
        _: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        context.evaluate(system_expr("Null"))
    }
    let mut session = Session::empty();
    session.current_memory_limit = Some(1_000_000_000);
    session.note_accounting_arm();
    let spec = BuiltinSpec::ordinary(
        "RouteOwner523",
        "Accounting adapter.",
        ordinary,
        MemoryGrowthContract::Delegated,
    );
    session.register_accounting_builtin(spec, accounted);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
    session.register_builtin(spec);
    session
        .enforce_active_memory_limit_for_projected_bytes(0)
        .unwrap();
}
