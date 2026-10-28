//! REPL request ownership: brief-553 U3/U4/U7 and the host cancellation contract.

mod support;

use std::io::{self, Cursor, Write};

use rum_host::HostEvalOutcome;
use rum_repl::Repl;
use support::*;

const CLEANUP: &str = "before=before+1; WithCleanup[(InputString[\"p: \"]; after=after+1), (cleanupCount=cleanupCount+1; Print[\"cleaned\"])]";

fn assert_abandoned(session: &mut rum_host::HostSession, output: &str, cancellation: bool) {
    assert!(
        output.contains("p: "),
        "must encounter input request: {output}"
    );
    assert_usable(session);
    assert_eq!(
        value(session.eval("{before,after,cleanupCount}")),
        "List[1, 0, 1]"
    );
    assert_eq!(
        output.matches("cleaned").count(),
        1,
        "cleanup output once: {output}"
    );
    assert_eq!(
        output.matches("cancelled").count(),
        usize::from(cancellation),
        "{output}"
    );
    assert!(
        !output.contains("TerminatedEvaluation"),
        "one concise notice: {output}"
    );
}

#[test]
fn red_eof_abandons_and_leaves_borrowed_session_usable() {
    assert_suspends(CLEANUP, "p: ", false);
    let mut session = session();
    let output = run(&mut session, &format!("{CLEANUP}\n"));
    assert_abandoned(&mut session, &output, true);
    assert_eq!(output.matches("rum> ").count(), 1, "EOF exits: {output}");
}

#[test]
fn red_interrupt_abandons_and_leaves_borrowed_session_usable() {
    assert_suspends(CLEANUP, "p: ", false);
    let mut session = session();
    let mut output = Vec::new();
    let input = ScriptedReader::new([
        ReadStep::Line(format!("{CLEANUP}\n")),
        ReadStep::Error(io::ErrorKind::Interrupted),
        ReadStep::Line("1+1\n".into()),
        ReadStep::Line(":quit\n".into()),
    ]);
    Repl::new(input, &mut output).run(&mut session).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert_abandoned(&mut session, &output, true);
    assert!(
        output.ends_with("rum> 2\nrum> "),
        "interrupt returns to primary prompt: {output}"
    );
}

#[test]
fn read_error_retires_request_before_returning_original_error() {
    assert_suspends(CLEANUP, "p: ", false);
    let mut session = session();
    let mut output = Vec::new();
    let input = ScriptedReader::new([
        ReadStep::Line(format!("{CLEANUP}\n")),
        ReadStep::Error(io::ErrorKind::BrokenPipe),
    ]);
    let error = Repl::new(input, &mut output).run(&mut session).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(error.to_string(), "fixture read error");
    let output = String::from_utf8(output).unwrap();
    assert_abandoned(&mut session, &output, false);
    assert!(
        output.contains("! input required: p: "),
        "original reason: {output}"
    );
}

enum WriteFailure {
    Prompt,
    Flush,
    Prefix,
}
struct FailingWriter {
    bytes: Vec<u8>,
    when: WriteFailure,
    failed: bool,
}
impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let fail = !self.failed
            && match self.when {
                WriteFailure::Prompt => bytes.windows(3).any(|part| part == b"p: "),
                WriteFailure::Prefix => bytes.windows(6).any(|part| part == b"prefix"),
                WriteFailure::Flush => false,
            };
        if fail {
            self.failed = true;
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "fixture write error",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if !self.failed
            && matches!(self.when, WriteFailure::Flush)
            && self.bytes.windows(3).any(|part| part == b"p: ")
        {
            self.failed = true;
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "fixture write error",
            ));
        }
        Ok(())
    }
}

#[test]
fn write_error_retires_request_including_prompt_flush_and_prefix() {
    let source = format!("Print[\"prefix\"]; {CLEANUP}");
    assert_suspends(&source, "p: ", false);
    for when in [
        WriteFailure::Prompt,
        WriteFailure::Flush,
        WriteFailure::Prefix,
    ] {
        let mut session = session();
        let mut writer = FailingWriter {
            bytes: Vec::new(),
            when,
            failed: false,
        };
        let error = Repl::new(Cursor::new(format!("{source}\n")), &mut writer)
            .run(&mut session)
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(error.to_string(), "fixture write error");
        assert_usable(&mut session);
        assert_eq!(
            value(session.eval("{before,after,cleanupCount}")),
            "List[1, 0, 1]"
        );
        let output = String::from_utf8(writer.bytes).unwrap();
        assert_eq!(
            output.matches("cleaned").count(),
            1,
            "working writer receives cleanup once: {output}"
        );
        assert!(
            !output.contains("cancelled"),
            "automatic abandonment: {output}"
        );
    }
}

#[test]
fn red_password_without_secure_reader_abandons() {
    let source = CLEANUP.replace(
        "InputString[\"p: \"]",
        "InputString[\"p: \", FieldMasked->True]",
    );
    assert_suspends(&source, "p: ", true);
    let history = HistoryFile::new();
    let mut session = session();
    let mut output = Vec::new();
    // This following line is source, never a fallback visible password answer.
    let input = Cursor::new(format!("{source}\n1+1\n:quit\n"));
    Repl::new(input, &mut output)
        .with_history_path(&history.0)
        .run(&mut session)
        .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert_abandoned(&mut session, &output, false);
    assert!(output.contains("password input unavailable"), "{output}");
    assert_eq!(
        output.matches("! input required: p: ").count(),
        1,
        "original reason: {output}"
    );
    assert!(
        output.ends_with("rum> 2\nrum> "),
        "must not read an answer: {output}"
    );
    assert_eq!(
        std::fs::read_to_string(&history.0).unwrap(),
        format!("{source}\n1+1\n:quit\n")
    );
}

#[test]
fn trace_abandonment_runs_cleanup_once_and_keeps_session_usable() {
    let guest = format!("TraceDetail[{CLEANUP}]");
    assert_suspends(&guest, "p: ", false);
    let mut session = session();
    let output = run(&mut session, &format!(":trace --rules --full {CLEANUP}\n"));
    assert_abandoned(&mut session, &output, true);
}

#[test]
fn rejected_overlaps_preserve_the_callers_pending_request() {
    for source in ["1+1", ":trace --rules --full 1+1"] {
        let mut session = session();
        let HostEvalOutcome::InputRequired(suspension) = session.eval(CLEANUP) else {
            panic!("fixture must suspend");
        };
        let output = run(&mut session, &format!("{source}\n:quit\n"));
        assert!(output.contains("cannot start a new evaluation"), "{output}");
        assert!(!output.contains("cancelled"), "{output}");
        assert_eq!(session.pending_input_request(), Some(suspension.request));
        value(session.resume_with_input("still owned"));
        assert_eq!(
            value(session.eval("{before,after,cleanupCount}")),
            "List[1, 1, 1]"
        );
        assert_usable(&mut session);
    }
}

#[test]
fn eof_at_a_later_prompt_cancels_the_original_request() {
    let source = CLEANUP.replace(
        "InputString[\"p: \"]",
        "answer=InputString[\"first: \"]; InputString[\"p: \"]",
    );
    assert_suspends(&source, "first: ", false);
    let mut session = session();
    let output = run(&mut session, &format!("{source}\nkept answer\n"));
    assert_abandoned(&mut session, &output, true);
    assert_eq!(value(session.eval("answer")), "\"kept answer\"");
    assert_eq!(output.matches("first: ").count(), 1, "{output}");
}

#[test]
fn cancellation_keeps_prefix_output_and_guest_cleanup_messages_once() {
    let source = format!(
        "Print[\"prefix\"]; Message[General::cancel]; {}",
        CLEANUP.replace(
            "Print[\"cleaned\"]",
            "Print[\"cleaned\"]; Message[General::cancel]"
        )
    );
    assert_suspends(&source, "p: ", false);
    let mut session = session();
    let output = run(&mut session, &format!("{source}\n"));
    assert_usable(&mut session);
    assert_eq!(
        value(session.eval("{before,after,cleanupCount}")),
        "List[1, 0, 1]"
    );
    assert_eq!(output.matches("prefix").count(), 1, "{output}");
    assert_eq!(output.matches("cleaned").count(), 1, "{output}");
    assert_eq!(output.matches("! input cancelled").count(), 1, "{output}");
    assert_eq!(
        output.matches("! General::cancel:").count(),
        2,
        "keep both guest messages: {output}"
    );
}

struct BrokenWriter {
    broken: bool,
}

impl Write for BrokenWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.broken |= bytes.windows(3).any(|part| part == b"p: ");
        if self.broken {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "permanently broken",
            ))
        } else {
            Ok(bytes.len())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn permanently_broken_output_cannot_prevent_guest_cleanup() {
    assert_suspends(CLEANUP, "p: ", false);
    let mut session = session();
    let mut writer = BrokenWriter { broken: false };
    let error = Repl::new(Cursor::new(format!("{CLEANUP}\n")), &mut writer)
        .run(&mut session)
        .unwrap_err();
    assert_eq!(error.to_string(), "permanently broken");
    assert_usable(&mut session);
    assert_eq!(
        value(session.eval("{before,after,cleanupCount}")),
        "List[1, 0, 1]"
    );
}

#[test]
fn eval_route_reports_inside_message_once_and_outer_cleanup_once() {
    let directory = std::env::temp_dir().join(format!("rum-repl-b2-inner-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let inner = directory.join("Inner.rum");
    std::fs::write(
        &inner,
        "probe::inside = \"inside message\"; Message[probe::inside]; InputString[\"in\"]\n",
    )
    .unwrap();
    let source = format!(
        "probe::outer = \"outer message\"; WithCleanup[Get[{:?}], Message[probe::outer]]",
        inner
    );
    let mut session = rum_host::HostSession::builder()
        .with_config(rum_host::HostConfig::default())
        .build();
    let mut output = Vec::new();
    let run = Repl::new(Cursor::new(format!("{source}\n")), &mut output).run(&mut session);
    let _ = std::fs::remove_dir_all(&directory);
    run.unwrap();
    let output = String::from_utf8(output).unwrap();
    assert_eq!(output.matches("inside message").count(), 1, "{output}");
    assert_eq!(output.matches("outer message").count(), 1, "{output}");
}
