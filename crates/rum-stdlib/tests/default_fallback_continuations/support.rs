use crate::oracle::{View, finish, session};
use rum_runtime::RuntimeInputKind;

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
    let run = crate::suspension_support::drive::execute(
        &mut s,
        case.source,
        &answers.iter().map(String::as_str).collect::<Vec<_>>(),
        queued,
    )
    .unwrap();
    assert!(
        run.requests
            .iter()
            .all(|r| r.kind == RuntimeInputKind::InputString)
    );
    let out = run.output;
    let prompts = run.requests.into_iter().map(|r| r.prompt).collect();
    crate::suspension_support::session::empty_queue(&mut s).unwrap();
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
