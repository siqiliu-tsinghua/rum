//! Retire only the input request admitted by a CLI helper, retaining its reason.
use rum_host::{
    HostCancelHandle, HostEvalOutcome, HostFailureCode, HostInputSuspension, HostSession,
    HostTerminationReason,
};

/// The host's single-source checkpoint is replayed on cancellation; batch
/// cancellation starts its output checkpoint at abandonment instead.
pub(super) enum CancellationOutput {
    Cumulative,
    CleanupOnly,
}

pub(super) fn retire_input_request(
    session: &mut HostSession,
    cancel: &HostCancelHandle,
    suspension: &mut HostInputSuspension,
    output: CancellationOutput,
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

    let (prior_prints, prior_messages) = match output {
        CancellationOutput::Cumulative => (suspension.prints.len(), suspension.messages.len()),
        CancellationOutput::CleanupOnly => (0, 0),
    };
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
