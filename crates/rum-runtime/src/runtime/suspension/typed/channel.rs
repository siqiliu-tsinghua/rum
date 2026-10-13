//! Closed channel and keyed transport strategies. Only Codec::Mode selects a mode.
use super::registry::*;

// Public inside this private module: no reachable key constructor or raw channel.
pub struct CodecKey {
    private: (),
}
impl CodecKey {
    pub(super) fn new() -> Self {
        Self { private: () }
    }
}

pub enum Slot {
    Value(Expr),
    State(ContinuationState),
}

// Existing nominal payloads only. None is transported as State until its flip.
pub enum ContinuationState {
    OptionalExpr(Option<Expr>),
    ExprVec(Vec<Expr>),
    BindingsVec(Vec<SymbolExprMap>),
    OptionalExprVec(Option<Vec<Expr>>),
    OptionalRewriteRules(Option<Vec<RewriteRuleInfo>>),
    OptionalCasesLevel(Option<CasesLevelSpec>),
    Bool(bool),
    String(String),
    MatchOutcome(CompiledMatchOutcome),
    CasesResolution(CasesTraversalResolution),
    TraversalResolution(TraversalWithoutLimitResolution),
    CasesCollection(CasesCollectionOutcome),
    PositionCollection(PositionCollectionOutcome),
    DeleteCases(DeleteCasesOutcome),
    TraversalCount(TraversalCountOutcome),
    Replace(ReplaceOutcome),
    ReplaceAll(ReplaceAllTraversalOutcome),
    ReplacePartTargets(Vec<ReplacePartTarget>),
    CapturedEvaluation(EvalCapturedOutcome),
    PreparedArguments(PreparedCallArguments),
    PartSelectors(EvaluatedPartSelectors),
}

pub trait Codec: Sized {
    type Mode: Transport<Self>;
}

pub(super) fn encode<T: Codec>(value: T, key: CodecKey) -> Slot {
    T::Mode::encode(value, key)
}

pub(super) fn decode<T: Codec>(slot: Slot, key: CodecKey) -> Result<T, Slot> {
    T::Mode::decode(slot, key)
}

pub trait Transport<T>: Sized {
    fn encode(value: T, key: CodecKey) -> Slot;
    fn decode(slot: Slot, key: CodecKey) -> Result<T, Slot>;
}

pub struct ValueMode;
pub struct LegacyMode;
pub struct StateMode;

pub trait LegacyCodec: Sized {
    fn old_encode(self, key: CodecKey) -> Expr;
    fn old_decode(expr: Expr, key: CodecKey) -> Result<Self, Expr>;
}

pub trait StateCodec: Sized {
    fn pack(self, key: CodecKey) -> ContinuationState;
    fn unpack(state: ContinuationState, key: CodecKey) -> Result<Self, ContinuationState>;
}

impl Transport<Expr> for ValueMode {
    fn encode(value: Expr, _: CodecKey) -> Slot {
        Slot::Value(value)
    }
    fn decode(slot: Slot, _: CodecKey) -> Result<Expr, Slot> {
        match slot {
            Slot::Value(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl<T: LegacyCodec> Transport<T> for LegacyMode {
    fn encode(value: T, key: CodecKey) -> Slot {
        Slot::Value(value.old_encode(key))
    }
    fn decode(slot: Slot, key: CodecKey) -> Result<T, Slot> {
        match slot {
            Slot::Value(value) => T::old_decode(value, key).map_err(Slot::Value),
            other => Err(other),
        }
    }
}

impl<T: StateCodec> Transport<T> for StateMode {
    fn encode(value: T, key: CodecKey) -> Slot {
        Slot::State(value.pack(key))
    }
    fn decode(slot: Slot, key: CodecKey) -> Result<T, Slot> {
        match slot {
            Slot::State(value) => T::unpack(value, key).map_err(Slot::State),
            other => Err(other),
        }
    }
}
