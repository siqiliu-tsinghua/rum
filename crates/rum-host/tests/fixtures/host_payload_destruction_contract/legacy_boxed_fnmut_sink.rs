use rum_runtime::Session;
use std::sync::{Arc, Mutex};

pub fn install(session: &mut Session) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink_received = Arc::clone(&received);
    let sink: Box<dyn FnMut(&str) + Send> = Box::new(move |line| {
        sink_received.lock().unwrap().push(line.to_string());
    });
    session.set_print_sink(sink);
}
