use super::*;
use crate::{KernelExecuteResponse, KernelInputRequestEnvelope, KernelIoPubEnvelope};

pub(super) fn request(source: &str) -> KernelRequestEnvelope {
    KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new("original", "output-once", "tester", "execute_request"),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new(source)),
    }
}

pub(super) fn input_reply(
    input: &KernelInputRequestEnvelope,
    answer: &str,
) -> KernelInputReplyEnvelope {
    KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new("answer", "output-once", "tester", "input_reply"),
        parent_header: Some(input.header.clone()),
        metadata: KernelMetadata::new(),
        reply: KernelInputReply::new(answer),
    }
}

pub(super) fn response(dispatch: &KernelDispatchResult) -> &KernelExecuteResponse {
    let KernelResponse::Execute(response) = &dispatch.reply.as_ref().unwrap().response else {
        panic!("expected execute reply")
    };
    response
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Effects {
    pub(super) stdout: Vec<String>,
    pub(super) stderr: Vec<String>,
    trace: Vec<String>,
    terminal: Vec<KernelOutput>,
}

impl Effects {
    pub(super) fn from_outputs(outputs: impl IntoIterator<Item = KernelOutput>) -> Self {
        let mut effects = Self::default();
        for output in outputs {
            match output {
                KernelOutput::Stream(stream) => match stream.name {
                    KernelStreamName::Stdout => effects.stdout.push(stream.text),
                    KernelStreamName::Stderr => effects.stderr.push(stream.text),
                },
                KernelOutput::DisplayData(display) => {
                    assert_eq!(display.metadata.get("bundle_kind"), Some("trace"));
                    let plain = display.data.get("text/plain").unwrap();
                    assert!(!plain.is_empty(), "no empty trace bundles");
                    assert_eq!(display.data.get("application/vnd.rum.trace"), Some(plain));
                    let fence = char::from(96).to_string().repeat(3);
                    assert_eq!(
                        display.data.get("text/markdown"),
                        Some(format!("{fence}text\n{plain}\n{fence}").as_str())
                    );
                    effects.trace.push(plain.to_string());
                }
                other => effects.terminal.push(other),
            }
        }
        // A trace display contains the new events in that segment. Joining the
        // bundles preserves every event and separator, including equal events.
        if !effects.trace.is_empty() {
            effects.trace = vec![effects.trace.join("\n")];
        }
        effects
    }
}

fn output(envelope: KernelIoPubEnvelope) -> Option<KernelOutput> {
    match envelope.payload {
        KernelIoPubPayload::Stream(stream) => Some(KernelOutput::Stream(stream)),
        KernelIoPubPayload::DisplayData(display) => Some(KernelOutput::DisplayData(display)),
        KernelIoPubPayload::ExecuteResult(result) => Some(KernelOutput::ExecuteResult(result)),
        KernelIoPubPayload::Error(error) => Some(KernelOutput::Error(error)),
        KernelIoPubPayload::Status(_) | KernelIoPubPayload::ExecuteInput(_) => None,
    }
}

pub(super) struct Run {
    pub(super) pump: KernelEventPump,
    transport: InMemoryTransport,
    segment_start: usize,
    parent: KernelMessageHeader,
}

impl Run {
    pub(super) fn start(source: &str, session: Session) -> Self {
        let original = request(source);
        let mut run = Self {
            pump: KernelEventPump::new(
                KernelAdapter::new(session),
                KernelWireCodec::new(KernelWireConfig::default()),
            ),
            transport: InMemoryTransport::default(),
            segment_start: 0,
            parent: original.header.clone(),
        };
        let mut wire = run.pump.codec().encode_request_envelope(&original).unwrap();
        wire.routing_ids = vec![b"original-route".to_vec()];
        run.pump
            .handle_wire_request(&mut run.transport, wire)
            .unwrap();
        run.complete();
        run
    }

    fn complete(&mut self) {
        self.pump.adapter_mut().wait_for_execute_worker_finished();
        assert!(
            self.pump
                .pump_execute_completion(&mut self.transport)
                .unwrap()
        );
        assert!(
            !self
                .pump
                .pump_execute_completion(&mut self.transport)
                .unwrap()
        );
        // Check family order in the actual wire sends of each segment.
        let mut previous = 0;
        for wire in &self.transport.messages[self.segment_start..] {
            if wire.channel != KernelChannel::IoPub {
                continue;
            }
            let envelope = self.pump.codec().decode_iopub_message(wire).unwrap();
            assert_eq!(envelope.parent_header, Some(self.parent.clone()));
            let rank = match envelope.payload {
                KernelIoPubPayload::Stream(s) if s.name == KernelStreamName::Stdout => 1,
                KernelIoPubPayload::Stream(_) => 2,
                KernelIoPubPayload::DisplayData(_) => 3,
                KernelIoPubPayload::ExecuteResult(_) | KernelIoPubPayload::Error(_) => 4,
                KernelIoPubPayload::Status(_) | KernelIoPubPayload::ExecuteInput(_) => continue,
            };
            assert!(rank >= previous, "family order changed within a segment");
            previous = rank;
        }
    }

    pub(super) fn at_prompt(&self, prompt: &str, prints: &[&str], messages: &[&str], trace: bool) {
        assert!(self.pump.adapter().awaiting_stdin());
        assert_eq!(
            self.pump.adapter().execution_state(),
            KernelExecutionState::Busy
        );
        assert_eq!(self.pump.adapter().execution_count(), 1);
        assert!(
            self.transport
                .messages
                .iter()
                .all(|w| w.channel != KernelChannel::Shell)
        );
        let segment = &self.transport.messages[self.segment_start..];
        let last = segment.last().expect("a prompt was sent");
        assert_eq!(
            last.channel,
            KernelChannel::Stdin,
            "all segment output precedes input_request"
        );
        assert_eq!(
            segment
                .iter()
                .filter(|w| w.channel == KernelChannel::Stdin)
                .count(),
            1
        );
        let input = self
            .pump
            .codec()
            .decode_input_request_message(last)
            .unwrap();
        assert_eq!(input.request.prompt, prompt);
        assert_eq!(input.parent_header, Some(self.parent.clone()));
        let effects = self.effects_since(self.segment_start);
        assert_eq!(effects.stdout, prints);
        assert_eq!(effects.stderr, messages);
        assert_eq!(!effects.trace.is_empty(), trace);
        assert!(
            effects.terminal.is_empty(),
            "no result or error while waiting"
        );
    }

    pub(super) fn answer(&mut self, answer: &str) {
        let input = self.pump.adapter().pending_input_request().unwrap().clone();
        let wire = self
            .pump
            .codec()
            .encode_input_reply_envelope(&input_reply(&input, answer))
            .unwrap();
        self.segment_start = self.transport.messages.len();
        self.pump
            .handle_wire_input_reply(&mut self.transport, wire)
            .unwrap();
        self.complete();
    }

    pub(super) fn interrupt(&mut self) {
        let mut interrupt = request("");
        interrupt.channel = KernelChannel::Control;
        interrupt.header =
            KernelMessageHeader::new("interrupt", "output-once", "tester", "interrupt_request");
        interrupt.request = KernelRequest::Interrupt(KernelInterruptRequest);
        let wire = self
            .pump
            .codec()
            .encode_request_envelope(&interrupt)
            .unwrap();
        self.segment_start = self.transport.messages.len();
        self.pump
            .handle_wire_request(&mut self.transport, wire)
            .unwrap();
        self.complete();
    }

    fn effects_since(&self, start: usize) -> Effects {
        Effects::from_outputs(
            self.transport.messages[start..]
                .iter()
                .filter(|wire| wire.channel == KernelChannel::IoPub)
                .filter_map(|wire| output(self.pump.codec().decode_iopub_message(wire).unwrap())),
        )
    }

    pub(super) fn assert_done(&mut self, expected: &KernelExecuteResponse) {
        let actual = self.effects_since(0);
        let expected_effects =
            Effects::from_outputs(expected.outputs.iter().cloned().map(wire_output));
        println!(
            "FAMILY stdout={} stderr={} trace={} terminal={}",
            actual.stdout == expected_effects.stdout,
            actual.stderr == expected_effects.stderr,
            actual.trace == expected_effects.trace,
            actual.terminal == expected_effects.terminal
        );
        assert_eq!(
            actual, expected_effects,
            "complete output sequences per family"
        );
        let replies = self
            .transport
            .messages
            .iter()
            .filter(|w| w.channel == KernelChannel::Shell)
            .collect::<Vec<_>>();
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0].routing_ids, [b"original-route".to_vec()]);
        let envelope = self
            .pump
            .codec()
            .decode_response_message(replies[0])
            .unwrap();
        assert_eq!(envelope.parent_header, Some(self.parent.clone()));
        let KernelResponse::Execute(reply) = envelope.response else {
            panic!("execute reply")
        };
        assert_eq!(reply.execution_count, Some(1));
        assert_eq!(reply.status, expected.status);
        let expected_reply = self
            .pump
            .codec()
            .decode_response_message(
                &self
                    .pump
                    .codec()
                    .encode_response_envelope(&crate::KernelResponseEnvelope {
                        channel: KernelChannel::Shell,
                        header: KernelMessageHeader::new(
                            "queued",
                            "output-once",
                            "tester",
                            "execute_reply",
                        ),
                        parent_header: Some(self.parent.clone()),
                        metadata: KernelMetadata::new(),
                        response: KernelResponse::Execute(expected.clone()),
                    })
                    .unwrap(),
            )
            .unwrap();
        let KernelResponse::Execute(expected_reply) = expected_reply.response else {
            unreachable!()
        };
        assert_eq!(reply.error, expected_reply.error);
        let payloads = self
            .transport
            .messages
            .iter()
            .filter(|w| w.channel == KernelChannel::IoPub)
            .map(|w| self.pump.codec().decode_iopub_message(w).unwrap().payload)
            .collect::<Vec<_>>();
        assert_eq!(
            payloads
                .iter()
                .filter(|p| matches!(p, KernelIoPubPayload::ExecuteInput(_)))
                .count(),
            1
        );
        assert_eq!(
            payloads
                .iter()
                .filter_map(|p| match p {
                    KernelIoPubPayload::Status(s) => Some(s.execution_state),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [KernelExecutionState::Busy, KernelExecutionState::Idle]
        );
        assert!(!self.pump.adapter().awaiting_stdin());
        assert!(!self.pump.adapter().execute_running());
        let next = self
            .pump
            .adapter_mut()
            .execute(KernelExecuteRequest::new("Print[\"prefix\"]; 2"));
        assert_eq!(Effects::from_outputs(next.outputs).stdout, ["prefix\n"]);
    }
}

pub(super) fn queued(source: &str, answers: &[&str], tracing: bool) -> KernelExecuteResponse {
    let mut session = HostSessionBuilder::new().with_tracing(tracing).build();
    for answer in answers {
        session.queue_input_response(*answer);
    }
    KernelAdapter::new(session).execute(KernelExecuteRequest::new(source))
}

pub(super) struct NestedGet {
    directory: std::path::PathBuf,
    pub(super) source: String,
}
impl NestedGet {
    pub(super) fn new() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "rum-jupyter-output-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let inner = directory.join("inner.rum");
        let outer = directory.join("outer.rum");
        std::fs::write(
            &inner,
            "Print[\"inner-before\"]; InputString[\"nested: \"]; Print[\"inner-after\"]; 42",
        )
        .unwrap();
        std::fs::write(
            &outer,
            format!(
                "Print[\"outer-before\"]; Get[{:?}]; Print[\"outer-after\"]; 42",
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

fn wire_output(value: KernelOutput) -> KernelOutput {
    let (msg_type, payload) = match value {
        KernelOutput::Stream(s) => ("stream", KernelIoPubPayload::Stream(s)),
        KernelOutput::DisplayData(d) => ("display_data", KernelIoPubPayload::DisplayData(d)),
        KernelOutput::ExecuteResult(r) => ("execute_result", KernelIoPubPayload::ExecuteResult(r)),
        KernelOutput::Error(e) => ("error", KernelIoPubPayload::Error(e)),
    };
    let codec = KernelWireCodec::new(KernelWireConfig::default());
    let envelope = KernelIoPubEnvelope {
        channel: KernelChannel::IoPub,
        header: KernelMessageHeader::new("queued", "output-once", "tester", msg_type),
        parent_header: None,
        metadata: KernelMetadata::new(),
        payload,
    };
    output(
        codec
            .decode_iopub_message(&codec.encode_iopub_envelope(&envelope).unwrap())
            .unwrap(),
    )
    .unwrap()
}
