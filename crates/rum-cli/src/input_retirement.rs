//! Retire only the input request admitted by a CLI helper, retaining its reason.
use rum_host::{
    HostCancelHandle, HostEvalOutcome, HostFailureCode, HostInputSuspension, HostSession,
    HostTerminationReason,
};

pub(super) fn retire_input_request(
    session: &mut HostSession,
    cancel: &HostCancelHandle,
    suspension: &mut HostInputSuspension,
) {
    cancel.cancel();
    // The documented cancellation route retires the old request before it
    // evaluates this source. In particular, no answer is supplied to a batch.
    let (prints, mut messages) = match session.eval("") {
        HostEvalOutcome::Success(response)
            if response.termination == Some(HostTerminationReason::Cancelled) =>
        {
            (response.prints, response.messages)
        }
        HostEvalOutcome::Failure(failure) if failure.code == HostFailureCode::Cancelled => {
            (failure.prints, failure.messages)
        }
        outcome => panic!("host input cancellation did not retire its request: {outcome:?}"),
    };

    // Skip what this front end already emitted for the request, by count.
    let prior_prints = suspension.prints.len();
    let prior_messages = suspension.messages.len();
    // Keep guest diagnostics, including any guest-authored General::cancel;
    // only the final, runtime-synthesized cancellation notice is redundant.
    if messages.len() > prior_messages
        && messages.last().is_some_and(|message| {
            matches!(message.symbol.as_str(), "General" | "System`General")
                && message.tag == "cancel"
        })
    {
        messages.pop();
    }
    suspension
        .messages
        .extend(messages.into_iter().skip(prior_messages));

    // Normal evaluation helpers stream prints exclusively through their sink.
    // Setup can still buffer them; deliver only its new cleanup prints. Output
    // failure cannot prevent retirement or replace the original input reason.
    for print in prints.iter().skip(prior_prints) {
        let _ = super::write_stdout_line(print);
    }
}
