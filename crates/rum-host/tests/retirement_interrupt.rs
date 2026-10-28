//! Retirement drains consume the cancel that caused them. A further cancel of
//! the same generation, or the suspended request's own deadline, stops the
//! cleanup that is running and skips cleanups that have not started.
//! Contract: guide/embedding.md, Cancel And Timeout (retiring a suspended request).
use rum_host::{
    BoundedDestruction, HostCancelHandle, HostConfig, HostEvalOutcome, HostMessage, HostSession,
    HostTerminationReason,
};
use rum_runtime::kernel_api::{FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

struct MemFiles(BTreeMap<String, String>);
impl BoundedDestruction for MemFiles {}
impl FileIo for MemFiles {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        let key = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| FileIoError::new(format!("missing {key}")))
    }
    fn probe(&self, path: &Path) -> FileProbe {
        let key = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if self.0.contains_key(key) {
            FileProbe::file(PathBuf::from(key))
        } else {
            FileProbe::missing()
        }
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only"))
    }
}

fn session_with(files: &[(&str, &str)]) -> HostSession {
    let map = files
        .iter()
        .map(|(name, source)| ((*name).to_string(), (*source).to_string()))
        .collect();
    HostSession::builder()
        .with_config(HostConfig {
            file_io: Some(FileIoProvider::new(MemFiles(map))),
            ..HostConfig::default()
        })
        .build()
}

fn on_thread<T: Send + 'static>(
    secs: u64,
    label: &str,
    body: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (tx, rx) = mpsc::channel();
    let handle = thread::Builder::new()
        .name(label.into())
        .stack_size(16 << 20)
        .spawn(move || {
            let _ = tx.send(body());
        })
        .expect("worker thread");
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(value) => {
            handle.join().expect("worker panicked after sending");
            value
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("{label} RED hung past {secs}s");
            std::process::exit(101);
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => match handle.join() {
            Err(panic) => std::panic::resume_unwind(panic),
            Ok(()) => panic!("{label} ended without a result"),
        },
    }
}

const SKIPPED_ONE: &str =
    "Pending cleanups skipped because a host interrupt stopped the retirement: 1.";

/// What the retiring `eval("")` returned.
#[derive(Debug)]
struct Retired {
    elapsed_ms: u128,
    termination: Option<HostTerminationReason>,
    value: String,
    tags: Vec<String>,
    skipped: Vec<String>,
}

fn tags(messages: &[HostMessage]) -> Vec<String> {
    messages
        .iter()
        .map(|message| format!("{}::{}", message.symbol, message.tag))
        .collect()
}

/// Retires the pending suspension with `eval("")`, timed from `started`.
fn retire(session: &mut HostSession, started: Instant) -> Retired {
    let outcome = session.eval("");
    let elapsed_ms = started.elapsed().as_millis();
    match outcome {
        HostEvalOutcome::Success(success) => Retired {
            elapsed_ms,
            termination: success.termination,
            value: success.value.text,
            tags: tags(&success.messages),
            skipped: success
                .messages
                .iter()
                .filter(|message| message.tag == "skipped")
                .map(|message| message.text.clone())
                .collect(),
        },
        other => panic!("expected a success outcome, got {other:?}"),
    }
}

fn cancel_again_after(cancel: &HostCancelHandle, delay: Duration) {
    let again = cancel.clone();
    thread::spawn(move || {
        thread::sleep(delay);
        again.cancel();
    });
}

fn text(session: &mut HostSession, source: &str) -> String {
    match session.eval(source) {
        HostEvalOutcome::Success(success) => success.value.text,
        other => panic!("read {source} failed: {other:?}"),
    }
}

fn assert_input(outcome: HostEvalOutcome) {
    assert!(
        matches!(outcome, HostEvalOutcome::InputRequired(_)),
        "expected input, got {outcome:?}"
    );
}

fn assert_success(outcome: HostEvalOutcome) {
    assert!(
        matches!(outcome, HostEvalOutcome::Success(_)),
        "expected success, got {outcome:?}"
    );
}

#[test]
fn f1_second_cancel_interrupts_retirement_cleanup() {
    let retired = on_thread(8, "B1F1", || {
        let mut session = HostSession::new();
        let cancel = session.cancel_handle();
        assert_input(session.eval("WithCleanup[InputString[\"x\"], While[True, Null]]"));
        cancel.cancel();
        cancel_again_after(&cancel, Duration::from_secs(1));
        retire(&mut session, Instant::now())
    });
    eprintln!("B1F1 {retired:?}");
    // The retirement consumes the first cancel, so the cleanup runs until the
    // second one, 1 s later, and stops within 2 s of it.
    assert!((900..3_000).contains(&retired.elapsed_ms), "{retired:?}");
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.value, "TerminatedEvaluation[\"Cancelled\"]");
    assert_eq!(retired.tags, ["General::cancel"]);
}

#[test]
fn f2_monitor_display_returns_body_without_cancel() {
    let (value, termination, tags) = on_thread(8, "B1F2", || {
        let mut session = HostSession::new();
        match session.eval(
            "markerD = 0; Monitor[1+2, WithCleanup[InputString[\"d\"], markerD = markerD + 1]]",
        ) {
            HostEvalOutcome::Success(success) => (
                success.value.text,
                success.termination,
                tags(&success.messages),
            ),
            other => panic!("expected a success outcome, got {other:?}"),
        }
    });
    eprintln!("B1F2 value={value} termination={termination:?} tags={tags:?}");
    assert_eq!(value, "3");
    assert_eq!(termination, None);
    assert!(tags.is_empty(), "{tags:?}");
}

#[test]
fn f3_escalation_skips_later_cleanups() {
    let (retired, outer, block, file) = on_thread(8, "B1F3", || {
        let mut session = session_with(&[(
            "Inner.rum",
            "fileAssign630 = 1\nBlock[{blockVar630 = 3}, WithCleanup[WithCleanup[InputString[\"x\"], While[True, Null]], outerMarker630 = 1]]\n",
        )]);
        assert_success(session.eval("blockVar630 = 9; fileAssign630 = 0; outerMarker630 = 0"));
        let cancel = session.cancel_handle();
        assert_input(session.eval("Get[\"Inner`\"]"));
        cancel.cancel();
        cancel_again_after(&cancel, Duration::from_secs(1));
        let retired = retire(&mut session, Instant::now());
        (
            retired,
            text(&mut session, "outerMarker630"),
            text(&mut session, "blockVar630"),
            text(&mut session, "fileAssign630"),
        )
    });
    eprintln!("B1F3 {retired:?} outer={outer} block={block} file={file}");
    assert!((900..3_000).contains(&retired.elapsed_ms), "{retired:?}");
    assert_eq!(outer, "0", "the outer cleanup must not run");
    assert_eq!(block, "9", "the Block variable is restored");
    assert_eq!(file, "0", "the file's assignment is rolled back");
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.tags, ["WithCleanup::skipped", "General::cancel"]);
    assert_eq!(retired.skipped, [SKIPPED_ONE]);
}

#[test]
fn control_b_original_deadline_bounds_the_cleanup() {
    let retired = on_thread(8, "B1CTRL-b", || {
        let mut session = HostSession::new();
        let cancel = session.cancel_handle();
        let started = Instant::now();
        assert_input(session.eval_with_timeout(
            "WithCleanup[InputString[\"x\"], While[True, Null]]",
            Duration::from_millis(500),
        ));
        cancel.cancel();
        retire(&mut session, started)
    });
    eprintln!("B1CTRL b {retired:?}");
    assert!(
        (400..800).contains(&retired.elapsed_ms),
        "the original 500 ms deadline must still bound the cleanup: {retired:?}"
    );
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.tags, ["General::cancel"]);
}

#[test]
fn expired_deadline_before_retirement_skips_every_cleanup() {
    // Time spent waiting for input counts toward the timeout. When it has run
    // out before the cancel retires the suspension, the drain enters no
    // cleanup and says so.
    let (retired, marker) = on_thread(8, "B1-expired", || {
        let mut session = HostSession::new();
        assert_success(session.eval("expired630 = 0"));
        let cancel = session.cancel_handle();
        assert_input(session.eval_with_timeout(
            "WithCleanup[InputString[\"x\"], expired630 = 1]",
            Duration::from_millis(100),
        ));
        thread::sleep(Duration::from_millis(250));
        cancel.cancel();
        let retired = retire(&mut session, Instant::now());
        (retired, text(&mut session, "expired630"))
    });
    eprintln!("B1 expired-deadline {retired:?} marker={marker}");
    assert_eq!(marker, "0", "no cleanup is entered after the deadline");
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.tags, ["WithCleanup::skipped", "General::cancel"]);
    assert_eq!(retired.skipped, [SKIPPED_ONE]);
}

#[test]
fn control_d_long_finite_cleanup_runs_to_completion() {
    let (retired, k) = on_thread(30, "B1CTRL-d", || {
        let mut session = HostSession::new();
        let cancel = session.cancel_handle();
        assert_input(
            session.eval("k630 = 0; WithCleanup[InputString[\"x\"], Do[k630 = k630 + 1, {20000}]]"),
        );
        cancel.cancel();
        let retired = retire(&mut session, Instant::now());
        (retired, text(&mut session, "k630"))
    });
    eprintln!("B1CTRL d {retired:?} k630={k}");
    assert_eq!(k, "20000");
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.tags, ["General::cancel"]);
}

#[test]
fn control_nested_contract() {
    let (retired, list, pending) = on_thread(8, "B1CTRL-nested", || {
        let mut session = HostSession::new();
        assert_success(session.eval(
            "cancelCleanupInner404 = 0; cancelCleanupOuter404 = 0; \
             cancelCleanupAfterInput404 = 0; cancelBodyAfterInput404 = 0",
        ));
        let cancel = session.cancel_handle();
        assert_input(session.eval(
            "WithCleanup[WithCleanup[(InputString[\"body\"]; cancelBodyAfterInput404 = 1), \
             (cancelCleanupInner404 += 1; InputString[\"nested cleanup\"]; \
             cancelCleanupAfterInput404 = 1)], cancelCleanupOuter404 += 1]",
        ));
        cancel.cancel();
        let retired = retire(&mut session, Instant::now());
        let list = text(
            &mut session,
            "List[cancelCleanupInner404, cancelCleanupOuter404, \
             cancelCleanupAfterInput404, cancelBodyAfterInput404]",
        );
        (retired, list, session.pending_input_request().is_some())
    });
    eprintln!("B1CTRL nested {retired:?} list={list} pending={pending}");
    assert_eq!(list, "List[1, 1, 0, 0]");
    assert!(!pending);
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.tags, ["General::cancel"]);
}

#[test]
fn control_runs_once() {
    let (retired, hits, tail) = on_thread(8, "B1CTRL-once", || {
        let mut session = HostSession::new();
        assert_success(session.eval("hits630 = 0; tail630 = 0"));
        let cancel = session.cancel_handle();
        assert_input(
            session.eval("WithCleanup[InputString[\"x\"], hits630 = hits630 + 1]; tail630 = 1"),
        );
        cancel.cancel();
        let retired = retire(&mut session, Instant::now());
        (
            retired,
            text(&mut session, "hits630"),
            text(&mut session, "tail630"),
        )
    });
    eprintln!("B1CTRL runs-once {retired:?} hits={hits} tail={tail}");
    assert_eq!(hits, "1");
    assert_eq!(tail, "0");
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.tags, ["General::cancel"]);
}

#[test]
fn control_file_rollback_returned() {
    let (retired, marker, rolled) = on_thread(8, "B1CTRL-file", || {
        let mut session = session_with(&[("Inner.rum", "marker630 = 1\nInputString[\"in\"]\n")]);
        assert_success(session.eval("marker630 = 0; rolled630 = -1"));
        let cancel = session.cancel_handle();
        assert_input(session.eval("WithCleanup[Get[\"Inner`\"], rolled630 = marker630]"));
        cancel.cancel();
        let retired = retire(&mut session, Instant::now());
        (
            retired,
            text(&mut session, "marker630"),
            text(&mut session, "rolled630"),
        )
    });
    eprintln!("B1CTRL file-rollback {retired:?} marker={marker} rolled={rolled}");
    assert_eq!(marker, "0");
    assert_eq!(rolled, "0");
    assert_eq!(retired.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(retired.tags, ["General::cancel"]);
}
