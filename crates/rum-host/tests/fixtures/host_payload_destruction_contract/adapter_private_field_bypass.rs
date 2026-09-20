use rum_host::{ClosurePrintSink, HostSession};

pub fn install(session: &mut HostSession) {
    session.set_print_sink(ClosurePrintSink {
        closure: |_: &str| {},
    });
}
