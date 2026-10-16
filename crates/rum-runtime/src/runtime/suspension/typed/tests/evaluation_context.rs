//! Queue context across widening, chains, driver deferrals and fresh tokens.
use super::*;
use crate::runtime::EvaluationStackEntry;

type Context = (Vec<(String, u64)>, usize, u64);
fn context(s: &Session) -> Context {
    (
        s.evaluation_stack
            .iter()
            .map(|e| (e.expr.to_string(), e.epoch))
            .collect(),
        s.evaluation_depth,
        s.evaluation_epoch,
    )
}
fn seeded() -> Session {
    let mut s = Session::new();
    s.evaluation_epoch = 7;
    s.evaluation_stack = ["outer", "middle", "inner"]
        .into_iter()
        .enumerate()
        .map(|(i, name)| EvaluationStackEntry {
            expr: Expr::symbol(name),
            epoch: i as u64,
        })
        .collect();
    s.evaluation_depth = 9;
    s.current_memory_limit = Some(1_000_000_000);
    s.note_accounting_arm();
    assert!(matches!(
        s.eval_input_string(&[Expr::string("context")]),
        Err(EvalError::InputRequired { .. })
    ));
    s
}
fn unwind(s: &mut Session, length: usize, depth: usize) {
    s.truncate_accounted_evaluation_stack(length);
    s.evaluation_depth = depth;
    super::super::accounting::reconcile(s);
}
#[derive(Clone, Copy)]
enum Route {
    Complete,
    Evaluate,
    Token,
    Finalize,
}
struct ContextFrame {
    expected: Context,
    route: Route,
    seen: Events,
    name: &'static str,
}
impl FrameRootTrace for ContextFrame {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for ContextFrame {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        self.seen.lock().unwrap().push("abandoned");
        FrameAbandonAction::Discard
    }
}
impl ContextFrame {
    fn check(&self, s: &Session) {
        assert_eq!(context(s), self.expected, "{}", self.name);
        super::super::accounting::reconcile(s);
        self.seen.lock().unwrap().push(self.name);
    }
    fn next(&self) -> Self {
        Self {
            expected: self.expected.clone(),
            route: Route::Finalize,
            seen: self.seen.clone(),
            name: "next",
        }
    }
}
impl TypedFrame for ContextFrame {
    type Input = Expr;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        s: &mut Session,
        value: Result<Expr, EvalError>,
        token: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        self.check(s);
        match self.route {
            Route::Complete => TypedAction::complete(value),
            Route::Finalize => TypedAction::finalize(CheckFinish(value)),
            Route::Evaluate => TypedAction::evaluate(
                s.lower_text("InputString[\"deferred\"]").unwrap(),
                self.next(),
            ),
            Route::Token => {
                let produced = s.eval_input_string(&[Expr::string("token")]);
                let witness = slot_witness(&produced);
                let request = request(produced);
                TypedAction::finish_work(
                    Err::<Expr, _>(token.suspend(s, witness, request, self.next())),
                    |value| CheckFinish(Ok(value)),
                )
            }
        }
    }
}
struct CheckFinish(Result<Expr, EvalError>);
impl SlotFinalizer for CheckFinish {
    type Output = Expr;
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        self.0
    }
}
impl crate::runtime::suspension::PureFrameFinalizer for CheckFinish {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        self.0
    }
}
impl crate::runtime::suspension::SuspensionFrame for ContextFrame {
    fn resume(
        self: Arc<Self>,
        s: &mut Session,
        value: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        use crate::runtime::suspension::FrameAction;
        self.check(s);
        match self.route {
            Route::Complete => FrameAction::complete(value),
            Route::Finalize => FrameAction::finalize(CheckFinish(value)),
            Route::Evaluate => FrameAction::evaluate(
                s.lower_text("InputString[\"deferred\"]").unwrap(),
                self.next(),
            ),
            Route::Token => unreachable!("fresh tokens belong to the typed arm"),
        }
    }
}
fn frame(s: &Session, seen: &Events, name: &'static str, route: Route) -> ContextFrame {
    ContextFrame {
        expected: context(s),
        route,
        seen: seen.clone(),
        name,
    }
}

#[test]
fn g14_widening_and_chain_preserve_every_cell_record() {
    let mut s = seeded();
    let seen = Events::default();
    s.insert_suspension_frame(frame(&s, &seen, "legacy", Route::Complete));
    let produced: Result<Expr, EvalError> = Err(EvalError::InputRequired {
        request: s.pending_input_request().unwrap().clone(),
    });
    let witness = slot_witness(&produced);
    let request = request(produced);
    let _ = witness
        .chain(frame(&s, &seen, "chain-one", Route::Complete))
        .then(frame(&s, &seen, "chain-two", Route::Complete))
        .suspend(&mut s, request);
    unwind(&mut s, 1, 2);
    s.insert_suspension_frame(frame(&s, &seen, "outer", Route::Complete));
    unwind(&mut s, 0, 0);
    assert_eq!(
        s.resume_suspended_evaluation("answer").unwrap(),
        Expr::string("answer")
    );
    assert_eq!(
        *seen.lock().unwrap(),
        ["legacy", "chain-one", "chain-two", "outer"]
    );
    super::super::accounting::reconcile(&s);
}

#[test]
fn g14_both_deferred_evaluate_arms_and_fresh_tokens_keep_nested_context() {
    for (typed, route) in [
        (false, Route::Evaluate),
        (true, Route::Evaluate),
        (true, Route::Token),
    ] {
        let mut s = seeded();
        let seen = Events::default();
        let first = frame(&s, &seen, "first", route);
        if typed {
            s.insert_typed_suspension_frame(first);
        } else {
            s.insert_suspension_frame(first);
        }
        unwind(&mut s, 1, 2);
        s.insert_typed_suspension_frame(frame(&s, &seen, "suffix", Route::Complete));
        unwind(&mut s, 0, 0);
        assert!(matches!(
            s.resume_suspended_evaluation("one"),
            Err(EvalError::InputRequired { .. })
        ));
        super::super::accounting::reconcile(&s);
        assert_eq!(
            s.resume_suspended_evaluation("two").unwrap(),
            Expr::string("two")
        );
        assert_eq!(*seen.lock().unwrap(), ["first", "next", "suffix"]);
        super::super::accounting::reconcile(&s);
    }
}

#[test]
fn g14_return_event_rejects_wrong_success_slot_and_preserves_error() {
    for wrong_slot in [false, true] {
        let mut s = seeded();
        let seen = Events::default();
        if wrong_slot {
            s.insert_typed_suspension_frame(Convert::<
                Expr,
                super::super::fault_support::StateBool,
            > {
                map: |value| value.map(|_| super::super::fault_support::StateBool),
                events: seen.clone(),
                name: "state",
            });
        } else {
            s.insert_typed_suspension_frame(Convert::<Expr, Expr> {
                map: |_| Err(EvalError::Abort),
                events: seen.clone(),
                name: "error",
            });
        }
        unwind(&mut s, 1, 2);
        let produced: Result<Expr, EvalError> = Err(EvalError::InputRequired {
            request: s.pending_input_request().unwrap().clone(),
        });
        let witness = slot_witness(&produced);
        let request = request(produced);
        let consumer = frame(&s, &seen, "consumer", Route::Complete);
        let _ = witness.suspend_evaluation_return(&mut s, request, consumer);
        unwind(&mut s, 0, 0);
        // An error must retain its identity even when a success check would expire.
        s.current_time_deadline =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
        let probe = s.continuation_fault_probe();
        let error = s.resume_suspended_evaluation("answer").unwrap_err();
        if wrong_slot {
            assert!(matches!(error, EvalError::ContinuationSlotMismatch(_)));
            assert_eq!(probe.occurrences(), 1);
            assert_eq!(*seen.lock().unwrap(), ["state", "abandoned"]);
            s.continuation_fault_outcome::<Expr>(Err(error));
        } else {
            assert!(matches!(error, EvalError::Abort));
            assert_eq!(probe.occurrences(), 0);
            assert_eq!(*seen.lock().unwrap(), ["error", "consumer"]);
        }
        assert!(s.resuming_suspension.is_none());
        assert!(s.pending_suspension.is_none());
        super::super::accounting::reconcile(&s);
    }
}
