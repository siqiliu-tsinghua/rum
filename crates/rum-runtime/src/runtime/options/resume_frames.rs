use super::*;
use crate::runtime::suspension::{FrameAction, SuspensionFrame};
use rum_core::SharedArgs;
use std::sync::Arc;

/// Rebuild the immediate option rule after its delayed value resumes.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct RuntimeOptionSuspensionFrame {
    pub(super) name: Expr,
}

impl SuspensionFrame for RuntimeOptionSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        FrameAction::complete(
            outcome.map(|value| Expr::call(system_expr("Rule"), vec![self.name.clone(), value])),
        )
    }
}

/// Frozen loop inputs and the already evaluated prefix are all roots. Defaults
/// are captured once, so guest changes to Options do not restart the loop.
#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MergedOptionsSuspensionFrame {
    pub(super) target: Symbol,
    pub(super) defaults: SharedArgs,
    pub(super) explicit: SharedArgs,
    pub(super) next_index: usize,
    pub(super) merged: SharedArgs,
}

impl SuspensionFrame for MergedOptionsSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction {
        let resumed = outcome.and_then(|value| {
            session
                .resume_merged_options_after_value(&self, value)
                .map(ContinuationValue::into_continuation_expr)
        });
        FrameAction::complete(resumed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Roots {
        exprs: Vec<Expr>,
        symbols: Vec<Symbol>,
    }

    impl FrameRootVisitor for Roots {
        fn visit_expr(&mut self, expr: &Expr) {
            self.exprs.push(expr.clone());
        }

        fn visit_symbol(&mut self, symbol: &Symbol) {
            self.symbols.push(symbol.clone());
        }
    }

    #[test]
    fn option_loop_frame_traces_inputs_and_accumulated_prefix() {
        let frame = MergedOptionsSuspensionFrame {
            target: Symbol::system("StringReplace"),
            defaults: vec![Expr::symbol("default")].into(),
            explicit: vec![Expr::symbol("explicit")].into(),
            next_index: 1,
            merged: vec![Expr::symbol("prefix")].into(),
        };
        let mut roots = Roots::default();
        frame.trace_roots(&mut roots);
        assert_eq!(roots.symbols, vec![frame.target]);
        assert_eq!(
            roots.exprs,
            vec![
                Expr::symbol("default"),
                Expr::symbol("explicit"),
                Expr::symbol("prefix")
            ]
        );
    }

    #[test]
    fn option_loop_frame_resumes_with_another_arc_owner() {
        let frame = Arc::new(MergedOptionsSuspensionFrame {
            target: Symbol::system("StringReplace"),
            defaults: vec![Expr::symbol("already_evaluated")].into(),
            explicit: vec![Expr::symbol("pending"), Expr::symbol("remaining")].into(),
            next_index: 2,
            merged: vec![Expr::symbol("prefix")].into(),
        });
        let retained = frame.clone();
        let FrameAction::Complete(Ok(value)) =
            frame.resume(&mut Session::new(), Ok(Expr::symbol("resolved")))
        else {
            panic!("option loop should complete without requiring unique ownership");
        };
        assert_eq!(
            Vec::<Expr>::from_continuation_expr(value),
            vec![
                Expr::symbol("prefix"),
                Expr::symbol("resolved"),
                Expr::symbol("remaining")
            ]
        );
        assert_eq!(retained.merged.as_ref(), &[Expr::symbol("prefix")]);
    }
}
