use rum_host::HostSession;
use std::sync::mpsc;

pub fn install(session: &mut HostSession) {
    let (evaluation_started_tx, _evaluation_started_rx) = mpsc::sync_channel(0);
    session.set_print_sink(move |line: &str| {
        assert_eq!(line, "evaluation-started");
        evaluation_started_tx
            .send(())
            .expect("canceller should receive evaluation-start signal");
    });
}
