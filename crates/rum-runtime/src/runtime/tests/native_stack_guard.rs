use super::*;
use crate::runtime::native_stack_guard::{
    run_with_native_stack_guard, with_thread_native_stack_base,
};

#[test]
fn native_stack_guard_polls_at_the_stride_and_fails_sticky() {
    let mut session = Session::new();
    session.resource_limits.native_stack_limit = 0;

    let result = with_thread_native_stack_base(usize::MAX, || {
        run_with_native_stack_guard(&session, |depth| {
            let depth = depth.child().expect("edge one is below the poll stride");
            let depth = depth.child().expect("edge two is below the poll stride");
            let depth = depth.child().expect("edge three is below the poll stride");
            assert!(depth.child().is_none());
            17
        })
    });

    assert!(matches!(
        result,
        Err(EvalError::StackLimitExceeded { limit: 0 })
    ));
}

#[test]
fn logical_depth_never_refuses_when_the_byte_budget_is_available() {
    let mut session = Session::new();
    session.resource_limits.native_stack_limit = 0;

    let result = with_thread_native_stack_base(1, || {
        run_with_native_stack_guard(&session, |mut depth| {
            for _ in 0..10_000 {
                depth = depth
                    .child()
                    .expect("logical depth alone must not exhaust the guard");
            }
            23
        })
    });

    assert_eq!(result.expect("the byte budget remains available"), 23);
}
