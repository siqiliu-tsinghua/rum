//! A completed fixed-point pass must publish before its next pass resumes.
use crate::runtime::kernel_api::{BuiltinSpec, MemoryGrowthContract};
use crate::runtime::profiling::logical::test_support;
use crate::runtime::{EvalError, Expr, Session, SharedArgs};

const PASS_KEY: &str = "Runtime { phase: \"evaluate.once_unstable\", source: None }";

// The pure next-pass producer used by the frozen fixed-point witnesses.
fn fixedpoint_step(_: &mut Session, _: &Expr, args: &SharedArgs) -> Result<Expr, EvalError> {
    assert!(args.len() >= 2);
    let stage = args[args.len() - 2].to_string().parse::<i64>().unwrap();
    let limit = args[args.len() - 1].to_string().parse::<i64>().unwrap();
    if stage < limit {
        return Ok(Expr::call(
            Expr::system_symbol("C8FixedPointStep"),
            vec![
                Expr::call(
                    Expr::system_symbol("Input"),
                    vec![Expr::string(format!("c8step{}", stage + 1))],
                ),
                Expr::small_integer(stage + 1),
                Expr::small_integer(limit),
            ],
        ));
    }
    let payload = &args[..args.len() - 2];
    Ok(if payload.len() == 1 {
        payload[0].clone()
    } else {
        Expr::call(Expr::system_symbol("Sequence"), payload.to_vec())
    })
}

fn run(queued: bool, pauses: usize, terminal: &str) -> (Vec<(String, [u64; 10])>, String) {
    let _enabled = test_support::enabled();
    let mut session = crate::runtime::test_builtins::session_with_test_control_builtins();
    session.register_builtin(BuiltinSpec::ordinary(
        "C8FixedPointStep",
        "Pure next-pass test producer.",
        fixedpoint_step,
        MemoryGrowthContract::Delegated,
    ));
    session.enable_tracing(true);
    let replies: Vec<_> = (0..pauses)
        .map(|index| if index + 1 == pauses { terminal } else { "73" })
        .collect();
    if queued {
        for reply in &replies {
            session.queue_input_response(*reply);
        }
    }
    let source = format!("C8FixedPointStep[Input[\"c8step1\"],1,{pauses}]");
    let mut outcome = session.eval_request(&source);
    if !queued {
        for (index, reply) in replies.iter().enumerate() {
            assert!(outcome.is_err(), "pause {}: {outcome:?}", index + 1);
            assert_eq!(
                session
                    .pending_input_request()
                    .expect("real Input request")
                    .prompt,
                format!("c8step{}", index + 1),
            );
            outcome = session.resume_eval_request_with_input(*reply);
        }
    }
    assert!(session.pending_input_request().is_none());
    let language = format!("{outcome:?} trace={:?}", session.take_trace());
    let rows = Session::logical_profile_test_rows();
    assert!(
        rows.iter()
            .any(|(key, counts)| key == PASS_KEY && counts[0] > 0)
    );
    assert!(
        rows.iter()
            .all(|(_, counts)| counts[8] == 0 && counts[9] == 0)
    );
    Session::logical_profile_test_assert_quiescent();
    assert_eq!(Session::logical_profile_test_leases(), 0);
    (rows, language)
}

fn compare(pauses: usize) {
    for terminal in ["Throw[83,c8error]", "Return[83]"] {
        let queued = run(true, pauses, terminal);
        let resumed = run(false, pauses, terminal);
        assert_eq!(resumed.1, queued.1, "language: {pauses} pauses, {terminal}");
        assert_eq!(
            resumed.0, queued.0,
            "every ten-counter record: {pauses} pauses, {terminal}"
        );
    }
}

#[test]
fn fixedpoint_two_pause_pass_counters_match_queued() {
    compare(2);
}

#[test]
fn fixedpoint_three_pause_pass_counters_match_queued() {
    compare(3);
}
