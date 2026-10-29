// Port of the director calibration probe `.temp/director/630/probe-b3/src/lib.rs`.
// Falsifiers b3f*: red before the cleanup-error arm (the cleanup's error is dropped).
// Controls b3k*: green before and after. b3p1 records the guest-deadline probe.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use rum_host::{
    ClosurePrintSink, HostCancelHandle, HostConfig, HostEvalOutcome, HostEvalSuccess, HostFailure,
    HostMessage, HostSession, HostTerminationReason,
};
use rum_runtime::{
    FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode, kernel_api::BoundedDestruction,
};

const REC: &str = "Block[{$RecursionLimit = 20}, f630[x_] := f630[x + 1]; f630[1]]";
const EXIT_TEXT: &str =
    "Exit[3] in a cleanup was not honored because its request was being abandoned.";

fn tags_of(messages: &[HostMessage]) -> Vec<String> {
    messages
        .iter()
        .map(|m| format!("{}::{}", m.symbol, m.tag))
        .collect()
}

fn tags(s: &HostEvalSuccess) -> Vec<String> {
    tags_of(&s.messages)
}

fn describe(o: &HostEvalOutcome) -> String {
    match o {
        HostEvalOutcome::Success(s) => format!(
            "Success value={} termination={:?} messages={:?} prints={:?}",
            s.value.text,
            s.termination,
            s.messages
                .iter()
                .map(|m| format!("{}::{}:{}", m.symbol, m.tag, m.text))
                .collect::<Vec<_>>(),
            s.prints
        ),
        HostEvalOutcome::InputRequired(p) => format!(
            "InputRequired prompt={:?} messages={:?} prints={:?}",
            p.request.prompt,
            tags_of(&p.messages),
            p.prints
        ),
        HostEvalOutcome::Failure(f) => format!("Failure {}", describe_failure(f)),
    }
}

fn describe_failure(f: &HostFailure) -> String {
    format!(
        "code={:?} exit_status={:?} message={:?} messages={:?} prints={:?}",
        f.code,
        f.exit_status,
        f.message,
        f.messages
            .iter()
            .map(|m| format!("{}::{}:{}", m.symbol, m.tag, m.text))
            .collect::<Vec<_>>(),
        f.prints
    )
}

fn success(o: HostEvalOutcome) -> HostEvalSuccess {
    match o {
        HostEvalOutcome::Success(s) => s,
        other => panic!("expected success, got {}", describe(&other)),
    }
}

fn text(session: &mut HostSession, source: &str) -> String {
    success(session.eval(source)).value.text
}

/// B1's bounded worker. A hang exits 101 instead of waiting out the suite.
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

fn cancel_again_after(cancel: &HostCancelHandle, delay: Duration) {
    let again = cancel.clone();
    thread::spawn(move || {
        thread::sleep(delay);
        again.cancel();
    });
}

fn cancel_suspended(session: &mut HostSession, first: HostEvalOutcome) -> HostEvalOutcome {
    assert!(
        matches!(first, HostEvalOutcome::InputRequired(_)),
        "did not suspend: {}",
        describe(&first)
    );
    session.cancel_handle().cancel();
    session.resume_with_input("never".to_string())
}

fn cancelled(session: &mut HostSession, source: &str) -> HostEvalSuccess {
    let first = session.eval(source);
    let resumed = cancel_suspended(session, first);
    eprintln!("DIR630B3 {source:?}: {}", describe(&resumed));
    success(resumed)
}

fn nested(inner_cleanup: &str) -> String {
    format!(
        "outerRan630 = 0; WithCleanup[WithCleanup[InputString[\"body\"], {inner_cleanup}], outerRan630 = 1]"
    )
}

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

fn session_with_files(files: &[(&str, &str)]) -> HostSession {
    let map = files
        .iter()
        .map(|(name, source)| ((*name).to_string(), (*source).to_string()))
        .collect();
    let config = HostConfig {
        file_io: Some(FileIoProvider::new(MemFiles(map))),
        ..HostConfig::default()
    };
    HostSession::builder().with_config(config).build()
}

// ---- Falsifiers: red before the arm, green after ----

#[test]
fn b3f1_cancelled_cleanup_resource_limit() {
    let mut s = HostSession::new();
    let r = cancelled(&mut s, &nested(REC));
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(text(&mut s, "outerRan630"), "1");
    assert_eq!(tags(&r), ["General::reslim", "General::cancel"]);
}

#[test]
fn b3f2_cancelled_cleanup_uncaught_throw() {
    let mut s = HostSession::new();
    let r = cancelled(&mut s, &nested("Throw[630]"));
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(text(&mut s, "outerRan630"), "1");
    assert_eq!(tags(&r), ["Throw::nocatch", "General::cancel"]);
}

#[test]
fn b3f3_cancelled_cleanup_exit_is_reported_not_honored() {
    let mut s = HostSession::new();
    let r = cancelled(&mut s, &nested("Exit[3]"));
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(r.value.text, "TerminatedEvaluation[\"Cancelled\"]");
    assert_eq!(text(&mut s, "outerRan630"), "1");
    assert_eq!(tags(&r), ["Exit::abandoned", "General::cancel"]);
    assert_eq!(r.messages[0].text, EXIT_TEXT);
    assert_eq!(text(&mut s, "1 + 1"), "2", "the session keeps running");
}

#[test]
fn b3f4_cleanup_outside_a_rolled_back_file() {
    let mut s = session_with_files(&[("Inner.rum", "fileRan630 = 1\nInputString[\"body\"]\n")]);
    text(
        &mut s,
        "outer630 = 0; fileRan630 = 0; probe::outside = \"outside message\"",
    );
    let r = cancelled(
        &mut s,
        "WithCleanup[Get[\"Inner`\"], (outer630 = 1; Print[\"outside-cleanup\"]; Message[probe::outside]; Throw[630])]",
    );
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(r.prints, ["outside-cleanup"]);
    assert_eq!(text(&mut s, "outer630"), "1");
    assert_eq!(text(&mut s, "fileRan630"), "0");
    assert_eq!(
        tags(&r),
        ["probe::outside", "Throw::nocatch", "General::cancel"]
    );
}

#[test]
fn b3f5_print_sink_cancel_during_eval() {
    let mut s = HostSession::new();
    let cancel = s.cancel_handle();
    s.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        if line == "arm" {
            cancel.cancel();
        }
    }));
    let source = format!("Print[\"arm\"]; {}", nested("Throw[630]"));
    let outcome = s.eval(source.as_str());
    eprintln!("DIR630B3 print-sink: {}", describe(&outcome));
    let r = success(outcome);
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(text(&mut s, "outerRan630"), "1");
    assert_eq!(tags(&r), ["Throw::nocatch", "General::cancel"]);
}

#[test]
fn b3f6_throw_in_cleanup_inside_an_abandoned_catch() {
    let mut s = HostSession::new();
    let r = cancelled(
        &mut s,
        "Catch[WithCleanup[InputString[\"x\"], (caught630 = 1; Throw[630])]]",
    );
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(text(&mut s, "caught630"), "1");
    assert_eq!(tags(&r), ["Throw::nocatch", "General::cancel"]);
}

// ---- Guest deadline on the drain. Record both rows; pin only if it follows the rule. ----

#[test]
fn b3p1_guest_deadline_on_the_drain() {
    let mut s = HostSession::new();
    let first =
        s.eval("tc630 = 0; TimeConstrained[WithCleanup[InputString[\"x\"], tc630 = 1], 1/10]");
    assert!(
        matches!(first, HostEvalOutcome::InputRequired(_)),
        "did not suspend: {}",
        describe(&first)
    );
    std::thread::sleep(std::time::Duration::from_millis(300));
    s.cancel_handle().cancel();
    let resumed = s.resume_with_input("never".to_string());
    let tc630 = text(&mut s, "tc630");
    eprintln!("B3P1VERDICT tc630={tc630} {}", describe(&resumed));
    let r = success(resumed);
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    // The restored guest deadline is already past, so the cleanup's assignment
    // does not complete and TimeConstraintExceeded escapes with no host
    // interrupt pending.
    assert_eq!(tc630, "0");
    assert_eq!(tags(&r), ["General::timelim", "General::cancel"]);
    eprintln!("B3P1TAGS {:?}", tags(&r));
}

// ---- Controls: green before and after ----

#[test]
fn b3k1_completed_cleanup_resource_limit() {
    let mut s = HostSession::new();
    assert!(matches!(
        s.eval(nested(REC).as_str()),
        HostEvalOutcome::InputRequired(_)
    ));
    let r = success(s.resume_with_input("given".to_string()));
    assert_eq!(r.termination, Some(HostTerminationReason::StackLimit));
    assert_eq!(tags(&r), ["General::reslim"]);
    assert_eq!(text(&mut s, "outerRan630"), "1");
}

#[test]
fn b3k2_completed_cleanup_uncaught_throw() {
    let mut s = HostSession::new();
    assert!(matches!(
        s.eval(nested("Throw[630]").as_str()),
        HostEvalOutcome::InputRequired(_)
    ));
    match s.resume_with_input("given".to_string()) {
        HostEvalOutcome::Failure(f) => assert_eq!(tags_of(&f.messages), ["Throw::nocatch"]),
        other => panic!("{}", describe(&other)),
    }
    assert_eq!(text(&mut s, "outerRan630"), "1");
}

#[test]
fn b3k3_cancelled_cleanup_that_succeeds() {
    let mut s = HostSession::new();
    let r = cancelled(&mut s, &nested("Print[\"inner-cleanup\"]"));
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(tags(&r), ["General::cancel"]);
    assert_eq!(r.prints, ["inner-cleanup"]);
    assert_eq!(text(&mut s, "outerRan630"), "1");
}

#[test]
fn b3k4_completed_cleanup_exit_is_honored_without_message() {
    let mut s = HostSession::new();
    assert!(matches!(
        s.eval(nested("Exit[3]").as_str()),
        HostEvalOutcome::InputRequired(_)
    ));
    let outcome = s.resume_with_input("given".to_string());
    eprintln!("DIR630B3 completed exit: {}", describe(&outcome));
    match outcome {
        HostEvalOutcome::Failure(f) => {
            assert_eq!(f.exit_status, Some(3));
            assert!(f.messages.is_empty(), "{:?}", tags_of(&f.messages));
        }
        other => panic!("{}", describe(&other)),
    }
}

#[test]
fn b3k5_cleanup_inside_nested_get_rolls_back() {
    let mut s = session_with_files(&[(
        "Inner.rum",
        "WithCleanup[InputString[\"in\"], (Print[\"inside-cleanup\"]; Message[probe::inside]; fileRan630 = 1; Throw[630])]\n",
    )]);
    text(&mut s, "fileRan630 = 0; probe::inside = \"inside message\"");
    let r = cancelled(&mut s, "Get[\"Inner`\"]");
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(r.prints, ["inside-cleanup"]);
    assert_eq!(tags(&r), ["General::cancel"]);
    assert_eq!(text(&mut s, "fileRan630"), "0");
}

#[test]
fn b3k6_public_file_cleanup_inside_rolls_back() {
    let mut s = session_with_files(&[(
        "Outer.rum",
        "WithCleanup[InputString[\"in\"], (Print[\"pub-inside\"]; Message[probe::inside]; marker630 = 1; Throw[630])]\n",
    )]);
    text(&mut s, "marker630 = 0; probe::inside = \"inside message\"");
    let first = s.eval_file("Outer.rum");
    let resumed = cancel_suspended(&mut s, first);
    eprintln!("DIR630B3 public-inside: {}", describe(&resumed));
    let r = success(resumed);
    assert_eq!(r.termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(r.prints, ["pub-inside"]);
    assert_eq!(tags(&r), ["General::cancel"]);
    assert_eq!(text(&mut s, "marker630"), "0");
}

#[test]
fn b3k7_checkpoint_rollback_discard_stays_silent() {
    let mut s = HostSession::new();
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let slot = Arc::clone(&seen);
    s.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        slot.lock().expect("print lock").push(line.to_string());
    }));
    let r = success(
        s.eval("ValueQ[WithCleanup[InputString[\"v\"], (Print[\"valueq-cleanup\"]; Throw[630])]]"),
    );
    let prints = seen.lock().expect("print lock").clone();
    assert!(
        prints.contains(&"valueq-cleanup".to_string()),
        "control did not fire: {prints:?}"
    );
    assert_eq!(r.value.text, "True");
    assert!(tags(&r).is_empty(), "{:?}", tags(&r));
}

#[test]
fn b3k8_failed_snapshot_restore_stays_silent() {
    let mut s = HostSession::new();
    let failure = s
        .restore_from_bytes(
            b"marker630 = 0; WithCleanup[InputString[\"snap\"], (marker630 = 1; Throw[630])]",
        )
        .expect_err("snapshot input should fail the restore");
    eprintln!("DIR630B3 snapshot: {}", describe_failure(&failure));
    assert_eq!(text(&mut s, "marker630"), "1", "control did not fire");
    assert!(
        failure.messages.is_empty(),
        "{:?}",
        tags_of(&failure.messages)
    );
}

#[test]
fn b3k9_helper_best_effort_discard_stays_silent() {
    let mut s = HostSession::new();
    let r = success(s.eval(
        "markerS = 0; Monitor[1+2, 0, StepMonitor :> WithCleanup[InputString[\"s\"], (markerS = 1; Throw[630])]]",
    ));
    assert_eq!(text(&mut s, "markerS"), "1", "control did not fire");
    assert_eq!(r.value.text, "3");
    assert!(tags(&r).is_empty(), "{:?}", tags(&r));
}

/// B1's F3 without the file. The second cancel ends the inner cleanup, so the
/// outer one is skipped. `Cancelled` stays out of the drain report, which
/// keeps the skip notice ahead of `General::cancel`.
#[test]
fn b3k10_escalation_outside_a_file() {
    let (outer, message_tags, termination) = on_thread(8, "B3K10", || {
        let mut session = HostSession::new();
        let cancel = session.cancel_handle();
        let first = session.eval(
            "outerMarker630 = 0; WithCleanup[WithCleanup[InputString[\"x\"], \
             While[True, Null]], outerMarker630 = 1]",
        );
        assert!(
            matches!(first, HostEvalOutcome::InputRequired(_)),
            "{}",
            describe(&first)
        );
        cancel.cancel();
        cancel_again_after(&cancel, Duration::from_secs(1));
        let retired = success(session.eval(""));
        let message_tags = tags(&retired);
        let outer = text(&mut session, "outerMarker630");
        eprintln!(
            "B3K10VERDICT outer={outer} tags={message_tags:?} termination={:?}",
            retired.termination
        );
        (outer, message_tags, retired.termination)
    });
    assert_eq!(outer, "0", "the outer cleanup must not run");
    assert_eq!(termination, Some(HostTerminationReason::Cancelled));
    assert_eq!(message_tags, ["WithCleanup::skipped", "General::cancel"]);
}
