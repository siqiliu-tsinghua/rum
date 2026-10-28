//! KernelAdapter's immediate/dispatch contracts and host request cancellation.
//! InputRequired is terminal only after the admitted request has been retired.
use super::*;
use crate::{KernelExecuteResponse, KernelInputRequestEnvelope};

mod controls;
mod transport;

fn request(id: &str, source: &str, allow_stdin: bool) -> KernelRequestEnvelope {
    let mut execute = KernelExecuteRequest::new(source);
    execute.allow_stdin = allow_stdin;
    KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(id, "input-abandonment", "tester", "execute_request"),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Execute(execute),
    }
}

fn start(adapter: &mut KernelAdapter, source: &str, allow: bool) -> KernelDispatchResult {
    let initial = adapter.dispatch_request(request("execute", source, allow));
    assert!(initial.reply.is_none());
    finish_adapter_execute(adapter, initial)
}

/// An overlapping execute is refused at admission, before any worker starts.
fn refused(adapter: &mut KernelAdapter, source: &str) -> KernelExecuteResponse {
    let initial = adapter.dispatch_request(request("execute", source, false));
    response(&initial).clone()
}

fn response(dispatch: &KernelDispatchResult) -> &KernelExecuteResponse {
    match &dispatch.reply.as_ref().expect("terminal reply").response {
        KernelResponse::Execute(response) => response,
        other => panic!("expected execute reply: {other:?}"),
    }
}

fn result_text(response: &KernelExecuteResponse) -> &str {
    match &response.outcome {
        KernelExecuteOutcome::Success(success) => &success.value.text,
        other => panic!("expected success: {other:?}"),
    }
}

fn assert_input_error(response: &KernelExecuteResponse, prompt: &str) {
    assert_eq!(response.status, KernelReplyStatus::Error);
    let error = response.error.as_ref().expect("input error");
    assert_eq!(error.error_name, "InputRequired");
    assert_eq!(error.error_value, format!("input required: {prompt}"));
    let KernelExecuteOutcome::InputRequired(suspension) = &response.outcome else {
        panic!("original input reason must survive retirement");
    };
    assert_eq!(suspension.request.prompt, prompt);
    assert_eq!(
        response
            .outputs
            .iter()
            .filter(|o| matches!(o, KernelOutput::Error(_)))
            .count(),
        1
    );
}

fn assert_reusable(adapter: &mut KernelAdapter) {
    assert_eq!(
        result_text(&adapter.execute(KernelExecuteRequest::new("1+1"))),
        "2"
    );
}

fn invoke(adapter: &mut KernelAdapter, code: &str, immediate: bool) -> KernelExecuteResponse {
    if immediate {
        adapter.execute(KernelExecuteRequest::new(code))
    } else {
        response(&start(adapter, code, false)).clone()
    }
}

fn streams(response: &KernelExecuteResponse, name: KernelStreamName) -> Vec<&str> {
    response
        .outputs
        .iter()
        .filter_map(|output| match output {
            KernelOutput::Stream(stream) if stream.name == name => Some(stream.text.as_str()),
            _ => None,
        })
        .collect()
}

fn input_reply(input: &KernelInputRequestEnvelope, answer: &str) -> KernelInputReplyEnvelope {
    KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new("answer", "input-abandonment", "tester", "input_reply"),
        parent_header: Some(input.header.clone()),
        metadata: KernelMetadata::new(),
        reply: KernelInputReply::new(answer),
    }
}

// Promoted from brief-552's five socket-free reuse reds.
#[test]
fn red_sync_execute_abandons() {
    let mut adapter = KernelAdapter::new(Session::new());
    let first = adapter.execute(KernelExecuteRequest::new("InputString[\"p: \"]"));
    assert_input_error(&first, "p: ");
    assert_reusable(&mut adapter);
}

#[test]
fn immediate_without_stdin_abandons() {
    let mut adapter = KernelAdapter::new(Session::new());
    let first = adapter.execute(KernelExecuteRequest::new("InputString[\"p: \"]").without_stdin());
    assert_input_error(&first, "p: ");
    assert_reusable(&mut adapter);
}

#[test]
fn red_worker_without_stdin_abandons() {
    let mut adapter = KernelAdapter::new(Session::new());
    let first = start(&mut adapter, "InputString[\"p: \"]", false);
    assert_input_error(response(&first), "p: ");
    assert!(first.stdin.is_empty());
    let error = first
        .iopub
        .iter()
        .find(|output| matches!(output.payload, KernelIoPubPayload::Error(_)))
        .unwrap();
    assert_eq!(error.metadata.get("error_name"), Some("InputRequired"));
    assert_eq!(
        result_text(response(&start(&mut adapter, "1+1", false))),
        "2"
    );
}

#[test]
fn red_shell_wrapper_without_stdin_abandons() {
    let mut adapter = KernelAdapter::new(Session::new());
    assert!(
        adapter
            .handle_shell_request(request("shell", "InputString[\"p: \"]", false))
            .is_none()
    );
    let done = wait_for_execute_completion(&mut adapter);
    assert_input_error(response(&done), "p: ");
    assert!(done.stdin.is_empty());
    assert_reusable(&mut adapter);
}

#[test]
fn immediate_cleanup_and_prefix_are_delivered_once() {
    for allow in [true, false] {
        let mut adapter = KernelAdapter::new(Session::new());
        let mut execute = KernelExecuteRequest::new(cleanup_source());
        execute.allow_stdin = allow;
        let first = adapter.execute(execute);
        assert_cleanup(&mut adapter, &first);
    }
}

#[test]
fn worker_cleanup_and_prefix_are_delivered_once() {
    let mut adapter = KernelAdapter::new(Session::new());
    let done = start(&mut adapter, cleanup_source(), false);
    assert_cleanup(&mut adapter, response(&done));
    let stdout: Vec<_> = done
        .iopub
        .iter()
        .filter_map(|out| match &out.payload {
            KernelIoPubPayload::Stream(stream) if stream.name == KernelStreamName::Stdout => {
                Some(stream.text.as_str())
            }
            _ => None,
        })
        .collect();
    assert_eq!(stdout, ["prefix\n", "cleanup\n"]);
    assert_eq!(
        done.iopub
            .iter()
            .filter(|out| matches!(out.payload, KernelIoPubPayload::Error(_)))
            .count(),
        1
    );
    assert!(matches!(&done.iopub.last().unwrap().payload,
        KernelIoPubPayload::Status(status) if status.execution_state == KernelExecutionState::Idle));
}

fn cleanup_source() -> &'static str {
    "cleanupCount=0; tail=0; Print[\"prefix\"]; WithCleanup[InputString[\"p: \"], cleanupCount=cleanupCount+1; Print[\"cleanup\"]]; tail=1; Print[\"tail\"]"
}

fn assert_cleanup(adapter: &mut KernelAdapter, first: &KernelExecuteResponse) {
    assert_input_error(first, "p: ");
    assert_eq!(
        streams(first, KernelStreamName::Stdout),
        ["prefix\n", "cleanup\n"]
    );
    assert!(streams(first, KernelStreamName::Stderr).is_empty());
    assert_eq!(
        result_text(&adapter.execute(KernelExecuteRequest::new("{cleanupCount,tail}"))),
        "List[1, 0]"
    );
    assert_reusable(adapter);
}

#[test]
fn immediate_prefix_output_is_delivered_once() {
    prefix_output(true);
}

#[test]
fn worker_prefix_output_is_delivered_once() {
    prefix_output(false);
}

fn prefix_output(immediate: bool) {
    let mut adapter = KernelAdapter::new(Session::new());
    let first = invoke(
        &mut adapter,
        "Print[\"prefix\"]; InputString[\"p: \"]; Print[\"tail\"]",
        immediate,
    );
    assert_input_error(&first, "p: ");
    assert_eq!(streams(&first, KernelStreamName::Stdout), ["prefix\n"]);
    assert_reusable(&mut adapter);
}

#[test]
fn cleanup_messages_keep_guest_cancel_and_original_input_reason() {
    for immediate in [true, false] {
        let mut adapter = KernelAdapter::new(Session::new());
        let first = invoke(
            &mut adapter,
            "notice::before=\"before-message\"; notice::cleanup=\"cleanup-message\"; Message[notice::before]; WithCleanup[InputString[\"p: \"], Message[notice::cleanup]; Message[General::cancel]]",
            immediate,
        );
        assert_input_error(&first, "p: ");
        let KernelExecuteOutcome::InputRequired(suspension) = &first.outcome else {
            unreachable!()
        };
        let tags: Vec<_> = suspension
            .messages
            .iter()
            .map(|message| message.tag.as_str())
            .collect();
        assert_eq!(tags, ["before", "cleanup", "cancel"]);
        assert_eq!(streams(&first, KernelStreamName::Stderr).len(), 3);
        assert_reusable(&mut adapter);
    }
}

struct NestedGet {
    directory: std::path::PathBuf,
    source: String,
}

impl NestedGet {
    fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "rum-jupyter-input-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let inner = directory.join("inner.rum");
        let outer = directory.join("outer.rum");
        std::fs::write(
            &inner,
            "marker=3\nWithCleanup[InputString[\"nested: \"], Print[\"inner:\",marker]]\ntail=1",
        )
        .unwrap();
        std::fs::write(
            &outer,
            format!(
                "marker=2\nWithCleanup[Get[{:?}], Print[\"outer:\",marker]]\ntail=2",
                inner.to_str().unwrap()
            ),
        )
        .unwrap();
        Self {
            source: format!("Get[{:?}]", outer.to_str().unwrap()),
            directory,
        }
    }
}

impl Drop for NestedGet {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.directory).unwrap();
    }
}

#[test]
fn immediate_nested_get_abandons_and_rolls_back() {
    for allow in [true, false] {
        nested_abandonment(Some(allow));
    }
}

#[test]
fn worker_nested_get_abandons_and_rolls_back() {
    nested_abandonment(None);
}

fn nested_abandonment(allow: Option<bool>) {
    let nested = NestedGet::new();
    let mut session = Session::new();
    eval_text(&mut session, "marker=1; tail=0");
    let mut adapter = KernelAdapter::new(session);
    let first = if let Some(allow) = allow {
        let mut request = KernelExecuteRequest::new(&nested.source);
        request.allow_stdin = allow;
        adapter.execute(request)
    } else {
        response(&start(&mut adapter, &nested.source, false)).clone()
    };
    assert_input_error(&first, "nested: ");
    assert_eq!(
        streams(&first, KernelStreamName::Stdout),
        ["inner:3\n", "outer:2\n"]
    );
    assert_eq!(
        result_text(&adapter.execute(KernelExecuteRequest::new("{marker,tail,$InputFileName}"))),
        "List[1, 0, \"\"]"
    );
    assert_reusable(&mut adapter);
}

#[test]
fn worker_retirement_runs_cleanup_off_transport_thread() {
    use std::sync::mpsc;
    let (send, receive) = mpsc::channel();
    let mut session = Session::new();
    session.set_print_sink(rum_host::ClosurePrintSink::assert_bounded_destruction(
        move |line| {
            send.send((line.to_string(), std::thread::current().id()))
                .unwrap();
        },
    ));
    let mut adapter = KernelAdapter::new(session);
    let done = start(
        &mut adapter,
        "WithCleanup[InputString[\"p: \"], Print[\"cleanup\"]]",
        false,
    );
    assert_input_error(response(&done), "p: ");
    let (line, thread) = receive
        .try_recv()
        .expect("guest cleanup must run before the terminal reply");
    assert_eq!(line, "cleanup");
    assert_ne!(thread, std::thread::current().id());
    assert!(receive.try_recv().is_err());
    assert_reusable(&mut adapter);
}

#[test]
fn execute_route_reports_inside_message_once_and_outer_cleanup_once() {
    let directory =
        std::env::temp_dir().join(format!("rum-jupyter-b2-inner-{}", std::process::id()));
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
    let mut adapter = KernelAdapter::new(Session::new());
    let first = adapter.execute(KernelExecuteRequest::new(&source));
    let _ = std::fs::remove_dir_all(&directory);
    assert_input_error(&first, "in");
    let stderr = streams(&first, KernelStreamName::Stderr).join("");
    assert_eq!(stderr.matches("inside message").count(), 1, "{stderr}");
    assert_eq!(stderr.matches("outer message").count(), 1, "{stderr}");
}
