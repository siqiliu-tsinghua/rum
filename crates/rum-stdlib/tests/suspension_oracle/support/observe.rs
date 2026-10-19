use rum_core::Expr;
use rum_runtime::{
    EvalTerminationReason, RuntimeMessage, SessionEvalResponse, SessionRequestError,
    SessionRequestFailure, TraceEvent,
};

pub type Output = Result<SessionEvalResponse, SessionRequestFailure>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub value: Option<Expr>,
    pub error: Option<SessionRequestError>,
    pub termination: Option<EvalTerminationReason>,
    pub messages: Vec<RuntimeMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<TraceEvent>,
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
                trace: r.trace.clone(),
            },
            Err(f) => Self {
                value: None,
                error: Some(f.error.clone()),
                termination: None,
                messages: f.messages.clone(),
                prints: f.prints.clone(),
                trace: f.trace.clone(),
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
    MessagePrefix,
    TracePrefix,
    PauseMessages(usize),
    PausePrints(usize),
    PauseTrace(usize),
    QueueNotEmpty,
    FreshRequest,
    Value,
    Error,
    Termination,
    Messages,
    Prints,
    Trace,
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
    if a.trace != b.trace {
        return Err(Rejection::Trace);
    }
    Ok(())
}

/// Every already public element must remain an ordered prefix at the next response.
pub fn prefix(previous: &Observation, next: &Observation) -> Result<(), Rejection> {
    if !next.prints.starts_with(&previous.prints) {
        return Err(Rejection::OutputPrefix);
    }
    if !next.messages.starts_with(&previous.messages) {
        return Err(Rejection::MessagePrefix);
    }
    if !next.trace.starts_with(&previous.trace) {
        return Err(Rejection::TracePrefix);
    }
    Ok(())
}

// The frozen timing fixtures declare only duration leaves. Preserve all other
// trace data and ordering while normalizing their recorded timing results.
use rum_core::Atom;
fn named(expr: &Expr, name: &str) -> bool {
    matches!(expr, Expr::Atom(Atom::Symbol(s)) if s.context()=="System`" && s.name()==name)
}

fn rewrite(expr: &Expr, outputs: &[(Expr, Expr)], report: bool, count: &mut usize) -> Expr {
    if let Some((_, masked)) = outputs.iter().find(|(actual, _)| actual == expr) {
        *count += 1;
        return masked.clone();
    }
    let Expr::Call { head, args } = expr else {
        return expr.clone();
    };
    if report
        && named(head, "Rule")
        && args.len() == 2
        && matches!(&args[0], Expr::Atom(Atom::String(s)) if s=="Timing")
    {
        *count += 1;
        return Expr::call(
            head.as_ref().clone(),
            vec![args[0].clone(), super::normalize::duration(&args[1], &[])],
        );
    }
    Expr::call(
        rewrite(head, outputs, report, count),
        args.iter()
            .map(|a| rewrite(a, outputs, report, count))
            .collect::<Vec<_>>(),
    )
}

pub fn normalize_profile_durations(out: &mut Observation, paths: &[Vec<usize>], report: bool) {
    let mut outputs = Vec::new();
    for event in &out.trace {
        if let TraceEvent::EvaluationPass {
            before: Expr::Call { head, .. },
            after,
        } = event
        {
            if ["Timing", "AbsoluteTiming", "RepeatedTiming"]
                .iter()
                .any(|n| named(head, n))
                && !paths.is_empty()
            {
                let mut masked = after.clone();
                for path in paths {
                    masked = super::normalize::duration(&masked, path);
                }
                outputs.push((after.clone(), masked));
            }
        }
    }
    let mut count = 0;
    for event in &mut out.trace {
        match event {
            TraceEvent::EvaluationPass { before, after } => {
                *before = rewrite(before, &outputs, report, &mut count);
                *after = rewrite(after, &outputs, report, &mut count);
            }
            TraceEvent::DefinitionStored { value, .. } => {
                *value = rewrite(value, &outputs, report, &mut count)
            }
            TraceEvent::RuleApplied {
                input,
                rule,
                result,
                ..
            } => {
                *input = rewrite(input, &outputs, report, &mut count);
                *rule = rewrite(rule, &outputs, report, &mut count);
                *result = rewrite(result, &outputs, report, &mut count);
            }
        }
    }
    if count > 0 {
        eprintln!(
            "623_DURATION_MASK\tevents={}\treplacements={count}",
            out.trace.len()
        );
    }
}
