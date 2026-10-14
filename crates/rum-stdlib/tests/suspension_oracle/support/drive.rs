use super::observe::{self, Observation, Output, Rejection};
use super::session;
use rum_runtime::{RuntimeInputRequest, Session, SessionRequestError};

pub struct Run {
    pub output: Output,
    pub requests: Vec<RuntimeInputRequest>,
    pub pauses: Vec<Observation>,
}

// Shared by catalog descriptors and the historical suites. The final output is
// cumulative. Each intermediate output channel must remain an ordered prefix.
pub fn execute(
    s: &mut Session,
    source: &str,
    answers: &[&str],
    queued: bool,
) -> Result<Run, Rejection> {
    execute_planned(s, source, answers, queued, None)
}

pub fn execute_planned(
    s: &mut Session,
    source: &str,
    answers: &[&str],
    queued: bool,
    expected: Option<&[(rum_runtime::RuntimeInputKind, String)]>,
) -> Result<Run, Rejection> {
    let _fault_check = super::faults::Check::new(s);
    if queued {
        for answer in answers {
            s.queue_input_response(*answer);
        }
    }
    let mut output = s.eval_request(source);
    let mut requests = Vec::new();
    let mut previous = None;
    let mut pauses = Vec::new();
    loop {
        let observation = Observation::from_output(&output);
        if let Some(previous) = &previous {
            if let Err(error) = observe::prefix(previous, &observation) {
                session::retire(s);
                return Err(error);
            }
        }
        previous = Some(observation.clone());
        let Err(failure) = &output else {
            break;
        };
        let SessionRequestError::InputRequired { request } = &failure.error else {
            break;
        };
        if queued || requests.len() >= answers.len() {
            session::retire(s);
            return Err(Rejection::UnexpectedPause(requests.len()));
        }
        if let Some(expected) = expected {
            let index = requests.len();
            let Some((kind, prompt)) = expected.get(index) else {
                session::retire(s);
                return Err(Rejection::UnexpectedPause(index));
            };
            if request.kind != *kind {
                session::retire(s);
                return Err(Rejection::Kind(index));
            }
            if request.prompt != *prompt {
                session::retire(s);
                return Err(Rejection::Prompt(index));
            }
        }
        pauses.push(observation);
        let request = request.clone();
        if s.pending_input_request() != Some(&request)
            || s.reject_overlapping_evaluation_request().is_ok()
        {
            session::retire(s);
            return Err(Rejection::PendingOwner);
        }
        // Exercise the actual entrypoint, including preservation of the owner.
        if s.eval_request("Global`oracleOverlap=1").is_ok()
            || s.pending_input_request() != Some(&request)
        {
            session::retire(s);
            return Err(Rejection::OverlapAccepted);
        }
        requests.push(request);
        output = s.resume_eval_request_with_input(answers[requests.len() - 1]);
    }
    if s.pending_input_request().is_some() || s.reject_overlapping_evaluation_request().is_err() {
        session::retire(s);
        return Err(Rejection::PendingOwner);
    }
    Ok(Run {
        output,
        requests,
        pauses,
    })
}
