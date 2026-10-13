//! Pure wire delegation; these providers never receive a channel Slot.
use super::channel::{CodecKey, LegacyCodec};
use super::registry::*;
use crate::runtime::suspension::ContinuationValue;

impl LegacyCodec for Option<Expr> {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for Vec<Expr> {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for Vec<SymbolExprMap> {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for Option<Vec<Expr>> {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for Option<Vec<RewriteRuleInfo>> {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for Option<CasesLevelSpec> {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for bool {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for String {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for CompiledMatchOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for CasesTraversalResolution {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for TraversalWithoutLimitResolution {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for CasesCollectionOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for PositionCollectionOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for DeleteCasesOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for TraversalCountOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for ReplaceOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for ReplaceAllTraversalOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for Vec<ReplacePartTarget> {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for EvalCapturedOutcome {
    fn old_encode(self, _: CodecKey) -> Expr {
        <Self as ContinuationValue>::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(<Self as ContinuationValue>::from_continuation_expr(expr))
    }
}

impl LegacyCodec for PreparedCallArguments {
    fn old_encode(self, _: CodecKey) -> Expr {
        PreparedCallArguments::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        crate::runtime::eval_call_prepare::prepared_call_arguments_from_continuation_expr(expr)
    }
}

impl LegacyCodec for EvaluatedPartSelectors {
    fn old_encode(self, _: CodecKey) -> Expr {
        EvaluatedPartSelectors::into_continuation_expr(self)
    }
    fn old_decode(expr: Expr, _: CodecKey) -> Result<Self, Expr> {
        Ok(EvaluatedPartSelectors::from_continuation_expr(expr))
    }
}
