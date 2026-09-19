use rum_host::{
    Capability, CapabilityPolicy, HostCompletenessStatus, HostConfig, HostEvalOutcome,
    HostEvalRequest, HostEvalSuccess, HostFailureCode, HostIsCompleteRequest, HostSession,
};
use rum_runtime::kernel_api::{
    BoundedDestruction, EntropyError, EntropyProvider, EntropySource, FileIo, FileIoError,
    FileIoProvider, FileProbe, FileWriteMode,
};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

fn successful_eval(outcome: HostEvalOutcome) -> HostEvalSuccess {
    match outcome {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected success, got {other:?}"),
    }
}

fn assert_success_text(outcome: HostEvalOutcome, expected: &str) -> HostEvalSuccess {
    let success = successful_eval(outcome);
    assert_eq!(success.value.text, expected);
    success
}

#[derive(Debug)]
struct InMemoryFileIo {
    files: BTreeMap<String, String>,
    reads: Arc<Mutex<Vec<String>>>,
}

impl InMemoryFileIo {
    fn new(files: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        Self {
            files: files
                .into_iter()
                .map(|(name, source)| (name.to_string(), source.to_string()))
                .collect(),
            reads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn provider(self: &Arc<Self>) -> FileIoProvider {
        FileIoProvider::from_arc(self.clone())
    }

    fn key(path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
    }

    fn read_log(&self) -> Vec<String> {
        self.reads.lock().unwrap().clone()
    }
}

/// The string map and shared string-vector log drop iteratively, including at the last Arc.
impl BoundedDestruction for InMemoryFileIo {}

impl FileIo for InMemoryFileIo {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        let key = Self::key(path).unwrap_or_else(|| path.display().to_string());
        self.reads.lock().unwrap().push(key.clone());
        self.files
            .get(&key)
            .cloned()
            .ok_or_else(|| FileIoError::new(format!("missing {key}")))
    }

    fn probe(&self, path: &Path) -> FileProbe {
        let Some(key) = Self::key(path) else {
            return FileProbe::missing();
        };

        if self.files.contains_key(&key) {
            FileProbe::file(path.to_path_buf())
        } else {
            FileProbe::missing()
        }
    }

    fn write_string(
        &self,
        _path: &Path,
        _contents: &str,
        _mode: FileWriteMode,
    ) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only in-memory provider"))
    }
}

#[derive(Debug)]
struct FixedEntropy {
    pattern: Vec<u8>,
    fills: AtomicUsize,
}

impl FixedEntropy {
    fn new(pattern: &[u8]) -> Arc<Self> {
        Arc::new(Self {
            pattern: pattern.to_vec(),
            fills: AtomicUsize::new(0),
        })
    }

    fn provider(self: &Arc<Self>) -> EntropyProvider {
        EntropyProvider::from_arc(self.clone())
    }

    fn fills(&self) -> usize {
        self.fills.load(Ordering::Relaxed)
    }
}

/// The flat byte vector and atomic counter have bounded destruction, even at the last Arc.
impl BoundedDestruction for FixedEntropy {}

impl EntropySource for FixedEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        self.fills.fetch_add(1, Ordering::Relaxed);
        for (index, byte) in buf.iter_mut().enumerate() {
            *byte = self.pattern[index % self.pattern.len()];
        }
        Ok(())
    }
}

#[test]
fn restore_from_snapshot_source_that_requests_input_returns_synthesized_failure() {
    let mut session = HostSession::new();

    let failure = session
        .restore_from_bytes(b"x = InputString[\"name: \"]")
        .expect_err("snapshot restore should reject input-requiring source");

    assert_eq!(failure.code, HostFailureCode::Other);
    assert_eq!(
        failure.message,
        "snapshot restore unexpectedly requested input"
    );
    assert!(failure.messages.is_empty());
    assert!(failure.prints.is_empty());
}

#[test]
fn host_config_uses_injected_file_io_for_get_and_needs() {
    let file_io = Arc::new(InMemoryFileIo::new([
        ("direct.rum", "directValue = 31\ndirectValue"),
        (
            "Injected.rum",
            "BeginPackage[\"Injected`\"]\nvalue = 41\nEndPackage[]",
        ),
    ]));
    let config = HostConfig::default()
        .with_file_io(Some(file_io.provider()))
        .with_package_search_paths(["virtual-root"]);
    let mut session = HostSession::builder().with_config(config).build();

    assert_success_text(session.eval("Get[\"direct.rum\"]"), "31");
    assert_success_text(
        session.eval("Needs[\"Injected`\", \"Injected.rum\"]"),
        "Null",
    );
    assert_success_text(session.eval("Injected`value"), "41");

    let reads = file_io.read_log();
    assert!(reads.iter().any(|path| path == "direct.rum"), "{reads:?}");
    assert!(reads.iter().any(|path| path == "Injected.rum"), "{reads:?}");
}

#[test]
fn host_config_uses_fixed_entropy_for_reseed_random() {
    let entropy = FixedEntropy::new(&[0, 1, 2, 3, 4, 5, 6, 7]);
    let config = HostConfig::default().with_entropy_source(Some(entropy.provider()));
    let mut first = HostSession::builder().with_config(config.clone()).build();
    let mut second = HostSession::builder().with_config(config).build();

    let source =
        "ReseedRandom[]; {RandomInteger[1000000], RandomInteger[1000000], RandomInteger[1000000]}";
    let first_value = successful_eval(first.eval(source)).value.text;
    let second_value = successful_eval(second.eval(source)).value.text;

    assert_eq!(first_value, second_value);
    assert_ne!(first_value, "List[0, 0, 0]");
    assert_eq!(entropy.fills(), 2);
}

#[test]
fn host_config_tracing_and_assert_flags_flow_into_built_sessions() {
    let mut traced = HostSession::builder()
        .with_config(HostConfig::default().with_tracing(true))
        .build();

    let traced_success = successful_eval(traced.eval("f[x_Integer] := x; f[3]"));
    assert_eq!(traced_success.value.text, "3");
    assert!(
        traced_success
            .trace
            .iter()
            .any(|event| matches!(event, rum_host::HostTraceEvent::RuleApplied { .. }))
    );
    assert!(traced.configuration().tracing_enabled);

    let asserts = HostSession::builder()
        .with_config(HostConfig::default().with_asserts_enabled(true))
        .build();
    assert!(asserts.configuration().asserts_enabled);
}

#[test]
fn host_session_builder_policy_and_core_builtin_controls_are_applied() {
    let policy = CapabilityPolicy::allow([Capability::Pure]);
    let mut denied = HostSession::builder()
        .with_capability_policy(policy)
        .build();

    let file = successful_eval(denied.eval("Get[\"direct.rum\"]"));
    assert_eq!(file.value.text, "$Failed");
    assert!(
        file.messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox")
    );

    let random = successful_eval(denied.eval("ReseedRandom[]"));
    assert_eq!(random.value.text, "$Failed");
    assert!(
        random
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox")
    );

    assert_success_text(denied.eval("SeedRandom[42]; RandomInteger[100]"), "39");

    let mut disabled = HostSession::builder().with_core_builtins(false).build();
    assert!(!disabled.configuration().install_core_builtins);
    assert_success_text(disabled.eval("1 + 2"), "Plus[1, 2]");
}

#[test]
fn file_and_entropy_providers_from_arc_share_identity_and_state() {
    let file_io = Arc::new(InMemoryFileIo::new([("state.rum", "state = 5\nstate")]));
    let file_provider = file_io.provider();
    let same_file_provider = FileIoProvider::from_arc(file_io.clone());
    let other_file_provider = FileIoProvider::new(InMemoryFileIo::new([("state.rum", "0")]));

    assert_eq!(file_provider, same_file_provider);
    assert_ne!(file_provider, other_file_provider);
    assert_eq!(
        same_file_provider
            .as_file_io()
            .read_to_string(Path::new("state.rum"))
            .unwrap(),
        "state = 5\nstate"
    );
    assert_eq!(file_io.read_log(), vec!["state.rum".to_string()]);

    let entropy = FixedEntropy::new(&[9, 8, 7, 6]);
    let entropy_provider = entropy.provider();
    let same_entropy_provider = EntropyProvider::from_arc(entropy.clone());
    let other_entropy_provider = EntropyProvider::new(FixedEntropy {
        pattern: vec![1],
        fills: AtomicUsize::new(0),
    });

    assert_eq!(entropy_provider, same_entropy_provider);
    assert_ne!(entropy_provider, other_entropy_provider);

    let mut bytes = [0_u8; 6];
    same_entropy_provider
        .as_entropy_source()
        .fill(&mut bytes)
        .unwrap();
    assert_eq!(bytes, [9, 8, 7, 6, 9, 8]);
    assert_eq!(entropy.fills(), 1);
}

#[test]
fn host_request_string_adapters_and_defaults_cover_public_facade() {
    let eval_request: HostEvalRequest = String::from("2 + 2").into();
    let complete_request: HostIsCompleteRequest = String::from("f[1]").into();
    let mut session = HostSession::default();

    assert_success_text(session.eval(eval_request), "4");
    assert_eq!(
        session.is_complete(complete_request).status,
        HostCompletenessStatus::Complete
    );

    let builder_session = rum_host::HostSessionBuilder::default().build();
    assert!(builder_session.configuration().install_core_builtins);
}
