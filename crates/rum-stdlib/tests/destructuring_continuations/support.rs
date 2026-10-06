use crate::oracle::{View, finish, session};
use rum_core::Expr;
use rum_runtime::{RuntimeInputKind, SessionRequestError};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Observation {
    pub view: View,
    pub log: String,
    pub prompts: Vec<String>,
    pub stored: Expr,
}

fn run(source: &str, inputs: &[&str], queued: bool, inspect: &str) -> Observation {
    let mut s = session(None);
    if queued {
        for answer in inputs {
            s.queue_input_response(*answer);
        }
    }
    let mut out = s.eval_request(source);
    let mut prompts = Vec::new();
    while let Err(failure) = &out {
        let SessionRequestError::InputRequired { request } = &failure.error else {
            break;
        };
        assert!(!queued, "queued control exhausted answers");
        assert_eq!(request.kind, RuntimeInputKind::InputString);
        assert_eq!(s.pending_input_request(), Some(request));
        assert!(s.reject_overlapping_evaluation_request().is_err());
        prompts.push(request.prompt.clone());
        println!(
            "PAUSE\t{prompts:?}\t{:?}\t{:?}",
            failure.messages, failure.prints
        );
        out = s.resume_eval_request_with_input(
            *inputs
                .get(prompts.len() - 1)
                .expect("unexpected extra prompt"),
        );
    }
    let (view, log, prompts) = finish(&mut s, out, prompts);
    assert!(s.reject_overlapping_evaluation_request().is_ok());
    let stored = s.eval_request(inspect).unwrap().value;
    Observation {
        view,
        log,
        prompts,
        stored,
    }
}

pub(super) fn compare(
    label: &str,
    source: &str,
    prompts: &[&str],
    inspect: &str,
    expected_value: Option<&str>,
    expected_log: &str,
    expected_stored: &str,
) -> Observation {
    let answers = vec!["answer"; prompts.len()];
    let queued = run(source, &answers, true, inspect);
    let resumed = run(source, &answers, false, inspect);
    println!("ORACLE\t{label}\tqueued={queued:?}\tresumed={resumed:?}");
    assert!(queued.prompts.is_empty());
    assert_eq!(
        resumed.prompts, prompts,
        "actual suspension, exact prompts and order"
    );
    assert_eq!(
        resumed.view, queued.view,
        "value/error/termination/messages/prints"
    );
    assert_eq!(resumed.log, queued.log, "effect log");
    assert_eq!(resumed.stored, queued.stored, "all stored variables");
    for result in [&queued, &resumed] {
        assert_eq!(
            result
                .view
                .value
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            expected_value
        );
        assert_eq!(result.view.termination, "None");
        assert_eq!(result.log, expected_log);
        assert_eq!(result.stored.to_string(), expected_stored);
    }
    resumed
}
