use super::*;

/// Original provider contents and resolved path. Both are root-free R11 data.
pub(super) struct GuestFileSource {
    pub(super) path: PathBuf,
    pub(super) text: String,
}
impl FrameRootTrace for GuestFileSource {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

#[derive(rum_derive::FrameRootTrace)]
pub(in crate::runtime) struct GuestFileTransaction {
    source: Arc<GuestFileSource>,
    snapshot: Arc<LoaderSnapshot>,
    depth: usize,
}

#[derive(rum_derive::FrameRootTrace)]
struct GuestFileCloseFrame {
    transaction: GuestFileTransaction,
}
impl SuspensionFrame for GuestFileCloseFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(session.finish_guest_file(&self.transaction, outcome))
    }
}
impl FrameAbandon for GuestFileCloseFrame {
    fn abandon(self: Arc<Self>, context: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        context.rollback_guest_file(&self.transaction);
        FrameAbandonAction::Discard
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct GetCompleteFrame {
    source: Arc<GuestFileSource>,
    message_start: usize,
    print_start: usize,
}
impl SuspensionFrame for GetCompleteFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(session.finish_guest_get(&self, outcome))
    }
}

impl Session {
    pub(in crate::runtime) fn start_guest_get(&mut self, path: PathBuf) -> Result<Expr, EvalError> {
        let source = self
            .file_io()
            .ok_or(EvalError::NoFileIoProvider)?
            .read_to_string(&path)
            .map_err(|source| EvalError::Io {
                path: path.display().to_string(),
                source,
            });
        let text = match source {
            Ok(text) => text,
            Err(error) => {
                self.emit_loader_message("Get", &error);
                return Ok(system_expr("$Failed"));
            }
        };
        let source = Arc::new(GuestFileSource { path, text });
        let completion = GetCompleteFrame {
            source: source.clone(),
            message_start: self.messages.len(),
            print_start: self.prints.len(),
        };
        let outcome = self.start_guest_file_load(source);
        if matches!(outcome, Err(EvalError::InputRequired { .. })) {
            crate::resume_or_suspend!(insert self, completion);
            outcome
        } else {
            self.finish_guest_get(&completion, outcome)
        }
    }

    fn start_guest_file_load(&mut self, source: Arc<GuestFileSource>) -> Result<Expr, EvalError> {
        let transaction = GuestFileTransaction {
            source: source.clone(),
            snapshot: Arc::new(self.capture_loader_snapshot()),
            depth: self.load_stack.len(),
        };
        self.load_stack.push(source.path.clone());
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::LoadStack);
        let outcome = self.start_guest_script(source);
        if matches!(outcome, Err(EvalError::InputRequired { .. })) {
            crate::resume_or_suspend!(insert self, GuestFileCloseFrame { transaction });
            outcome
        } else {
            self.finish_guest_file(&transaction, outcome)
        }
    }

    fn finish_guest_file(
        &mut self,
        transaction: &GuestFileTransaction,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        assert!(
            !matches!(outcome, Err(EvalError::InputRequired { .. })),
            "input does not close a file"
        );
        self.close_guest_file(transaction, outcome.is_err());
        outcome
    }

    pub(in crate::runtime) fn close_guest_file(
        &mut self,
        transaction: &GuestFileTransaction,
        rollback: bool,
    ) {
        assert_eq!(
            self.load_stack.len(),
            transaction.depth + 1,
            "guest files close inner to outer exactly once"
        );
        assert_eq!(
            self.load_stack.last(),
            Some(&transaction.source.path),
            "guest file owner must match its load-stack entry"
        );
        if rollback {
            // Clone only at terminal rollback: the queued immutable Arc protocol
            // retains the frame until this transition has finished.
            self.restore_loader_snapshot(transaction.snapshot.as_ref().clone());
        }
        self.load_stack.pop().expect("validated guest file entry");
        self.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::LoadStack);
        #[cfg(test)]
        super::tests::record_close(self, &transaction.source.path, transaction.depth, rollback);
    }

    fn finish_guest_get(
        &mut self,
        completion: &GetCompleteFrame,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        debug_assert!(completion.message_start <= self.messages.len());
        debug_assert!(completion.print_start <= self.prints.len());
        match outcome {
            Ok(value) => Ok(value), // Session-owned stream ranges are already committed.
            Err(
                error @ (EvalError::InputRequired { .. }
                | EvalError::Exit { .. }
                | EvalError::Cancelled
                | EvalError::TimeConstraintExceeded
                | EvalError::MemoryConstraintExceeded { .. }),
            ) => Err(error),
            Err(error) => {
                self.emit_loader_message_with_source("Get", &error, Some(&completion.source.text));
                Ok(system_expr("$Failed"))
            }
        }
    }
}
