use super::*;
use std::time::Instant;

/// A scalar marker, set only while a guest form is executing. Its file depth
/// and cursor identify the matching source/transaction retained by the frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct GuestFormIdentity {
    pub(super) file_depth: usize,
    pub(super) form: usize,
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(super) struct GuestCallerStack {
    entries: Vec<EvaluationStackEntry>,
    depth: usize,
}

impl GuestCallerStack {
    pub(super) fn capture(session: &Session) -> Self {
        Self {
            entries: session.evaluation_stack.clone(),
            depth: session.evaluation_depth,
        }
    }

    fn restore(&self, session: &mut Session) {
        session.replace_accounted_evaluation_stack(self.entries.clone());
        session.evaluation_depth = self.depth;
    }
}

/// Cold input-boundary payload. Failure packaging may replace last_message_list
/// and truncate handled errors, so save their actual contents, not stale indices.
/// No deadline or accounting initializer is called when this payload is installed.
#[derive(Clone, rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct GuestFormEnvironment {
    identity: GuestFormIdentity,
    message_checkpoint: Option<usize>,
    message_dirty: bool,
    last_message_list: Vec<RuntimeMessage>,
    handled_errors: Vec<RuntimeMessage>,
    eval_deadline: Option<Instant>,
    time_deadline: Option<Instant>,
    memory_limit: Option<usize>,
    interrupt_poll_counter: u64,
    interrupt_armed: bool,
}

impl Session {
    pub(in crate::runtime) fn capture_guest_form_environment(
        &self,
    ) -> Option<Box<GuestFormEnvironment>> {
        let identity = self.guest_form?;
        Some(Box::new(GuestFormEnvironment {
            identity,
            message_checkpoint: self.current_message_list_checkpoint,
            message_dirty: self.current_message_list_dirty,
            last_message_list: self.last_message_list.clone(),
            handled_errors: self.handled_non_loader_eval_errors.clone(),
            eval_deadline: self.eval_deadline,
            time_deadline: self.current_time_deadline,
            memory_limit: self.current_memory_limit,
            interrupt_poll_counter: self.interrupt_poll_counter,
            interrupt_armed: self.interrupt_armed,
        }))
    }
}

impl GuestFormEnvironment {
    pub(in crate::runtime) fn install(self, session: &mut Session) {
        session.guest_form = Some(self.identity);
        session.current_message_list_checkpoint = self.message_checkpoint;
        session.current_message_list_dirty = self.message_dirty;
        session.last_message_list = self.last_message_list;
        session.handled_non_loader_eval_errors = self.handled_errors;
        session.eval_deadline = self.eval_deadline;
        session.current_time_deadline = self.time_deadline;
        session.current_memory_limit = self.memory_limit;
        session.interrupt_poll_counter = self.interrupt_poll_counter;
        session.interrupt_armed = self.interrupt_armed;
        session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::LastMessageList,
        );
        session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
        );
    }
}

#[derive(Clone, rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct GuestFormScope {
    pub(super) message_checkpoint: usize,
    pub(super) print_checkpoint: usize,
    pub(super) handled_error_checkpoint: usize,
    previous_message_checkpoint: Option<usize>,
    previous_message_dirty: bool,
    previous_guest_form: Option<GuestFormIdentity>,
    caller_stack: Arc<GuestCallerStack>,
}

impl GuestFormScope {
    pub(super) fn enter(
        session: &mut Session,
        identity: GuestFormIdentity,
        caller_stack: Arc<GuestCallerStack>,
    ) -> Self {
        let message_checkpoint = session.messages.len();
        Self {
            message_checkpoint,
            print_checkpoint: session.prints.len(),
            handled_error_checkpoint: session.handled_non_loader_eval_errors.len(),
            previous_message_checkpoint: session
                .current_message_list_checkpoint
                .replace(message_checkpoint),
            previous_message_dirty: std::mem::replace(
                &mut session.current_message_list_dirty,
                false,
            ),
            previous_guest_form: session.guest_form.replace(identity),
            caller_stack,
        }
    }

    /// On a native unwind evaluate already restored its stack. Resumed and
    /// abandoned forms must reinstall the full caller prefix, including entries
    /// deliberately omitted by the input boundary's two-entry stack shortening.
    pub(in crate::runtime) fn restore(&self, session: &mut Session, restore_stack: bool) {
        session.current_message_list_checkpoint = self.previous_message_checkpoint;
        session.current_message_list_dirty = self.previous_message_dirty;
        session.guest_form = self.previous_guest_form;
        session
            .handled_non_loader_eval_errors
            .truncate(self.handled_error_checkpoint);
        session.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::HandledNonLoaderEvalErrors,
        );
        if restore_stack {
            self.caller_stack.restore(session);
        }
    }
}
