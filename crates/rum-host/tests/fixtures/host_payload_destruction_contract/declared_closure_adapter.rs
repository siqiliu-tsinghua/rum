use rum_host::{ClosurePrintSink, HostSession};
use std::sync::{Arc, Mutex};

pub fn install(session: &mut HostSession) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink_received = Arc::clone(&received);
    // The last Arc holder drops a flat Vec of Strings, with no recursive chain.
    session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        sink_received.lock().unwrap().push(line.to_string());
    }));
}
