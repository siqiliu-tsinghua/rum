use rum_host::PrintSink;

pub struct NamedSink;

impl PrintSink for NamedSink {
    fn print_line(&mut self, _line: &str) {}
}
