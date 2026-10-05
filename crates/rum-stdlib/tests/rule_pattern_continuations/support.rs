use rum_core::Expr;
use rum_runtime::{
    RuntimeMessage, Session, SessionEvalResponse, SessionRequestError, SessionRequestFailure,
};

pub(super) fn session(limit: Option<usize>) -> Session {
    let mut s = Session::new();
    rum_stdlib::install_core(&mut s);
    s.eval_request("log={};probe::tag=\"G4 message\"").unwrap();
    if let Some(limit) = limit {
        s.set_iteration_work_limit(limit);
    }
    s
}
#[derive(PartialEq, Eq)]
pub(super) struct View {
    pub(super) status: String,
    pub(super) value: Option<Expr>,
    pub(super) termination: String,
    pub(super) messages: Vec<RuntimeMessage>,
    pub(super) prints: Vec<String>,
}
impl std::fmt::Debug for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("status", &self.status)
            .field("value", &self.value.as_ref().map(ToString::to_string))
            .field("termination", &self.termination)
            .field("messages", &self.messages)
            .field("prints", &self.prints)
            .finish()
    }
}
fn view(out: Result<SessionEvalResponse, SessionRequestFailure>) -> View {
    match out {
        Ok(r) => View {
            status: "OK".into(),
            value: Some(r.value),
            termination: format!("{:?}", r.termination),
            messages: r.messages,
            prints: r.prints,
        },
        Err(f) => View {
            status: format!("{:?}", f.error),
            value: None,
            termination: "None".into(),
            messages: f.messages,
            prints: f.prints,
        },
    }
}
pub(super) fn finish(
    s: &mut Session,
    out: Result<SessionEvalResponse, SessionRequestFailure>,
    prompts: Vec<String>,
) -> (View, String, Vec<String>) {
    let v = view(out);
    assert!(
        s.pending_input_request().is_none(),
        "pending owner leaked: {v:?}"
    );
    let log = s.eval_request("log").unwrap().value.to_string();
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
    (v, log, prompts)
}
fn run(
    source: &str,
    inputs: &[&str],
    queued: bool,
    setup: &str,
    limit: Option<usize>,
) -> (View, String, Vec<String>) {
    let mut s = session(limit);
    if !setup.is_empty() {
        s.eval_request(setup).unwrap();
    }
    if queued {
        for input in inputs {
            s.queue_input_response(*input);
        }
    }
    let mut out = s.eval_request(source);
    let mut prompts = Vec::new();
    while let Err(f) = &out {
        let SessionRequestError::InputRequired { request } = &f.error else {
            break;
        };
        assert!(!queued, "queued control exhausted answers");
        assert_eq!(
            s.pending_input_request(),
            Some(request),
            "exact pending owner"
        );
        prompts.push(request.prompt.clone());
        let input = inputs
            .get(prompts.len() - 1)
            .expect("unexpected extra suspension");
        out = s.resume_eval_request_with_input(*input);
    }
    finish(&mut s, out, prompts)
}
fn record(
    label: &str,
    resumed: &(View, String, Vec<String>),
    queued: &(View, String, Vec<String>),
    expected_prompts: usize,
) {
    println!("CASE\t{label}\tresumed={resumed:?}\tqueued={queued:?}");
    println!(
        "ROW\t{label}\t{:?}\t{:?}\t{:?}\t{:?}\t{}\t{expected_prompts}\t{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
        resumed
            .0
            .value
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "<error>".into()),
        resumed.1,
        queued
            .0
            .value
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "<error>".into()),
        queued.1,
        resumed.2.len(),
        resumed.0 == queued.0 && resumed.1 == queued.1,
        resumed.0.status,
        queued.0.status,
        resumed.0.termination,
        queued.0.termination,
        resumed.0.messages,
        queued.0.messages,
        resumed.0.prints,
        queued.0.prints
    );
}
pub(super) fn compare(
    label: &str,
    source: &str,
    inputs: &[&str],
    expected_prompts: &[&str],
    setup: &str,
) {
    let resumed = run(source, inputs, false, setup, None);
    let queued = run(source, inputs, true, setup, None);
    record(label, &resumed, &queued, expected_prompts.len());
    println!("PROMPTS\t{label}\t{:?}\t{:?}", resumed.2, expected_prompts);
    assert!(!resumed.2.is_empty(), "must actually suspend");
    assert!(queued.2.is_empty(), "queued control must complete");
    assert_eq!(resumed.2, expected_prompts, "exact prompt count and order");
    assert_eq!(
        resumed.0, queued.0,
        "value, error, termination, ordered messages and prints"
    );
    assert_eq!(
        resumed.1, queued.1,
        "each completed effect runs exactly once"
    );
}
