use super::{
    KernelAdapter, KernelChannel, KernelQueuedTransport, KernelTransport, KernelWireCodec,
    KernelWireError, KernelWireMessage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelPumpEvent {
    Channel(KernelChannel),
    ExecuteCompletion,
}

pub struct KernelEventPump {
    adapter: KernelAdapter,
    codec: KernelWireCodec,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KernelTransportLoopReport {
    pub control_messages: usize,
    pub shell_messages: usize,
    pub stdin_messages: usize,
    pub execute_completions: usize,
    pub stopped_for_shutdown: bool,
}

impl KernelTransportLoopReport {
    pub const fn total_messages(&self) -> usize {
        self.control_messages + self.shell_messages + self.stdin_messages + self.execute_completions
    }
}

impl KernelEventPump {
    pub fn new(adapter: KernelAdapter, codec: KernelWireCodec) -> Self {
        Self { adapter, codec }
    }

    pub fn adapter(&self) -> &KernelAdapter {
        &self.adapter
    }

    pub fn adapter_mut(&mut self) -> &mut KernelAdapter {
        &mut self.adapter
    }

    pub fn codec(&self) -> &KernelWireCodec {
        &self.codec
    }

    pub fn handle_wire_request<T: KernelTransport>(
        &mut self,
        transport: &mut T,
        request: KernelWireMessage,
    ) -> Result<(), KernelWireError> {
        let routing_ids = request.routing_ids.clone();
        let envelope = self.codec.decode_request_message(&request)?;
        let dispatch = self
            .adapter
            .dispatch_request_with_routing(envelope, routing_ids.clone());
        self.codec
            .send_dispatch_with_routing(transport, &dispatch, routing_ids)
    }

    pub fn handle_wire_input_reply<T: KernelTransport>(
        &mut self,
        transport: &mut T,
        reply: KernelWireMessage,
    ) -> Result<(), KernelWireError> {
        let envelope = self.codec.decode_input_reply_message(&reply)?;
        if let Some((dispatch, routing_ids)) = self
            .adapter
            .handle_input_reply_with_routing(envelope)
            .map_err(KernelWireError::from)?
        {
            self.codec
                .send_dispatch_with_routing(transport, &dispatch, routing_ids)?;
        }
        Ok(())
    }

    pub fn pump_execute_completion<T: KernelTransport>(
        &mut self,
        transport: &mut T,
    ) -> Result<bool, KernelWireError> {
        let Some((dispatch, routing_ids)) = self.adapter.poll_execute_completion_with_routing()
        else {
            return Ok(false);
        };

        self.codec
            .send_dispatch_with_routing(transport, &dispatch, routing_ids)?;
        Ok(true)
    }

    pub fn pump_once<T: KernelQueuedTransport>(
        &mut self,
        transport: &mut T,
        channel: KernelChannel,
    ) -> Result<bool, KernelWireError> {
        if let Some(error) = transport.take_pending_error() {
            return Err(error);
        }

        if self.adapter.execute_running() && channel != KernelChannel::Control {
            return Ok(false);
        }

        if let Some(request) = transport.pop_incoming(channel) {
            match channel {
                KernelChannel::Shell | KernelChannel::Control => {
                    self.handle_wire_request(transport, request)?;
                }
                KernelChannel::Stdin => {
                    self.handle_wire_input_reply(transport, request)?;
                }
                KernelChannel::IoPub => return Ok(false),
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn pump_next<T: KernelQueuedTransport>(
        &mut self,
        transport: &mut T,
    ) -> Result<Option<KernelPumpEvent>, KernelWireError> {
        if self.pump_execute_completion(transport)? {
            return Ok(Some(KernelPumpEvent::ExecuteCompletion));
        }

        if self.adapter.shutdown_requested() {
            return Ok(None);
        }

        let channels: &[KernelChannel] = if self.adapter.execute_running() {
            &[KernelChannel::Control]
        } else if self.adapter.awaiting_stdin() {
            &[
                KernelChannel::Control,
                KernelChannel::Stdin,
                KernelChannel::Shell,
            ]
        } else {
            &[
                KernelChannel::Control,
                KernelChannel::Shell,
                KernelChannel::Stdin,
            ]
        };

        for &channel in channels {
            if self.pump_once(transport, channel)? {
                return Ok(Some(KernelPumpEvent::Channel(channel)));
            }
        }

        Ok(None)
    }

    pub fn pump_until_idle<T: KernelQueuedTransport>(
        &mut self,
        transport: &mut T,
    ) -> Result<KernelTransportLoopReport, KernelWireError> {
        let mut report = KernelTransportLoopReport::default();

        loop {
            if self.adapter.shutdown_requested() {
                report.stopped_for_shutdown = transport.has_pending_incoming();
                break;
            }

            let Some(channel) = self.pump_next(transport)? else {
                if self.adapter.execute_running() {
                    std::thread::yield_now();
                    continue;
                }
                break;
            };

            match channel {
                KernelPumpEvent::Channel(KernelChannel::Control) => report.control_messages += 1,
                KernelPumpEvent::Channel(KernelChannel::Shell) => report.shell_messages += 1,
                KernelPumpEvent::Channel(KernelChannel::Stdin) => report.stdin_messages += 1,
                KernelPumpEvent::Channel(KernelChannel::IoPub) => {}
                KernelPumpEvent::ExecuteCompletion => report.execute_completions += 1,
            }

            if self.adapter.shutdown_requested() {
                report.stopped_for_shutdown = transport.has_pending_incoming();
                break;
            }
        }

        Ok(report)
    }
}
