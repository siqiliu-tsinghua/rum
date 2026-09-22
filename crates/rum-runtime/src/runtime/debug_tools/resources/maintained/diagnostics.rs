//! Buffer appends charge the incoming occurrence and actual capacity growth.
use super::super::memory::{
    render_message_selector_memory_proxy_bytes, render_runtime_message_memory_proxy_bytes,
    render_trace_event_memory_proxy_bytes,
};
use super::*;

impl Session {
    pub(in crate::runtime) fn push_accounted_messages(&mut self, value: RuntimeMessage) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed {
            let value = &value;
            render_runtime_message_memory_proxy_bytes(value) - size_of::<RuntimeMessage>()
        } else {
            0
        };
        let capacity = self.messages.capacity();
        self.messages.push(value);
        if armed {
            self.charge_memory_owner(MemoryOwner::Messages, payload);
            self.charge_memory_owner(
                MemoryOwner::Messages,
                (self.messages.capacity() - capacity) * size_of::<RuntimeMessage>(),
            );
        }
    }

    pub(in crate::runtime) fn split_accounted_messages(
        &mut self,
        at: usize,
    ) -> Vec<RuntimeMessage> {
        let capacity = self.messages.capacity();
        let removed = self.messages.split_off(at);
        if self.current_memory_limit.is_some() {
            for value in &removed {
                self.release_memory_owner(
                    MemoryOwner::Messages,
                    render_runtime_message_memory_proxy_bytes(value) - size_of::<RuntimeMessage>(),
                );
            }
            self.release_memory_owner(
                MemoryOwner::Messages,
                (capacity - self.messages.capacity()) * size_of::<RuntimeMessage>(),
            );
        }
        removed
    }

    pub(in crate::runtime) fn push_accounted_prints(&mut self, value: String) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed {
            let value = &value;
            value.capacity()
        } else {
            0
        };
        let capacity = self.prints.capacity();
        self.prints.push(value);
        if armed {
            self.charge_memory_owner(MemoryOwner::Prints, payload);
            self.charge_memory_owner(
                MemoryOwner::Prints,
                (self.prints.capacity() - capacity) * size_of::<String>(),
            );
        }
    }

    pub(in crate::runtime) fn split_accounted_prints(&mut self, at: usize) -> Vec<String> {
        let capacity = self.prints.capacity();
        let removed = self.prints.split_off(at);
        if self.current_memory_limit.is_some() {
            for value in &removed {
                self.release_memory_owner(MemoryOwner::Prints, value.capacity());
            }
            self.release_memory_owner(
                MemoryOwner::Prints,
                (capacity - self.prints.capacity()) * size_of::<String>(),
            );
        }
        removed
    }

    pub(in crate::runtime) fn push_accounted_handled_non_loader_eval_errors(
        &mut self,
        value: RuntimeMessage,
    ) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed {
            let value = &value;
            render_runtime_message_memory_proxy_bytes(value) - size_of::<RuntimeMessage>()
        } else {
            0
        };
        let capacity = self.handled_non_loader_eval_errors.capacity();
        self.handled_non_loader_eval_errors.push(value);
        if armed {
            self.charge_memory_owner(MemoryOwner::HandledNonLoaderEvalErrors, payload);
            self.charge_memory_owner(
                MemoryOwner::HandledNonLoaderEvalErrors,
                (self.handled_non_loader_eval_errors.capacity() - capacity)
                    * size_of::<RuntimeMessage>(),
            );
        }
    }

    pub(in crate::runtime) fn push_accounted_trace_events(&mut self, value: TraceEvent) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed {
            let value = &value;
            render_trace_event_memory_proxy_bytes(value) - size_of::<TraceEvent>()
        } else {
            0
        };
        let capacity = self.trace_events.capacity();
        self.trace_events.push(value);
        if armed {
            self.charge_memory_owner(MemoryOwner::TraceEvents, payload);
            self.charge_memory_owner(
                MemoryOwner::TraceEvents,
                (self.trace_events.capacity() - capacity) * size_of::<TraceEvent>(),
            );
        }
    }

    pub(in crate::runtime) fn push_accounted_trace_event_depths(&mut self, value: usize) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed { 0 } else { 0 };
        let capacity = self.trace_event_depths.capacity();
        self.trace_event_depths.push(value);
        if armed {
            self.charge_memory_owner(MemoryOwner::TraceEventDepths, payload);
            self.charge_memory_owner(
                MemoryOwner::TraceEventDepths,
                (self.trace_event_depths.capacity() - capacity) * size_of::<usize>(),
            );
        }
    }

    pub(in crate::runtime) fn push_accounted_disabled_message_selectors(
        &mut self,
        value: MessageSelector,
    ) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed {
            let value = &value;
            render_message_selector_memory_proxy_bytes(value) - size_of::<MessageSelector>()
        } else {
            0
        };
        let capacity = self.disabled_message_selectors.capacity();
        self.disabled_message_selectors.push(value);
        if armed {
            self.charge_memory_owner(MemoryOwner::DisabledMessageSelectors, payload);
            self.charge_memory_owner(
                MemoryOwner::DisabledMessageSelectors,
                (self.disabled_message_selectors.capacity() - capacity)
                    * size_of::<MessageSelector>(),
            );
        }
    }
}
