#![allow(dead_code)]

use std::collections::VecDeque;
use std::io::{self, BufRead, Cursor, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use rum_host::{HostConfig, HostEvalOutcome, HostSession, HostTerminationReason};
use rum_repl::Repl;

pub fn session() -> HostSession {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_package_search_paths([fixtures]))
        .build();
    value(session.eval("before=0; after=0; cleanupCount=0"));
    session
}

pub fn run(session: &mut HostSession, input: &str) -> String {
    let mut output = Vec::new();
    Repl::new(Cursor::new(input), &mut output)
        .run(session)
        .unwrap();
    String::from_utf8(output).unwrap()
}

pub fn value(outcome: HostEvalOutcome) -> String {
    match outcome {
        HostEvalOutcome::Success(response) => {
            assert_eq!(response.termination, None, "{response:?}");
            response.value.text
        }
        other => panic!("expected success: {other:?}"),
    }
}

pub fn assert_usable(session: &mut HostSession) {
    assert!(session.pending_input_request().is_none());
    assert_eq!(value(session.eval("1+1")), "2");
}

// Prove the fixture really requests input, independently of REPL rendering.
pub fn assert_suspends(source: &str, prompt: &str, password: bool) {
    let mut session = session();
    let cancel = session.cancel_handle();
    let HostEvalOutcome::InputRequired(suspension) = session.eval(source) else {
        panic!("fixture must suspend: {source}");
    };
    assert_eq!(suspension.request.prompt, prompt);
    assert_eq!(suspension.request.password, password);
    assert_eq!(session.pending_input_request(), Some(suspension.request));
    cancel.cancel();
    let HostEvalOutcome::Success(response) = session.resume_with_input("") else {
        panic!("fixture cancellation must complete");
    };
    assert_eq!(response.termination, Some(HostTerminationReason::Cancelled));
    assert_usable(&mut session);
}

pub enum ReadStep {
    Line(String),
    Error(io::ErrorKind),
    Eof,
}

pub struct ScriptedReader {
    steps: VecDeque<ReadStep>,
}

impl ScriptedReader {
    pub fn new(steps: impl IntoIterator<Item = ReadStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }
}

impl Read for ScriptedReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("REPL must use the same line reader for answers");
    }
}

impl BufRead for ScriptedReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        panic!("REPL must use the same line reader for answers");
    }

    fn consume(&mut self, _: usize) {
        panic!("REPL must use the same line reader for answers");
    }

    fn read_line(&mut self, buffer: &mut String) -> io::Result<usize> {
        match self.steps.pop_front().unwrap_or(ReadStep::Eof) {
            ReadStep::Line(line) => {
                buffer.push_str(&line);
                Ok(line.len())
            }
            ReadStep::Error(kind) => Err(io::Error::new(kind, "fixture read error")),
            ReadStep::Eof => Ok(0),
        }
    }
}

pub struct HistoryFile(pub PathBuf);

impl HistoryFile {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/rum-repl-tests")
            .join(format!(
                "history-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "").unwrap();
        Self(path)
    }
}

impl Drop for HistoryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
