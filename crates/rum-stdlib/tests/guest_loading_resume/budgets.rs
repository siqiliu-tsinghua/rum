use super::*;
use std::time::Duration;

#[test]
fn red_guest_load_inherits_memory_constraint() {
    let (mut s, _) = session(&[("Mem.rum", "x = Range[50000];\nLength[x]")]);
    let out = s
        .eval_request("MemoryConstrained[Get[\"Mem`\"], 100000]")
        .unwrap();
    println!(
        "BUDGET memory value={} termination={:?}",
        out.value, out.termination
    );
    assert_eq!(out.value.to_string(), "$Aborted");
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
}

#[test]
fn red_guest_forms_share_request_deadline() {
    // Each form waits at least 100 ms, followed by enough loop checkpoints to
    // observe the deadline. Eight forms take at least 800 ms against one 400 ms
    // budget. Fixed minimum work cannot get faster after a noisy calibration.
    let form = "Pause[1/10]; Do[Global`y = i, {i, 2000}];";
    let timeout = Duration::from_millis(400);
    let (mut control, _) = session(&[]);
    control.set_eval_timeout(Some(timeout));
    let single = control.eval_request(form).unwrap();
    assert_eq!(single.value.to_string(), "Null");
    assert!(single.termination.is_none(), "one form must fit the budget");

    let source = format!("{}\n\"three-done\"", vec![form; 8].join("\n"));
    let (mut s, _) = session(&[("Three.rum", &source)]);
    s.set_eval_timeout(Some(timeout));
    let out = s.eval_request("Get[\"Three`\"]").unwrap();
    println!(
        "BUDGET timeout per_form_minimum=100ms forms=8 deadline={timeout:?} single={} value={} termination={:?}",
        single.value, out.value, out.termination
    );
    assert_eq!(
        out.value.to_string(),
        "TerminatedEvaluation[\"TimeConstraint\"]"
    );
    assert_eq!(format!("{:?}", out.termination), "Some(TimeConstraint)");
    assert_eq!(s.eval_request("1+1").unwrap().value.to_string(), "2");
}
