use super::*;

#[derive(Clone, Copy, rum_derive::FrameRootTrace)]
struct ListConsumer;

impl OptionReentry for ListConsumer {
    fn symbol_name(self) -> &'static str {
        "TestOptions"
    }
    fn resume(
        self,
        _session: &mut Session,
        head: &Expr,
        _args: &[Expr],
        options: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(head.clone(), options))
    }
}

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
        reentry: ListConsumer,
        head: system_expr("List"),
        args: vec![Expr::symbol("argument")].into(),
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
            system_expr("List"),
            Expr::symbol("argument"),
            Expr::symbol("default"),
            Expr::symbol("explicit"),
            Expr::symbol("prefix")
        ]
    );
}

#[test]
fn option_loop_frame_resumes_with_another_arc_owner() {
    let frame = Arc::new(MergedOptionsSuspensionFrame {
        reentry: ListConsumer,
        head: system_expr("List"),
        args: vec![Expr::symbol("argument")].into(),
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
        value,
        Expr::call(
            system_expr("List"),
            vec![
                Expr::symbol("prefix"),
                Expr::symbol("resolved"),
                Expr::symbol("remaining")
            ]
        )
    );
    assert_eq!(retained.merged.as_ref(), &[Expr::symbol("prefix")]);
}
