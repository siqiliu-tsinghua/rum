//! Host eval/resume remains usable across guest package loading and retirement.
use rum_host::{
    BoundedDestruction, ClosurePrintSink, HostConfig, HostEvalOutcome, HostSession,
    HostTerminationReason,
};
use rum_runtime::kernel_api::{FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode};
use std::path::Path;
use std::sync::{Arc, Mutex};

struct Files {
    source: String,
    reads: Arc<Mutex<usize>>,
}
impl BoundedDestruction for Files {}
impl FileIo for Files {
    fn read_to_string(&self, _: &Path) -> Result<String, FileIoError> {
        *self.reads.lock().unwrap() += 1;
        Ok(self.source.clone())
    }
    fn probe(&self, path: &Path) -> FileProbe {
        FileProbe::file(path.to_owned())
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only"))
    }
}
fn session(source: &str) -> (HostSession, Arc<Mutex<usize>>) {
    let reads = Arc::new(Mutex::new(0));
    let config = HostConfig {
        file_io: Some(FileIoProvider::new(Files {
            source: source.into(),
            reads: reads.clone(),
        })),
        ..HostConfig::default()
    };
    (HostSession::builder().with_config(config).build(), reads)
}
const TWICE: &str = r#"Global`marker=1
Print["prefix"]
InputString["first: "]
InputString["second: "]
Twice`foo=42
77"#;
fn suspended(outcome: HostEvalOutcome, prompt: &str) {
    match outcome {
        HostEvalOutcome::InputRequired(input) => assert_eq!(input.request.prompt, prompt),
        other => panic!("expected {prompt:?} suspension, got {other:?}"),
    }
}
fn value(outcome: HostEvalOutcome, expected: &str) {
    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, expected);
            assert!(success.termination.is_none());
        }
        other => panic!("expected {expected:?}, got {other:?}"),
    }
}

#[test]
fn all_guest_entries_resume_twice_without_poisoning_or_reopening_provider() {
    for (source, expected) in [
        (r#"Get["Twice`"]"#, "77"),
        (r#"Needs["Twice`"]"#, "Null"),
        (r#"BeginPackage["Main`",{"Twice`"}]"#, r#""Main`""#),
        (r#"DeclarePackage["Twice`",{"foo"}];Twice`foo"#, "42"),
    ] {
        let (mut s, reads) = session(TWICE);
        suspended(s.eval(source), "first: ");
        assert_eq!(*reads.lock().unwrap(), 1);
        assert!(matches!(s.eval("1+1"), HostEvalOutcome::Failure(_)));
        assert!(matches!(
            s.eval_file("Unrelated.rum"),
            HostEvalOutcome::Failure(_)
        ));
        assert_eq!(
            *reads.lock().unwrap(),
            1,
            "overlap must reject before provider access"
        );
        suspended(s.resume_with_input("one"), "second: ");
        value(s.resume_with_input("two"), expected);
        assert_eq!(*reads.lock().unwrap(), 1);
        assert!(s.pending_input_request().is_none());
        value(s.eval("1+1"), "2");
    }
}

#[test]
fn reset_and_replacement_retire_either_position_without_guest_cleanup() {
    for replace in [false, true] {
        for stop in 0..2 {
            let (mut s, _) = session(TWICE);
            let prints = Arc::new(Mutex::new(Vec::new()));
            let sink = prints.clone();
            s.set_print_sink(ClosurePrintSink::assert_bounded_destruction(move |line| {
                sink.lock().unwrap().push(line.to_string());
            }));
            let old = s.cancel_handle();
            suspended(
                s.eval(r#"WithCleanup[Get["Twice`"],Print["cleanup"]]"#),
                "first: ",
            );
            if stop == 1 {
                suspended(s.resume_with_input("one"), "second: ");
            }
            if replace {
                s = session(TWICE).0;
            } else {
                s.reset();
            }
            assert_eq!(
                *prints.lock().unwrap(),
                vec!["prefix"],
                "reset/replacement must never evaluate cleanup"
            );
            assert!(s.pending_input_request().is_none());
            old.cancel();
            value(s.eval("1+1"), "2");
            suspended(s.eval(r#"InputString["fresh: "]"#), "fresh: ");
            old.cancel();
            value(s.resume_with_input("new"), r#""new""#);
        }
    }
}

#[test]
fn cancellation_at_either_position_rolls_back_and_old_handle_cannot_cancel_new_input() {
    for source in [
        r#"Get["Twice`"]"#,
        r#"Needs["Twice`"]"#,
        r#"BeginPackage["Main`",{"Twice`"}]"#,
        r#"DeclarePackage["Twice`",{"foo"}];Twice`foo"#,
    ] {
        for stop in 0..2 {
            let (mut s, _) = session(TWICE);
            value(s.eval("Global`marker=0"), "0");
            let old = s.cancel_handle();
            suspended(s.eval(source), "first: ");
            if stop == 1 {
                suspended(s.resume_with_input("one"), "second: ");
            }
            old.cancel();
            match s.resume_with_input("unused") {
                HostEvalOutcome::Success(success) => {
                    assert_eq!(success.termination, Some(HostTerminationReason::Cancelled))
                }
                other => panic!("expected cancellation, got {other:?}"),
            }
            assert!(s.pending_input_request().is_none());
            value(s.eval("Global`marker"), "0");
            value(s.eval("1+1"), "2");
            suspended(s.eval(r#"InputString["fresh: "]"#), "fresh: ");
            old.cancel();
            value(s.resume_with_input("new"), r#""new""#);
        }
    }
}

#[test]
fn error_after_resumed_load_is_a_failure_value_and_session_remains_usable() {
    let (mut s, _) = session(
        r#"Global`marker=1
InputString["error: "]
Throw[9]"#,
    );
    value(s.eval("Global`marker=0"), "0");
    suspended(s.eval(r#"Get["Bad`"]"#), "error: ");
    value(s.resume_with_input("one"), "$Failed");
    value(s.eval("Global`marker"), "0");
    value(s.eval("1+1"), "2");
}
