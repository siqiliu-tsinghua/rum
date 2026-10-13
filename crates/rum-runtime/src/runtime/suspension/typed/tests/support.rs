//! Shared dormant frame and finalizer fixtures.
use super::super::*;
use rum_core::Expr;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) struct Echo {
    root: Expr,
    resumes: Arc<AtomicUsize>,
}

impl FrameRootTrace for Echo {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        visitor.visit_expr(&self.root);
    }
}
impl FrameAbandon for Echo {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
impl TypedFrame for Echo {
    type Input = Expr;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        _: &mut Session,
        input: Result<Expr, EvalError>,
        token: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        assert_eq!(std::mem::size_of_val(&token), 0);
        assert_eq!(
            Arc::strong_count(&self),
            2,
            "adapter plus current call only"
        );
        self.resumes.fetch_add(1, Ordering::SeqCst);
        TypedAction::complete(input)
    }
    fn output_boundary(&self) -> OutputBoundary {
        OutputBoundary::all(2, 3, 4)
    }
}

pub(super) struct Finish(pub(super) Option<Expr>);
impl SlotFinalizer for Finish {
    type Output = Option<Expr>;
    fn finish(self: Box<Self>) -> Result<Self::Output, EvalError> {
        Ok(self.0)
    }
}

pub(super) fn echo() -> (Echo, Arc<AtomicUsize>) {
    let resumes = Arc::new(AtomicUsize::new(0));
    (
        Echo {
            root: Expr::string("retained"),
            resumes: resumes.clone(),
        },
        resumes,
    )
}
