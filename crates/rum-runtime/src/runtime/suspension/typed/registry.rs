pub(super) use crate::runtime::debug_tools::test_report::EvalCapturedOutcome;
pub(super) use crate::runtime::definitions::{
    CasesTraversalResolution, ReplaceAllTraversalOutcome, TraversalWithoutLimitResolution,
};
pub(super) use crate::runtime::eval_call_prepare::PreparedCallArguments;
pub(super) use crate::runtime::kernel_data::evaluated_part_selectors::EvaluatedPartSelectors;
pub(super) use crate::runtime::kernel_data::rewrite_data::{DeleteCasesOutcome, ReplaceOutcome};
pub(super) use crate::runtime::kernel_data::structure_data::replace::ReplacePartTarget;
pub(super) use crate::runtime::model::{
    CasesCollectionOutcome, CasesLevelSpec, PositionCollectionOutcome, RewriteRuleInfo,
    SymbolExprMap, TraversalCountOutcome,
};
pub(super) use crate::runtime::pattern_matching::CompiledMatchOutcome;
pub(super) use rum_core::Expr;

use super::SlotValue;
use super::channel::{Codec, CodecKey, ContinuationState, LegacyMode, StateCodec, ValueMode};

impl Codec for Expr {
    type Mode = ValueMode;
}
impl SlotValue for Expr {}

impl Codec for Option<Expr> {
    type Mode = LegacyMode;
}
impl SlotValue for Option<Expr> {}
impl StateCodec for Option<Expr> {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::OptionalExpr(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::OptionalExpr(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for Vec<Expr> {
    type Mode = LegacyMode;
}
impl SlotValue for Vec<Expr> {}
impl StateCodec for Vec<Expr> {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::ExprVec(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::ExprVec(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for Vec<SymbolExprMap> {
    type Mode = LegacyMode;
}
impl SlotValue for Vec<SymbolExprMap> {}
impl StateCodec for Vec<SymbolExprMap> {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::BindingsVec(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::BindingsVec(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for Option<Vec<Expr>> {
    type Mode = LegacyMode;
}
impl SlotValue for Option<Vec<Expr>> {}
impl StateCodec for Option<Vec<Expr>> {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::OptionalExprVec(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::OptionalExprVec(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for Option<Vec<RewriteRuleInfo>> {
    type Mode = LegacyMode;
}
impl SlotValue for Option<Vec<RewriteRuleInfo>> {}
impl StateCodec for Option<Vec<RewriteRuleInfo>> {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::OptionalRewriteRules(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::OptionalRewriteRules(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for Option<CasesLevelSpec> {
    type Mode = LegacyMode;
}
impl SlotValue for Option<CasesLevelSpec> {}
impl StateCodec for Option<CasesLevelSpec> {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::OptionalCasesLevel(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::OptionalCasesLevel(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for bool {
    type Mode = LegacyMode;
}
impl SlotValue for bool {}
impl StateCodec for bool {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::Bool(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::Bool(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for String {
    type Mode = LegacyMode;
}
impl SlotValue for String {}
impl StateCodec for String {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::String(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::String(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for CompiledMatchOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for CompiledMatchOutcome {}
impl StateCodec for CompiledMatchOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::MatchOutcome(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::MatchOutcome(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for CasesTraversalResolution {
    type Mode = LegacyMode;
}
impl SlotValue for CasesTraversalResolution {}
impl StateCodec for CasesTraversalResolution {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::CasesResolution(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::CasesResolution(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for TraversalWithoutLimitResolution {
    type Mode = LegacyMode;
}
impl SlotValue for TraversalWithoutLimitResolution {}
impl StateCodec for TraversalWithoutLimitResolution {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::TraversalResolution(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::TraversalResolution(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for CasesCollectionOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for CasesCollectionOutcome {}
impl StateCodec for CasesCollectionOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::CasesCollection(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::CasesCollection(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for PositionCollectionOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for PositionCollectionOutcome {}
impl StateCodec for PositionCollectionOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::PositionCollection(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::PositionCollection(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for DeleteCasesOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for DeleteCasesOutcome {}
impl StateCodec for DeleteCasesOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::DeleteCases(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::DeleteCases(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for TraversalCountOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for TraversalCountOutcome {}
impl StateCodec for TraversalCountOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::TraversalCount(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::TraversalCount(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for ReplaceOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for ReplaceOutcome {}
impl StateCodec for ReplaceOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::Replace(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::Replace(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for ReplaceAllTraversalOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for ReplaceAllTraversalOutcome {}
impl StateCodec for ReplaceAllTraversalOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::ReplaceAll(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::ReplaceAll(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for Vec<ReplacePartTarget> {
    type Mode = LegacyMode;
}
impl SlotValue for Vec<ReplacePartTarget> {}
impl StateCodec for Vec<ReplacePartTarget> {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::ReplacePartTargets(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::ReplacePartTargets(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for EvalCapturedOutcome {
    type Mode = LegacyMode;
}
impl SlotValue for EvalCapturedOutcome {}
impl StateCodec for EvalCapturedOutcome {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::CapturedEvaluation(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::CapturedEvaluation(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for PreparedCallArguments {
    type Mode = LegacyMode;
}
impl SlotValue for PreparedCallArguments {}
impl StateCodec for PreparedCallArguments {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::PreparedArguments(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::PreparedArguments(value) => Ok(value),
            other => Err(other),
        }
    }
}

impl Codec for EvaluatedPartSelectors {
    type Mode = LegacyMode;
}
impl SlotValue for EvaluatedPartSelectors {}
impl StateCodec for EvaluatedPartSelectors {
    fn pack(self, _: CodecKey) -> ContinuationState {
        ContinuationState::PartSelectors(self)
    }
    fn unpack(state: ContinuationState, _: CodecKey) -> Result<Self, ContinuationState> {
        match state {
            ContinuationState::PartSelectors(value) => Ok(value),
            other => Err(other),
        }
    }
}
