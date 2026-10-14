//! Private injection fixtures; production codecs and frames remain unchanged.
use super::super::channel::{Codec, StateCodec, StateMode};
use super::*;
use crate::runtime::{BuiltinSpec, MemoryGrowthContract, SessionRequestError};
use rum_core::SharedArgs;
use std::cell::RefCell;
use std::sync::Mutex;

#[derive(Debug)]
pub(super) struct StateBool;
impl Codec for StateBool {
    type Mode = StateMode;
}
impl SlotValue for StateBool {}
impl StateCodec for StateBool {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::Bool(true)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::Bool(_) => Ok(Self),
            other => Err(other),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum FaultKind {
    ValueForState,
    StateForValue,
    WrongState,
}
impl FaultKind {
    fn slot(self) -> Slot {
        match self {
            Self::ValueForState => Slot::Value(Expr::string("wrong arm")),
            Self::StateForValue => Slot::State(ContinuationState::Bool(true)),
            Self::WrongState => Slot::State(ContinuationState::String("wrong variant".into())),
        }
    }
    pub(super) fn expected(self) -> &'static str {
        match self {
            Self::StateForValue => std::any::type_name::<Expr>(),
            _ => std::any::type_name::<StateBool>(),
        }
    }
    pub(super) fn got(self) -> &'static str {
        match self {
            Self::ValueForState => "Value",
            Self::StateForValue => "State::Bool",
            Self::WrongState => "State::String",
        }
    }
    pub(super) fn assert_request(self, error: &SessionRequestError) {
        assert!(
            matches!(error, SessionRequestError::ContinuationSlotMismatch { expected, got } if *expected == self.expected() && *got == self.got()),
            "{error:?}"
        );
    }
    pub(super) fn assert_eval(self, error: &EvalError) {
        assert!(
            matches!(error, EvalError::ContinuationSlotMismatch(fault) if fault.expected() == self.expected() && fault.got() == self.got()),
            "{error:?}"
        );
    }
}

pub(super) type Events = Arc<Mutex<Vec<&'static str>>>;
#[derive(Clone)]
struct Config {
    kind: FaultKind,
    events: Events,
    nested: bool,
}
thread_local! { static CONFIG: RefCell<Option<Config>> = const { RefCell::new(None) }; }

struct Emit {
    config: Config,
}
impl FrameRootTrace for Emit {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl FrameAbandon for Emit {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        FrameAbandonAction::Discard
    }
}
impl TypedFrame for Emit {
    type Input = Expr;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        s: &mut Session,
        input: Result<Expr, EvalError>,
        _: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        input.unwrap();
        self.config.events.lock().unwrap().push("emit");
        if self.config.nested {
            let expr = s.lower_text("InputString[\"nested-owner\"]").unwrap();
            assert!(matches!(
                s.evaluate(expr),
                Err(EvalError::InputRequired { .. })
            ));
            s.insert_typed_suspension_frame(Spy::<Expr>::new(&self.config.events, "nested"));
        }
        TypedAction::malformed_slot(self.config.kind.slot())
    }
}

pub(super) struct Spy<I: SlotValue> {
    events: Events,
    name: &'static str,
    input: std::marker::PhantomData<fn(I)>,
}
impl<I: SlotValue> Spy<I> {
    pub(super) fn new(events: &Events, name: &'static str) -> Self {
        Self {
            events: events.clone(),
            name,
            input: std::marker::PhantomData,
        }
    }
}
impl<I: SlotValue> FrameRootTrace for Spy<I> {
    fn trace_roots(&self, _: &mut dyn FrameRootVisitor) {}
}
impl<I: SlotValue> FrameAbandon for Spy<I> {
    fn abandon(self: Arc<Self>, _: &mut AbandonmentContext<'_>) -> FrameAbandonAction {
        self.events.lock().unwrap().push(self.name);
        FrameAbandonAction::Discard
    }
}
impl<I: SlotValue> TypedFrame for Spy<I> {
    type Input = I;
    type Output = Expr;
    fn resume<'r>(
        self: Arc<Self>,
        _: &mut Session,
        _: Result<I, EvalError>,
        _: ContinueToken<'r, Self>,
    ) -> TypedAction<Expr> {
        panic!("rejected consumer or unrun suffix executed: {}", self.name)
    }
}

fn trip(s: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    let config = CONFIG.with(|c| c.borrow().as_ref().unwrap().clone());
    let expr = s.lower_text("InputString[\"fault\"]").unwrap();
    let result = s.evaluate(expr);
    let Err(EvalError::InputRequired { request }) = result else {
        panic!("injection needs a real input pause")
    };
    install(s, &config, true);
    Err(EvalError::InputRequired { request })
}

fn direct(_: &mut Session, _: &Expr, _: &SharedArgs) -> Result<Expr, EvalError> {
    Err(EvalError::ContinuationSlotMismatch(
        crate::runtime::SlotMismatch::for_test("direct expected", "direct got"),
    ))
}

fn install(s: &mut Session, config: &Config, consumer: bool) {
    s.insert_typed_suspension_frame(Emit {
        config: config.clone(),
    });
    if consumer {
        match config.kind {
            FaultKind::StateForValue => {
                s.insert_typed_suspension_frame(Spy::<Expr>::new(&config.events, "consumer"))
            }
            _ => s.insert_typed_suspension_frame(Spy::<StateBool>::new(&config.events, "consumer")),
        }
        s.insert_typed_suspension_frame(Spy::<Expr>::new(&config.events, "suffix-one"));
        s.insert_typed_suspension_frame(Spy::<Expr>::new(&config.events, "suffix-two"));
    }
}

pub(super) fn completion(s: &mut Session, kind: FaultKind, events: &Events) {
    let expr = s.lower_text("InputString[\"completion\"]").unwrap();
    assert!(matches!(
        s.evaluate(expr),
        Err(EvalError::InputRequired { .. })
    ));
    install(
        s,
        &Config {
            kind,
            events: events.clone(),
            nested: false,
        },
        false,
    );
}

pub(super) fn session(kind: FaultKind, nested: bool) -> (Session, Events) {
    let events = Events::default();
    CONFIG.with(|c| {
        *c.borrow_mut() = Some(Config {
            kind,
            events: events.clone(),
            nested,
        })
    });
    let mut s = crate::runtime::test_builtins::session_with_test_control_builtins();
    s.resource_limits.eval_memory_limit = Some(1_000_000_000);
    for (name, handler) in [
        ("SlotTrip", trip as crate::runtime::BuiltinFn),
        ("SlotDirect", direct as crate::runtime::BuiltinFn),
    ] {
        s.register_builtin(BuiltinSpec::ordinary(
            name,
            "Runtime-private fault injection.",
            handler,
            MemoryGrowthContract::Delegated,
        ));
    }
    (s, events)
}

pub(super) fn restored(s: &mut Session) {
    super::accounting::reconcile(s);
    assert!(s.pending_input_request().is_none());
    assert!(s.pending_eval_request.is_none());
    assert!(s.pending_suspension.is_none());
    assert!(s.resuming_suspension.is_none());
    assert!(s.abandonment_worklist.is_empty());
    assert!(!s.abandonment_active);
    assert!(s.dynamic_own_values.is_empty());
    assert!(s.option_value_scopes.is_empty());
    assert!(s.load_stack.is_empty());
    assert!(s.guest_form.is_none());
    assert!(s.module_activations.is_empty());
    assert!(s.handled_non_loader_eval_errors.is_empty());
    assert_eq!(s.trace_event_depths.len(), s.trace_events.len());
    assert!(s.reject_overlapping_evaluation_request().is_ok());
    assert_eq!(s.eval_request("1+1").unwrap().value, Expr::small_integer(2));
}
