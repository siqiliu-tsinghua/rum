//! P0001's public retention caller, with direct occurrence-charge observers.
//! The observer scope belongs to one run; no session or answer state is reused.
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{
    BuiltinSpec, CallbackValue, FrameAction, FrameRootTrace, FrameRootVisitor, FrozenValues,
    HoldAttribute, MemoryGrowthContract, RetentionContinuation,
};
use rum_runtime::{EvalError, Session};
use std::cell::RefCell;

/// Observe the dispatch build through the checked-only NonAmplifying contract.
/// The deliberate violation runs in its own session, never a fixture session.
pub(crate) fn checked_dispatch_enabled() -> bool {
    fn amplifying_result(
        _session: &mut Session,
        _head: &Expr,
        _args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        Ok(Expr::string("x".repeat(4096)))
    }
    let observed = std::panic::catch_unwind(|| {
        let mut session = Session::new();
        session.register_builtin(BuiltinSpec::ordinary(
            "CM739NonAmplifying",
            "Feature-isolated dispatch configuration discriminator.",
            amplifying_result,
            MemoryGrowthContract::NonAmplifying,
        ));
        session.eval_request("CM739NonAmplifying[]")
    });
    match observed {
        Ok(result) => {
            assert!(
                result.is_ok(),
                "configuration probe must evaluate: {result:?}"
            );
            false
        }
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied());
            if !message.is_some_and(|text| {
                text.starts_with("builtin CM739NonAmplifying declared NonAmplifying but returned ")
                    && text.contains("argument proxy bytes (allowance: 1024 bytes)")
            }) {
                std::panic::resume_unwind(payload);
            }
            true
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct NativeObservation {
    pub entries: usize,
    pub resumes: usize,
    pub charges: Vec<(&'static str, usize)>,
}

std::thread_local! {
    static OBSERVATION: RefCell<Option<NativeObservation>> = const { RefCell::new(None) };
    static FAULTS: RefCell<Vec<crate::oracle::support::faults::Check>> = const { RefCell::new(Vec::new()) };
}

pub(crate) struct ObserverScope;

impl ObserverScope {
    pub fn enter() -> Self {
        FAULTS.with(|probes| assert!(probes.borrow().is_empty()));
        OBSERVATION.with(|slot| {
            assert!(slot.borrow().is_none(), "independent native observer scope");
            *slot.borrow_mut() = Some(NativeObservation::default());
        });
        Self
    }

    pub fn snapshot(&self) -> NativeObservation {
        OBSERVATION.with(|slot| slot.borrow().as_ref().unwrap().clone())
    }

    pub fn assert_clean(&self) {
        FAULTS.with(|probes| {
            for probe in probes.borrow().iter() {
                probe.assert_unchanged();
            }
        });
    }
}

impl Drop for ObserverScope {
    fn drop(&mut self) {
        OBSERVATION.with(|slot| *slot.borrow_mut() = None);
        FAULTS.with(|probes| probes.borrow_mut().clear());
    }
}

fn record(update: impl FnOnce(&mut NativeObservation)) {
    OBSERVATION.with(|slot| update(slot.borrow_mut().as_mut().expect("active receipt run")));
}

fn observe(bytes: Option<usize>, branch: &'static str) {
    let bytes = bytes.expect("P0001 actual occurrence receipt is valid");
    record(|observation| observation.charges.push((branch, bytes)));
}

#[derive(FrameRootTrace)]
struct CallbackFinish;

impl RetentionContinuation for CallbackFinish {
    fn resume(
        &self,
        session: &mut Session,
        frozen: &FrozenValues,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        record(|observation| observation.resumes += 1);
        let result = session
            .accounting_context()
            .expect("P0001 restores the armed context")
            .with_retention(|mut cx, empty| {
                let mut values = cx.resume_values(empty, frozen)?;
                cx.push(&mut values, outcome?)?;
                observe(values.actual_receipt_bytes(), "resumed-value");
                cx.finish_list(values)
            });
        FrameAction::complete(result)
    }
}

fn callback_charge(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    assert_eq!(args.len(), 1);
    record(|observation| observation.entries += 1);
    session
        .accounting_context()
        .expect("P0001 caller is inside MemoryConstrained")
        .with_retention(|cx, empty| {
            let callback = if rum_runtime::cm740_probe::driven() {
                cx.evaluate_cm740_driven(empty, args[0].clone())
            } else {
                cx.evaluate(empty, args[0].clone())
            }
            .map(|value| {
                observe(value.actual_receipt_bytes(), "callback-value");
                value
            });
            match callback.and_then(CallbackValue::adopt) {
                Ok((cx, values)) => cx.finish_list(values),
                Err(failure) => Ok(failure.suspend(|| CallbackFinish)),
            }
        })
}

#[derive(FrameRootTrace)]
struct PrefixFinish;

impl RetentionContinuation for PrefixFinish {
    fn resume(
        &self,
        session: &mut Session,
        frozen: &FrozenValues,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        record(|observation| observation.resumes += 1);
        let result = session.with_unarmed_retention(|mut cx, empty| {
            let mut values = cx.resume_values(empty, frozen)?;
            cx.push(&mut values, outcome?)?;
            cx.finish_list(values)
        });
        FrameAction::complete(result)
    }
}

fn retained_prefix(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    assert_eq!(args.len(), 1);
    record(|observation| observation.entries += 1);
    session.with_unarmed_retention(|mut cx, mut values| {
        cx.push(&mut values, Expr::small_integer(101))?;
        cx.push(&mut values, Expr::small_integer(103))?;
        match if rum_runtime::cm740_probe::driven() {
            cx.evaluate_cm740_driven(values, args[0].clone())
        } else {
            cx.evaluate(values, args[0].clone())
        }
        .and_then(CallbackValue::adopt)
        {
            Ok((cx, values)) => cx.finish_list(values),
            Err(failure) => Ok(failure.suspend(|| PrefixFinish)),
        }
    })
}

pub(crate) fn install(session: &mut Session) {
    FAULTS.with(|probes| {
        probes
            .borrow_mut()
            .push(crate::oracle::support::faults::Check::new(session))
    });
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "C34CallbackCharge",
        "P0001 empty-prefix retention callback fixture.",
        HoldAttribute::HoldAll,
        callback_charge,
        MemoryGrowthContract::Incremental,
    ));
    session.register_builtin(BuiltinSpec::ordinary_with_hold(
        "C32RetainedPrefix",
        "P0002 retained callback prefix fixture.",
        HoldAttribute::HoldAll,
        retained_prefix,
        MemoryGrowthContract::Incremental,
    ));
}

pub(crate) fn request_session(program: &super::guest::Program) -> Session {
    let mut session = Session::new();
    rum_stdlib::install_core(&mut session);
    install(&mut session);
    session.enable_tracing(program.trace);
    session
}

pub(crate) fn request_outcome(
    output: Result<rum_runtime::SessionEvalResponse, rum_runtime::SessionRequestFailure>,
) -> crate::HostEvalOutcome {
    // Reuse the unchanged facade's complete response conversion.
    match output {
        Ok(response) => {
            crate::HostEvalOutcome::Success(crate::model::eval_success_from_runtime(response))
        }
        Err(failure) => crate::model::eval_failure_from_runtime(failure),
    }
}

pub(crate) fn start_accepted_request(
    session: &mut Session,
    program: &super::guest::Program,
    source: &str,
) -> Result<(crate::HostEvalOutcome, rum_runtime::RequestCancelHandle), super::guest::RunFailure> {
    use super::guest::RunFailure;
    if matches!(program.key.row(), "P0508" | "P1577") {
        let handle = session.request_cancel_handle();
        return Ok((request_outcome(session.eval_request(source)), handle));
    }
    let expr = if program.key.row() == "P1563" {
        Some(
            session
                .lower_request(source)
                .map_err(|_| RunFailure::Program)?
                .expr,
        )
    } else if program.key.row() == "P1562" {
        None
    } else {
        return Err(RunFailure::Program);
    };
    let accepted = session
        .accept_cancellable_request()
        .map_err(|_| RunFailure::Producer)?;
    let handle = accepted.cancel_handle();
    let output = match expr {
        Some(expr) => session.eval_accepted_expr_request(accepted, expr),
        None => session.eval_accepted_request(accepted, source),
    };
    Ok((request_outcome(output), handle))
}

pub(crate) fn run_accepted_request(
    program: &super::guest::Program,
    queued: bool,
) -> Result<super::guest::RunObservation, super::guest::RunFailure> {
    use super::guest::{self, PauseObservation, RunFailure, RunObservation};
    use crate::HostEvalOutcome;
    if program.key.context().starts_with("guest_heads.") {
        return Err(RunFailure::Program);
    }
    let lease = Session::logical_profile_test_acquire();
    let observer = ObserverScope::enter();
    let mut session = request_session(program);
    let setup = request_outcome(session.eval_request(&program.setup));
    let setup_native = observer.snapshot();
    if !matches!(setup, HostEvalOutcome::Success(_)) || setup_native.entries != 0 {
        return Err(RunFailure::Setup);
    }
    if queued {
        for (_, answer) in &program.pauses {
            session.queue_input_response(answer);
        }
    }
    rum_runtime::test_logical_profile_reset(&lease);
    rum_runtime::cm740_probe::begin(queued, "CM740-NATIVE");
    let (mut output, _handle) = start_accepted_request(&mut session, program, &program.source)?;
    let mut pauses = Vec::new();
    if !queued {
        for (ordinal, (_, answer)) in program.pauses.iter().enumerate() {
            if !matches!(output, HostEvalOutcome::InputRequired(_)) {
                return Err(RunFailure::PauseCount);
            }
            pauses.push(PauseObservation {
                ordinal,
                phase: guest::phase(program, ordinal),
                producer_route: program.producer_route.clone(),
                output: output.clone(),
                native: observer.snapshot(),
            });
            output = request_outcome(session.resume_eval_request_with_input(answer));
        }
    }
    if matches!(output, HostEvalOutcome::InputRequired(_)) {
        return Err(RunFailure::PauseCount);
    }
    // Read the two original C29 bits on the executing thread, immediately at
    // terminal return. No projection or fresh request may precede this read.
    let (active, reserved) = session.actual_accepted_request_state();
    let native = observer.snapshot();
    let mut counters = Session::logical_profile_test_rows();
    counters.push((
        "C29TerminalAcceptedRequestState".into(),
        [
            u64::from(active),
            u64::from(reserved),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
    ));
    rum_runtime::cm740_probe::end();
    Session::logical_profile_test_assert_quiescent();
    let projections = ["{log,scope}", "DownValues[f]", "UpValues[f]"]
        .map(|source| request_outcome(session.eval_request(source)))
        .to_vec();
    let pending = session.pending_input_request().cloned().map(Into::into);
    let queue_probe = request_outcome(session.eval_request("InputString[\"cm-queue\"]"));
    let queue_finish = request_outcome(session.resume_eval_request_with_input("cm-drained"));
    let fresh = request_outcome(session.eval_request("1+1"));
    observer.assert_clean();
    Ok(RunObservation {
        setup: super::guest::SetupObservation::Executed(setup),
        setup_native,
        pauses,
        output,
        projections,
        checkpoint_snapshot: None,
        native,
        counters,
        evaluation_states: Vec::new(),
        adapter: None,
        pending,
        queue_probe,
        queue_finish,
        fresh,
    })
}

pub(crate) fn run_resuming_driver(
    program: &super::guest::Program,
    queued: bool,
) -> Result<super::guest::RunObservation, super::guest::RunFailure> {
    use super::guest::{self, PauseObservation, RunFailure, RunObservation};
    use crate::HostEvalOutcome;
    if !matches!(program.key.row(), "P0508" | "P1577")
        || program.key.context().starts_with("guest_heads.")
    {
        return Err(RunFailure::Program);
    }
    let lease = Session::logical_profile_test_acquire();
    let observer = ObserverScope::enter();
    let mut session = request_session(program);
    let setup = request_outcome(session.eval_request(&program.setup));
    let setup_native = observer.snapshot();
    if !matches!(setup, HostEvalOutcome::Success(_)) || setup_native.entries != 0 {
        return Err(RunFailure::Setup);
    }
    if queued {
        for (_, answer) in &program.pauses {
            session.queue_input_response(answer);
        }
    }
    rum_runtime::test_logical_profile_reset(&lease);
    rum_runtime::cm740_probe::begin(queued, "CM740-NATIVE");
    let output = session.eval_request(&program.source);
    // Each sample is a direct executing-thread read immediately after the
    // request returns, including intermediate pauses and terminal completion.
    let mut evaluation_states = vec![session.actual_evaluation_state()];
    let mut output = request_outcome(output);
    let mut pauses = Vec::new();
    if !queued {
        for (ordinal, (_, answer)) in program.pauses.iter().enumerate() {
            if !matches!(output, HostEvalOutcome::InputRequired(_)) {
                return Err(RunFailure::PauseCount);
            }
            pauses.push(PauseObservation {
                ordinal,
                phase: guest::phase(program, ordinal),
                producer_route: program.producer_route.clone(),
                output: output.clone(),
                native: observer.snapshot(),
            });
            let resumed = session.resume_eval_request_with_input(answer);
            evaluation_states.push(session.actual_evaluation_state());
            output = request_outcome(resumed);
        }
    }
    if matches!(output, HostEvalOutcome::InputRequired(_)) {
        return Err(RunFailure::PauseCount);
    }
    let native = observer.snapshot();
    let mut counters = Session::logical_profile_test_rows();
    let mut maximum = [0; 10];
    for &(depth, stack) in &evaluation_states {
        maximum[0] = maximum[0].max(depth as u64);
        maximum[1] = maximum[1].max(stack as u64);
    }
    counters.push((
        if program.key.row() == "P0508" {
            "C28BetweenRequestsEvaluationStateMax"
        } else {
            "C29ResumingDriverEvaluationStateMax"
        }
        .into(),
        maximum,
    ));
    rum_runtime::cm740_probe::end();
    Session::logical_profile_test_assert_quiescent();
    let projections = ["{log,scope}", "DownValues[f]", "UpValues[f]"]
        .map(|source| request_outcome(session.eval_request(source)))
        .to_vec();
    let pending = session.pending_input_request().cloned().map(Into::into);
    let queue_probe = request_outcome(session.eval_request("InputString[\"cm-queue\"]"));
    let queue_finish = request_outcome(session.resume_eval_request_with_input("cm-drained"));
    let fresh = request_outcome(session.eval_request("1+1"));
    observer.assert_clean();
    Ok(RunObservation {
        setup: super::guest::SetupObservation::Executed(setup),
        setup_native,
        pauses,
        output,
        projections,
        checkpoint_snapshot: None,
        native,
        counters,
        evaluation_states,
        adapter: None,
        pending,
        queue_probe,
        queue_finish,
        fresh,
    })
}

// Imported from the sealed C31 pure next-pass driver; no production body is copied.
pub(crate) fn c8_fixedpoint_configure(s: &mut rum_runtime::Session) {
    use rum_runtime::kernel_api::{BuiltinSpec, MemoryGrowthContract};
    s.register_builtin(BuiltinSpec::ordinary(
        "C8FixedPointStep",
        "Copied pure next-pass driver",
        c8_fixedpoint_step,
        MemoryGrowthContract::Delegated,
    ));
}
fn c8_fixedpoint_step(
    _: &mut rum_runtime::Session,
    _: &rum_core::Expr,
    args: &rum_core::SharedArgs,
) -> Result<rum_core::Expr, rum_runtime::EvalError> {
    use rum_core::{Atom, Expr, Symbol};
    let sys = |name: &str| Expr::Atom(Atom::Symbol(Symbol::system(name)));
    assert!(args.len() >= 2);
    let stage = args[args.len() - 2].to_string().parse::<i64>().unwrap();
    let limit = args[args.len() - 1].to_string().parse::<i64>().unwrap();
    if stage < limit {
        return Ok(Expr::call(
            sys("C8FixedPointStep"),
            vec![
                Expr::call(
                    sys("Input"),
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
        Expr::call(sys("Sequence"), payload.to_vec())
    })
}

// Counts imported from the sealed C31 real records, never from a current run.
pub(crate) const C31_EVENT: &str =
    "C31Synchronous Runtime { phase: \"evaluate.fixed_point_equal\", source: None }";
pub(crate) fn c31_expected_count(variant: &str, context: &str) -> Option<u64> {
    match (variant, context) {
        ("AlreadyResumed", "error_before_tail.last-pause-throw") => Some(5),
        ("AlreadyResumed", "phase_observation.trace") => Some(8),
        ("AlreadyResumed", "prefix_and_tail.log") => Some(14),
        ("AlreadyResumed", "return_boundary.last-pause-return") => Some(5),
        ("AlreadyResumed", "return_boundary.loop-store-reset") => Some(9),
        ("AlreadyResumed", "value.base") => Some(8),
        ("Error", "error_before_tail.last-pause-throw") => Some(4),
        ("Error", "phase_observation.trace") => Some(4),
        ("Error", "prefix_and_tail.log") => Some(5),
        ("Error", "return_boundary.last-pause-return") => Some(4),
        ("Error", "return_boundary.loop-store-reset") => Some(4),
        ("Error", "value.base") => Some(4),
        ("Middle", "error_before_tail.last-pause-throw") => Some(4),
        ("Middle", "phase_observation.trace") => Some(6),
        ("Middle", "prefix_and_tail.log") => Some(13),
        ("Middle", "return_boundary.last-pause-return") => Some(4),
        ("Middle", "return_boundary.loop-store-reset") => Some(8),
        ("Middle", "value.base") => Some(6),
        ("Nested", "error_before_tail.last-pause-throw") => Some(4),
        ("Nested", "phase_observation.trace") => Some(7),
        ("Nested", "prefix_and_tail.log") => Some(14),
        ("Nested", "return_boundary.last-pause-return") => Some(4),
        ("Nested", "return_boundary.loop-store-reset") => Some(8),
        ("Nested", "value.base") => Some(7),
        ("Twice", "error_before_tail.last-pause-throw") => Some(6),
        ("Twice", "phase_observation.trace") => Some(8),
        ("Twice", "prefix_and_tail.log") => Some(15),
        ("Twice", "return_boundary.last-pause-return") => Some(6),
        ("Twice", "return_boundary.loop-store-reset") => Some(10),
        ("Twice", "value.base") => Some(8),
        _ => None,
    }
}
pub(crate) const CHECKPOINT_EVENT: &str = "C68MessageCheckpointState";

/// Exact selected-producer membership from the sealed disabled pause stacks.
/// Later pauses may belong to a different continuation even at the same prompt.
pub(crate) fn late_guest_pause(row: &str, prompt: &str, control_caller: bool) -> Option<bool> {
    match (row, prompt) {
        ("P0416", "KeyMap.callback:2" | "KeyMap.callback:4" | "Rule.lhsPreparation:2") => {
            Some(true)
        }
        ("P0416", "associations caller" | "rules caller" | "Rule.lhsPreparation:4") => Some(false),
        ("P0820" | "P0825", "Catch.pattern:2") => Some(true),
        ("P0820" | "P0825", "control caller" | "Catch.pattern:4") => Some(false),
        ("P1477", "Failure.propertyParameters:2") => Some(true),
        ("P1477", "control caller" | "Failure.propertyParameters:4") => Some(false),
        ("P2593", "First.lazyConsumer:2") => Some(true),
        ("P2593", "predicate caller" | "First.lazyConsumer:4") => Some(false),
        ("P0267" | "P0312" | "P0323", "c7tail") => Some(true),
        ("P0267" | "P0312" | "P0323", "c7prior" | "c7guard" | "c7tail2") => Some(false),
        ("P0410", "c10set") => Some(true),
        ("P0410", "c10setPrior" | "c10set2") => Some(false),
        (
            "P0492",
            "control caller"
            | "AbsoluteTiming.body:2"
            | "Accumulate.arithmetic:2"
            | "Accumulate.arithmetic:4"
            | "AlignColumns.explicitOption:2"
            | "AlignColumns.explicitOption:4"
            | "b724tuple",
        ) => Some(true),
        ("P0492", "debug caller" | "AbsoluteTiming.body:4" | "AbortProtect.body:4") => Some(false),
        ("P0492", "AbortProtect.body:2") => Some(!control_caller),
        _ => None,
    }
}

/// C49/C47 read this flag at the host return, before any later request.
pub(crate) fn timeout_event(row: &str) -> Option<&'static str> {
    match row {
        "P0813" => Some("C49HostOwnsEvalTimeoutAtTerminalBoundary"),
        "P2600" => Some("C47HostOwnsEvalTimeoutAtTerminalBoundary"),
        _ => None,
    }
}

/// C68's sealed 30-cell matrix per row: 20 (false,false), 10 (false,true).
/// Error variants and uncaught Throw outcomes retain the dirty bit.
pub(crate) fn checkpoint_expected(variant: &str, context: &str) -> Option<[u64; 10]> {
    if !matches!(
        variant,
        "Middle" | "Twice" | "Error" | "AlreadyResumed" | "Nested"
    ) || !matches!(
        context,
        "error_before_tail.last-pause-throw"
            | "phase_observation.trace"
            | "prefix_and_tail.log"
            | "return_boundary.last-pause-return"
            | "return_boundary.loop-store-reset"
            | "value.base"
    ) {
        return None;
    }
    Some([
        0,
        u64::from(variant == "Error" || context == "error_before_tail.last-pause-throw"),
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ])
}
