use super::*;

#[test]
fn witnessed_chain_and_actions_remain_dormant() {
    let result: Result<Expr, EvalError> = Ok(Expr::string("input"));
    let (first, calls) = echo();
    let chain: FrameChain<Expr, Expr> = slot_witness(&result).chain(first).then(echo().0);
    assert_eq!(std::mem::size_of_val(&slot_witness(&result)), 0);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    drop(chain);

    let action = TypedAction::<Expr>::evaluate(Expr::string("evaluate"), echo().0);
    let Action::Evaluate { expr, next } = action.inner else {
        panic!("evaluate action")
    };
    assert_eq!(expr, Expr::string("evaluate"));
    let mut session = Session::new();
    let mut input = Ok(Slot::Value(Expr::string("child")));
    let resumed = next.resume(&mut session, &mut input);
    assert!(matches!(input, Err(EvalError::Abort)));
    assert!(
        matches!(resumed, Action::Complete(Ok(Slot::Value(expr))) if expr == Expr::string("child"))
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn deferred_completion_preserves_output_and_stops_without_finalizing() {
    let action: TypedAction<Option<Expr>> = TypedAction::finish_work(
        Ok::<_, ResumeStop<Option<Expr>>>(Expr::string("done")),
        |value| Finish(Some(value)),
    );
    let Action::Finalize(finalizer) = action.inner else {
        panic!("finalize action")
    };
    let slot = finalizer.finish().unwrap();
    assert_eq!(
        decode::<Option<Expr>>(slot, CodecKey::new()).ok().unwrap(),
        Some(Expr::string("done"))
    );
    let action = TypedAction::<Option<Expr>>::finish_work(
        Err::<Expr, _>(ResumeStop::failure(EvalError::Abort)),
        |_| -> Finish { panic!("stopped helper must not finalize") },
    );
    assert!(matches!(
        action.inner,
        Action::Complete(Err(EvalError::Abort))
    ));
}

#[test]
fn immediate_work_preserves_completion_and_token_stop() {
    let action = TypedAction::<Option<Expr>>::complete_work(Ok(Some(Expr::string("done"))));
    let Action::Complete(Ok(slot)) = action.inner else {
        panic!("complete")
    };
    assert_eq!(
        decode::<Option<Expr>>(slot, CodecKey::new()).ok().unwrap(),
        Some(Expr::string("done"))
    );
    let action =
        TypedAction::<Option<Expr>>::complete_work(Err(ResumeStop::failure(EvalError::Abort)));
    assert!(matches!(
        action.inner,
        Action::Complete(Err(EvalError::Abort))
    ));
}
