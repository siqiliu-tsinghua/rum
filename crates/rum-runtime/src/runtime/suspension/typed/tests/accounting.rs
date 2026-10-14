use super::*;
use rum_core::Symbol;

use crate::runtime::debug_tools::resources::occurrence::{Bytes, session_breakdown};

pub(super) fn reconcile(session: &Session) {
    if session.current_memory_limit.is_some() {
        session.assert_maintained_memory_accounting();
    }
    assert_eq!(
        session_breakdown(session).total(),
        Bytes::Exact(session.current_memory_proxy_bytes_for_test())
    );
}

struct Empty;
impl FrameRootTrace for Empty {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for Empty {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
impl TypedFrame for Empty {
    type Input = Expr;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        s: &mut Session,
        result: Result<Expr, EvalError>,
        _: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        reconcile(s);
        TypedAction::complete(result)
    }
}

fn armed_pause() -> Session {
    let mut s = Session::new();
    s.current_memory_limit = Some(1_000_000_000);
    s.note_accounting_arm();
    let input = s.lower_text("InputString[\"accounting\"]").unwrap();
    assert!(matches!(
        s.evaluate(input),
        Err(EvalError::InputRequired { .. })
    ));
    reconcile(&s);
    s
}

#[test]
fn typed_layout_charges_two_arcs_and_real_queue_cells_in_both_models() {
    let mut s = armed_pause();
    let before = s.current_memory_proxy_bytes_for_test();
    let legacy_cells = s
        .pending_suspension
        .as_ref()
        .unwrap()
        .occurrence_frame_capacity()
        * std::mem::size_of::<Arc<dyn crate::runtime::suspension::SuspensionFrame>>();
    s.insert_typed_suspension_frame(Empty);
    let owner = s.pending_suspension.as_ref().unwrap();
    let capacity = owner.occurrence_frame_capacity();
    // Outer Arc<TypedAdapter<Empty>>: two counts plus one inner Arc pointer.
    // Inner Arc<Empty>: two counts plus a zero-sized Empty. No shared helper.
    let exact_layout = 5 * std::mem::size_of::<usize>();
    let cells = capacity * std::mem::size_of::<entry::Entry>();
    assert_eq!(
        s.current_memory_proxy_bytes_for_test() - before,
        exact_layout + cells - legacy_cells
    );
    reconcile(&s);
    assert_eq!(
        s.resume_suspended_evaluation("answer").unwrap(),
        Expr::string("answer")
    );
    reconcile(&s);
    assert!(s.pending_suspension.is_none());
    assert!(s.resuming_suspension.is_none());
}

#[test]
fn typed_admission_is_checked_below_at_and_above_projected_boundary() {
    for delta in [-1isize, 0, 1] {
        let mut s = armed_pause();
        let projected = s.pending_typed_frame_allocation_bytes::<Empty>().unwrap();
        assert!(
            projected >= 5 * std::mem::size_of::<usize>() + std::mem::size_of::<entry::Entry>()
        );
        let before = s.current_memory_proxy_bytes_for_test();
        let capacity = s
            .pending_suspension
            .as_ref()
            .unwrap()
            .occurrence_frame_capacity();
        s.current_memory_limit = Some(before + projected - 1 + (delta + 1) as usize);
        let admitted = s.enforce_active_memory_limit_for_projected_bytes(projected);
        assert_eq!(admitted.is_ok(), delta >= 0);
        assert_eq!(
            s.pending_suspension
                .as_ref()
                .unwrap()
                .occurrence_frame_capacity(),
            capacity,
            "admission occurs before allocation/insertion"
        );
        if admitted.is_ok() {
            s.insert_typed_suspension_frame(Empty);
            reconcile(&s);
            assert!(s.current_memory_proxy_bytes_for_test() <= s.current_memory_limit.unwrap());
        }
        s.current_memory_limit = Some(1_000_000_000);
        s.abandon_pending_suspension(
            crate::runtime::suspension::AbandonReason::HelperBestEffortDiscard,
        );
        reconcile(&s);
        assert_eq!(s.eval_request("1+1").unwrap().value, Expr::small_integer(2));
    }
}

#[test]
fn adapter_delegates_roots_and_capture_boundary_without_running_frame() {
    struct Roots(Vec<Expr>);
    impl FrameRootVisitor for Roots {
        fn visit_expr(&mut self, expr: &Expr) {
            self.0.push(expr.clone());
        }
        fn visit_symbol(&mut self, _: &Symbol) {
            panic!("no bare symbols");
        }
    }
    let (frame, calls) = echo();
    let erased = adapter::erase(frame);
    let mut roots = Roots(Vec::new());
    erased.trace_roots(&mut roots);
    assert_eq!(roots.0, vec![Expr::string("retained")]);
    let boundary = erased.output_boundary();
    assert_eq!(
        (boundary.prints, boundary.messages, boundary.trace),
        (Some(2), Some(3), Some(4))
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
