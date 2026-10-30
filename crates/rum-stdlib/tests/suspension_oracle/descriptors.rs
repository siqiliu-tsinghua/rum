use super::support::{
    drive, lifecycle,
    observe::{self, Observation, Rejection},
    session,
};
use rum_core::Expr;
use rum_runtime::{
    EvalTerminationReason, RuntimeInputKind, RuntimeInputRequest, Session, SessionRequestError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Middle,
    Twice,
    Decline,
    Error,
    AlreadyResumed,
    Nested,
}
#[derive(Clone, Debug)]
pub struct Pause {
    pub prompt: String,
    pub kind: RuntimeInputKind,
    pub answer: String,
}
#[derive(Clone)]
pub struct Fixture {
    pub id: String,
    pub source: String,
    pub setup: String,
    pub pauses: Vec<Pause>,
    pub projections: Vec<String>,
    pub expected_projections: Vec<String>,
    pub expected_value: Option<String>,
    pub expected_log: Option<String>,
    pub expected_error: Option<SessionRequestError>,
    pub expected_prints: Option<Vec<String>>,
    pub expected_messages: Option<Vec<rum_runtime::RuntimeMessage>>,
    pub pause_prints: Option<Vec<Vec<String>>>,
    pub pause_messages: Option<Vec<Vec<rum_runtime::RuntimeMessage>>>,
    pub pause_trace: Option<Vec<Vec<rum_runtime::TraceEvent>>>,
    /// Zero-based argument paths of monotonic timing leaves, and nothing else.
    pub duration_paths: Vec<Vec<usize>>,
    pub files: Option<Vec<(String, String)>>,
    pub lifecycle_pauses: [usize; 2],
    pub configure: fn(&mut Session),
    /// Native collection for the logical-profiler context, after setup only.
    pub logical_profile: bool,
    pub profile_trace: bool,
}
impl Fixture {
    pub fn new(id: &str, source: &str) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            setup: String::new(),
            pauses: vec![],
            projections: vec![],
            expected_projections: vec![],
            expected_value: None,
            expected_log: None,
            expected_error: None,
            expected_prints: None,
            expected_messages: None,
            pause_prints: None,
            pause_messages: None,
            pause_trace: None,
            duration_paths: vec![],
            files: None,
            lifecycle_pauses: [0, 1],
            configure: |_| {},
            logical_profile: false,
            profile_trace: false,
        }
    }
    pub fn check_pause(&self, i: usize, observation: &Observation) -> Result<(), Rejection> {
        if self
            .pause_prints
            .as_ref()
            .is_some_and(|p| p.get(i) != Some(&observation.prints))
        {
            return Err(Rejection::PausePrints(i));
        }
        if self
            .pause_messages
            .as_ref()
            .is_some_and(|p| p.get(i) != Some(&observation.messages))
        {
            return Err(Rejection::PauseMessages(i));
        }
        if self
            .pause_trace
            .as_ref()
            .is_some_and(|p| p.get(i) != Some(&observation.trace))
        {
            return Err(Rejection::PauseTrace(i));
        }
        Ok(())
    }
    pub fn session(&self) -> Session {
        let mut s = session::new();
        (self.configure)(&mut s);
        if let Some(files) = &self.files {
            super::support::files::configure_memory(&mut s, files);
        }
        if !self.setup.is_empty() {
            s.eval_request(&self.setup).unwrap();
        }
        s
    }
}

#[derive(Debug)]
pub struct ResultView {
    pub observation: Observation,
    pub log: Expr,
    pub projections: Vec<Expr>,
    pub profile: Option<rum_runtime::LogicalProfileSnapshot>,
}

#[derive(Clone, Copy)]
pub struct Checks {
    pub transcript: bool,
    pub equality: bool,
    pub expectations: bool,
    pub queue: bool,
}
impl Checks {
    pub const ALL: Self = Self {
        transcript: true,
        equality: true,
        expectations: true,
        queue: true,
    };
}

pub fn run(f: &Fixture, queued: bool, checks: Checks) -> Result<ResultView, Rejection> {
    let profile_lease = f
        .logical_profile
        .then(rum_runtime::Session::logical_profile_test_acquire);
    let mut s = f.session();
    if let Some(lease) = profile_lease.as_ref() {
        s.set_eval_timeout(Some(std::time::Duration::from_secs(2)));
        if f.profile_trace {
            s.enable_tracing(true);
        }
        rum_runtime::test_logical_profile_reset(lease);
    }
    let answers: Vec<_> = f.pauses.iter().map(|p| p.answer.as_str()).collect();
    let expected: Vec<_> = f
        .pauses
        .iter()
        .map(|p| (p.kind, p.prompt.clone()))
        .collect();
    let r = drive::execute_planned(
        &mut s,
        &f.source,
        &answers,
        queued,
        checks.transcript.then_some(expected.as_slice()),
    )?;
    // Native read immediately after the subject, before sentinels or projections.
    let profile = f
        .logical_profile
        .then(rum_runtime::test_logical_profile_snapshot);
    if !queued && checks.transcript {
        if r.requests.len() != f.pauses.len() {
            return Err(Rejection::PauseCount {
                expected: f.pauses.len(),
                actual: r.requests.len(),
            });
        }
        for (i, (request, pause)) in r.requests.iter().zip(&f.pauses).enumerate() {
            if request.prompt != pause.prompt {
                return Err(Rejection::Prompt(i));
            }
            if request.kind != pause.kind {
                return Err(Rejection::Kind(i));
            }
        }
    }
    if !queued && checks.expectations {
        for (i, observation) in r.pauses.iter().enumerate() {
            f.check_pause(i, observation)?;
        }
    }
    let mut observation = Observation::from_output(&r.output);
    if let Some(value) = &mut observation.value {
        for path in &f.duration_paths {
            *value = super::support::normalize::duration(value, path);
        }
    }
    if f.logical_profile {
        super::support::observe::normalize_profile_durations(
            &mut observation,
            &f.duration_paths,
            f.source.contains("TestReport["),
        );
    }
    // Check before any guest-state projection could consume a spare answer.
    if checks.queue {
        session::empty_queue(&mut s)?;
    }
    let log = s.eval_request("Global`log").unwrap().value;
    let projections = f
        .projections
        .iter()
        .map(|p| s.eval_request(p).unwrap().value)
        .collect();
    session::fresh(&mut s)?;
    Ok(ResultView {
        observation,
        log,
        projections,
        profile,
    })
}

pub fn accept(f: &Fixture, checks: Checks) -> Result<ResultView, Rejection> {
    let r = run(f, false, checks)?;
    let q = run(f, true, checks)?;
    if checks.equality {
        observe::compare(&r.observation, &q.observation)?;
        if r.log != q.log {
            return Err(Rejection::Log);
        }
        for (i, (a, b)) in r.projections.iter().zip(&q.projections).enumerate() {
            if a != b {
                return Err(Rejection::Projection(i));
            }
        }
    }
    if checks.expectations {
        for v in [&r, &q] {
            if let Some(expected) = &f.expected_error {
                if v.observation.error.as_ref() != Some(expected) {
                    return Err(Rejection::Error);
                }
            }
            if let Some(expected) = &f.expected_messages {
                if &v.observation.messages != expected {
                    eprintln!(
                        "{} expected messages={expected:?} actual={:?}",
                        f.id, v.observation.messages
                    );
                    return Err(Rejection::Messages);
                }
            }
            if let Some(expected) = &f.expected_prints {
                if &v.observation.prints != expected {
                    return Err(Rejection::Prints);
                }
            }
            for (i, expected) in f.expected_projections.iter().enumerate() {
                if v.projections.get(i).map(ToString::to_string).as_ref() != Some(expected) {
                    eprintln!(
                        "{} projection {i}: expected={expected} actual={:?}; observation={:?}",
                        f.id,
                        v.projections.get(i),
                        v.observation
                    );
                    return Err(Rejection::Projection(i));
                }
            }
            if f.expected_value.as_ref().is_some_and(|e| {
                v.observation
                    .value
                    .as_ref()
                    .map(ToString::to_string)
                    .as_ref()
                    != Some(e)
            }) {
                eprintln!(
                    "expected={:?} actual={:?}",
                    f.expected_value,
                    v.observation.value.as_ref().map(ToString::to_string)
                );
                return Err(Rejection::ExpectedValue);
            }
            if f.expected_log
                .as_ref()
                .is_some_and(|e| v.log.to_string() != *e)
            {
                eprintln!(
                    "{} expected log={:?} actual={}",
                    f.id, f.expected_log, v.log
                );
                return Err(Rejection::ExpectedLog);
            }
        }
    }
    Ok(r)
}

/// Buffered lifecycle coverage checks the same exact pause observations as
/// completion; the streaming counterpart separately verifies sink exclusivity.
pub fn exercise_fixture(f: &Fixture, verify: impl Fn(&mut Session, usize, bool)) -> usize {
    let source = format!(
        "WithCleanup[({});Print[\"oracle suffix\"],Print[\"oracle cleanup\"]]",
        f.source
    );
    for (owner_pause, stop) in f.lifecycle_pauses.into_iter().enumerate() {
        for reset in [false, true] {
            let mut s = f.session();
            let cancel = s.request_cancel_handle();
            let mut out = s.eval_request(&source);
            let mut previous = None;
            for (i, pause) in f.pauses.iter().enumerate().take(stop + 1) {
                let current = Observation::from_output(&out);
                if let Some(previous) = &previous {
                    observe::prefix(previous, &current).unwrap();
                }
                f.check_pause(i, &current).unwrap();
                let Some(SessionRequestError::InputRequired { request }) = &current.error else {
                    panic!("{} lifecycle pause {i}: {out:?}", f.id);
                };
                assert_eq!(request.prompt, pause.prompt);
                assert_eq!(request.kind, pause.kind);
                assert_eq!(s.pending_input_request(), Some(request));
                assert!(s.eval_request("Global`oracleOverlap=1").is_err());
                assert_eq!(s.pending_input_request(), Some(request));
                previous = Some(current);
                if i < stop {
                    out = s.resume_eval_request_with_input(&pause.answer);
                }
            }
            if reset {
                s.queue_input_response("discarded with reset");
                s = f.session();
                cancel.cancel();
            } else {
                cancel.cancel();
                let result = s
                    .finish_cancelled_pending_evaluation_request()
                    .unwrap()
                    .unwrap();
                assert_eq!(result.termination, Some(EvalTerminationReason::Cancelled));
                let current = Observation::from_output(&Ok(result));
                let previous = previous.unwrap();
                observe::prefix(&previous, &current).unwrap();
                let mut prints = previous.prints;
                prints.push("oracle cleanup".into());
                assert_eq!(current.prints, prints, "{} cancelled buffered output", f.id);
                assert_eq!(current.messages.len(), previous.messages.len() + 1);
                assert_eq!(current.messages.last().unwrap().tag, "cancel");
                assert_eq!(current.trace, previous.trace);
            }
            assert!(s.pending_input_request().is_none());
            verify(&mut s, owner_pause, reset);
            session::fresh(&mut s).unwrap();
            session::empty_queue(&mut s).unwrap();
        }
    }
    let plans = f
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
        .collect::<Vec<_>>();
    6 + lifecycle::exercise_observed_at(
        || f.session(),
        &f.source,
        &plans,
        f.lifecycle_pauses,
        verify,
    )
}
