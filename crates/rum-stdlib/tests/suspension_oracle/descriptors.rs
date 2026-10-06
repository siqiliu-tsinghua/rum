use super::support::{
    drive,
    observe::{self, Observation, Rejection},
    session,
};
use rum_core::Expr;
use rum_runtime::{RuntimeInputKind, Session};

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
    pub configure: fn(&mut Session),
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
            configure: |_| {},
        }
    }
    pub fn session(&self) -> Session {
        let mut s = session::new();
        (self.configure)(&mut s);
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

fn run(f: &Fixture, queued: bool, checks: Checks) -> Result<ResultView, Rejection> {
    let mut s = f.session();
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
    let observation = Observation::from_output(&r.output);
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
            for (i, expected) in f.expected_projections.iter().enumerate() {
                if v.projections.get(i).map(ToString::to_string).as_ref() != Some(expected) {
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
