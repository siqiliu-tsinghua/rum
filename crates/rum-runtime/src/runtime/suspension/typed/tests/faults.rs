use super::super::channel::{LegacyMode, StateMode, Transport, ValueMode};
use super::*;

#[test]
fn channel_modes_reject_the_other_arm_and_wrong_state_variant() {
    assert!(matches!(
        <ValueMode as Transport<Expr>>::decode(
            Slot::State(ContinuationState::Bool(true)),
            CodecKey::new()
        ),
        Err(Slot::State(ContinuationState::Bool(true)))
    ));
    assert!(matches!(
        <LegacyMode as Transport<bool>>::decode(
            Slot::State(ContinuationState::Bool(true)),
            CodecKey::new()
        ),
        Err(Slot::State(ContinuationState::Bool(true)))
    ));
    assert!(
        matches!(<StateMode as Transport<bool>>::decode(Slot::Value(Expr::string("guest")), CodecKey::new()), Err(Slot::Value(expr)) if expr == Expr::string("guest"))
    );
    assert!(
        matches!(<StateMode as Transport<bool>>::decode(Slot::State(ContinuationState::String("wrong".into())), CodecKey::new()), Err(Slot::State(ContinuationState::String(s))) if s == "wrong")
    );
    assert!(matches!(
        <StateMode as Transport<bool>>::decode(
            <StateMode as Transport<bool>>::encode(true, CodecKey::new()),
            CodecKey::new()
        ),
        Ok(true)
    ));
}

#[test]
fn adapter_rejects_before_user_resume_and_forwards_errors() {
    let (frame, calls) = echo();
    let erased = adapter::erase(frame);
    let mut session = Session::new();
    let mut input = Ok(Slot::State(ContinuationState::Bool(true)));
    let Action::DecodeFailure(error) = erased.clone().resume(&mut session, &mut input) else {
        panic!("adapter must reject the mismatched slot before resuming");
    };
    assert!(matches!(input, Err(EvalError::Abort)));
    assert_eq!(error.expected, std::any::type_name::<Expr>());
    assert!(matches!(
        error.actual,
        Slot::State(ContinuationState::Bool(true))
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let mut input = Err(EvalError::Abort);
    let action = erased.resume(&mut session, &mut input);
    assert!(matches!(input, Err(EvalError::Abort)));
    assert!(matches!(action, Action::Complete(Err(EvalError::Abort))));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let mut input = Err(EvalError::InvalidInteger {
        literal: "owned error payload".into(),
    });
    let action = erased.resume(&mut session, &mut input);
    assert!(matches!(input, Err(EvalError::Abort)));
    assert!(matches!(
        action,
        Action::Complete(Err(EvalError::InvalidInteger { literal }))
            if literal == "owned error payload"
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
