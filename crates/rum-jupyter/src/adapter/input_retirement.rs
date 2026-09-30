//! Retire an admitted, unserviceable input request without changing its reason.
use rum_host::{
    HostCancelHandle, HostEvalOutcome, HostFailureCode, HostInputSuspension, HostSession,
    HostStructuredOutput, HostStructuredOutputKind, HostTerminationReason,
};

pub(super) fn retire_input_request(
    session: &mut HostSession,
    cancel: &HostCancelHandle,
    mut suspension: HostInputSuspension,
) -> HostEvalOutcome {
    cancel.cancel();
    // The documented host route consumes cancellation of the old request before
    // evaluating new source. No answer, reset, or replay is involved.
    let (prints, mut messages, trace) = match session.eval("") {
        HostEvalOutcome::Success(response)
            if response.termination == Some(HostTerminationReason::Cancelled) =>
        {
            (response.prints, response.messages, response.trace)
        }
        HostEvalOutcome::Failure(failure) if failure.code == HostFailureCode::Cancelled => {
            (failure.prints, failure.messages, failure.trace)
        }
        outcome => panic!("host input cancellation did not retire its request: {outcome:?}"),
    };

    // Single-source eval cancellation replays its cumulative output checkpoint.
    // Only the newly appended runtime notice is redundant with InputRequired;
    // guest-authored General::cancel messages must remain visible.
    if messages.len() > suspension.messages.len()
        && messages.last().is_some_and(|message| {
            matches!(message.symbol.as_str(), "General" | "System`General")
                && message.tag == "cancel"
        })
    {
        messages.pop();
    }
    suspension
        .prints
        .extend(prints.into_iter().skip(suspension.prints.len()));
    suspension
        .messages
        .extend(messages.into_iter().skip(suspension.messages.len()));
    suspension
        .trace
        .extend(trace.into_iter().skip(suspension.trace.len()));

    // Keep the public descriptive outcome consistent with the rendered output,
    // without exposing the cancellation's synthetic result or message.
    suspension.outputs = suspension
        .prints
        .iter()
        .map(|text| HostStructuredOutput {
            kind: HostStructuredOutputKind::Print,
            text: text.clone(),
        })
        .chain(
            suspension
                .messages
                .iter()
                .map(|message| HostStructuredOutput {
                    kind: HostStructuredOutputKind::Message,
                    text: format!("{}::{}: {}", message.symbol, message.tag, message.text),
                }),
        )
        .chain(suspension.trace.iter().map(|event| HostStructuredOutput {
            kind: HostStructuredOutputKind::Trace,
            text: format!("{event:?}"),
        }))
        .collect();
    HostEvalOutcome::InputRequired(suspension)
}
