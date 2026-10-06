use crate::oracle::{View, finish, session};
use rum_core::Expr;
use rum_runtime::RuntimeInputKind;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Observation {
    pub view: View,
    pub log: String,
    pub prompts: Vec<String>,
    pub stored: Expr,
}

fn run(source: &str, inputs: &[&str], queued: bool, inspect: &str) -> Observation {
    let mut s = session(None);
    let run = crate::suspension_support::drive::execute(&mut s, source, inputs, queued).unwrap();
    assert!(
        run.requests
            .iter()
            .all(|r| r.kind == RuntimeInputKind::InputString)
    );
    let out = run.output;
    let prompts = run.requests.into_iter().map(|r| r.prompt).collect();
    crate::suspension_support::session::empty_queue(&mut s).unwrap();
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
