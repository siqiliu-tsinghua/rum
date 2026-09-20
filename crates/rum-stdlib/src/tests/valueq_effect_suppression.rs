use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::install_core;
use crate::tests::install_test_file_io;
use rum_runtime::kernel_api::{BoundedDestruction, EntropyError, EntropyProvider, EntropySource};
use rum_runtime::{Capability, CapabilityPolicy, ClosurePrintSink, EvalError, Session};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new() -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rum-valueq-effects-{}-{timestamp}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Debug)]
struct FixedEntropy;

/// This fieldless unit struct has no state to destroy.
impl BoundedDestruction for FixedEntropy {}

impl EntropySource for FixedEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        buf.fill(0x5a);
        Ok(())
    }
}

fn core_session() -> Session {
    let mut session = Session::new();
    install_test_file_io(&mut session);
    session.set_entropy_source(Some(EntropyProvider::new(FixedEntropy)));
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn rum_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

#[test]
fn valueq_restores_prng_on_success_error_suspension_and_nesting() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[123]; {ValueQ[RandomInteger[1000000]], RandomInteger[1000000]}"
        ),
        "List[True, 488421]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[123]; {ValueQ[RandomInteger[1000000]; Abort[]], RandomInteger[1000000]}"
        ),
        "List[False, 488421]"
    );

    let suspended = session
        .eval_request(
            "SeedRandom[123]; {ValueQ[RandomInteger[1000000]; InputString[\"name: \"]], RandomInteger[1000000]}",
        )
        .unwrap();
    assert_eq!(suspended.value.to_string(), "List[True, 488421]");
    assert!(suspended.messages.is_empty(), "{:?}", suspended.messages);
    assert!(suspended.prints.is_empty(), "{:?}", suspended.prints);
    assert!(session.pending_input_request().is_none());

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[123]; {ValueQ[ValueQ[RandomInteger[1000000]]], RandomInteger[1000000]}"
        ),
        "List[True, 488421]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[123]; {BlockRandom[{ValueQ[RandomInteger[1000000]], RandomInteger[1000000]}], RandomInteger[1000000]}"
        ),
        "List[List[True, 488421], 488421]"
    );
}

#[test]
fn valueq_suppresses_an_installed_print_sink_and_restores_it() {
    let mut session = core_session();
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink_received = Arc::clone(&received);
    session.set_print_sink(Box::new(ClosurePrintSink::assert_bounded_destruction(
        move |text| {
            sink_received.lock().unwrap().push(text.to_string());
        },
    )));

    let success = session.eval_request("ValueQ[Print[\"LEAK\"]]").unwrap();
    assert_eq!(success.value.to_string(), "True");
    assert!(success.prints.is_empty(), "{:?}", success.prints);

    let error = session
        .eval_request("ValueQ[Print[\"ERROR\"]; Abort[]]")
        .unwrap();
    assert_eq!(error.value.to_string(), "False");
    assert!(error.prints.is_empty(), "{:?}", error.prints);

    let input = session
        .eval_request("ValueQ[Print[\"INPUT\"]; InputString[\"name: \"]]")
        .unwrap();
    assert_eq!(input.value.to_string(), "True");
    assert!(input.messages.is_empty(), "{:?}", input.messages);
    assert!(input.prints.is_empty(), "{:?}", input.prints);
    assert!(session.pending_input_request().is_none());

    assert_eq!(
        eval_to_string(&mut session, "ValueQ[ValueQ[Print[\"NESTED\"]]]"),
        "True"
    );
    assert_eq!(
        eval_to_string(&mut session, "BlockRandom[ValueQ[Print[\"BLOCK\"]]]"),
        "True"
    );
    assert!(received.lock().unwrap().is_empty());

    let outside = session.eval_request("Print[\"OUTSIDE\"]").unwrap();
    assert_eq!(outside.value.to_string(), "Null");
    assert!(outside.prints.is_empty(), "{:?}", outside.prints);
    assert_eq!(*received.lock().unwrap(), vec!["OUTSIDE".to_string()]);
}

#[test]
fn valueq_refuses_file_write_and_process_then_restores_the_policy() {
    let dir = TempDirGuard::new();
    let refused_file = dir.path().join("refused.rum");
    let outside_file = dir.path().join("outside.rum");
    let mut session = core_session();
    let policy = CapabilityPolicy::allow([
        Capability::Pure,
        Capability::FileWrite,
        Capability::Process,
        Capability::Interactive,
    ]);
    session.set_capability_policy(policy);
    session.eval_text("x = 42").unwrap();

    let refused = session
        .eval_request(&format!(
            "ValueQ[DumpSave[File[\"{}\"], x]]",
            rum_path(&refused_file)
        ))
        .unwrap();
    assert_eq!(refused.value.to_string(), "False");
    assert!(refused.messages.is_empty(), "{:?}", refused.messages);
    assert!(!refused_file.exists());
    assert_eq!(session.capability_policy(), policy);

    let outside = session
        .eval_request(&format!(
            "DumpSave[File[\"{}\"], x]",
            rum_path(&outside_file)
        ))
        .unwrap();
    assert_eq!(outside.value.to_string(), "List[\"Global`x\"]");
    assert!(outside.messages.is_empty(), "{:?}", outside.messages);
    assert!(outside_file.is_file());
    assert_eq!(fs::read_to_string(&outside_file).unwrap(), "Set[x, 42]\n");

    let process = session.eval_request("ValueQ[Exit[7]]").unwrap();
    assert_eq!(process.value.to_string(), "False");
    assert!(process.messages.is_empty(), "{:?}", process.messages);
    assert_eq!(session.capability_policy(), policy);

    match session
        .eval_text("Exit[9]")
        .expect_err("Process capability should be restored after ValueQ")
    {
        EvalError::Exit { code } => assert_eq!(code, 9),
        other => panic!("expected Exit[9], got {other:?}"),
    }
}

#[test]
fn valueq_allows_read_clock_diagnostic_and_random_capabilities() {
    let dir = TempDirGuard::new();
    let source = dir.path().join("read-effect.rum");
    fs::write(&source, "readEffect = 42; 7").unwrap();
    let mut session = core_session();

    session
        .eval_text(&format!(
            "readProbe[] := readSucceeded /; SameQ[Get[File[\"{}\"]], 7]",
            rum_path(&source)
        ))
        .unwrap();
    assert_eq!(eval_to_string(&mut session, "ValueQ[readProbe[]]"), "True");
    assert_eq!(eval_to_string(&mut session, "readEffect"), "readEffect");

    session
        .eval_text(
            "clockProbe[] := clockSucceeded /; NumberQ[AbsoluteTime[]]; \
             diagnosticProbe[] := diagnosticSucceeded /; IntegerQ[MemoryInUse[]]; \
             randomProbe[] := randomSucceeded /; ByteArrayQ[RandomBytes[2]]",
        )
        .unwrap();
    assert_eq!(
        eval_to_string(
            &mut session,
            "{ValueQ[clockProbe[]], ValueQ[diagnosticProbe[]], ValueQ[randomProbe[]]}"
        ),
        "List[True, True, True]"
    );
}

#[test]
fn valueq_restores_the_exact_prior_policy_on_every_outcome() {
    let mut session = core_session();
    let policy = CapabilityPolicy::allow([
        Capability::Pure,
        Capability::FileWrite,
        Capability::Process,
        Capability::Interactive,
    ]);
    session.set_capability_policy(policy);

    for (source, expected) in [
        ("ValueQ[1 + 1]", "True"),
        ("ValueQ[Abort[]]", "False"),
        ("ValueQ[InputString[\"name: \"]]", "True"),
        ("ValueQ[ValueQ[1 + 1]]", "True"),
    ] {
        let outcome = session.eval_request(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "source {source}");
        assert!(outcome.messages.is_empty(), "source {source}");
        assert!(outcome.prints.is_empty(), "source {source}");
        assert!(session.pending_input_request().is_none(), "source {source}");
        assert_eq!(session.capability_policy(), policy, "source {source}");
    }
}
