use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    thread::{self, JoinHandle},
};

#[cfg(test)]
use std::sync::mpsc::{self, Receiver, Sender};

use rum_host::{
    HostCancelHandle, HostEvalOutcome, HostEvalRequest, HostEvalSuccess, HostFailure,
    HostFailureCode, HostSession, HostTerminationReason,
};

use crate::{KernelExecuteOutcome, KernelExecuteRequest, KernelMessageHeader};

#[derive(Clone, Debug)]
pub(super) struct ExecuteContext {
    pub(super) parent_header: KernelMessageHeader,
    pub(super) request: KernelExecuteRequest,
    pub(super) execution_count: Option<u64>,
}

pub(super) enum ExecuteState {
    Idle,
    Running(ExecuteWorker),
}

pub(super) struct ExecuteWorker {
    handle: JoinHandle<(HostSession, KernelExecuteOutcome)>,
    cancel: HostCancelHandle,
    context: ExecuteContext,
    interrupted: bool,
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
    Eval { code: String },
    Resume { input: String },
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
        worker.interrupted = true;
        worker.cancel.cancel();
        true
    }

    pub(super) fn start_eval(
        &mut self,
        session: HostSession,
        context: ExecuteContext,
        #[cfg(test)] gate: Option<ExecuteWorkerGateWait>,
    ) {
        debug_assert!(!self.is_running());
        *self = Self::Running(ExecuteWorker::spawn_eval(
            session,
            context,
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
        *self = Self::Running(ExecuteWorker::spawn_resume(
            session,
            context,
            input,
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
    fn spawn_eval(
        session: HostSession,
        context: ExecuteContext,
        #[cfg(test)] gate: Option<ExecuteWorkerGateWait>,
    ) -> Self {
        let cancel = session.cancel_handle();
        let code = context.request.code.clone();
        #[cfg(test)]
        let (signals, started, finished) = worker_signals(gate);
        Self {
            handle: spawn_worker(
                session,
                ExecuteSegment::Eval { code },
                #[cfg(test)]
                signals,
            ),
            cancel,
            context,
            interrupted: false,
            #[cfg(test)]
            started: Some(started),
            #[cfg(test)]
            finished: Some(finished),
        }
    }

    fn spawn_resume(
        session: HostSession,
        context: ExecuteContext,
        input: String,
        #[cfg(test)] gate: Option<ExecuteWorkerGateWait>,
    ) -> Self {
        let cancel = session.cancel_handle();
        #[cfg(test)]
        let (signals, started, finished) = worker_signals(gate);
        Self {
            handle: spawn_worker(
                session,
                ExecuteSegment::Resume { input },
                #[cfg(test)]
                signals,
            ),
            cancel,
            context,
            interrupted: false,
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

    pub(super) fn join(self) -> (ExecuteContext, HostSession, KernelExecuteOutcome, bool) {
        let (session, outcome) = self
            .handle
            .join()
            .expect("execute worker catches eval/resume panics and returns its session");
        (self.context, session, outcome, self.interrupted)
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

        let outcome = match catch_unwind(AssertUnwindSafe(|| run_segment(&mut session, segment))) {
            Ok(outcome) => outcome,
            Err(payload) => KernelExecuteOutcome::Failure(worker_panic_failure(&*payload)),
        };
        #[cfg(test)]
        let _ = signals.finished.send(());
        (session, outcome)
    })
}

fn run_segment(session: &mut HostSession, segment: ExecuteSegment) -> KernelExecuteOutcome {
    let outcome = match segment {
        ExecuteSegment::Eval { code } => session.eval(HostEvalRequest::new(code)),
        ExecuteSegment::Resume { input } => session.resume_with_input(input),
    };
    host_outcome_to_kernel(outcome)
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
