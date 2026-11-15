//! Separate actual cancel/reset observations at each designated producer pause.
use super::{
    guest::{self, Program, RunFailure},
    native::ObserverScope,
};
use crate::{ClosurePrintSink, HostEvalOutcome, HostInputRequest, HostTerminationReason};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LifecycleObservation {
    pub reset: bool,
    pub stop: usize,
    pub source_hash: String,
    pub pauses: Vec<HostEvalOutcome>,
    pub pending_before: Option<HostInputRequest>,
    pub overlap: HostEvalOutcome,
    pub prefix: Vec<String>,
    pub delivered: Vec<String>,
    pub finish: Option<HostEvalOutcome>,
    pub pending_after: Option<HostInputRequest>,
    pub queue_probe: HostEvalOutcome,
    pub queue_finish: HostEvalOutcome,
    pub fresh: HostEvalOutcome,
}

pub(crate) fn run(program: &Program) -> Result<Vec<LifecycleObservation>, RunFailure> {
    if matches!(program.key.row(), "P0508" | "P1562" | "P1563" | "P1577") {
        return accepted_request_lifecycle(program);
    }
    let source = format!(
        "WithCleanup[({});Print[\"cm-suffix\"],Print[\"cm-cleanup\"]]",
        program.source
    );
    let mut records = Vec::new();
    for stop in 0..program.pauses.len() {
        if !guest::credited_phase(&guest::phase(program, stop)) {
            continue;
        }
        for reset in [false, true] {
            let _lease = rum_runtime::Session::logical_profile_test_acquire();
            let observer = ObserverScope::enter();
            let mut session = guest::session(program);
            if !guest::setup_matches(
                program,
                &guest::setup_for_source(program, &mut session, &source),
            ) {
                return Err(RunFailure::Setup);
            }
            let delivered = Arc::new(Mutex::new(Vec::<String>::new()));
            let sink = delivered.clone();
            session.set_print_sink(ClosurePrintSink::assert_bounded_destruction(
                move |text: &str| sink.lock().unwrap().push(text.to_string()),
            ));
            let handle = session.cancel_handle();
            let mut output = guest::start(program, &mut session, &source);
            let mut pauses = Vec::new();
            for (index, (request, answer)) in program.pauses.iter().enumerate().take(stop + 1) {
                if !matches!(&output, HostEvalOutcome::InputRequired(p) if &p.request == request) {
                    return Err(RunFailure::Lifecycle);
                }
                pauses.push(output.clone());
                if index < stop {
                    output = guest::resume(program, &mut session, answer);
                }
            }
            let pending_before = session.pending_input_request();
            let overlap = session.eval("cm-overlap=1");
            if session.pending_input_request() != pending_before {
                return Err(RunFailure::Lifecycle);
            }
            let prefix = delivered.lock().unwrap().clone();
            let finish = if reset {
                session.queue_input_response("cm-reset-must-discard");
                session.reset();
                handle.cancel();
                None
            } else {
                handle.cancel();
                Some(guest::resume(program, &mut session, "cm-cancel-ignored"))
            };
            let after = delivered.lock().unwrap().clone();
            let pending_after = session.pending_input_request();
            let queue_probe = session.eval("InputString[\"cm-life-queue\"]");
            let queue_finish = session.resume_with_input("cm-life-drained");
            let fresh = session.eval("1+1");
            observer.assert_clean();
            records.push(LifecycleObservation {
                reset,
                stop,
                source_hash: crate::oracle::obligations::content_digest(source.as_bytes()),
                pauses,
                pending_before,
                overlap,
                prefix,
                delivered: after,
                finish,
                pending_after,
                queue_probe,
                queue_finish,
                fresh,
            });
        }
    }
    validate(program, &records)?;
    Ok(records)
}

fn accepted_request_lifecycle(program: &Program) -> Result<Vec<LifecycleObservation>, RunFailure> {
    use super::native::{request_outcome, request_session, start_accepted_request};
    let source = format!(
        "WithCleanup[({});Print[\"cm-suffix\"],Print[\"cm-cleanup\"]]",
        program.source
    );
    let mut records = Vec::new();
    for stop in 0..program.pauses.len() {
        if !guest::credited_phase(&guest::phase(program, stop)) {
            continue;
        }
        for reset in [false, true] {
            let _lease = rum_runtime::Session::logical_profile_test_acquire();
            let observer = ObserverScope::enter();
            let mut session = request_session(program);
            if !matches!(
                request_outcome(session.eval_request(&program.setup)),
                HostEvalOutcome::Success(_)
            ) {
                return Err(RunFailure::Setup);
            }
            let delivered = Arc::new(Mutex::new(Vec::<String>::new()));
            let sink = delivered.clone();
            session.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
                move |text: &str| sink.lock().unwrap().push(text.to_string()),
            )));
            let (mut output, handle) = start_accepted_request(&mut session, program, &source)?;
            let mut pauses = Vec::new();
            for (index, (request, answer)) in program.pauses.iter().enumerate().take(stop + 1) {
                if !matches!(&output, HostEvalOutcome::InputRequired(p) if &p.request == request) {
                    return Err(RunFailure::Lifecycle);
                }
                pauses.push(output.clone());
                if index < stop {
                    output = request_outcome(session.resume_eval_request_with_input(answer));
                }
            }
            let pending_before = session.pending_input_request().cloned().map(Into::into);
            let overlap = request_outcome(session.eval_request("cm-overlap=1"));
            if session.pending_input_request().cloned().map(Into::into) != pending_before {
                return Err(RunFailure::Lifecycle);
            }
            let prefix = delivered.lock().unwrap().clone();
            let finish = if reset {
                // The kernel lifecycle replaces the Session, as the existing
                // support/lifecycle.rs does. Both accepted routes above run
                // through their real runtime entrypoints before replacement.
                session.queue_input_response("cm-reset-must-discard");
                observer.assert_clean();
                session = request_session(program);
                handle.cancel();
                None
            } else {
                handle.cancel();
                Some(request_outcome(
                    session.resume_eval_request_with_input("cm-cancel-ignored"),
                ))
            };
            let after = delivered.lock().unwrap().clone();
            let pending_after = session.pending_input_request().cloned().map(Into::into);
            let queue_probe =
                request_outcome(session.eval_request("InputString[\"cm-life-queue\"]"));
            let queue_finish =
                request_outcome(session.resume_eval_request_with_input("cm-life-drained"));
            let fresh = request_outcome(session.eval_request("1+1"));
            observer.assert_clean();
            records.push(LifecycleObservation {
                reset,
                stop,
                source_hash: crate::oracle::obligations::content_digest(source.as_bytes()),
                pauses,
                pending_before,
                overlap,
                prefix,
                delivered: after,
                finish,
                pending_after,
                queue_probe,
                queue_finish,
                fresh,
            });
        }
    }
    validate(program, &records)?;
    Ok(records)
}

pub(crate) fn validate(
    program: &Program,
    records: &[LifecycleObservation],
) -> Result<(), RunFailure> {
    let expected: Vec<_> = (0..program.pauses.len())
        .filter(|&stop| guest::credited_phase(&guest::phase(program, stop)))
        .flat_map(|stop| [false, true].map(|reset| (stop, reset)))
        .collect();
    if records
        .iter()
        .map(|r| (r.stop, r.reset))
        .collect::<Vec<_>>()
        != expected
    {
        return Err(RunFailure::Lifecycle);
    }
    for record in records {
        if record.pauses.len() != record.stop + 1
            || record.pending_before.as_ref() != Some(&program.pauses[record.stop].0)
            || !matches!(&record.overlap, HostEvalOutcome::Failure(f) if f.code == crate::HostFailureCode::Other)
            || record
                .prefix
                .iter()
                .any(|p| p == "cm-suffix" || p == "cm-cleanup")
            || !record.delivered.starts_with(&record.prefix)
            || record.delivered.iter().any(|p| p == "cm-suffix")
            || record.pending_after.is_some()
        {
            return Err(RunFailure::Lifecycle);
        }
        if record.reset {
            if record.finish.is_some() || record.delivered != record.prefix {
                return Err(RunFailure::Lifecycle);
            }
        } else if record
            .delivered
            .iter()
            .filter(|p| p.as_str() == "cm-cleanup")
            .count()
            != 1
            || !(if guest::adapter_program(program) {
                matches!(&record.finish, Some(HostEvalOutcome::Failure(f)) if f.code == crate::HostFailureCode::Cancelled)
            } else {
                matches!(&record.finish, Some(HostEvalOutcome::Success(s)) if s.termination == Some(HostTerminationReason::Cancelled))
            })
        {
            return Err(RunFailure::Lifecycle);
        }
        if !matches!(&record.queue_probe, HostEvalOutcome::InputRequired(p) if p.request.prompt == "cm-life-queue")
            || !matches!(&record.queue_finish, HostEvalOutcome::Success(s) if s.value.text == "\"cm-life-drained\"")
            || !matches!(&record.fresh, HostEvalOutcome::Success(s) if s.value.text == "2")
        {
            return Err(RunFailure::Lifecycle);
        }
    }
    Ok(())
}
