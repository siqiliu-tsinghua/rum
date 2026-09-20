use std::sync::{Arc, Mutex};

use super::*;
use rum_runtime::ClosurePrintSink;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn print_sink_streams_prints_and_leaves_outcome_prints_empty() {
    let mut session = core_session();
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink_received = Arc::clone(&received);

    session.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |text| {
            sink_received.lock().unwrap().push(text.to_string());
        },
    )));

    let outcome = session.eval_request("Do[Print[i], {i,1,3}]").unwrap();
    assert_eq!(
        *received.lock().unwrap(),
        vec!["1".to_string(), "2".to_string(), "3".to_string()]
    );
    assert!(outcome.prints.is_empty());
}

#[test]
fn prints_accumulate_without_sink() {
    let mut session = core_session();

    let outcome = session.eval_request("Do[Print[i], {i,1,3}]").unwrap();
    assert_eq!(outcome.prints, vec!["1", "2", "3"]);
}
