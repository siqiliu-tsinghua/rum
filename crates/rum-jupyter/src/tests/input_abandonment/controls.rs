use super::*;

fn resume(
    adapter: &mut KernelAdapter,
    paused: &KernelDispatchResult,
    answer: &str,
) -> KernelDispatchResult {
    assert!(paused.reply.is_none());
    assert_eq!(paused.stdin.len(), 1);
    let started = adapter
        .handle_input_reply(input_reply(&paused.stdin[0], answer))
        .unwrap()
        .unwrap();
    assert!(started.reply.is_none());
    wait_for_execute_completion(adapter)
}

#[test]
fn worker_nested_get_still_resumes() {
    let nested = NestedGet::new();
    let mut session = Session::new();
    eval_text(&mut session, "marker=1; tail=0");
    let mut adapter = KernelAdapter::new(session);
    let paused = start(&mut adapter, &nested.source, true);
    assert_eq!(paused.stdin[0].request.prompt, "nested: ");
    let done = resume(&mut adapter, &paused, "answer");
    assert_eq!(result_text(response(&done)), "2");
    assert_eq!(
        result_text(&adapter.execute(KernelExecuteRequest::new("{marker,tail}"))),
        "List[3, 2]"
    );
    assert_reusable(&mut adapter);
}

#[test]
fn worker_twice_still_resumes() {
    let mut adapter = KernelAdapter::new(Session::new());
    let first = start(
        &mut adapter,
        "a=Input[\"first: \"]; b=InputString[\"second: \"]; {a,b}",
        true,
    );
    assert_eq!(first.stdin[0].request.prompt, "first: ");
    let second = resume(&mut adapter, &first, "6*7");
    assert_eq!(second.stdin[0].request.prompt, "second: ");
    let done = resume(&mut adapter, &second, "literal");
    assert_eq!(result_text(response(&done)), "List[42, \"literal\"]");
    assert_reusable(&mut adapter);
}

#[test]
fn worker_password_flag_still_resumes() {
    let mut adapter = KernelAdapter::new(Session::new());
    let paused = start(
        &mut adapter,
        "InputString[\"secret: \",FieldMasked->True]",
        true,
    );
    assert!(paused.stdin[0].request.password);
    assert_eq!(paused.stdin[0].metadata.get("password"), Some("true"));
    let done = resume(&mut adapter, &paused, "secret");
    assert_eq!(result_text(response(&done)), "\"secret\"");
    assert_reusable(&mut adapter);
}

#[test]
fn refused_immediate_admission_keeps_original_request() {
    refused_admission(true);
}

#[test]
fn refused_worker_admission_keeps_original_request() {
    refused_admission(false);
}

fn refused_admission(immediate: bool) {
    let mut adapter = KernelAdapter::new(Session::new());
    let paused = start(&mut adapter, "InputString[\"owner: \"]", true);
    let rejected = invoke(&mut adapter, "InputString[\"other: \"]", immediate);
    assert!(
        matches!(rejected.outcome, KernelExecuteOutcome::Failure(ref failure) if failure.code == rum_host::HostFailureCode::Other)
    );
    assert_eq!(adapter.pending_input_request(), Some(&paused.stdin[0]));
    let done = resume(&mut adapter, &paused, "original answer");
    assert_eq!(result_text(response(&done)), "\"original answer\"");
    assert_reusable(&mut adapter);
}

#[test]
fn shell_wrapper_is_reply_only_and_caller_services_stdin() {
    let mut adapter = KernelAdapter::new(Session::new());
    assert!(
        adapter
            .handle_shell_request(request("shell", "InputString[\"p: \"]", true))
            .is_none()
    );
    let paused = wait_for_execute_completion(&mut adapter);
    assert!(
        paused.iopub.is_empty(),
        "initial prologue was dropped by the wrapper"
    );
    let done = resume(&mut adapter, &paused, "answer");
    assert_eq!(result_text(response(&done)), "\"answer\"");
    assert_reusable(&mut adapter);
}
