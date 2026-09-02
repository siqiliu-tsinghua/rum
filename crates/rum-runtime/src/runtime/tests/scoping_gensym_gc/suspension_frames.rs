use std::sync::Arc;

use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::suspension::{
    AbandonmentContext, FrameAbandon, FrameAbandonAction, FrameAction, SuspensionFrame,
};

const PATH_A_SUBCASE: &str = "frame-roots-path-a";
const PATH_B_SUBCASE: &str = "frame-roots-path-b";
const MATCHER_SUBCASE: &str = "frame-roots-matcher";
const BARE_SYMBOL_SUBCASE: &str = "frame-roots-bare-symbol";
const CURRENT_AND_SUFFIX_SUBCASE: &str = "frame-roots-current-and-suffix";

fn enter_threshold_one_subprocess(test_function: &str, subcase: &str) -> bool {
    if in_gensym_gc_watermark_subprocess(subcase) {
        return true;
    }
    run_gensym_gc_watermark_subprocess(test_function, subcase, 1);
    false
}

fn allocate_named_candidate(session: &mut Session, base: &str, value: i64) -> Symbol {
    let before = session.gc_candidate_gensyms.clone();
    let outcome = session
        .eval_text(&format!(
            "Module[{{{base}}}, {base} = {value}; Hold[{base}]]"
        ))
        .expect("candidate allocation should finish");
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .filter(|candidate| !before.contains(candidate))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        outcome.value.to_string(),
        format!("Hold[{}]", candidates[0])
    );
    candidates[0]
}

fn allocate_current_and_suffix_candidates(session: &mut Session) -> (Symbol, Symbol) {
    let before = session.gc_candidate_gensyms.clone();
    let outcome = session
        .eval_text(
            "Module[{gcCurrent, gcSuffix}, \
             gcCurrent = 711; gcSuffix = 712; Hold[{gcCurrent, gcSuffix}]]",
        )
        .expect("the two frame candidates should allocate together");
    let mut candidates = session
        .gc_candidate_gensyms
        .iter()
        .filter(|candidate| !before.contains(candidate))
        .copied()
        .collect::<Vec<_>>();
    candidates.sort_by_key(Symbol::to_string);
    assert_eq!(candidates.len(), 2);
    let current = candidates
        .iter()
        .copied()
        .find(|candidate| candidate.name().starts_with("gcCurrent$"))
        .expect("the current-frame candidate should be present");
    let suffix = candidates
        .iter()
        .copied()
        .find(|candidate| candidate.name().starts_with("gcSuffix$"))
        .expect("the suffix-frame candidate should be present");
    assert_eq!(
        outcome.value.to_string(),
        format!("Hold[List[{current}, {suffix}]]")
    );
    (current, suffix)
}

fn assert_input_required(failure: &SessionRequestFailure, prompt: &str) {
    match &failure.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, prompt),
        other => panic!("expected an input request for {prompt:?}, got {other:?}"),
    }
}

#[test]
fn path_a_second_request_keeps_frame_only_candidate_live() {
    if !enter_threshold_one_subprocess(
        "suspension_frames::path_a_second_request_keeps_frame_only_candidate_live",
        PATH_A_SUBCASE,
    ) {
        return;
    }

    let mut session = session_with_test_control_builtins();
    let candidate = allocate_named_candidate(&mut session, "gcA", 701);
    assert_eq!(eval_string(&mut session, &candidate.to_string()), "701");

    let failure = session
        .eval_request(&format!("List[InputString[\"A: \"], {candidate}]"))
        .expect_err("Path A should suspend");
    assert_input_required(&failure, "A: ");

    let sweeps_before = session.gensym_gc_sweep_count;
    let trigger = session
        .eval_request("Module[{gcATrigger}, gcATrigger = 1; Hold[gcATrigger]]")
        .expect("the second request should cross the watermark");
    assert_eq!(trigger.value.to_string(), "Hold[gcATrigger$2]");
    assert!(session.gensym_gc_sweep_count > sweeps_before);

    let resumed = session
        .resume_suspended_evaluation("ready")
        .expect("the original Path A suspension should resume");
    assert_eq!(resumed.to_string(), "List[\"ready\", 701]");
}

#[test]
fn path_b_resume_sweep_keeps_remaining_frame_candidate_live() {
    if !enter_threshold_one_subprocess(
        "suspension_frames::path_b_resume_sweep_keeps_remaining_frame_candidate_live",
        PATH_B_SUBCASE,
    ) {
        return;
    }

    let mut session = session_with_test_control_builtins();
    let candidate = allocate_named_candidate(&mut session, "gcB", 702);
    assert_eq!(eval_string(&mut session, &candidate.to_string()), "702");

    let failure = session
        .eval_request(&format!("List[Input[\"B: \"], {candidate}]"))
        .expect_err("Path B should suspend");
    assert_input_required(&failure, "B: ");

    let sweeps_before = session.gensym_gc_sweep_count;
    let resumed = session
        .resume_eval_request_with_input("Module[{gcBTrigger}, gcBTrigger = 1; Hold[gcBTrigger]]")
        .expect("Path B should resume through its Module trigger");
    assert!(session.gensym_gc_sweep_count > sweeps_before);
    assert_eq!(resumed.value.to_string(), "List[Hold[gcBTrigger$2], 702]");
}

#[test]
fn matcher_guard_binding_survives_a_sweep_while_suspended() {
    if !enter_threshold_one_subprocess(
        "suspension_frames::matcher_guard_binding_survives_a_sweep_while_suspended",
        MATCHER_SUBCASE,
    ) {
        return;
    }

    let mut session = session_with_test_control_builtins();
    let candidate = allocate_named_candidate(&mut session, "gcM", 703);
    assert_eq!(eval_string(&mut session, &candidate.to_string()), "703");
    session
        .eval_text(&format!(
            "gcMRoot = Hold[{candidate}]; \
             gcMFn[Hold[z_] /; MatchQ[InputString[\"M: \"], \"yes\"]] := z; \
             gcMThunk := With[{{v = gcMRoot}}, Clear[gcMRoot, gcMThunk]; v]"
        ))
        .expect("matcher probe definition should install");

    let failure = session
        .eval_request("gcMFn[gcMThunk]")
        .expect_err("the matcher guard should suspend");
    assert_input_required(&failure, "M: ");

    let sweeps_before = session.gensym_gc_sweep_count;
    session
        .eval_text("Module[{gcMTrigger}, gcMTrigger = 1; Hold[gcMTrigger]]")
        .expect("the matcher probe should cross the watermark");
    assert!(session.gensym_gc_sweep_count > sweeps_before);

    let resumed = session
        .resume_eval_request_with_input("yes")
        .expect("the matcher guard should resume");
    assert_eq!(resumed.value.to_string(), "703");
}

#[test]
fn block_frame_bare_symbol_is_reclaimed_after_resume_and_next_sweep() {
    if !enter_threshold_one_subprocess(
        "suspension_frames::block_frame_bare_symbol_is_reclaimed_after_resume_and_next_sweep",
        BARE_SYMBOL_SUBCASE,
    ) {
        return;
    }

    let mut session = session_with_test_control_builtins();
    let candidate = allocate_named_candidate(&mut session, "gcS", 704);
    assert_eq!(eval_string(&mut session, &candidate.to_string()), "704");

    let failure = session
        .eval_request(&format!("Block[{{{candidate}}}, InputString[\"S: \"]]"))
        .expect_err("the Block body should suspend");
    assert_input_required(&failure, "S: ");

    let first_sweeps_before = session.gensym_gc_sweep_count;
    session
        .eval_request("Module[{gcSTrigger}, gcSTrigger = 1; Hold[gcSTrigger]]")
        .expect("the suspended Block should survive a real sweep");
    assert!(session.gensym_gc_sweep_count > first_sweeps_before);

    let resumed = session
        .resume_suspended_evaluation("ready")
        .expect("the Block should restore its definition after resuming");
    assert_eq!(resumed.to_string(), "\"ready\"");

    let second_sweeps_before = session.gensym_gc_sweep_count;
    session
        .eval_request("Module[{gcSTrigger2}, gcSTrigger2 = 1; Hold[gcSTrigger2]]")
        .expect("the post-resume request should force another sweep");
    assert!(session.gensym_gc_sweep_count > second_sweeps_before);
    assert_eq!(
        eval_string(&mut session, &candidate.to_string()),
        candidate.to_string()
    );
    assert!(!session.definitions.contains_key(&candidate));
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct CurrentFrameSweepProbe {
    candidate: Expr,
}

impl SuspensionFrame for CurrentFrameSweepProbe {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        if let Err(error) = outcome {
            return FrameAction::complete(Err(error));
        }
        session
            .eval_text("Module[{gcCurrentTrigger}, gcCurrentTrigger = 1; Hold[gcCurrentTrigger]]")
            .expect("the executing frame should force a real Module sweep");
        FrameAction::complete(session.evaluate(self.candidate.clone()))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct SuffixFrameProbe {
    candidate: Expr,
}

impl SuspensionFrame for SuffixFrameProbe {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let result = outcome.and_then(|current| {
            session
                .evaluate(self.candidate.clone())
                .map(|suffix| Expr::call(system_expr("List"), vec![current, suffix]))
        });
        FrameAction::complete(result)
    }
}

#[test]
fn executing_frame_and_queued_suffix_remain_rooted_during_resume_sweep() {
    if !enter_threshold_one_subprocess(
        "suspension_frames::executing_frame_and_queued_suffix_remain_rooted_during_resume_sweep",
        CURRENT_AND_SUFFIX_SUBCASE,
    ) {
        return;
    }

    let mut session = session_with_test_control_builtins();
    let (current, suffix) = allocate_current_and_suffix_candidates(&mut session);
    assert_eq!(eval_string(&mut session, &current.to_string()), "711");
    assert_eq!(eval_string(&mut session, &suffix.to_string()), "712");

    session.begin_input_suspension(RuntimeInputRequest::new("current: "));
    session.insert_suspension_frame(CurrentFrameSweepProbe {
        candidate: symbol_expr(current),
    });
    session.insert_suspension_frame(SuffixFrameProbe {
        candidate: symbol_expr(suffix),
    });

    let sweeps_before = session.gensym_gc_sweep_count;
    let resumed = session
        .resume_suspended_evaluation("go")
        .expect("both rooted probe frames should resume");
    assert!(session.gensym_gc_sweep_count > sweeps_before);
    assert_eq!(resumed.to_string(), "List[711, 712]");
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct LargeSuspensionMemoryProbe {
    values: SharedArgs,
}

impl SuspensionFrame for LargeSuspensionMemoryProbe {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let result = outcome.and_then(|_| session.evaluate(Expr::integer(BigInt::from(1_u8))));
        FrameAction::complete(result)
    }
}

#[test]
fn large_frame_state_trips_memory_limit_and_session_recovers() {
    let mut session = session_with_test_control_builtins();
    let failure = session
        .eval_request("InputString[\"memory: \"]")
        .expect_err("the memory probe should establish a real request suspension");
    assert_input_required(&failure, "memory: ");
    let before_frame = session.current_memory_proxy_bytes_for_test();
    let values = (0..4096)
        .map(|index| Expr::string(format!("frame-payload-{index:04}")))
        .collect::<Vec<_>>();
    session.insert_suspension_frame(LargeSuspensionMemoryProbe {
        values: SharedArgs::new(values),
    });
    let corrected = session.current_memory_proxy_bytes_for_test();
    let old_flat_estimate = before_frame.saturating_add(16);
    assert!(
        corrected > old_flat_estimate.saturating_add(100_000),
        "the test payload must materially separate the corrected proxy from the old flat estimate"
    );

    let limit = old_flat_estimate + (corrected - old_flat_estimate) / 2;
    let mut limits = session.resource_limits();
    limits.eval_memory_limit = Some(limit);
    session.set_resource_limits(limits);
    session.current_memory_limit = Some(limit);

    let response = session
        .resume_eval_request_with_input("resume")
        .expect("the memory ceiling should produce a classified termination value");
    assert_eq!(
        response.value.to_string(),
        "TerminatedEvaluation[\"MemoryLimit\"]"
    );
    assert_eq!(
        response.termination,
        Some(EvalTerminationReason::MemoryLimit)
    );
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "memlim")
    );

    let recovered = session
        .eval_request("1 + 1")
        .expect("the session should recover after clearing the oversized suspension");
    assert_eq!(recovered.value.to_string(), "2");
    assert_eq!(recovered.termination, None);
}
