//! G14 logical context is independent of tracked length and native stack state.
use super::*;
use crate::kernel_api::{
    ContinueToken, FrameRootTrace, FrameRootVisitor, TypedAction, TypedFrame, slot_witness,
};
use crate::runtime::native_stack_guard::{native_stack_exhausted, with_thread_native_stack_base};
use crate::runtime::suspension::{
    AbandonmentContext, FrameAbandon, FrameAbandonAction, FrameAction, SuspensionFrame,
};
use std::sync::{Arc, Mutex};

type Observation = (Vec<(String, u64)>, usize, u64);
fn observe(s: &Session) -> Observation {
    (
        s.evaluation_stack
            .iter()
            .map(|e| (e.expr.to_string(), e.epoch))
            .collect(),
        s.evaluation_depth.get(),
        s.evaluation_epoch,
    )
}
fn next_check_depth(mut depth: EvaluationDepth) -> usize {
    let mut entered_at = depth.get();
    while !depth.native_check_due(entered_at) {
        depth.enter_level(entered_at);
        entered_at += 1;
    }
    entered_at
}
fn set_high_water_check_depth(s: &mut Session, check_depth: usize) {
    let entered = check_depth - NATIVE_STACK_DEPTH_POLL_STRIDE;
    s.evaluation_depth.restore(entered);
    for d in (0..entered).rev() {
        s.evaluation_depth.leave_level(d);
    }
}
struct Check {
    expected: Observation,
    seen: Arc<Mutex<Vec<Observation>>>,
}
impl FrameRootTrace for Check {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for Check {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
impl Check {
    fn check(&self, s: &Session) {
        assert_eq!(
            next_check_depth(s.evaluation_depth),
            s.evaluation_depth.get() + NATIVE_STACK_DEPTH_POLL_STRIDE
        );
        let actual = observe(s);
        self.seen.lock().unwrap().push(actual.clone());
        assert_eq!(actual, self.expected, "G14 frame context");
        if s.current_memory_limit.is_some() {
            s.assert_maintained_memory_accounting();
        }
    }
}
impl SuspensionFrame for Check {
    fn resume(self: Arc<Self>, s: &mut Session, result: Result<Expr, EvalError>) -> FrameAction {
        self.check(s);
        FrameAction::complete(result)
    }
}
impl TypedFrame for Check {
    type Input = Expr;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        s: &mut Session,
        result: Result<Expr, EvalError>,
        _: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        self.check(s);
        TypedAction::complete(result)
    }
}
fn seeded() -> Session {
    let mut s = Session::new();
    s.evaluation_epoch = 17;
    s.evaluation_stack = ["outer", "middle", "inner"]
        .into_iter()
        .enumerate()
        .map(|(i, name)| EvaluationStackEntry {
            expr: Expr::symbol(name),
            epoch: 14 + i as u64,
        })
        .collect();
    s.evaluation_depth.restore(8);
    s.begin_input_suspension(RuntimeInputRequest {
        kind: RuntimeInputKind::InputString,
        prompt: "context".into(),
        password: false,
    });
    s
}

#[test]
fn full_snapshot_retains_all_original_entry_epochs() {
    let s = seeded();
    let saved: Vec<_> = s
        .pending_suspension
        .as_ref()
        .unwrap()
        .active_stack
        .iter()
        .map(|e| (e.expr.to_string(), e.epoch))
        .collect();
    assert_eq!(saved, observe(&s).0);
}

fn routes(typed: bool, armed: bool) {
    let mut s = seeded();
    if armed {
        s.current_memory_limit = Some(1_000_000_000);
        s.note_accounting_arm();
    }
    let seen = Arc::new(Mutex::new(Vec::new()));
    for (length, depth) in [(3, 8), (1, 3), (0, 0)] {
        let mut entries = s.evaluation_stack.clone();
        entries.truncate(length);
        s.replace_accounted_evaluation_stack(entries);
        s.evaluation_depth.restore(depth);
        let expected = observe(&s);
        let frame = Check {
            expected,
            seen: seen.clone(),
        };
        if typed {
            let result: Result<Expr, EvalError> = Err(EvalError::InputRequired {
                request: s.pending_input_request().unwrap().clone(),
            });
            let witness = slot_witness(&result);
            let Err(EvalError::InputRequired { request }) = result else {
                unreachable!()
            };
            let _ = witness.suspend(&mut s, request, frame);
        } else {
            s.insert_suspension_frame(frame);
        }
    }
    assert_eq!(
        s.resume_suspended_evaluation("x").unwrap(),
        Expr::string("x")
    );
    assert_eq!(seen.lock().unwrap().len(), 3);
    assert_eq!(s.evaluation_depth.get(), 0);
    assert!(s.evaluation_stack.is_empty());
    if armed {
        s.assert_maintained_memory_accounting();
    }
}
#[test]
fn legacy_contexts_restore_independent_depth() {
    routes(false, false);
}
#[test]
fn typed_witness_contexts_restore_independent_depth() {
    routes(true, false);
}
#[test]
fn armed_contexts_reconcile_every_truncation() {
    routes(true, true);
}

#[test]
fn native_resume_restores_displaced_poll_state() {
    let mut s = Session::new();
    let expr = s.lower_text("InputString[\"native\"]").unwrap();
    assert!(matches!(
        s.evaluate(expr),
        Err(EvalError::InputRequired { .. })
    ));
    set_high_water_check_depth(&mut s, 29);
    with_thread_native_stack_base(123, || {
        s.resume_suspended_evaluation("x").unwrap();
        assert!(!native_stack_exhausted(123, 0));
        assert!(native_stack_exhausted(122, 0));
    });
    assert_eq!(
        (
            s.evaluation_depth.get(),
            next_check_depth(s.evaluation_depth)
        ),
        (0, 29)
    );
}

#[test]
fn evaluation_return_source_boundary_is_closed() {
    let resolution = include_str!("../resolution.rs");
    let caller = "self.evaluate_once_unstable_after_input_check_with_cycle_tracking(";
    assert_eq!(resolution.matches(caller).count(), 2);
    fn callers(path: &Path, needle: &str) -> usize {
        fs::read_dir(path)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                if path
                    .file_name()
                    .is_some_and(|name| name == "tests" || name == "tests.rs")
                {
                    return 0;
                }
                if path.is_dir() {
                    callers(&path, needle)
                } else if path.extension().is_some_and(|extension| extension == "rs") {
                    fs::read_to_string(path).unwrap().matches(needle).count()
                } else {
                    0
                }
            })
            .sum()
    }
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime");
    assert_eq!(
        callers(&runtime, caller),
        2,
        "every native evaluator caller needs the approved return boundary"
    );
    assert_eq!(resolution.matches("evaluation_return =>").count(), 2);
    let witness = include_str!("../suspension/typed/witness.rs");
    let start = witness
        .find("fn suspend_evaluation_return")
        .expect("private return helper");
    let helper = &witness[start..];
    assert!(
        helper.find("insert_evaluation_return").unwrap() < helper.find("self.suspend(").unwrap(),
        "marker precedes its fixed-point consumer"
    );
}

#[test]
fn three_witnesses_record_exact_full_snapshots_and_return_levels() {
    for (id, source, count, depth, length) in [
        ("false-cycle", "Table[f[1],{3}]", 3, 4, 4),
        ("depth-leak", "Table[(t=i;f[1]),{i,1100}]", 1100, 5, 3),
        ("passing", "Table[InputString[\"rhs\"],{3}]", 3, 2, 2),
    ] {
        let mut s = session_with_test_control_builtins();
        s.eval_request("f[x_]:=(InputString[\"rhs\"];7)").unwrap();
        s.start_context_observations_for_test();
        let mut result = s.eval_request(source);
        let mut pauses = 0;
        while matches!(&result, Err(f) if matches!(f.error, SessionRequestError::InputRequired { .. }))
        {
            pauses += 1;
            assert!(pauses <= count);
            let snapshot = &s.pending_suspension.as_ref().unwrap().active_stack;
            assert_eq!(snapshot.len(), length, "{id} pause {pauses}");
            result = s.resume_eval_request_with_input("x");
        }
        assert_eq!(pauses, count);
        assert!(result.unwrap().termination.is_none());
        let observations = s.take_context_observations_for_test();
        let snapshots: Vec<_> = observations.iter().filter(|o| o.phase == "pause").collect();
        assert_eq!(snapshots.len(), count);
        for snapshot in snapshots {
            assert_eq!(
                (snapshot.entries.len(), snapshot.depth),
                (length, depth),
                "{id}"
            );
            assert!(
                snapshot
                    .entries
                    .iter()
                    .all(|(_, epoch)| *epoch == snapshot.epoch)
            );
        }
        assert!(observations.iter().all(|o| o.entries.len() <= o.depth));
        // Queued native returns pop only tracked roots but always lower depth.
        // These are the distinct evaluator levels, independent of helper count.
        let queued_returns = match id {
            "false-cycle" => vec![(3, 3), (2, 2), (1, 1)],
            "depth-leak" => vec![(2, 4), (1, 3), (0, 2), (0, 1)],
            "passing" => vec![(1, 1)],
            _ => unreachable!(),
        };
        let next_pause = observations
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, o)| o.phase == "pause")
            .unwrap()
            .0;
        let first_segment = &observations[..next_pause];
        assert_eq!(
            first_segment
                .iter()
                .filter(|o| o.phase == "return")
                .map(|o| (o.entries.len(), o.depth))
                .collect::<Vec<_>>(),
            queued_returns,
            "{id} native return boundaries"
        );
        for sample in first_segment.iter().filter(|o| o.phase == "frame") {
            let level = (sample.entries.len(), sample.depth);
            assert!(
                level == (length, depth) || queued_returns.contains(&level),
                "{id} helper level {level:?}"
            );
            assert_eq!(
                sample.entries,
                observations[0].entries[..sample.entries.len()],
                "{id} original prefix and entry epochs"
            );
        }
        let returns: Vec<_> = observations
            .iter()
            .filter(|o| o.phase == "return")
            .collect();
        assert_eq!(returns.last().unwrap().depth, 0);
        assert!(returns.last().unwrap().entries.is_empty());
        for o in observations.iter().take(24) {
            println!(
                "G14 CONTEXT {id} phase={} depth={} epoch={} entries={:?}",
                o.phase, o.depth, o.epoch, o.entries
            );
        }
        println!(
            "G14 CONTEXT {id} pauses={count} events={}",
            observations.len()
        );
    }
}

#[test]
fn intervening_epoch_mutation_is_not_rewound() {
    let mut s = seeded();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let mut expected = observe(&s);
    expected.2 = 29;
    s.insert_suspension_frame(Check { expected, seen });
    s.evaluation_epoch = 29;
    s.clear_accounted_evaluation_stack();
    s.evaluation_depth.restore(0);
    s.resume_suspended_evaluation("x").unwrap();
    assert_eq!(s.evaluation_epoch, 29);
}

#[test]
fn truncation_releases_occurrences_preserves_capacity_and_rearms() {
    let mut s = Session::new();
    let shared = Expr::call(Expr::symbol("held"), vec![Expr::string("root".repeat(40))]);
    let mut entries = Vec::with_capacity(16);
    entries.extend((0..4).map(|epoch| EvaluationStackEntry {
        expr: shared.clone(),
        epoch,
    }));
    s.replace_accounted_evaluation_stack(entries);
    let capacity = s.evaluation_stack.capacity();
    for length in [3, 2, 1, 0] {
        s.current_memory_limit = Some(1_000_000_000);
        s.note_accounting_arm();
        s.truncate_accounted_evaluation_stack(length);
        s.assert_maintained_memory_accounting();
        assert_eq!(s.evaluation_stack.capacity(), capacity);
        s.current_memory_limit = None;
    }
    let counters = s.ensure_accounting_shadow().snapshot();
    assert_eq!(
        (counters.pushes, counters.pops, counters.live),
        (0, 0, 0),
        "restored occurrences are not native shadow guards"
    );
}

#[test]
fn local_arming_second_pause_and_cancellation_keep_accounting_exact() {
    for cancel in [false, true] {
        let mut s = session_with_test_control_builtins();
        let source =
            "MemoryConstrained[Table[(InputString[\"a\"];InputString[\"b\"];7),{2}],1000000000]";
        let mut result = s.eval_request(source);
        for pause in 0..4 {
            assert!(
                matches!(&result, Err(f) if matches!(f.error, SessionRequestError::InputRequired { .. }))
            );
            s.assert_maintained_memory_accounting();
            let counters = s.ensure_accounting_shadow().snapshot();
            assert_eq!(counters.pushes, counters.pops);
            assert_eq!(counters.live, 0);
            if cancel && pause == 1 {
                s.request_cancel_handle().cancel();
                result = Ok(s
                    .finish_cancelled_pending_evaluation_request()
                    .unwrap()
                    .unwrap());
                break;
            }
            result = s.resume_eval_request_with_input("x");
        }
        let result = result.unwrap();
        if cancel {
            assert_eq!(result.termination, Some(EvalTerminationReason::Cancelled));
        } else {
            assert_eq!(result.value.to_string(), "List[7, 7]");
        }
        assert!(s.pending_suspension.is_none());
        assert!(s.resuming_suspension.is_none());
        assert_eq!(s.eval_request("1+1").unwrap().value, Expr::small_integer(2));
        s.current_memory_limit = Some(1_000_000_000);
        s.note_accounting_arm();
        s.assert_maintained_memory_accounting();
    }
}

#[test]
fn resumed_frame_checks_native_headroom_without_entering_a_new_logical_level() {
    let mut s = Session::new();
    assert!(matches!(
        s.eval_input_string(&[Expr::string("bounded native")]),
        Err(EvalError::InputRequired { .. })
    ));
    let seen = Arc::new(Mutex::new(Vec::new()));
    s.insert_suspension_frame(Check {
        expected: observe(&s),
        seen: seen.clone(),
    });
    s.resource_limits.native_stack_limit = 0;
    set_high_water_check_depth(&mut s, 29);
    assert!(matches!(
        s.resume_suspended_evaluation("x"),
        Err(EvalError::StackLimitExceeded { limit: 0 })
    ));
    assert_eq!(
        seen.lock().unwrap().len(),
        1,
        "the error still unwinds the callable frame"
    );
    assert_eq!(
        (
            s.evaluation_depth.get(),
            next_check_depth(s.evaluation_depth)
        ),
        (0, 29)
    );
}
