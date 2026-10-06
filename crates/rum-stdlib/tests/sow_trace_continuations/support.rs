use rum_runtime::{RuntimeMessage, Session, SessionEvalResponse, SessionRequestFailure};
pub(super) fn session() -> Session {
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s.eval_request("log={};probe::tag=\"G7 message\"").unwrap();
    s
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct View {
    typed: crate::suspension_support::observe::Observation,
    pub(super) status: String,
    pub(super) value: String,
    pub(super) termination: String,
    pub(super) messages: Vec<RuntimeMessage>,
    pub(super) prints: Vec<String>,
}
fn view(out: Result<SessionEvalResponse, SessionRequestFailure>) -> View {
    let typed = crate::suspension_support::observe::Observation::from_output(&out);
    match out {
        Ok(r) => View {
            typed,
            status: "OK".into(),
            value: r.value.to_string(),
            termination: format!("{:?}", r.termination),
            messages: r.messages,
            prints: r.prints,
        },
        Err(f) => View {
            typed,
            status: format!("{:?}", f.error),
            value: "<error>".into(),
            termination: "None".into(),
            messages: f.messages,
            prints: f.prints,
        },
    }
}
pub(super) fn record(file: &str, line: &str) {
    println!("{file}\t{line}");
}
pub(super) fn finish(
    s: &mut Session,
    out: Result<SessionEvalResponse, SessionRequestFailure>,
) -> (View, String) {
    let v = view(out);
    assert!(
        s.pending_input_request().is_none(),
        "no pending input after completion"
    );
    assert!(
        s.reject_overlapping_evaluation_request().is_ok(),
        "no suspension owner after completion"
    );
    let log = s.eval_request("Global`log").unwrap().value.to_string();
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
    (v, log)
}
fn run(_label: &str, source: &str, inputs: &[&str], queued: bool) -> (View, String, Vec<String>) {
    let mut s = session();
    let run = crate::suspension_support::drive::execute(&mut s, source, inputs, queued).unwrap();
    let out = run.output;
    let prompts = run.requests.into_iter().map(|r| r.prompt).collect();
    crate::suspension_support::session::empty_queue(&mut s).unwrap();
    let (v, log) = finish(&mut s, out);
    (v, log, prompts)
}
pub(super) fn compare(label: &str, source: &str, inputs: &[&str], expected_prompts: &[&str]) {
    let r = run(label, source, inputs, false);
    let q = run(label, source, inputs, true);
    let equal = r.0 == q.0 && r.1 == q.1 && r.2 == expected_prompts;
    let line = format!(
        "{label}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
        r.0.value,
        r.1,
        q.0.value,
        q.1,
        r.2,
        expected_prompts,
        equal,
        r.0.status,
        q.0.status,
        r.0.termination,
        q.0.termination,
        r.0.messages,
        q.0.messages,
        r.0.prints,
        q.0.prints
    );
    record("oracle.tsv", &line);
    println!("ROW\t{line}");
    if !expected_prompts.is_empty() {
        assert!(!r.2.is_empty(), "must actually suspend");
    }
    assert!(q.2.is_empty());
    assert_eq!(r.2, expected_prompts, "exact prompts and order");
    assert_eq!(
        (&r.0, &r.1),
        (&q.0, &q.1),
        "rum value/error/termination/messages/prints/log equality"
    );
}
