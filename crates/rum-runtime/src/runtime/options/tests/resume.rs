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
    let mut session = Session::new();
    let FrameAction::Complete(Ok(refused)) = frame
        .clone()
        .resume(&mut session, Ok(Expr::symbol("resolved")))
    else {
        panic!("unregistered resume must refuse");
    };
    assert_eq!(refused, system_expr("$Failed"));
    assert_eq!(session.messages().last().unwrap().tag, "optreentry");
    session.register_option_reentry::<ListConsumer>();
    let FrameAction::Complete(Ok(value)) = frame.resume(&mut session, Ok(Expr::symbol("resolved")))
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

#[test]
fn option_reentry_refused_until_registered() {
    let mut session = Session::new();
    let head = system_expr("List");
    let run = |session: &mut Session| {
        session
            .merged_option_exprs_for_system_symbol(
                ListConsumer,
                &head,
                &[],
                &[Expr::small_integer(17)],
            )
            .unwrap()
    };
    assert_eq!(run(&mut session), system_expr("$Failed"));
    assert_eq!(session.messages().last().unwrap().tag, "optreentry");
    session.register_option_reentry::<ListConsumer>();
    session.register_option_reentry::<ListConsumer>();
    assert_eq!(
        run(&mut session),
        Expr::call(head, vec![Expr::small_integer(17)])
    );
    println!("ListConsumer: unregistered => General::optreentry / $Failed; registered => List[17]");
}
