//! Threads that never registered with the profiling test lease.

use crate::runtime::Session;
use crate::runtime::profiling::logical::test_support;

fn trivial_expr(session: &mut Session) -> crate::runtime::Expr {
    let syntax = rum_syntax::parse_expr("1").expect("parse 1");
    session.lower_syntax_expr(&syntax).expect("lower 1")
}

#[test]
fn unregistered_thread_span_does_not_record_while_a_profiling_guard_is_held() {
    let _enabled = test_support::enabled();
    let before = Session::logical_profile_test_rows();
    std::thread::spawn(|| {
        let mut session = Session::new();
        let expr = trivial_expr(&mut session);
        let _ = session.logical_profile_test_span("unregistered-span", expr);
    })
    .join()
    .expect("unregistered span");
    assert_eq!(
        Session::logical_profile_test_rows(),
        before,
        "unregistered thread recorded into the held epoch"
    );
}

#[test]
fn unregistered_thread_profile_reset_leaves_the_held_epoch() {
    let _enabled = test_support::enabled();
    let mut session = Session::new();
    let expr = trivial_expr(&mut session);
    session
        .logical_profile_test_span("leased-span", expr)
        .expect("leased span");
    let held = test_support::epoch();
    let before = Session::logical_profile_test_rows();
    assert!(!before.is_empty(), "the leased span recorded no row");
    std::thread::spawn(|| {
        let mut session = Session::new();
        let _ = session.eval_text("RumProfileReset[]");
    })
    .join()
    .expect("unregistered reset");
    assert!(
        held.is_valid(),
        "unregistered reset invalidated the held epoch"
    );
    assert_eq!(
        Session::logical_profile_test_rows(),
        before,
        "unregistered reset cleared the held epoch"
    );
}
