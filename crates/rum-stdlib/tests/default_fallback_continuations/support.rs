use crate::oracle::{View, finish, session};
use rum_runtime::{RuntimeInputKind, SessionRequestError};

pub(super) struct Case {
    pub label: &'static str,
    pub source: &'static str,
    pub prompts: &'static [&'static str],
    pub value: Option<&'static str>,
    pub log: &'static str,
}

fn run(case: &Case, queued: bool) -> (View, String, Vec<String>) {
    let mut s = session(None);
    let answers: Vec<_> = (0..case.prompts.len())
        .map(|i| format!("answer-{i}"))
        .collect();
    if queued {
        for answer in &answers {
            s.queue_input_response(answer.clone());
        }
    }
    let mut out = s.eval_request(case.source);
    let mut prompts = Vec::new();
    while let Err(failure) = &out {
        let SessionRequestError::InputRequired { request } = &failure.error else {
            break;
        };
        assert!(!queued, "queued control exhausted answers: {}", case.label);
        assert_eq!(request.kind, RuntimeInputKind::InputString);
        assert_eq!(s.pending_input_request(), Some(request));
        assert!(s.reject_overlapping_evaluation_request().is_err());
        prompts.push(request.prompt.clone());
        println!(
            "PAUSE\t{}\t{prompts:?}\t{:?}\t{:?}",
            case.label, failure.messages, failure.prints
        );
        out = s.resume_eval_request_with_input(
            answers
                .get(prompts.len() - 1)
                .expect("unexpected extra prompt")
                .clone(),
        );
    }
    let result = finish(&mut s, out, prompts);
    assert!(s.reject_overlapping_evaluation_request().is_ok());
    result
}

pub(super) fn compare(case: Case) -> View {
    let queued = run(&case, true);
    let resumed = run(&case, false);
    println!(
        "ORACLE\t{}\tqueued={queued:?}\tresumed={resumed:?}",
        case.label
    );
    assert!(queued.2.is_empty());
    assert_eq!(
        resumed.2, case.prompts,
        "actual suspension, exact prompts and order"
    );
    assert_eq!(
        resumed.0, queued.0,
        "value/error/termination/ordered messages/prints"
    );
    assert_eq!(resumed.1, queued.1, "effect log");
    for result in [&queued, &resumed] {
        assert_eq!(
            result.0.value.as_ref().map(ToString::to_string).as_deref(),
            case.value
        );
        assert_eq!(result.0.termination, "None");
        assert_eq!(result.1, case.log);
    }
    resumed.0
}
