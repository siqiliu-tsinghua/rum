//! Visibility and rollback of provisional buffered capture output.
use super::{Session, SuspendedEvaluation};
use crate::runtime::debug_tools::resources::MemoryOwner;
use crate::runtime::model::OutputMark;

/// Each buffered channel's provisional output, stamped after its mark; `None` owns nothing.
/// Queried only when packaging a pause or abandoning its continuation.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OutputBoundary {
    pub(in crate::runtime) prints: Option<OutputMark>,
    pub(in crate::runtime) messages: Option<OutputMark>,
    pub(in crate::runtime) trace: Option<OutputMark>,
}

impl OutputBoundary {
    pub(in crate::runtime) fn all(
        prints: OutputMark,
        messages: OutputMark,
        trace: OutputMark,
    ) -> Self {
        Self {
            prints: Some(prints),
            messages: Some(messages),
            trace: Some(trace),
        }
    }

    pub(in crate::runtime) fn messages(checkpoint: OutputMark) -> Self {
        Self {
            messages: Some(checkpoint),
            ..Self::default()
        }
    }

    pub(in crate::runtime) fn trace(checkpoint: OutputMark) -> Self {
        Self {
            trace: Some(checkpoint),
            ..Self::default()
        }
    }

    fn include(&mut self, other: Self) {
        fn earliest(a: Option<OutputMark>, b: Option<OutputMark>) -> Option<OutputMark> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            }
        }
        self.prints = earliest(self.prints, other.prints);
        self.messages = earliest(self.messages, other.messages);
        self.trace = earliest(self.trace, other.trace);
    }
}

impl SuspendedEvaluation {
    fn output_boundary(&self) -> OutputBoundary {
        let mut boundary = OutputBoundary::default();
        self.frames.visit(|frame| {
            boundary.include(frame.output_boundary());
        });
        boundary
    }
}

impl Session {
    pub(in crate::runtime) fn pending_output_boundary(&self) -> OutputBoundary {
        match self.pending_suspension.as_ref() {
            Some(pending) => pending.output_boundary(),
            None => OutputBoundary::default(),
        }
    }

    /// Discard a capture's provisional suffix. The abandonment driver applies
    /// this inner to outer before restoring each frame's remaining state.
    pub(in crate::runtime) fn rollback_capture_output(&mut self, boundary: OutputBoundary) {
        if let Some(checkpoint) = boundary.prints {
            self.prints.truncate_after(checkpoint);
            self.refresh_memory_owner(MemoryOwner::Prints);
        }
        if let Some(checkpoint) = boundary.messages {
            self.messages.truncate_after(checkpoint);
            self.refresh_memory_owner(MemoryOwner::Messages);
        }
        if let Some(checkpoint) = boundary.trace {
            self.trace_events.truncate_after(checkpoint);
            self.refresh_memory_owner(MemoryOwner::TraceEvents);
        }
    }
}
