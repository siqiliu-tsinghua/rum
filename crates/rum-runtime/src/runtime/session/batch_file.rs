use super::super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

/// The public batch cannot resume. Its outer file transaction remains rooted
/// until abandonment reaches it, after all inner scopes and file transactions.
#[derive(rum_derive::FrameRootTrace)]
struct PublicFileRollbackFrame {
    path: PathBuf,
    depth: usize,
    snapshot: LoaderSnapshot,
}

impl SuspensionFrame for PublicFileRollbackFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        _outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        unreachable!("a public file rollback obligation is abandonment-only")
    }
}

impl FrameAbandon for PublicFileRollbackFrame {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        context.rollback_public_file(&self.path, self.depth, &self.snapshot);
        FrameAbandonAction::Discard
    }
}

impl Session {
    #[cold]
    pub(super) fn defer_public_file_rollback(
        &mut self,
        path: &Path,
        depth: usize,
        snapshot: LoaderSnapshot,
    ) {
        crate::resume_or_suspend!(insert self, PublicFileRollbackFrame {
            path: path.to_owned(),
            depth,
            snapshot,
        });
    }

    pub(in crate::runtime) fn close_public_file(
        &mut self,
        path: &Path,
        depth: usize,
        rollback: Option<LoaderSnapshot>,
    ) {
        assert_eq!(
            self.load_stack.len(),
            depth + 1,
            "public files close inner to outer exactly once"
        );
        assert_eq!(
            self.load_stack.last().map(PathBuf::as_path),
            Some(path),
            "public file owner must match its load-stack entry"
        );
        self.load_stack.pop().expect("validated public file entry");
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::LoadStack);
        if let Some(snapshot) = rollback {
            self.restore_loader_snapshot(snapshot);
        }
    }
}
