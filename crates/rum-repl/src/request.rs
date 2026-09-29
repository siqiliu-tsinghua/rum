//! Own one admitted host request through input waits and terminal rendering.

use std::io::{self, BufRead, Write};

use rum_host::{
    HostCancelHandle, HostEvalOutcome, HostEvalSuccess, HostFailureCode, HostInputRequest,
    HostMessage, HostSession, HostTerminationReason,
};
use rum_terminal::{render_host_failure, render_host_message};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RequestExit {
    Continue,
    Eof,
}

pub(super) enum InputEvent {
    Answer(String),
    Eof,
    Interrupted,
}

/// An answer reader must own (or borrow) the source reader's buffering.
/// A future terminal reader may advertise password entry only when it can
/// suppress all echo, including placeholders, and restore terminal state.
pub(super) trait RequestInput {
    fn password_available(&self) -> bool;
    fn read_answer(&mut self, password: bool) -> io::Result<InputEvent>;
}

pub(super) struct PlainInput<'a, R>(pub(super) &'a mut R);

impl<R: BufRead> RequestInput for PlainInput<'_, R> {
    fn password_available(&self) -> bool {
        false
    }

    fn read_answer(&mut self, password: bool) -> io::Result<InputEvent> {
        if password {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "password input unavailable",
            ));
        }
        let mut answer = String::new();
        match self.0.read_line(&mut answer) {
            Ok(0) => Ok(InputEvent::Eof),
            Ok(_) => {
                // Remove one line ending, retaining whitespace and any preceding CR.
                if answer.ends_with('\n') {
                    answer.pop();
                    if answer.ends_with('\r') {
                        answer.pop();
                    }
                } else if answer.ends_with('\r') {
                    answer.pop();
                }
                Ok(InputEvent::Answer(answer))
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => Ok(InputEvent::Interrupted),
            Err(error) => Err(error),
        }
    }
}

/// Host outputs are cumulative across suspensions, including cancellation.
#[derive(Default)]
struct Delivered {
    prints: usize,
    messages: usize,
}

impl Delivered {
    fn write(
        &mut self,
        output: &mut impl Write,
        prints: &[String],
        messages: &[HostMessage],
    ) -> io::Result<()> {
        for (index, line) in prints.iter().enumerate().skip(self.prints) {
            // A failed write may have delivered a prefix. Never replay that event.
            self.prints = index + 1;
            writeln!(output, "{line}")?;
        }
        for (index, message) in messages.iter().enumerate().skip(self.messages) {
            self.messages = index + 1;
            writeln!(output, "{}", render_host_message(message))?;
        }
        Ok(())
    }
}

pub(super) fn drive(
    session: &mut HostSession,
    input: &mut impl RequestInput,
    output: &mut impl Write,
    source: &str,
    render: impl FnOnce(&HostEvalSuccess) -> String,
) -> io::Result<RequestExit> {
    // A rejected overlap owns no cancellation handle and must preserve its caller.
    let cancel = session
        .pending_input_request()
        .is_none()
        .then(|| session.cancel_handle());
    let mut outcome = session.eval(source);
    let mut delivered = Delivered::default();

    loop {
        match outcome {
            HostEvalOutcome::Success(mut response) => {
                delivered.write(output, &response.prints, &response.messages)?;
                response.messages.clear();
                writeln!(output, "{}", render(&response))?;
                return Ok(RequestExit::Continue);
            }
            HostEvalOutcome::Failure(mut failure) => {
                delivered.write(output, &failure.prints, &failure.messages)?;
                failure.messages.clear();
                writeln!(output, "{}", render_host_failure(&failure))?;
                return Ok(RequestExit::Continue);
            }
            HostEvalOutcome::InputRequired(suspension) => {
                let cancel = cancel
                    .as_ref()
                    .expect("only an admitted evaluation can request input");
                let request = &suspension.request;
                // Keep every fallible operation inside the retirement boundary.
                let wait = (|| {
                    delivered.write(output, &suspension.prints, &suspension.messages)?;
                    if request.password && !input.password_available() {
                        return Ok(None);
                    }
                    let prompt = if request.prompt.is_empty() {
                        "input> "
                    } else {
                        &request.prompt
                    };
                    output.write_all(prompt.as_bytes())?;
                    output.flush()?;
                    input.read_answer(request.password).map(Some)
                })();

                match wait {
                    Ok(Some(InputEvent::Answer(answer))) => {
                        outcome = session.resume_with_input(answer);
                    }
                    Ok(Some(InputEvent::Eof)) => {
                        abandon(
                            session,
                            cancel,
                            output,
                            &mut delivered,
                            request,
                            Abandon::Eof,
                        )?;
                        return Ok(RequestExit::Eof);
                    }
                    Ok(Some(InputEvent::Interrupted)) => {
                        abandon(
                            session,
                            cancel,
                            output,
                            &mut delivered,
                            request,
                            Abandon::Interrupt,
                        )?;
                        return Ok(RequestExit::Continue);
                    }
                    Ok(None) => {
                        abandon(
                            session,
                            cancel,
                            output,
                            &mut delivered,
                            request,
                            Abandon::Password,
                        )?;
                        return Ok(RequestExit::Continue);
                    }
                    Err(error) => {
                        // Even an unusable writer cannot prevent host retirement.
                        // Preserve the original I/O error after best-effort reporting.
                        let _ = abandon(
                            session,
                            cancel,
                            output,
                            &mut delivered,
                            request,
                            Abandon::Io,
                        );
                        return Err(error);
                    }
                }
            }
        }
    }
}

enum Abandon {
    Eof,
    Interrupt,
    Password,
    Io,
}

fn abandon(
    session: &mut HostSession,
    cancel: &HostCancelHandle,
    output: &mut impl Write,
    delivered: &mut Delivered,
    request: &HostInputRequest,
    reason: Abandon,
) -> io::Result<()> {
    cancel.cancel();
    // Cancellation wins before this dummy value is converted or delivered.
    let outcome = session.resume_with_input("");
    let notice = match reason {
        Abandon::Eof | Abandon::Interrupt => writeln!(output, "\n! input cancelled"),
        Abandon::Password | Abandon::Io => {
            let prompt = if request.prompt.is_empty() {
                String::new()
            } else {
                format!(": {}", request.prompt)
            };
            let detail = if matches!(reason, Abandon::Password) {
                " (password input unavailable)"
            } else {
                ""
            };
            writeln!(output, "\n! input required{prompt}{detail}")
        }
    };
    // Attempt cleanup delivery even if the notice could not be written.
    let cleanup = match outcome {
        HostEvalOutcome::Success(mut response) => {
            if response.termination != Some(HostTerminationReason::Cancelled) {
                let _ = delivered.write(output, &response.prints, &response.messages);
                return Err(io::Error::other(format!(
                    "input cancellation returned unexpected termination: {:?}",
                    response.termination
                )));
            }
            omit_cancel_notice(&mut response.messages, delivered.messages);
            delivered.write(output, &response.prints, &response.messages)
        }
        HostEvalOutcome::Failure(mut failure) => {
            if failure.code != HostFailureCode::Cancelled {
                let _ = delivered.write(output, &failure.prints, &failure.messages);
                failure.messages.clear();
                let _ = writeln!(output, "{}", render_host_failure(&failure));
                return Err(io::Error::other("input cancellation failed"));
            }
            omit_cancel_notice(&mut failure.messages, delivered.messages);
            delivered.write(output, &failure.prints, &failure.messages)
        }
        HostEvalOutcome::InputRequired(_) => {
            return Err(io::Error::other(
                "input cancellation left a pending request",
            ));
        }
    };
    notice.and(cleanup)
}

fn omit_cancel_notice(messages: &mut Vec<HostMessage>, delivered: usize) {
    // The typed cancellation supplies the reason. Replace its final diagnostic
    // with our one notice (or the original input-required reason on automatic
    // abandonment), retaining earlier guest messages, even with the same tag.
    if messages.len() > delivered
        && messages.last().is_some_and(|message| {
            matches!(message.symbol.as_str(), "General" | "System`General")
                && message.tag == "cancel"
        })
    {
        messages.pop();
    }
}
