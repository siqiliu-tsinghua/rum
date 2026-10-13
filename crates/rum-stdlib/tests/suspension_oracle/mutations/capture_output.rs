use super::super::descriptors::Fixture;
use super::super::support::{
    observe::{self, Observation, Rejection},
    session,
};
#[test]
fn lost_message_prefix_is_rejected_when_old_print_only_check_passes() {
    let mut s = session::new();
    let pause = Observation::from_output(
        &s.eval_request(r#"Message[Global`probe::tag];InputString["prefix"];7"#),
    );
    let done = Observation::from_output(&s.resume_eval_request_with_input("answer"));
    observe::prefix(&pause, &done).unwrap();
    assert!(!pause.messages.is_empty());
    let mut corrupt = done.clone();
    corrupt.messages.clear();
    assert!(corrupt.prints.starts_with(&pause.prints));
    println!("WEAK message_prefix: PASS");
    let rejected = observe::prefix(&pause, &corrupt);
    assert_eq!(rejected, Err(Rejection::MessagePrefix));
    println!("STRICT message_prefix: {rejected:?}");
    assert_eq!(observe::prefix(&pause, &done), Ok(()));
    println!("CONTROL message_prefix: PASS");
    let mut f = Fixture::new("message pause", "");
    f.pause_messages = Some(vec![vec![]]);
    assert_eq!(f.check_pause(0, &pause), Err(Rejection::PauseMessages(0)));
    println!("STRICT message_pause: PauseMessages(0)");
}
#[test]
fn lost_trace_prefix_is_rejected_when_old_print_only_check_passes() {
    let mut s = session::new();
    s.enable_tracing(true);
    let pause = Observation::from_output(&s.eval_request(r#"1+2;InputString["prefix"];4+5"#));
    let done = Observation::from_output(&s.resume_eval_request_with_input("answer"));
    observe::prefix(&pause, &done).unwrap();
    assert!(!pause.trace.is_empty());
    let mut corrupt = done.clone();
    corrupt.trace.clear();
    assert!(corrupt.prints.starts_with(&pause.prints));
    println!("WEAK trace_prefix: PASS");
    let rejected = observe::prefix(&pause, &corrupt);
    assert_eq!(rejected, Err(Rejection::TracePrefix));
    println!("STRICT trace_prefix: {rejected:?}");
    assert_eq!(observe::prefix(&pause, &done), Ok(()));
    println!("CONTROL trace_prefix: PASS");
    let mut f = Fixture::new("trace pause", "");
    f.pause_trace = Some(vec![vec![]]);
    assert_eq!(f.check_pause(0, &pause), Err(Rejection::PauseTrace(0)));
    println!("STRICT trace_pause: PauseTrace(0)");
}
