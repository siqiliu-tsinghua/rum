//! S2 insertion routes execute real queues; no production frame is converted.
use super::*;
use crate::runtime::{RuntimeInputRequest, suspension::ContinuationValue};
use std::sync::{Mutex, OnceLock, Weak};

type Events = Arc<Mutex<Vec<&'static str>>>;

struct Convert<I: SlotValue, O: SlotValue> {
    map: fn(Result<I, EvalError>) -> Result<O, EvalError>,
    events: Events,
    name: &'static str,
}

impl<I: SlotValue, O: SlotValue> FrameRootTrace for Convert<I, O> {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl<I: SlotValue, O: SlotValue> FrameAbandon for Convert<I, O> {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        self.events.lock().unwrap().push("abandon-convert");
        FrameAbandonAction::Discard
    }
}
impl<I: SlotValue, O: SlotValue> TypedFrame for Convert<I, O> {
    type Input = I;
    type Output = O;
    fn resume<'r>(
        self: Arc<Self>,
        _: &mut Session,
        input: Result<I, EvalError>,
        _: ContinueToken<'r, Self>,
    ) -> TypedAction<O> {
        self.events.lock().unwrap().push(self.name);
        TypedAction::complete((self.map)(input))
    }
}

fn input(session: &mut Session) -> Result<Expr, EvalError> {
    let expr = session.lower_text("InputString[\"typed\"]").unwrap();
    session.evaluate(expr)
}

fn request(result: Result<Expr, EvalError>) -> RuntimeInputRequest {
    match result {
        Err(EvalError::InputRequired { request }) => request,
        other => panic!("expected actual suspension: {other:?}"),
    }
}

fn option(events: &Events) -> Convert<Expr, Option<Expr>> {
    Convert {
        map: |value| value.map(Some),
        events: events.clone(),
        name: "option",
    }
}
fn unwrap(events: &Events) -> Convert<Option<Expr>, Expr> {
    Convert {
        map: |value| value.map(Option::unwrap),
        events: events.clone(),
        name: "unwrap",
    }
}
fn identity(events: &Events, name: &'static str) -> Convert<Expr, Expr> {
    Convert {
        map: |value| value,
        events: events.clone(),
        name,
    }
}

fn macro_optional(session: &mut Session, events: &Events) -> Result<Option<Expr>, EvalError> {
    crate::resume_typed_or_suspend!(session, input(session),
        ok(value) => Ok(Some(value)),
        suspend => {
            events.lock().unwrap().push("construct");
            option(events)
        },
    )
}

#[test]
fn macro_is_cold_and_a_witnessed_chain_returns_its_last_output() {
    let mut s = Session::new();
    let events = Events::default();
    s.queue_input_response("queued");
    assert_eq!(
        macro_optional(&mut s, &events).unwrap(),
        Some(Expr::string("queued"))
    );
    assert!(
        events.lock().unwrap().is_empty(),
        "ordinary path must not build the frame"
    );

    let produced = macro_optional(&mut s, &events);
    let witness = slot_witness(&produced);
    let Err(EvalError::InputRequired { request }) = produced else {
        panic!("pause")
    };
    let stopped = witness
        .chain(unwrap(&events))
        .then(identity(&events, "last"))
        .suspend(&mut s, request);
    assert!(matches!(stopped, Err(EvalError::InputRequired { .. })));
    assert_eq!(
        s.resume_suspended_evaluation("resumed").unwrap(),
        Expr::string("resumed")
    );
    assert_eq!(
        *events.lock().unwrap(),
        ["construct", "option", "unwrap", "last"]
    );
    assert!(s.pending_input_request().is_none());
}

struct Legacy {
    encode: bool,
    events: Events,
}
impl FrameRootTrace for Legacy {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for Legacy {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
impl crate::runtime::suspension::SuspensionFrame for Legacy {
    fn resume(
        self: Arc<Self>,
        _: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        self.events.lock().unwrap().push("legacy");
        crate::runtime::suspension::FrameAction::complete(outcome.map(|value| {
            if self.encode {
                Some(value).into_continuation_expr()
            } else {
                Option::<Expr>::from_continuation_expr(value).unwrap()
            }
        }))
    }
}

#[test]
fn legacy_and_typed_neighbors_exchange_the_unchanged_value_wire() {
    for legacy_first in [false, true] {
        let mut s = Session::new();
        let events = Events::default();
        let produced = input(&mut s);
        let witness = slot_witness(&produced);
        let request = request(produced);
        if legacy_first {
            s.insert_suspension_frame(Legacy {
                encode: true,
                events: events.clone(),
            });
            s.insert_typed_suspension_frame(unwrap(&events));
        } else {
            let _: Result<Option<Expr>, _> = witness.suspend(&mut s, request, option(&events));
            s.insert_suspension_frame(Legacy {
                encode: false,
                events: events.clone(),
            });
        }
        assert_eq!(
            s.resume_suspended_evaluation("wire").unwrap(),
            Expr::string("wire")
        );
        let expected = if legacy_first {
            vec!["legacy", "unwrap"]
        } else {
            vec!["option", "legacy"]
        };
        assert_eq!(*events.lock().unwrap(), expected);
    }
}

struct Evaluate {
    events: Events,
}
impl FrameRootTrace for Evaluate {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for Evaluate {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
impl TypedFrame for Evaluate {
    type Input = Expr;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        session: &mut Session,
        value: Result<Expr, EvalError>,
        _: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        assert_eq!(value.unwrap(), Expr::string("first"));
        self.events.lock().unwrap().push("evaluate");
        TypedAction::evaluate(
            session.lower_text("InputString[\"nested\"]").unwrap(),
            identity(&self.events, "successor"),
        )
    }
}

#[test]
fn evaluate_successor_and_nested_owner_precede_the_unrun_suffix() {
    let mut s = Session::new();
    s.current_memory_limit = Some(1_000_000_000);
    s.note_accounting_arm();
    let events = Events::default();
    let produced = input(&mut s);
    let witness = slot_witness(&produced);
    let request = request(produced);
    let _ = witness
        .chain(Evaluate {
            events: events.clone(),
        })
        .then(identity(&events, "suffix"))
        .suspend(&mut s, request);
    assert!(matches!(
        s.resume_suspended_evaluation("first"),
        Err(EvalError::InputRequired { .. })
    ));
    super::accounting::reconcile(&s);
    assert_eq!(*events.lock().unwrap(), ["evaluate"]);
    assert_eq!(
        s.resume_suspended_evaluation("second").unwrap(),
        Expr::string("second")
    );
    assert_eq!(*events.lock().unwrap(), ["evaluate", "successor", "suffix"]);
    super::accounting::reconcile(&s);
}

struct Repeat {
    remaining: usize,
    events: Events,
}
impl FrameRootTrace for Repeat {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for Repeat {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
fn repeat_work<'r>(
    s: &mut Session,
    fresh: Repeat,
    token: ContinueToken<'r, Repeat>,
) -> Result<Expr, ResumeStop<Option<Expr>>> {
    if fresh.remaining == 0 {
        return Ok(Expr::small_integer(4));
    }
    let outcome = input(s);
    let witness = slot_witness(&outcome);
    match outcome {
        Err(EvalError::InputRequired { request }) => Err(token.suspend(
            s,
            witness,
            request,
            Repeat {
                remaining: fresh.remaining - 1,
                events: fresh.events,
            },
        )),
        other => panic!("repeat must pause: {other:?}"),
    }
}
impl TypedFrame for Repeat {
    type Input = Expr;
    type Output = Option<Expr>;
    fn resume<'r>(
        self: Arc<Self>,
        s: &mut Session,
        input: Result<Expr, EvalError>,
        token: ContinueToken<'r, Self>,
    ) -> TypedAction<Option<Expr>> {
        assert_eq!(input.unwrap(), Expr::string("answer"));
        self.events.lock().unwrap().push("repeat");
        let fresh = Repeat {
            remaining: self.remaining,
            events: self.events.clone(),
        };
        TypedAction::finish_work(repeat_work(s, fresh, token), |value| Finish(Some(value)))
    }
}

#[test]
fn repeat_token_freezes_fresh_state_and_keeps_the_deferred_output_type() {
    let mut s = Session::new();
    let events = Events::default();
    let produced = input(&mut s);
    let witness = slot_witness(&produced);
    let request = request(produced);
    let _ = witness
        .chain(Repeat {
            remaining: 3,
            events: events.clone(),
        })
        .then(unwrap(&events))
        .suspend(&mut s, request);
    for _ in 0..3 {
        assert!(matches!(
            s.resume_suspended_evaluation("answer"),
            Err(EvalError::InputRequired { .. })
        ));
    }
    assert_eq!(
        s.resume_suspended_evaluation("answer").unwrap(),
        Expr::small_integer(4)
    );
    assert_eq!(
        *events.lock().unwrap(),
        ["repeat", "repeat", "repeat", "repeat", "unwrap"]
    );
}

struct DropBeforeFinish {
    inner_lifetime: Arc<()>,
    outer: Arc<OnceLock<Weak<dyn adapter::ErasedFrame>>>,
}
impl FrameRootTrace for DropBeforeFinish {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for DropBeforeFinish {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
struct CheckDrop {
    inner: Weak<()>,
    outer: Weak<dyn adapter::ErasedFrame>,
    result: Result<Expr, EvalError>,
}
impl SlotFinalizer for CheckDrop {
    type Output = Expr;
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        assert!(self.inner.upgrade().is_none(), "inner frame remains owned");
        assert!(
            self.outer.upgrade().is_none(),
            "outer adapter remains owned"
        );
        self.result
    }
}
impl TypedFrame for DropBeforeFinish {
    type Input = Expr;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        _: &mut Session,
        result: Result<Expr, EvalError>,
        _: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        TypedAction::finalize(CheckDrop {
            inner: Arc::downgrade(&self.inner_lifetime),
            outer: self.outer.get().unwrap().clone(),
            result,
        })
    }
}

#[test]
fn driver_drops_both_owning_arcs_before_running_the_finalizer() {
    let mut s = Session::new();
    s.current_memory_limit = Some(1_000_000_000);
    s.note_accounting_arm();
    let _ = request(input(&mut s));
    let outer = Arc::new(OnceLock::new());
    let frame = adapter::erase(DropBeforeFinish {
        inner_lifetime: Arc::new(()),
        outer: outer.clone(),
    });
    assert!(outer.set(Arc::downgrade(&frame)).is_ok());
    s.insert_typed_erased_frame(frame);
    super::accounting::reconcile(&s);
    assert_eq!(
        s.resume_suspended_evaluation("done").unwrap(),
        Expr::string("done")
    );
    super::accounting::reconcile(&s);
}

#[path = "evaluation_context.rs"]
mod evaluation_context;
