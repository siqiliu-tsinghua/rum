use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    thread::{self, JoinHandle},
};

#[cfg(test)]
use std::sync::mpsc::{self, Receiver, Sender};

use rum_host::{
    HostCancelHandle, HostEvalOutcome, HostEvalRequest, HostEvalSuccess, HostFailure,
    HostFailureCode, HostInputSuspension, HostSession, HostTerminationReason,
};

use crate::{KernelExecuteOutcome, KernelExecuteRequest, KernelMessageHeader};

#[derive(Debug)]
pub(super) struct ExecuteContext {
    pub(super) parent_header: KernelMessageHeader,
    pub(super) request: KernelExecuteRequest,
    pub(super) execution_count: Option<u64>,
    pub(super) cancel: HostCancelHandle,
    pub(super) routing_ids: Vec<Vec<u8>>,
    pub(super) interrupted: bool,
    pub(super) emitted: super::render::EmissionProgress,
}

impl ExecuteContext {
    pub(super) fn interrupt(&mut self) {
        self.interrupted = true;
        self.cancel.cancel();
    }
}

pub(super) enum ExecuteState {
    Idle,
    Running(ExecuteWorker),
}

pub(super) struct ExecuteWorker {
    handle: JoinHandle<(HostSession, KernelExecuteOutcome)>,
    context: ExecuteContext,
    #[cfg(test)]
    started: Option<Receiver<()>>,
    #[cfg(test)]
    finished: Option<Receiver<()>>,
}

#[cfg(test)]
pub(crate) struct ExecuteWorkerGate(Sender<()>);

#[cfg(test)]
pub(super) struct ExecuteWorkerGateWait(Receiver<()>);

#[cfg(test)]
struct ExecuteWorkerSignals {
    started: Sender<()>,
    finished: Sender<()>,
    gate: Option<ExecuteWorkerGateWait>,
}

enum ExecuteSegment {
    Eval { code: String, allow_stdin: bool },
    Resume { input: String },
    RetireInput { suspension: HostInputSuspension },
    CancelInput,
}

impl ExecuteState {
    pub(super) const fn idle() -> Self {
        Self::Idle
    }

    pub(super) fn is_running(&self) -> bool {
        matches!(self, Self::Running(_))
    }

    pub(super) fn cancel(&mut self) -> bool {
        let Self::Running(worker) = self else {
            return false;
        };
        worker.context.interrupt();
        true
    }

    pub(super) fn start_eval(
        &mut self,
        session: HostSession,
        context: ExecuteContext,
        #[cfg(test)] gate: Option<ExecuteWorkerGateWait>,
    ) {
        debug_assert!(!self.is_running());
        let segment = ExecuteSegment::Eval {
            code: context.request.code.clone(),
            allow_stdin: context.request.allow_stdin,
        };
        *self = Self::Running(ExecuteWorker::spawn(
            session,
            context,
            segment,
            #[cfg(test)]
            gate,
        ));
    }

    pub(super) fn start_resume(
        &mut self,
        session: HostSession,
        context: ExecuteContext,
        input: String,
        #[cfg(test)] gate: Option<ExecuteWorkerGateWait>,
    ) {
        debug_assert!(!self.is_running());
        *self = Self::Running(ExecuteWorker::spawn(
            session,
            context,
            ExecuteSegment::Resume { input },
            #[cfg(test)]
            gate,
        ));
    }

    pub(super) fn start_retirement(
        &mut self,
        session: HostSession,
        context: ExecuteContext,
        #[cfg(test)] gate: Option<ExecuteWorkerGateWait>,
    ) {
        debug_assert!(!self.is_running());
        *self = Self::Running(ExecuteWorker::spawn(
            session,
            context,
            ExecuteSegment::CancelInput,
            #[cfg(test)]
            gate,
        ));
    }

    #[cfg(test)]
    pub(super) fn wait_until_started(&mut self) {
        let Self::Running(worker) = self else {
            panic!("cannot wait for an execute worker that is not running");
        };
        worker.wait_until_started();
    }

    #[cfg(test)]
    pub(super) fn wait_until_finished(&mut self) {
        let Self::Running(worker) = self else {
            panic!("cannot wait for an execute worker that is not running");
        };
        worker.wait_until_finished();
    }

    pub(super) fn take_finished(&mut self) -> Option<ExecuteWorker> {
        let Self::Running(worker) = self else {
            return None;
        };
        #[cfg(not(test))]
        if !worker.handle.is_finished() {
            return None;
        }
        #[cfg(test)]
        if !worker.is_finished() {
            return None;
        }

        let Self::Running(worker) = std::mem::replace(self, Self::Idle) else {
            unreachable!("checked running state before replace");
        };
        Some(worker)
    }
}

impl ExecuteWorker {
    fn spawn(
        session: HostSession,
        context: ExecuteContext,
        segment: ExecuteSegment,
        #[cfg(test)] gate: Option<ExecuteWorkerGateWait>,
    ) -> Self {
        #[cfg(test)]
        let (signals, started, finished) = worker_signals(gate);
        Self {
            handle: spawn_worker(
                session,
                segment,
                context.cancel.clone(),
                #[cfg(test)]
                signals,
            ),
            context,
            #[cfg(test)]
            started: Some(started),
            #[cfg(test)]
            finished: Some(finished),
        }
    }

    #[cfg(test)]
    fn is_finished(&self) -> bool {
        self.finished.is_none() || self.handle.is_finished()
    }

    #[cfg(test)]
    fn wait_until_started(&mut self) {
        if let Some(started) = self.started.take() {
            started
                .recv()
                .expect("execute worker should signal that it started");
        }
    }

    #[cfg(test)]
    fn wait_until_finished(&mut self) {
        if let Some(finished) = self.finished.take() {
            finished
                .recv()
                .expect("execute worker should signal that it finished");
        }
    }

    pub(super) fn join(self) -> (ExecuteContext, HostSession, KernelExecuteOutcome) {
        let (session, outcome) = self
            .handle
            .join()
            .expect("execute worker catches eval/resume panics and returns its session");
        (self.context, session, outcome)
    }
}

#[cfg(test)]
impl ExecuteWorkerGate {
    pub(crate) fn release(self) {
        self.0
            .send(())
            .expect("gated execute worker should still be waiting for release");
    }
}

#[cfg(test)]
pub(super) fn execute_worker_gate() -> (ExecuteWorkerGate, ExecuteWorkerGateWait) {
    let (release, wait) = mpsc::channel();
    (ExecuteWorkerGate(release), ExecuteWorkerGateWait(wait))
}

#[cfg(test)]
fn worker_signals(
    gate: Option<ExecuteWorkerGateWait>,
) -> (ExecuteWorkerSignals, Receiver<()>, Receiver<()>) {
    let (started_sender, started) = mpsc::channel();
    let (finished_sender, finished) = mpsc::channel();
    (
        ExecuteWorkerSignals {
            started: started_sender,
            finished: finished_sender,
            gate,
        },
        started,
        finished,
    )
}

pub(super) fn host_outcome_to_kernel(outcome: HostEvalOutcome) -> KernelExecuteOutcome {
    match outcome {
        HostEvalOutcome::Success(response)
            if response.termination == Some(HostTerminationReason::Cancelled) =>
        {
            KernelExecuteOutcome::Failure(cancelled_failure_from_success(response))
        }
        HostEvalOutcome::Success(response) => KernelExecuteOutcome::Success(response),
        HostEvalOutcome::InputRequired(suspension) => {
            KernelExecuteOutcome::InputRequired(suspension)
        }
        HostEvalOutcome::Failure(failure) => KernelExecuteOutcome::Failure(failure),
    }
}

fn cancelled_failure_from_success(response: HostEvalSuccess) -> HostFailure {
    HostFailure {
        code: HostFailureCode::Cancelled,
        exit_status: None,
        message: "evaluation cancelled".to_string(),
        syntax_rendered: None,
        messages: response.messages,
        prints: response.prints,
        trace: response.trace,
        outputs: response.outputs,
    }
}

pub(super) fn outcome_requires_session_reset(outcome: &KernelExecuteOutcome) -> bool {
    matches!(
        outcome,
        KernelExecuteOutcome::Failure(failure) if failure.code.requires_reset()
    )
}

pub(super) fn outcome_is_cancelled(outcome: &KernelExecuteOutcome) -> bool {
    matches!(
        outcome,
        KernelExecuteOutcome::Failure(failure) if failure.code == HostFailureCode::Cancelled
    )
}

pub(super) fn force_cancelled_success(outcome: KernelExecuteOutcome) -> KernelExecuteOutcome {
    match outcome {
        KernelExecuteOutcome::Success(response) => {
            KernelExecuteOutcome::Failure(cancelled_failure_from_success(response))
        }
        other => other,
    }
}

fn spawn_worker(
    mut session: HostSession,
    segment: ExecuteSegment,
    cancel: HostCancelHandle,
    #[cfg(test)] mut signals: ExecuteWorkerSignals,
) -> JoinHandle<(HostSession, KernelExecuteOutcome)> {
    thread::spawn(move || {
        #[cfg(test)]
        let _ = signals.started.send(());

        #[cfg(test)]
        if signals
            .gate
            .take()
            .is_some_and(|gate| gate.0.recv().is_err())
        {
            let outcome = KernelExecuteOutcome::Failure(worker_panic_failure(
                &"test execute worker gate dropped before release",
            ));
            let _ = signals.finished.send(());
            return (session, outcome);
        }

        let outcome = match catch_unwind(AssertUnwindSafe(|| {
            run_segment(&mut session, segment, &cancel)
        })) {
            Ok(outcome) => outcome,
            Err(payload) => KernelExecuteOutcome::Failure(worker_panic_failure(&*payload)),
        };
        #[cfg(test)]
        let _ = signals.finished.send(());
        (session, outcome)
    })
}

fn run_segment(
    session: &mut HostSession,
    mut segment: ExecuteSegment,
    cancel: &HostCancelHandle,
) -> KernelExecuteOutcome {
    loop {
        let outcome = match segment {
            ExecuteSegment::Eval { code, allow_stdin } => {
                let outcome = session.eval(HostEvalRequest::new(code));
                if !allow_stdin {
                    if let HostEvalOutcome::InputRequired(suspension) = outcome {
                        // Continue on this worker: retirement can run guest cleanup.
                        // Nothing from the suspension has been emitted yet.
                        segment = ExecuteSegment::RetireInput { suspension };
                        continue;
                    }
                }
                outcome
            }
            ExecuteSegment::Resume { input } => session.resume_with_input(input),
            ExecuteSegment::CancelInput => {
                // Cancellation consumes the suspended request before new source.
                // Re-enter once here because retirement can run guest cleanup.
                cancel.cancel();
                let outcome = session.eval("");
                assert!(
                    matches!(&outcome,
                        HostEvalOutcome::Success(response)
                            if response.termination == Some(HostTerminationReason::Cancelled))
                        || matches!(&outcome,
                        HostEvalOutcome::Failure(failure)
                            if failure.code == HostFailureCode::Cancelled),
                    "host input cancellation did not retire its request: {outcome:?}"
                );
                outcome
            }
            ExecuteSegment::RetireInput { suspension } => {
                super::input_retirement::retire_input_request(session, cancel, suspension)
            }
        };
        return host_outcome_to_kernel(outcome);
    }
}

fn worker_panic_failure(payload: &(dyn Any + Send)) -> HostFailure {
    HostFailure {
        code: HostFailureCode::Panic,
        exit_status: None,
        message: format!(
            "jupyter execute worker caught panic: {}",
            panic_payload_message(payload)
        ),
        syntax_rendered: None,
        messages: Vec::new(),
        prints: Vec::new(),
        trace: Vec::new(),
        outputs: Vec::new(),
    }
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic".to_string()
    }
}
