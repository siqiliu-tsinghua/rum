use super::*;
use rum_core::Symbol;

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
