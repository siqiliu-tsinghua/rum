use rum_host::{BoundedDestruction, HostSession, PrintSink};

pub struct NamedSink;

// This fieldless unit struct has no captured or owned state to destroy.
impl BoundedDestruction for NamedSink {}

impl PrintSink for NamedSink {
    fn print_line(&mut self, _line: &str) {}
}

pub fn install(session: &mut HostSession) {
    session.set_print_sink(NamedSink);
}
