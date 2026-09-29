//! Host batch cancellation/reset and the partial-state restore contract.
use rum_host::{
    BoundedDestruction, ClosurePrintSink, HostConfig, HostEvalOutcome, HostFailureCode,
    HostSession, HostTerminationReason,
};
use rum_runtime::kernel_api::{FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

const OUTER: &str = "Global`marker=5\nAppendTo[Global`log,1]\nGet[\"Inner`\"]\n99";
const INNER: &str = "AppendTo[Global`log,2]\nInputString[\"p: \"]\n7";
struct Files(BTreeMap<String, String>);
impl BoundedDestruction for Files {}
impl FileIo for Files {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        self.0
            .get(path.file_name().unwrap().to_str().unwrap())
            .cloned()
            .ok_or_else(|| FileIoError::new("missing batch test file"))
    }
    fn probe(&self, path: &Path) -> FileProbe {
        if self
            .0
            .contains_key(path.file_name().unwrap().to_str().unwrap())
        {
            FileProbe::file(path.to_owned())
        } else {
            FileProbe::missing()
        }
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only"))
    }
}
fn session(files: &[(&str, &str)]) -> HostSession {
    HostSession::builder()
        .with_config(HostConfig {
            file_io: Some(FileIoProvider::new(Files(
                files
                    .iter()
                    .map(|(p, s)| ((*p).into(), (*s).into()))
                    .collect(),
            ))),
            ..HostConfig::default()
        })
        .build()
}
fn value(outcome: HostEvalOutcome, expected: &str) {
    match outcome {
        HostEvalOutcome::Success(s) => {
            assert_eq!(s.value.text, expected);
            assert!(s.termination.is_none());
        }
        other => panic!("expected {expected}, got {other:?}"),
    }
}
fn suspended(outcome: HostEvalOutcome, prompt: &str) {
    match outcome {
        HostEvalOutcome::InputRequired(s) => assert_eq!(s.request.prompt, prompt),
        other => panic!("expected suspension, got {other:?}"),
    }
}
fn cancelled(outcome: HostEvalOutcome) {
    match outcome {
        HostEvalOutcome::Success(s) => {
            assert_eq!(s.termination, Some(HostTerminationReason::Cancelled))
        }
        other => panic!("expected usable cancellation, got {other:?}"),
    }
}
fn no_resume(s: &mut HostSession) {
    match s.resume_with_input("unused") {
        HostEvalOutcome::Failure(f) => {
            assert_eq!(f.code, HostFailureCode::Other);
            assert!(f.message.contains("no suspended evaluation"));
        }
        other => panic!("expected no suspension, got {other:?}"),
    }
}

#[test]
fn nested_public_file_cancellation_keeps_host_usable() {
    let mut s = session(&[("Outer.rum", OUTER), ("Inner.rum", INNER)]);
    value(s.eval("Global`log={}; Global`marker=0"), "0");
    let old = s.cancel_handle();
    suspended(s.eval_file("Outer.rum"), "p: ");
    assert!(s.pending_input_request().is_some());
    no_resume(&mut s);
    old.cancel();
    cancelled(s.resume_with_input("unused"));
    assert!(s.pending_input_request().is_none());
    value(
        s.eval("{Global`log, Global`marker, $InputFileName}"),
        "List[List[], 0, \"\"]",
    );
    value(s.eval("1+1"), "2");
    suspended(s.eval("InputString[\"fresh: \"]"), "fresh: ");
    old.cancel();
    value(s.resume_with_input("fresh"), "\"fresh\"");
}

#[test]
fn reset_nested_public_file_runs_no_guest_cleanup() {
    let mut s = session(&[
        (
            "Outer.rum",
            "Global`marker=5\nPrint[\"prefix\"]\nWithCleanup[Get[\"Inner`\"],Print[\"cleanup\"]]\n99",
        ),
        ("Inner.rum", "InputString[\"p: \"]\n7"),
    ]);
    let prints = Arc::new(Mutex::new(Vec::new()));
    let sink = prints.clone();
    s.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
        sink.lock().unwrap().push(line.to_string());
    }));
    let old = s.cancel_handle();
    suspended(s.eval_file("Outer.rum"), "p: ");
    assert!(s.pending_input_request().is_some());
    s.reset();
    assert_eq!(*prints.lock().unwrap(), vec!["prefix"]);
    assert!(s.pending_input_request().is_none());
    old.cancel();
    value(s.eval("1+1"), "2");
    no_resume(&mut s);
}

fn failed_input_restore(s: &mut HostSession, source: &[u8]) {
    value(s.eval("old=9"), "9");
    let failure = s
        .restore_from_bytes(source)
        .expect_err("restore must reject input");
    assert_eq!(failure.code, HostFailureCode::Other);
    assert_eq!(
        failure.message,
        "snapshot restore unexpectedly requested input"
    );
    value(s.eval("{x, y}"), "List[1, y]");
    value(s.eval("old"), "old");
    assert!(s.pending_input_request().is_none());
    no_resume(s);
    value(s.eval("1+1"), "2");
    suspended(s.eval("InputString[\"fresh: \"]"), "fresh: ");
    value(s.resume_with_input("fresh"), "\"fresh\"");
}

#[test]
fn input_restore_failure_retires_owner_preserving_partial_state() {
    failed_input_restore(
        &mut HostSession::builder().build(),
        b"x=1\nInputString[\"p: \"]\ny=2",
    );
}

#[test]
fn nested_input_restore_failure_retires_owner_preserving_partial_state() {
    let mut s = session(&[
        ("Outer.rum", "x=2\nGet[\"Inner`\"]\nx=3"),
        ("Inner.rum", "x=4\nInputString[\"p: \"]\nx=5"),
    ]);
    failed_input_restore(&mut s, b"x=1\nGet[\"Outer`\"]\ny=2");
}

#[test]
fn throw_restore_failure_control_keeps_partial_state_usable() {
    let mut s = HostSession::builder().build();
    assert!(s.restore_from_bytes(b"x=1\nThrow[3]\ny=2").is_err());
    value(s.eval("{x,y}"), "List[1, y]");
    assert!(s.pending_input_request().is_none());
    no_resume(&mut s);
}
