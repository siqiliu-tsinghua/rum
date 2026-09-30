use super::*;
use rum_host::{
    HostFailure, HostFailureCode, HostInputRequest, HostInputSuspension, HostMessage,
    HostTraceEvent, HostTyped,
};

pub(super) fn input_request_from_outcome(
    outcome: &KernelExecuteOutcome,
) -> Option<HostInputRequest> {
    match outcome {
        KernelExecuteOutcome::InputRequired(suspension) => Some(suspension.request.clone()),
        _ => None,
    }
}

pub(super) fn kernel_input_request(request: &HostInputRequest) -> KernelInputRequest {
    let mut input = KernelInputRequest::new(&request.prompt);
    if request.password {
        input = input.password();
    }
    input
}

/// Positions in the host's cumulative per-family buffers, owned by one execute.
#[derive(Debug, Default)]
pub(super) struct EmissionProgress {
    prints: usize,
    messages: usize,
    trace: usize,
}

impl EmissionProgress {
    fn buffered_outputs(&mut self, outcome: &KernelExecuteOutcome) -> Vec<KernelOutput> {
        let (prints, messages, trace) = match outcome {
            KernelExecuteOutcome::Success(s) => (&s.prints, &s.messages, &s.trace),
            KernelExecuteOutcome::InputRequired(s) => (&s.prints, &s.messages, &s.trace),
            KernelExecuteOutcome::Failure(f) => {
                // A synthesized panic/poison failure carries no checkpoint.
                // Earlier published output survives; there is nothing to replay.
                if f.code.requires_reset()
                    && f.prints.is_empty()
                    && f.messages.is_empty()
                    && f.trace.is_empty()
                {
                    return Vec::new();
                }
                (&f.prints, &f.messages, &f.trace)
            }
            KernelExecuteOutcome::WireDecoded => return Vec::new(),
        };
        // Runtime outcomes preserve these prefixes, including failure and cancel.
        // Do not use the rebuilt, family-grouped HostStructuredOutput collection.
        let mut outputs = stdout_streams(&prints[self.prints..]);
        outputs.extend(runtime_message_streams(&messages[self.messages..]));
        let unseen_trace = &trace[self.trace..];
        if !unseen_trace.is_empty() {
            outputs.push(KernelOutput::DisplayData(KernelDisplayOutput {
                data: trace_mime_bundle(unseen_trace),
                metadata: trace_metadata(),
            }));
        }
        self.prints = prints.len();
        self.messages = messages.len();
        self.trace = trace.len();
        outputs
    }
}

pub(super) fn build_execute_response(
    execution_count: Option<u64>,
    outcome: KernelExecuteOutcome,
) -> KernelExecuteResponse {
    build_execute_response_since(execution_count, outcome, &mut EmissionProgress::default())
}

pub(super) fn build_execute_response_since(
    execution_count: Option<u64>,
    outcome: KernelExecuteOutcome,
    emitted: &mut EmissionProgress,
) -> KernelExecuteResponse {
    let outputs = build_execute_outputs(execution_count, &outcome, emitted);
    let error = match &outcome {
        KernelExecuteOutcome::Success(_) => None,
        KernelExecuteOutcome::InputRequired(suspension) => {
            Some(input_required_error_output(suspension))
        }
        KernelExecuteOutcome::Failure(failure) => Some(error_output(failure)),
        KernelExecuteOutcome::WireDecoded => None,
    };

    KernelExecuteResponse {
        status: if matches!(outcome, KernelExecuteOutcome::Success(_)) {
            KernelReplyStatus::Ok
        } else {
            KernelReplyStatus::Error
        },
        execution_count,
        payload: Vec::<KernelExecutePayload>::new(),
        user_expressions: Default::default(),
        error,
        outputs,
        outcome,
    }
}

fn build_execute_outputs(
    execution_count: Option<u64>,
    outcome: &KernelExecuteOutcome,
    emitted: &mut EmissionProgress,
) -> Vec<KernelOutput> {
    let mut outputs = emitted.buffered_outputs(outcome);
    match outcome {
        KernelExecuteOutcome::Success(response) => {
            outputs.push(KernelOutput::ExecuteResult(KernelExecuteResult {
                execution_count,
                data: execute_result_mime_bundle(&response.value.text, &response.value.typed),
                metadata: execute_result_metadata(execution_count),
            }));
        }
        KernelExecuteOutcome::InputRequired(suspension) => {
            outputs.push(KernelOutput::Error(input_required_error_output(suspension)));
        }
        KernelExecuteOutcome::Failure(failure) => {
            outputs.push(KernelOutput::Error(error_output(failure)));
        }
        KernelExecuteOutcome::WireDecoded => {}
    }
    outputs
}

pub(super) fn build_suspended_outputs(
    outcome: &KernelExecuteOutcome,
    emitted: &mut EmissionProgress,
) -> Vec<KernelOutput> {
    emitted.buffered_outputs(outcome)
}

fn stdout_streams(prints: &[String]) -> Vec<KernelOutput> {
    prints
        .iter()
        .map(|text| {
            KernelOutput::Stream(KernelStreamOutput {
                name: KernelStreamName::Stdout,
                text: format!("{text}\n"),
            })
        })
        .collect()
}

fn runtime_message_streams(messages: &[HostMessage]) -> Vec<KernelOutput> {
    messages
        .iter()
        .map(|message| {
            KernelOutput::Stream(KernelStreamOutput {
                name: KernelStreamName::Stderr,
                text: format!("{}::{}: {}\n", message.symbol, message.tag, message.text),
            })
        })
        .collect()
}

fn error_output(failure: &HostFailure) -> KernelErrorOutput {
    KernelErrorOutput {
        error_name: failure_code_name(failure.code).to_string(),
        error_value: failure.message.clone(),
        rendered: None,
    }
}

fn input_required_error_output(suspension: &HostInputSuspension) -> KernelErrorOutput {
    KernelErrorOutput {
        error_name: "InputRequired".to_string(),
        error_value: format!("input required: {}", suspension.request.prompt),
        rendered: None,
    }
}

const fn failure_code_name(code: HostFailureCode) -> &'static str {
    match code {
        HostFailureCode::Panic => "Panic",
        HostFailureCode::Poisoned => "Poisoned",
        HostFailureCode::Syntax => "Syntax",
        HostFailureCode::InvalidInteger => "InvalidInteger",
        HostFailureCode::Io => "Io",
        HostFailureCode::LoadPathNotFound => "LoadPathNotFound",
        HostFailureCode::EvaluationCycleDetected => "EvaluationCycleDetected",
        HostFailureCode::RecursionLimitExceeded => "RecursionLimitExceeded",
        HostFailureCode::MatchRecursionLimitExceeded => "MatchRecursionLimitExceeded",
        HostFailureCode::Cancelled => "Cancelled",
        HostFailureCode::TimeConstraintExceeded => "TimeConstraintExceeded",
        HostFailureCode::MemoryConstraintExceeded => "MemoryConstraintExceeded",
        HostFailureCode::GeneratedElementLimitExceeded => "GeneratedElementLimitExceeded",
        HostFailureCode::IntegerWorkLimitExceeded => "IntegerWorkLimitExceeded",
        HostFailureCode::IterationWorkLimitExceeded => "IterationWorkLimitExceeded",
        HostFailureCode::IterationLimitExceeded => "IterationLimitExceeded",
        HostFailureCode::EvaluationPassLimitExceeded => "EvaluationPassLimitExceeded",
        HostFailureCode::Exit => "Exit",
        HostFailureCode::Other => "Other",
    }
}

fn expr_mime_bundle(rendered_expr: &str) -> KernelMimeBundle {
    let mut bundle = KernelMimeBundle::text_plain(rendered_expr);
    bundle.insert("text/markdown", format!("```rum\n{rendered_expr}\n```"));
    bundle.insert("application/vnd.rum.expr", rendered_expr);
    bundle
}

fn execute_result_mime_bundle(rendered_expr: &str, typed: &HostTyped) -> KernelMimeBundle {
    let mut bundle = expr_mime_bundle(rendered_expr);
    if let Some(json) = json_from_typed(typed) {
        bundle.insert("application/json", json);
    }
    bundle
}

fn json_from_typed(value: &HostTyped) -> Option<String> {
    match value {
        HostTyped::Integer(value) => Some(value.to_string()),
        HostTyped::Str(value) => Some(json_string_from_typed(value)),
        HostTyped::Bool(value) => Some(value.to_string()),
        HostTyped::List(items) => {
            let items = items
                .iter()
                .map(json_from_typed)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("[{}]", items.join(",")))
        }
        HostTyped::Association(entries) => {
            let mut seen_keys = std::collections::HashSet::with_capacity(entries.len());
            let mut fields = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                let HostTyped::Str(key) = key else {
                    return None;
                };
                if !seen_keys.insert(key.as_str()) {
                    return None;
                }
                fields.push(format!(
                    "{}:{}",
                    json_string_from_typed(key),
                    json_from_typed(value)?
                ));
            }
            Some(format!("{{{}}}", fields.join(",")))
        }
        HostTyped::BigInteger(_)
        | HostTyped::Rational { .. }
        | HostTyped::Symbol(_)
        | HostTyped::Rule { .. }
        | HostTyped::Other { .. } => None,
    }
}

fn json_string_from_typed(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');

    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000c}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch <= '\u{001f}' => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }

    escaped.push('"');
    escaped
}

fn trace_mime_bundle(trace: &[HostTraceEvent]) -> KernelMimeBundle {
    let rendered = trace
        .iter()
        .map(render_trace_event)
        .collect::<Vec<_>>()
        .join("\n");

    let mut bundle = KernelMimeBundle::text_plain(&rendered);
    bundle.insert("text/markdown", format!("```text\n{rendered}\n```"));
    bundle.insert("application/vnd.rum.trace", rendered);
    bundle
}

pub(super) fn inspect_mime_bundle(response: &rum_host::HostInspectResponse) -> KernelMimeBundle {
    let plain = render_inspect_text(response);
    let markdown = render_inspect_markdown(response);

    let mut bundle = KernelMimeBundle::text_plain(&plain);
    bundle.insert("text/markdown", markdown);
    bundle.insert("application/vnd.rum.inspect", plain);
    bundle
}

fn execute_result_metadata(execution_count: Option<u64>) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("output_type", "execute_result");
    metadata.insert("bundle_kind", "expr");
    metadata.insert(
        "execution_count",
        execution_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    metadata
}

fn trace_metadata() -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("output_type", "display_data");
    metadata.insert("bundle_kind", "trace");
    metadata
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_from_typed_maps_supported_tree_and_escapes_strings() {
        let value = HostTyped::Association(vec![(
            HostTyped::Str("quote\"\\\n".to_string()),
            HostTyped::List(vec![
                HostTyped::Integer(-2),
                HostTyped::Bool(false),
                HostTyped::Str("tab\t".to_string()),
            ]),
        )]);

        assert_eq!(
            json_from_typed(&value).as_deref(),
            Some(r#"{"quote\"\\\n":[-2,false,"tab\t"]}"#)
        );
    }

    #[test]
    fn json_from_typed_rejects_ambiguous_or_partial_trees() {
        let duplicate_keys = HostTyped::Association(vec![
            (HostTyped::Str("a".to_string()), HostTyped::Integer(1)),
            (HostTyped::Str("a".to_string()), HostTyped::Integer(2)),
        ]);
        assert_eq!(json_from_typed(&duplicate_keys), None);

        let non_string_key = HostTyped::Association(vec![(
            HostTyped::Integer(1),
            HostTyped::Str("value".to_string()),
        )]);
        assert_eq!(json_from_typed(&non_string_key), None);

        let nested_unsupported = HostTyped::List(vec![
            HostTyped::Integer(1),
            HostTyped::Rational {
                numerator: "1".to_string(),
                denominator: "2".to_string(),
            },
        ]);
        assert_eq!(json_from_typed(&nested_unsupported), None);

        let rule = HostTyped::Rule {
            lhs: Box::new(HostTyped::Str("a".to_string())),
            rhs: Box::new(HostTyped::Integer(1)),
        };
        assert_eq!(json_from_typed(&rule), None);
    }

    #[test]
    fn error_output_covers_time_constraint_failures() {
        let outcome = KernelExecuteOutcome::Failure(host_failure(
            HostFailureCode::TimeConstraintExceeded,
            "TimeConstrained limit exceeded",
        ));

        let response = build_execute_response(Some(1), outcome);

        assert_eq!(response.status, KernelReplyStatus::Error);
        assert_eq!(response.outputs.len(), 1);
        assert_eq!(
            response
                .error
                .as_ref()
                .map(|error| error.error_name.as_str()),
            Some("TimeConstraintExceeded")
        );

        match &response.outputs[0] {
            KernelOutput::Error(output) => {
                assert_eq!(output.error_name, "TimeConstraintExceeded");
                assert!(
                    output.error_value.contains("TimeConstrained limit"),
                    "unexpected error text: {}",
                    output.error_value
                );
            }
            other => panic!("expected error output, got {other:?}"),
        }
    }

    #[test]
    fn error_output_covers_memory_constraint_failures() {
        let outcome = KernelExecuteOutcome::Failure(host_failure(
            HostFailureCode::MemoryConstraintExceeded,
            "MemoryConstrained limit (20) exceeded",
        ));

        let response = build_execute_response(Some(1), outcome);

        assert_eq!(response.status, KernelReplyStatus::Error);
        assert_eq!(response.outputs.len(), 1);
        assert_eq!(
            response
                .error
                .as_ref()
                .map(|error| error.error_name.as_str()),
            Some("MemoryConstraintExceeded")
        );

        match &response.outputs[0] {
            KernelOutput::Error(output) => {
                assert_eq!(output.error_name, "MemoryConstraintExceeded");
                assert!(
                    output.error_value.contains("MemoryConstrained limit (20)"),
                    "unexpected error text: {}",
                    output.error_value
                );
            }
            other => panic!("expected error output, got {other:?}"),
        }
    }

    #[test]
    fn error_output_covers_resource_limit_failures() {
        for (code, expected_name) in [
            (
                HostFailureCode::GeneratedElementLimitExceeded,
                "GeneratedElementLimitExceeded",
            ),
            (
                HostFailureCode::IntegerWorkLimitExceeded,
                "IntegerWorkLimitExceeded",
            ),
            (
                HostFailureCode::IterationWorkLimitExceeded,
                "IterationWorkLimitExceeded",
            ),
        ] {
            let outcome = KernelExecuteOutcome::Failure(host_failure(code, expected_name));

            let response = build_execute_response(Some(1), outcome);

            assert_eq!(response.status, KernelReplyStatus::Error);
            assert_eq!(
                response
                    .error
                    .as_ref()
                    .map(|error| error.error_name.as_str()),
                Some(expected_name)
            );
        }
    }

    #[test]
    fn render_rule_application_source_covers_replace() {
        assert_eq!(render_rule_application_source("Replace"), "replace");
        assert_eq!(
            render_rule_application_source("ReplaceList"),
            "replace-list"
        );
        assert_eq!(render_rule_application_source("ReplaceAt"), "replace-at");
    }

    fn host_failure(code: HostFailureCode, message: &str) -> HostFailure {
        HostFailure {
            code,
            exit_status: None,
            message: message.to_string(),
            syntax_rendered: None,
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

pub(super) fn inspect_metadata(target: Option<&str>, found: bool) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("found", if found { "true" } else { "false" });
    metadata.insert(
        "target",
        target
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "<none>".to_string()),
    );
    metadata.insert("bundle_kind", "inspect");
    metadata
}

pub(super) fn response_msg_type(response: &KernelResponse) -> &'static str {
    match response {
        KernelResponse::Execute(_) => "execute_reply",
        KernelResponse::Inspect(_) => "inspect_reply",
        KernelResponse::Complete(_) => "complete_reply",
        KernelResponse::IsComplete(_) => "is_complete_reply",
        KernelResponse::History(_) => "history_reply",
        KernelResponse::KernelInfo(_) => "kernel_info_reply",
        KernelResponse::CommInfo(_) => "comm_info_reply",
        KernelResponse::Debug(_) => "debug_reply",
        KernelResponse::Interrupt(_) => "interrupt_reply",
        KernelResponse::Shutdown(_) => "shutdown_reply",
    }
}

pub(super) fn response_metadata(
    response: &KernelResponse,
    channel: KernelChannel,
    parent_header: &KernelMessageHeader,
    execution_state: KernelExecutionState,
    lifecycle_state: KernelLifecycleState,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("msg_type", response_msg_type(response));
    metadata.insert("channel", channel.as_str());
    metadata.insert("execution_state", execution_state.as_str());
    metadata.insert("lifecycle_state", lifecycle_state.as_str());
    metadata.insert("parent_msg_type", &parent_header.msg_type);
    metadata.insert("protocol_version", &parent_header.protocol_version);

    let status = match response {
        KernelResponse::Execute(execute) => execute.status.as_str(),
        KernelResponse::Inspect(inspect) => inspect.status.as_str(),
        KernelResponse::Complete(complete) => complete.status.as_str(),
        KernelResponse::IsComplete(response) => response.reply_status.as_str(),
        KernelResponse::History(history) => history.status.as_str(),
        KernelResponse::KernelInfo(info) => info.status.as_str(),
        KernelResponse::CommInfo(comm_info) => comm_info.status.as_str(),
        KernelResponse::Debug(debug) => debug.status.as_str(),
        KernelResponse::Interrupt(interrupt) => interrupt.status.as_str(),
        KernelResponse::Shutdown(shutdown) => shutdown.status.as_str(),
    };

    metadata.insert("status", status);

    if let KernelResponse::IsComplete(response) = response {
        metadata.insert("completion_status", response.completion_status.as_str());
    }

    metadata
}

pub(super) fn complete_metadata(
    prefix: &str,
    replacement_start: usize,
    replacement_end: usize,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("cursor_start", replacement_start.to_string());
    metadata.insert("cursor_end", replacement_end.to_string());
    metadata.insert("prefix", prefix);
    metadata
}

pub(super) fn input_request_metadata(
    parent_header: &KernelMessageHeader,
    request: &KernelExecuteRequest,
    input: &KernelInputRequest,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("msg_type", "input_request");
    metadata.insert("channel", KernelChannel::Stdin.as_str());
    metadata.insert("parent_msg_type", &parent_header.msg_type);
    metadata.insert(
        "allow_stdin",
        if request.allow_stdin { "true" } else { "false" },
    );
    metadata.insert("password", if input.password { "true" } else { "false" });
    metadata
}

fn render_inspect_text(response: &rum_host::HostInspectResponse) -> String {
    let Some(info) = &response.info else {
        return response
            .target
            .as_deref()
            .map(|target| format!("{target}: <not found>"))
            .unwrap_or_else(|| "<no symbol>".to_string());
    };

    let mut lines = vec![
        format!("{} ({})", info.qualified_name, info.context),
        format!(
            "attributes: {}",
            if info.attributes.is_empty() {
                "<none>".to_string()
            } else {
                info.attributes.join(", ")
            }
        ),
        format!(
            "definitions: own={} delayed={} up={} down={} sub={}",
            info.has_own_value,
            info.has_delayed_value,
            info.has_up_values,
            info.has_down_values,
            info.has_sub_values
        ),
        format!(
            "messages: {}",
            if info.message_tags.is_empty() {
                "<none>".to_string()
            } else {
                info.message_tags.join(", ")
            }
        ),
    ];

    if let Some(summary) = &info.summary {
        lines.insert(1, format!("summary: {summary}"));
    }

    lines.join("\n")
}

fn render_inspect_markdown(response: &rum_host::HostInspectResponse) -> String {
    let Some(info) = &response.info else {
        return response
            .target
            .as_deref()
            .map(|target| format!("`{target}` not found"))
            .unwrap_or_else(|| "No symbol".to_string());
    };

    let attributes = if info.attributes.is_empty() {
        "<none>".to_string()
    } else {
        info.attributes.join(", ")
    };
    let messages = if info.message_tags.is_empty() {
        "<none>".to_string()
    } else {
        info.message_tags.join(", ")
    };

    format!(
        "### `{}`\n\n- Context: `{}`\n- Attributes: {}\n- Definitions: own=`{}`, delayed=`{}`, up=`{}`, down=`{}`, sub=`{}`\n- Messages: {}",
        info.qualified_name,
        info.context,
        attributes,
        info.has_own_value,
        info.has_delayed_value,
        info.has_up_values,
        info.has_down_values,
        info.has_sub_values,
        messages
    )
}

fn render_trace_event(event: &HostTraceEvent) -> String {
    match event {
        HostTraceEvent::EvaluationPass { before, after } => {
            format!("evaluation-pass: {before} => {after}")
        }
        HostTraceEvent::DefinitionStored {
            symbol,
            kind,
            value,
        } => format!("definition-stored: {symbol} [{kind}] = {value}"),
        HostTraceEvent::RuleApplied {
            source,
            input,
            rule,
            result,
        } => format!(
            "rule-applied: {} | input={input} | rule={rule} | result={result}",
            render_rule_application_source(source)
        ),
    }
}

fn render_rule_application_source(source: &str) -> &str {
    match source {
        "UpValue" => "upvalue",
        "DownValue" => "downvalue",
        "SubValue" => "subvalue",
        "Replace" => "replace",
        "ReplaceList" => "replace-list",
        "ReplaceAt" => "replace-at",
        "ReplaceAll" => "replace-all",
        "ReplaceRepeated" => "replace-repeated",
        other => other,
    }
}
