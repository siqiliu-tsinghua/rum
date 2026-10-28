//! Non-interactive helpers own retirement, even when their caller keeps the session.
//! Contract: getting-started.md (script input), embedding.md (batch cancellation).
use super::*;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicU64;

static FILE_COUNTER: AtomicU64 = AtomicU64::new(0);
const INPUT: &str = "InputString[\"p: \"]";
const CHILD_CASE: &str = "RUM_CLI_RETIREMENT_TEST";

struct File(PathBuf);

impl File {
    fn new(source: &str) -> Self {
        let path = env::temp_dir().join(format!(
            "rum-cli-retirement-{}-{}.rum",
            std::process::id(),
            FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, source).unwrap();
        Self(path)
    }

    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn value(session: &mut HostSession, source: &str, expected: &str) {
    match session.eval(source) {
        HostEvalOutcome::Success(response) => {
            assert_eq!(response.value.text, expected);
            assert!(response.termination.is_none());
        }
        other => panic!("expected {expected}, got {other:?}"),
    }
}

fn reusable(session: &mut HostSession) {
    value(session, "1+1", "2");
    assert!(session.pending_input_request().is_none());
}

fn invoke(
    kind: &str,
    session: &mut HostSession,
    source: &str,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match kind {
        "eval" => eval_source(session, source),
        "script" => eval_script_source(session, source),
        "file" => eval_file(session, File::new(source).path()),
        "setup" => eval_setup_binding(session, source).map(|()| ExitCode::SUCCESS),
        "request" => eval_request(session, "evaluation failed", |session| session.eval(source)),
        _ => panic!("unknown helper {kind}"),
    }
}

fn input_error(result: Result<ExitCode, Box<dyn std::error::Error>>, kind: &str, prompt: &str) {
    let expected = if kind == "setup" {
        format!("failed to bind script variable: ! input required: {prompt}")
    } else {
        format!("input required: {prompt}")
    };
    assert_eq!(result.unwrap_err().to_string(), expected);
}

fn helper_abandons(kind: &str) {
    let mut session = build_host_session();
    value(&mut session, "kept=17", "17");
    input_error(invoke(kind, &mut session, INPUT), kind, "p: ");
    reusable(&mut session);
    value(&mut session, "kept", "17");
}

#[test]
fn eval_helper_abandons() {
    helper_abandons("eval");
}

#[test]
fn script_helper_abandons() {
    helper_abandons("script");
}

#[test]
fn file_helper_abandons() {
    helper_abandons("file");
}

#[test]
fn unexpected_setup_input_abandons() {
    helper_abandons("setup");
}

#[test]
fn shared_request_helper_abandons() {
    helper_abandons("request");
}

fn child(test: &str, case: &str, closed_stdout: bool) -> std::process::Output {
    let mut child = Command::new(env::current_exe().unwrap())
        .args([
            "--exact",
            &format!("eval_tests::{test}"),
            "--nocapture",
            "--quiet",
        ])
        .env_remove("RUST_TEST_THREADS")
        .env(CHILD_CASE, case)
        .stdin(if closed_stdout {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if closed_stdout {
        use std::io::BufRead;
        let mut ready = String::new();
        io::BufReader::new(child.stderr.as_mut().unwrap())
            .read_line(&mut ready)
            .unwrap();
        assert_eq!(ready, "ready\n");
        drop(child.stdout.take().unwrap());
        child.stdin.take().unwrap().write_all(b"x").unwrap();
    }
    child.wait_with_output().unwrap()
}

fn wait_for_closed_stdout() {
    eprintln!("ready");
    io::stdin().read_exact(&mut [0]).unwrap();
}

fn assert_child_passed(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "child failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cleanup_output_appears_once_and_tail_never_runs() {
    if let Ok(kind) = env::var(CHILD_CASE) {
        let mut session = build_host_session();
        value(&mut session, "cleanupCount=0; tail=0", "0");
        let prefix = if kind == "setup" {
            ""
        } else {
            "Print[\"prefix\"]; "
        };
        let source = format!(
            "{prefix}WithCleanup[InputString[\"p: \"], cleanupCount=cleanupCount+1; Print[\"cleanup\"]]; tail=1; Print[\"tail\"]"
        );
        input_error(invoke(&kind, &mut session, &source), &kind, "p: ");
        reusable(&mut session);
        let count = if kind == "file" { "0" } else { "1" };
        value(&mut session, "cleanupCount", count);
        value(&mut session, "tail", "0");
        return;
    }
    for kind in ["eval", "script", "file", "setup"] {
        let output = child(
            "cleanup_output_appears_once_and_tail_never_runs",
            kind,
            false,
        );
        assert_child_passed(&output);
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            stdout.lines().filter(|line| *line == "cleanup").count(),
            1,
            "{kind}: {stdout}"
        );
        assert_eq!(
            stdout.lines().filter(|line| *line == "prefix").count(),
            usize::from(kind != "setup"),
            "{kind}: {stdout}"
        );
        assert!(!stdout.lines().any(|line| line == "tail"), "{stdout}");
        assert!(!stdout.contains("TerminatedEvaluation"), "{stdout}");
        assert!(!stderr.contains("cancel"), "{stderr}");
    }
}

#[test]
fn public_file_nested_get_rolls_back_inner_to_outer() {
    if env::var(CHILD_CASE).as_deref() == Ok("nested") {
        let inner = File::new(
            "marker=3\nWithCleanup[InputString[\"nested: \"], Print[\"inner:\",marker]]\ntail=1",
        );
        let outer = File::new(&format!(
            "marker=2\nWithCleanup[Get[{:?}], Print[\"outer:\",marker]]\ntail=2",
            inner.path()
        ));
        let mut session = build_host_session();
        value(&mut session, "marker=1; tail=0", "0");
        input_error(eval_file(&mut session, outer.path()), "file", "nested: ");
        reusable(&mut session);
        value(
            &mut session,
            "{marker,tail,$InputFileName}",
            "List[1, 0, \"\"]",
        );
        return;
    }
    let output = child(
        "public_file_nested_get_rolls_back_inner_to_outer",
        "nested",
        false,
    );
    assert_child_passed(&output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let cleanup: Vec<_> = stdout
        .lines()
        .filter(|line| line.starts_with("inner:") || line.starts_with("outer:"))
        .collect();
    assert_eq!(cleanup, ["inner:3", "outer:2"]);
}

#[test]
fn script_keeps_earlier_forms_after_nested_get_retirement() {
    let inner = File::new("before=9\nInputString[\"nested: \"]\ntail=1");
    let mut session = build_host_session();
    value(&mut session, "before=1; cleanupCount=0; tail=0", "0");
    let source = format!(
        "before=7\nWithCleanup[Get[{:?}], cleanupCount=cleanupCount+1]\ntail=2",
        inner.path()
    );
    input_error(
        eval_script_source(&mut session, &source),
        "script",
        "nested: ",
    );
    reusable(&mut session);
    value(
        &mut session,
        "{before,cleanupCount,tail,$InputFileName}",
        "List[7, 1, 0, \"\"]",
    );
}

#[test]
fn refused_admission_preserves_someone_elses_pending_request() {
    for kind in ["eval", "script", "file", "setup", "request"] {
        let mut session = build_host_session();
        assert!(matches!(
            session.eval("answer=InputString[\"original: \"]"),
            HostEvalOutcome::InputRequired(_)
        ));
        let pending = session.pending_input_request().unwrap();
        let error = invoke(kind, &mut session, "sideEffect=1").unwrap_err();
        assert!(
            error.to_string().contains("cannot start a new evaluation"),
            "{kind}: {error}"
        );
        assert_eq!(session.pending_input_request(), Some(pending));
        match session.resume_with_input("preserved") {
            HostEvalOutcome::Success(response) => {
                assert_eq!(response.value.text, "\"preserved\"");
                assert!(response.termination.is_none());
            }
            other => panic!("{kind}: pending owner was changed: {other:?}"),
        }
        reusable(&mut session);
        value(
            &mut session,
            "{answer,sideEffect}",
            "List[\"preserved\", sideEffect]",
        );
    }
}

#[test]
fn bindings_consume_exactly_their_queued_responses() {
    let mut session = build_host_session();
    bind_script_input(&mut session, "answer\n".into()).unwrap();
    bind_script_args_if_present(
        &mut session,
        Some(vec!["arg".into(), "InputString[]".into()]),
    )
    .unwrap();
    value(
        &mut session,
        "{$ScriptInput,$ScriptArgs}",
        "List[\"answer\\n\", List[\"arg\", \"InputString[]\"]]",
    );
    // A later guest request must still suspend: bindings are data, not answers.
    assert!(matches!(
        session.eval(INPUT),
        HostEvalOutcome::InputRequired(_)
    ));
    assert_eq!(session.pending_input_request().unwrap().prompt, "p: ");
    match session.resume_with_input("real answer") {
        HostEvalOutcome::Success(response) => assert_eq!(response.value.text, "\"real answer\""),
        other => panic!("{other:?}"),
    }
    reusable(&mut session);
}

#[test]
fn terminal_stdin_does_not_make_scripts_interactive() {
    let file = File::new(INPUT);
    for command in [
        CommandMode::Eval(INPUT.into()),
        CommandMode::File(file.path().into()),
    ] {
        let error = run_non_interactive(command, Some(vec!["unused".into()]), true).unwrap_err();
        assert_eq!(error.to_string(), "input required: p: ");
    }
}

#[test]
fn broken_pipe_during_cleanup_still_retires_input() {
    if env::var(CHILD_CASE).as_deref() == Ok("broken_cleanup") {
        wait_for_closed_stdout();
        let mut session = build_host_session();
        value(&mut session, "cleanupCount=0; tail=0", "0");
        let result = eval_source(
            &mut session,
            "WithCleanup[InputString[\"p: \"], cleanupCount=cleanupCount+1; Print[\"cleanup\"]]; tail=1",
        );
        // Check reuse before exit policy, so the red proves a retained session.
        reusable(&mut session);
        value(&mut session, "{cleanupCount,tail}", "List[1, 0]");
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
        // libtest itself writes to stdout; exit only after the helper assertions.
        std::process::exit(0);
    }
    let output = child(
        "broken_pipe_during_cleanup_still_retires_input",
        "broken_cleanup",
        true,
    );
    assert_child_passed(&output);
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn already_broken_pipe_returns_with_reusable_session() {
    if env::var(CHILD_CASE).as_deref() == Ok("broken_prefix") {
        wait_for_closed_stdout();
        let mut session = build_host_session();
        let result = eval_script_source(
            &mut session,
            "Print[\"prefix\"]\nInputString[\"p: \"]\nPrint[\"tail\"]",
        );
        reusable(&mut session);
        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
        std::process::exit(0);
    }
    let output = child(
        "already_broken_pipe_returns_with_reusable_session",
        "broken_prefix",
        true,
    );
    assert_child_passed(&output);
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cleanup_messages_appear_once_with_original_input_reason() {
    if let Ok(kind) = env::var(CHILD_CASE) {
        let mut session = build_host_session();
        let result = invoke(
            &kind,
            &mut session,
            "notice::before=\"before-message\"; notice::cleanup=\"cleanup-message\"; Message[notice::before]; WithCleanup[InputString[\"p: \"], Message[notice::cleanup]; Print[\"cleanup\"]]",
        );
        let error = result.unwrap_err();
        assert!(error.to_string().ends_with("input required: p: "));
        // Setup returns its rendered diagnostics in the error; main prints it.
        if kind == "setup" {
            eprintln!("{error}");
        }
        reusable(&mut session);
        return;
    }
    // Check the nontransactional script first; file rollback discards only its
    // cleanup message, while cleanup prints remain observable in both cases.
    for kind in ["script", "file", "eval", "setup"] {
        let output = child(
            "cleanup_messages_appear_once_with_original_input_reason",
            kind,
            false,
        );
        assert_child_passed(&output);
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            stdout.lines().filter(|line| *line == "cleanup").count(),
            1,
            "{kind}: {stdout}"
        );
        assert_eq!(
            stderr.matches("before-message").count(),
            1,
            "{kind}: {stderr}"
        );
        assert_eq!(
            stderr.matches("cleanup-message").count(),
            usize::from(kind != "file"),
            "{kind}: {stderr}"
        );
        assert_eq!(
            stderr.matches("input required: p: ").count(),
            1,
            "{kind}: {stderr}"
        );
        assert!(!stderr.contains("cancel"), "{kind}: {stderr}");
    }
}

#[test]
fn retirement_does_not_consume_a_queued_future_answer() {
    let mut session = build_host_session();
    let result = eval_request(&mut session, "evaluation failed", |session| {
        let outcome = session.eval_script(INPUT);
        assert!(matches!(outcome, HostEvalOutcome::InputRequired(_)));
        session.queue_input_response("future");
        outcome
    });
    input_error(result, "request", "p: ");
    reusable(&mut session);
    value(&mut session, "InputString[]", "\"future\"");
}

/// W1 through the CLI helpers. Script, eval, and setup keep the nested file's
/// already-reported inside message and the outer cleanup message. The file
/// route's rollback discards the outer cleanup, as in
/// `cleanup_messages_appear_once_with_original_input_reason`.
#[test]
fn cancelled_nested_get_reports_inside_once_and_outer_once() {
    if let Ok(kind) = env::var(CHILD_CASE) {
        let mut session = build_host_session();
        let inner = File::new(
            "probe::inside = \"inside message\"; Message[probe::inside]; InputString[\"in\"]",
        );
        let source = format!(
            "probe::outer = \"outer message\"; WithCleanup[Get[{:?}], Message[probe::outer]]",
            inner.path()
        );
        let error = invoke(&kind, &mut session, &source).unwrap_err();
        assert!(
            error.to_string().contains("input required: in"),
            "{kind}: {error}"
        );
        if kind == "setup" {
            eprintln!("{error}");
        }
        reusable(&mut session);
        return;
    }
    let mut failures = Vec::new();
    for kind in ["script", "eval", "setup", "file"] {
        let output = child(
            "cancelled_nested_get_reports_inside_once_and_outer_once",
            kind,
            false,
        );
        let stderr = String::from_utf8(output.stderr).unwrap();
        let inside = stderr.matches("inside message").count();
        let outer = stderr.matches("outer message").count();
        let reason = stderr.matches("input required: in").count();
        let cancel = stderr.contains("cancel");
        eprintln!(
            "CLIVERDICT kind={kind} status={} inside={inside} outer={outer} reason={reason} cancel={cancel}",
            output.status
        );
        let expect_outer = usize::from(kind != "file");
        if !output.status.success() || inside != 1 || outer != expect_outer || reason != 1 || cancel
        {
            failures.push(format!(
                "{kind}: status={} inside={inside} outer={outer} (expect {expect_outer}) reason={reason} cancel={cancel}\n{stderr}",
                output.status
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}
