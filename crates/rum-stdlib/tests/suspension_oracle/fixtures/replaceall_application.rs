//! G15-C1: ReplaceAll finishes its application before reporting its trace pass.
//! Queued and resumed execution share the documented rewrite/evaluation order
//! (guide: rules, evaluation, and control flow); final values alone are too weak.
use super::super::descriptors::{Checks, Fixture, accept, run};
use super::super::support::{
    faults, lifecycle,
    observe::{self, Observation},
    session,
};
use rum_runtime::{EvalTerminationReason, RuntimeInputRequest, SessionRequestError};

fn equality(
    name: &str,
    setup: &str,
    source: &str,
    prompts: &[&str],
    value: &str,
    log: &str,
    traced: bool,
) {
    let pauses: Vec<_> = prompts.iter().map(|p| ("InputString", *p, "1")).collect();
    let mut fixture = super::make(name, source, &pauses, Some(value), log);
    fixture.setup = format!("log = {{}};{setup}");
    if traced {
        fixture.configure = |session| session.enable_tracing(true);
    }
    fixture.expected_prints = Some(vec![]);
    fixture.expected_messages = Some(vec![]);
    fixture.pause_prints = Some(vec![vec![]; prompts.len()]);
    fixture.pause_messages = Some(vec![vec![]; prompts.len()]);
    if !traced {
        fixture.pause_trace = Some(vec![vec![]; prompts.len()]);
    }
    fixture.lifecycle_pauses = [0, usize::from(prompts.len() > 1)];
    // Every phase has its own prompt in the source. ALL checks each prompt,
    // its kind, pending ownership, overlap rejection, and the complete ordered
    // trace. The independent FaultProbe check cannot be disabled by Checks.
    let result = accept(&fixture, Checks::ALL);
    let lifecycle = exercise_lifecycle(&fixture);
    if let Err(error) = &result {
        let resumed = run(&fixture, false, Checks::ALL).unwrap();
        let queued = run(&fixture, true, Checks::ALL).unwrap();
        assert_eq!(resumed.observation.value, queued.observation.value);
        assert_eq!(resumed.observation.messages, queued.observation.messages);
        assert_eq!(resumed.observation.prints, queued.observation.prints);
        assert_eq!(resumed.log, queued.log);
        println!(
            "C1_FINAL_VALUE_EQUAL {name} value={:?} rejection={error:?}",
            resumed.observation.value
        );
        println!("{name} queued trace: {:?}", queued.observation.trace);
        println!("{name} resumed trace: {:?}", resumed.observation.trace);
    }
    super::super::support::faults::assert_all();
    let accepted = result.unwrap_or_else(|error| panic!("{name}: {error:?}"));
    println!(
        "G15-C1 {name} pauses={} trace={traced} lifecycle={lifecycle} PASS value={:?} log={}",
        prompts.len(),
        accepted.observation.value,
        accepted.log
    );
}

// The common buffered helper assumes an untraced cleanup. These fixtures keep
// tracing enabled and check the exact cleanup suffix as well as its prefix.
// A one-pause expression runs twice to exercise two distinct lifecycle stops.
fn exercise_lifecycle(fixture: &Fixture) -> usize {
    let mut f = fixture.clone();
    if f.pauses.len() == 1 {
        f.source = format!("({});({})", f.source, f.source);
        f.pauses.extend(f.pauses.clone());
        if let Some(values) = &mut f.pause_prints {
            values.extend(values.clone());
        }
        if let Some(values) = &mut f.pause_messages {
            values.extend(values.clone());
        }
        if let Some(values) = &mut f.pause_trace {
            values.extend(values.clone());
        }
    }
    let plans: Vec<_> = f
        .pauses
        .iter()
        .map(|p| {
            (
                RuntimeInputRequest {
                    kind: p.kind,
                    prompt: p.prompt.clone(),
                    password: false,
                },
                p.answer.clone(),
            )
        })
        .collect();
    let source = format!(
        "WithCleanup[({});Print[\"oracle suffix\"],Print[\"oracle cleanup\"]]",
        f.source
    );
    let mut cleanup_session = f.session();
    let cleanup_trace = cleanup_session
        .eval_request("Print[\"oracle cleanup\"]")
        .unwrap()
        .trace;
    for stop in [0, 1] {
        for reset in [false, true] {
            let mut s = f.session();
            let check = faults::Check::new(&s);
            let cancel = s.request_cancel_handle();
            let mut out = s.eval_request(&source);
            let mut previous = None;
            for (i, (planned, answer)) in plans.iter().enumerate().take(stop + 1) {
                let current = Observation::from_output(&out);
                if let Some(previous) = &previous {
                    observe::prefix(previous, &current).unwrap();
                }
                f.check_pause(i, &current).unwrap();
                let Some(SessionRequestError::InputRequired { request }) = &current.error else {
                    panic!("{} lifecycle pause {i}: {out:?}", f.id);
                };
                assert_eq!(request, planned);
                assert_eq!(s.pending_input_request(), Some(planned));
                assert!(s.eval_request("Global`oracleOverlap=1").is_err());
                assert_eq!(s.pending_input_request(), Some(planned));
                previous = Some(current);
                if i < stop {
                    out = s.resume_eval_request_with_input(answer);
                }
            }
            if reset {
                s.queue_input_response("discarded with reset");
                check.assert_unchanged();
                s = f.session();
                cancel.cancel();
            } else {
                cancel.cancel();
                let done = s
                    .finish_cancelled_pending_evaluation_request()
                    .unwrap()
                    .unwrap();
                assert_eq!(done.termination, Some(EvalTerminationReason::Cancelled));
                assert_eq!(
                    done.value.to_string(),
                    "TerminatedEvaluation[\"Cancelled\"]"
                );
                let current = Observation::from_output(&Ok(done));
                let previous = previous.unwrap();
                observe::prefix(&previous, &current).unwrap();
                let mut prints = previous.prints;
                prints.push("oracle cleanup".into());
                assert_eq!(current.prints, prints);
                assert_eq!(current.messages.len(), previous.messages.len() + 1);
                assert_eq!(current.messages.last().unwrap().tag, "cancel");
                let mut trace = previous.trace;
                trace.extend(cleanup_trace.clone());
                assert_eq!(current.trace, trace);
            }
            check.assert_unchanged();
            let _fresh_check = faults::Check::new(&s);
            assert!(s.pending_input_request().is_none());
            assert!(s.reject_overlapping_evaluation_request().is_ok());
            session::fresh(&mut s).unwrap();
            session::empty_queue(&mut s).unwrap();
        }
    }
    6 + lifecycle::exercise_observed_at(|| f.session(), &f.source, &plans, [0, 1], |_, _, _| {})
}

macro_rules! case {
    ($name:ident, $setup:expr, $source:expr, $prompts:expr, $value:expr, $log:expr, $traced:expr) => {
        #[test]
        fn $name() {
            equality(
                concat!("G15C1.", stringify!($name)),
                $setup,
                $source,
                $prompts,
                $value,
                $log,
                $traced,
            );
        }
    };
}

case!(
    all_red_atom,
    "",
    "ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard"],
    "2",
    "List[]",
    true
);

case!(
    all_red_list,
    "",
    "ReplaceAll[{1}, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard"],
    "List[2]",
    "List[]",
    true
);

case!(
    all_red_pattern_test,
    "",
    "ReplaceAll[1, (x_Integer ? (Function[z, (InputString[\"test\"]; True)])) :> (x + 1)]",
    &["test"],
    "2",
    "List[]",
    true
);

case!(
    all_red_guard_then_value,
    "",
    "ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (InputString[\"rhs\"]; x + 1)]",
    &["guard", "rhs"],
    "2",
    "List[]",
    true
);

case!(
    all_red_two_guards,
    "",
    "ReplaceAll[{1, 2}, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard", "guard"],
    "List[2, 3]",
    "List[]",
    true
);

case!(
    all_red_nested,
    "",
    "ReplaceAll[ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)], 2 -> 3]",
    &["guard"],
    "3",
    "List[]",
    true
);

case!(
    all_control_untraced,
    "",
    "ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard"],
    "2",
    "List[]",
    false
);

case!(
    all_control_rhs_logs,
    "",
    "ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (AppendTo[log, \"rhs\"]; x + 1)]",
    &["guard"],
    "2",
    "List[\"rhs\"]",
    false
);

case!(
    all_control_return,
    "f[] := (ReplaceAll[1, (x_Integer /; (InputString[\"guard\"]; True)) :> Return[5]]; 7)",
    "f[]",
    &["guard"],
    "5",
    "List[]",
    false
);

case!(
    all_control_target,
    "",
    "ReplaceAll[(InputString[\"target\"]; 1), 1 :> (1 + 1)]",
    &["target"],
    "2",
    "List[]",
    true
);

case!(
    all_control_value_only,
    "",
    "ReplaceAll[1, x_Integer :> (InputString[\"rhs\"]; x + 1)]",
    &["rhs"],
    "2",
    "List[]",
    true
);

case!(
    all_control_decline,
    "",
    "ReplaceAll[1, (x_Integer /; (InputString[\"decline\"]; False)) :> (x + 1)]",
    &["decline"],
    "1",
    "List[]",
    true
);

case!(
    unheld_red_atom,
    "",
    "ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard"],
    "2",
    "List[]",
    true
);

case!(
    unheld_red_list,
    "",
    "ReplaceAllUnheld[{1}, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard"],
    "List[2]",
    "List[]",
    true
);

case!(
    unheld_red_pattern_test,
    "",
    "ReplaceAllUnheld[1, (x_Integer ? (Function[z, (InputString[\"test\"]; True)])) :> (x + 1)]",
    &["test"],
    "2",
    "List[]",
    true
);

case!(
    unheld_red_guard_then_value,
    "",
    "ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (InputString[\"rhs\"]; x + 1)]",
    &["guard", "rhs"],
    "2",
    "List[]",
    true
);

case!(
    unheld_red_two_guards,
    "",
    "ReplaceAllUnheld[{1, 2}, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard", "guard"],
    "List[2, 3]",
    "List[]",
    true
);

case!(
    unheld_red_nested,
    "",
    "ReplaceAllUnheld[ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)], 2 -> 3]",
    &["guard"],
    "3",
    "List[]",
    true
);

case!(
    unheld_control_untraced,
    "",
    "ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (x + 1)]",
    &["guard"],
    "2",
    "List[]",
    false
);

case!(
    unheld_control_rhs_logs,
    "",
    "ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> (AppendTo[log, \"rhs\"]; x + 1)]",
    &["guard"],
    "2",
    "List[\"rhs\"]",
    false
);

case!(
    unheld_control_return,
    "f[] := (ReplaceAllUnheld[1, (x_Integer /; (InputString[\"guard\"]; True)) :> Return[5]]; 7)",
    "f[]",
    &["guard"],
    "5",
    "List[]",
    false
);

case!(
    unheld_control_target,
    "",
    "ReplaceAllUnheld[(InputString[\"target\"]; 1), 1 :> (1 + 1)]",
    &["target"],
    "2",
    "List[]",
    true
);

case!(
    unheld_control_value_only,
    "",
    "ReplaceAllUnheld[1, x_Integer :> (InputString[\"rhs\"]; x + 1)]",
    &["rhs"],
    "2",
    "List[]",
    true
);

case!(
    unheld_control_decline,
    "",
    "ReplaceAllUnheld[1, (x_Integer /; (InputString[\"decline\"]; False)) :> (x + 1)]",
    &["decline"],
    "1",
    "List[]",
    true
);
