use rum_core::Expr;
use rum_runtime::{
    EvalTerminationReason, RuntimeMessage, SessionEvalResponse, SessionRequestError,
    SessionRequestFailure,
};

pub type Output = Result<SessionEvalResponse, SessionRequestFailure>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub value: Option<Expr>,
    pub error: Option<SessionRequestError>,
    pub termination: Option<EvalTerminationReason>,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
}
impl Observation {
    pub fn from_output(out: &Output) -> Self {
        match out {
            Ok(r) => Self {
                value: Some(r.value.clone()),
                error: None,
                termination: r.termination,
                messages: r.messages.clone(),
                prints: r.prints.clone(),
            },
            Err(f) => Self {
                value: None,
                error: Some(f.error.clone()),
                termination: None,
                messages: f.messages.clone(),
                prints: f.prints.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    UnexpectedPause(usize),
    PauseCount { expected: usize, actual: usize },
    Prompt(usize),
    Kind(usize),
    PendingOwner,
    OverlapAccepted,
    OutputPrefix,
    QueueNotEmpty,
    FreshRequest,
    Value,
    Error,
    Termination,
    Messages,
    Prints,
    Log,
    Projection(usize),
    ExpectedValue,
    ExpectedLog,
}

pub fn compare(a: &Observation, b: &Observation) -> Result<(), Rejection> {
    if a.value != b.value {
        return Err(Rejection::Value);
    }
    if a.error != b.error {
        return Err(Rejection::Error);
    }
    if a.termination != b.termination {
        return Err(Rejection::Termination);
    }
    if a.messages != b.messages {
        return Err(Rejection::Messages);
    }
    if a.prints != b.prints {
        return Err(Rejection::Prints);
    }
    Ok(())
}
