//! Output-window falsifiers for brief 630 stage B2.
//!
//! W1 and W2 assert the cumulative contract. They are red at HEAD.
//! W3 asserts that a rolled-back, unreported item stays out. It is green at HEAD.
use rum_host::{
    BoundedDestruction, HostConfig, HostEvalOutcome, HostFailureCode, HostMessage, HostSession,
    HostTraceEvent,
};
use rum_runtime::kernel_api::{FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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

fn session_with(files: &[(&str, &str)], tracing: bool) -> HostSession {
    let map = files
        .iter()
        .map(|(name, source)| ((*name).to_string(), (*source).to_string()))
        .collect();
    HostSession::builder()
        .with_config(HostConfig {
            file_io: Some(FileIoProvider::new(MemFiles(map))),
            tracing_enabled: tracing,
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

fn tags(messages: &[HostMessage]) -> Vec<String> {
    messages
        .iter()
        .map(|message| format!("{}::{}", message.symbol, message.tag))
        .collect()
}

fn outcome_messages(outcome: &HostEvalOutcome) -> Vec<HostMessage> {
    match outcome {
        HostEvalOutcome::Success(success) => success.messages.clone(),
        HostEvalOutcome::InputRequired(suspension) => suspension.messages.clone(),
        HostEvalOutcome::Failure(failure) => failure.messages.clone(),
    }
}

fn outcome_trace(outcome: &HostEvalOutcome) -> Vec<HostTraceEvent> {
    match outcome {
        HostEvalOutcome::Success(success) => success.trace.clone(),
        HostEvalOutcome::InputRequired(suspension) => suspension.trace.clone(),
        HostEvalOutcome::Failure(failure) => failure.trace.clone(),
    }
}

fn is_panic(outcome: &HostEvalOutcome) -> bool {
    matches!(
        outcome,
        HostEvalOutcome::Failure(failure) if failure.code == HostFailureCode::Panic
    )
}

fn value_text(session: &mut HostSession, source: &str) -> String {
    match session.eval(source) {
        HostEvalOutcome::Success(success) => success.value.text,
        HostEvalOutcome::InputRequired(suspension) => {
            format!("InputRequired {:?}", suspension.request.prompt)
        }
        HostEvalOutcome::Failure(failure) => format!("Failure {:?}", failure.code),
    }
}

const INNER: &str =
    "probe::inside = \"inside message\"\nMessage[probe::inside]\nInputString[\"in\"]\n";
const PROGRAM: &str = "WithCleanup[Get[\"Inner`\"], Message[probe::outer]]";

fn suspend(session: &mut HostSession, route: &str) -> HostEvalOutcome {
    let cancel = session.cancel_handle();
    let first = match route {
        "eval" => session.eval(PROGRAM),
        "script" => session.eval_script(PROGRAM),
        other => panic!("unknown route {other}"),
    };
    cancel.cancel();
    first
}

struct W1Cell {
    route: &'static str,
    retire: &'static str,
    input_tags: Vec<String>,
    kernel_tags: Vec<String>,
    message_list: String,
    panic: bool,
}

fn w1_cell(route: &'static str, retire: &'static str) -> W1Cell {
    let mut session = session_with(&[("Inner.rum", INNER)], false);
    match session.eval("probe::outer = \"outer message\"") {
        HostEvalOutcome::Success(_) => {}
        other => panic!("setup failed: {other:?}"),
    }
    let first = suspend(&mut session, route);
    let input_tags = tags(&outcome_messages(&first));
    let retired = match retire {
        "eval" => session.eval(""),
        "resume" => session.resume_with_input(""),
        other => panic!("unknown retire {other}"),
    };
    let kernel_tags = tags(&outcome_messages(&retired));
    let message_list = value_text(&mut session, "$MessageList");
    let panic = is_panic(&retired);
    eprintln!(
        "W1CELL route={route} retire={retire} input={input_tags:?} kernel={kernel_tags:?} \
         message_list={message_list} panic={panic}"
    );
    W1Cell {
        route,
        retire,
        input_tags,
        kernel_tags,
        message_list,
        panic,
    }
}

fn w1_green(cell: &W1Cell) -> bool {
    !cell.panic
        && cell.input_tags == ["probe::inside"]
        && cell.kernel_tags == ["probe::inside", "probe::outer", "General::cancel"]
        && cell.message_list == "List[probe::outer, General::cancel]"
}

#[test]
fn w1_merge() {
    on_thread(60, "w1_merge", || {
        let cells = [
            w1_cell("eval", "eval"),
            w1_cell("eval", "resume"),
            w1_cell("script", "eval"),
            w1_cell("script", "resume"),
        ];
        let red = cells.iter().filter(|cell| !w1_green(cell)).count();
        eprintln!("W1VERDICT cells=4 red={red} green={}", 4 - red);
        for cell in &cells {
            assert!(
                w1_green(cell),
                "route={} retire={} input={:?} kernel={:?} list={}",
                cell.route,
                cell.retire,
                cell.input_tags,
                cell.kernel_tags,
                cell.message_list,
            );
        }
    });
}

fn prefix(kind: &str, n: usize) -> String {
    match kind {
        "messages" => (0..n)
            .map(|i| format!("probe::e{i} = \"m{i}\"\nMessage[probe::e{i}]\n"))
            .collect(),
        "trace" => (0..n).map(|_| "1 + 1\n".to_string()).collect(),
        "handled" => (0..n)
            .map(|_| "Check[Block[{$RecursionLimit = 4}, h630[] := h630[]; h630[]], 0]\n")
            .collect(),
        other => panic!("unknown kind {other}"),
    }
}

fn with_cancel(tags: &[String]) -> Vec<String> {
    let mut tags = tags.to_vec();
    tags.push("General::cancel".to_string());
    tags
}

fn cancel_count(tags: &[String]) -> usize {
    tags.iter()
        .filter(|tag| tag.as_str() == "General::cancel")
        .count()
}

struct W2Cell {
    class: &'static str,
}

fn w2_cell(route: &'static str, n: usize, kind: &'static str) -> W2Cell {
    let tracing = kind == "trace";
    let body = format!("{}InputString[\"in\"]\n", prefix(kind, n));
    let mut session = match route {
        "file" => session_with(&[("Outer.rum", &body)], tracing),
        "script" => session_with(&[("Inner.rum", "InputString[\"in\"]\n")], tracing),
        "eval-get" => session_with(&[("Inner.rum", &body)], tracing),
        "nested" => session_with(
            &[("Outer.rum", "Get[\"Inner`\"]\n"), ("Inner.rum", &body)],
            tracing,
        ),
        other => panic!("unknown route {other}"),
    };
    let cancel = session.cancel_handle();
    let first = match route {
        "file" => session.eval_file("Outer.rum"),
        "script" => {
            let source = format!("{}Get[\"Inner`\"]\n", prefix(kind, n));
            session.eval_script(&source)
        }
        "eval-get" => session.eval("Get[\"Inner`\"]"),
        "nested" => session.eval_file("Outer.rum"),
        _ => unreachable!(),
    };
    let suspended = matches!(first, HostEvalOutcome::InputRequired(_));
    cancel.cancel();
    let retired = if suspended {
        session.eval("")
    } else {
        first.clone()
    };
    let input_tags = tags(&outcome_messages(&first));
    let retired_tags = tags(&outcome_messages(&retired));
    let input_trace = outcome_trace(&first);
    let retired_trace = outcome_trace(&retired);
    let items_ok = retired_tags == with_cancel(&input_tags)
        && (kind != "trace" || retired_trace == input_trace);
    let class = if is_panic(&retired) {
        "panic"
    } else if !matches!(retired, HostEvalOutcome::Success(_)) {
        "other"
    } else if cancel_count(&retired_tags) != 1 {
        "lost-cancel"
    } else if !items_ok {
        "dropped-prefix"
    } else {
        let again =
            session.eval("Check[Block[{$RecursionLimit = 4}, h630[] := h630[]; h630[]], 0]");
        let again_tags = tags(&outcome_messages(&again));
        if again_tags == ["General::passlim"] {
            "green"
        } else {
            eprintln!("W2CHECK route={route} n={n} kind={kind} again={again_tags:?}");
            "check"
        }
    };
    eprintln!(
        "W2CELL route={route} n={n} kind={kind} class={class} suspended={suspended} \
         input={input_tags:?} retire={retired_tags:?} trace_in={} trace_out={}",
        input_trace.len(),
        retired_trace.len()
    );
    W2Cell { class }
}

#[test]
fn w2_window_matrix() {
    on_thread(120, "w2_window_matrix", || {
        let mut cells = Vec::new();
        for route in ["file", "script", "eval-get", "nested"] {
            for n in [0_usize, 1, 2] {
                for kind in ["messages", "trace", "handled"] {
                    cells.push(w2_cell(route, n, kind));
                }
            }
        }
        let count = |class: &str| cells.iter().filter(|cell| cell.class == class).count();
        let panic = count("panic");
        let lost = count("lost-cancel");
        let dropped = count("dropped-prefix");
        let check = count("check");
        let other = count("other");
        let green = count("green");
        eprintln!(
            "W2VERDICT cells={} panic={panic} lost-cancel={lost} dropped-prefix={dropped} \
             check={check} other={other} green={green}",
            cells.len()
        );
        assert_eq!(
            (panic, lost, dropped, check, other, green),
            (0, 0, 0, 0, 0, 36),
            "window matrix is not the cumulative contract"
        );
    });
}

#[test]
fn w3_rolled_back_unreported_stays_out() {
    on_thread(60, "w3_rolled_back_unreported_stays_out", || {
        let mut session = session_with(
            &[(
                "Inner.rum",
                "WithCleanup[InputString[\"in\"], (Print[\"inside-cleanup\"]; \
                 Message[probe::inside]; fileRan630 = 1; Throw[630])]\n",
            )],
            false,
        );
        match session.eval("fileRan630 = 0; probe::inside = \"inside message\"") {
            HostEvalOutcome::Success(_) => {}
            other => panic!("setup failed: {other:?}"),
        }
        let first = session.eval("Get[\"Inner`\"]");
        assert!(
            matches!(first, HostEvalOutcome::InputRequired(_)),
            "cleanup control did not suspend: {first:?}"
        );
        session.cancel_handle().cancel();
        let retired = session.eval("");
        let messages = outcome_messages(&retired);
        let retired_tags = tags(&messages);
        let prints = match &retired {
            HostEvalOutcome::Success(success) => success.prints.clone(),
            other => panic!("cleanup control did not succeed: {other:?}"),
        };
        let has_inside = messages
            .iter()
            .any(|message| message.text == "inside message");
        let file_ran = value_text(&mut session, "fileRan630");
        eprintln!(
            "W3CELL kind=cleanup-inside tags={retired_tags:?} prints={prints:?} \
             has_inside={has_inside} file_ran={file_ran}"
        );
        assert_eq!(prints, ["inside-cleanup"]);
        assert!(!has_inside, "{retired_tags:?}");
        assert!(!retired_tags.iter().any(|tag| tag == "Throw::nocatch"));
        assert_eq!(
            retired_tags
                .iter()
                .filter(|tag| tag.as_str() == "General::cancel")
                .count(),
            1,
            "{retired_tags:?}"
        );
        assert_eq!(file_ran, "0");

        let mut session = session_with(
            &[("Outer.rum", "InputString[\"in\"]\nMessage[probe::after]\n")],
            false,
        );
        match session.eval("probe::after = \"after message\"") {
            HostEvalOutcome::Success(_) => {}
            other => panic!("setup failed: {other:?}"),
        }
        let first = session.eval_file("Outer.rum");
        assert!(
            matches!(first, HostEvalOutcome::InputRequired(_)),
            "after-input control did not suspend: {first:?}"
        );
        let input_tags = tags(&outcome_messages(&first));
        session.cancel_handle().cancel();
        let retired = session.eval("");
        let messages = outcome_messages(&retired);
        let retired_tags = tags(&messages);
        let has_after = messages
            .iter()
            .any(|message| message.text == "after message");
        eprintln!(
            "W3CELL kind=after-input input={input_tags:?} retire={retired_tags:?} \
             has_after={has_after}"
        );
        assert!(input_tags.is_empty(), "{input_tags:?}");
        assert!(!has_after, "{retired_tags:?}");
        assert!(!retired_tags.iter().any(|tag| tag == "probe::after"));
        assert_eq!(retired_tags, ["General::cancel"]);
        eprintln!("W3VERDICT green");
    });
}
