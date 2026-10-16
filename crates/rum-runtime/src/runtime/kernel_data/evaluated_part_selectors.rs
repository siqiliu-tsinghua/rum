use super::super::*;
use crate::runtime::kernel_api::{ContinueToken, TypedAction, TypedFrame};
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

/// A Part selector sequence whose expressions have each been evaluated once.
///
/// The fields stay private so assignment-target parsers cannot receive raw
/// selector expressions by accident.
#[derive(Clone, FrameRootTrace)]
pub struct EvaluatedPartSelectors {
    storage: EvaluatedPartSelectorStorage,
}

#[derive(Clone, FrameRootTrace)]
enum EvaluatedPartSelectorStorage {
    Owned(Vec<Expr>),
    StablePartArgs { args: SharedArgs, start: usize },
}

impl EvaluatedPartSelectors {
    fn from_evaluated(values: Vec<Expr>) -> Self {
        Self {
            storage: EvaluatedPartSelectorStorage::Owned(values),
        }
    }

    pub(in crate::runtime) fn from_stable_positive_part_args(args: &SharedArgs) -> Option<Self> {
        let selectors = args.get(1..)?;
        if selectors.is_empty()
            || !selectors
                .iter()
                .all(|selector| expr_as_signed_isize(selector).is_some_and(|index| index > 0))
        {
            return None;
        }

        Some(Self {
            storage: EvaluatedPartSelectorStorage::StablePartArgs {
                args: args.clone(),
                start: 1,
            },
        })
    }

    pub fn as_slice(&self) -> &[Expr] {
        match &self.storage {
            EvaluatedPartSelectorStorage::Owned(values) => values,
            EvaluatedPartSelectorStorage::StablePartArgs { args, start } => &args[*start..],
        }
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub(in crate::runtime) fn into_continuation_expr(self) -> Expr {
        let values = match self.storage {
            EvaluatedPartSelectorStorage::Owned(values) => values,
            EvaluatedPartSelectorStorage::StablePartArgs { args, start } => args[start..].to_vec(),
        };
        Expr::call(
            system_expr("InternalContinuationEvaluatedPartSelectors"),
            values,
        )
    }

    pub(in crate::runtime) fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationEvaluatedPartSelectors") =>
            {
                Self::from_evaluated(args.into_vec())
            }
            other => panic!("expected evaluated Part selector continuation, got {other}"),
        }
    }
}

impl Session {
    /// Evaluates Part selectors exactly once from left to right.
    pub(in crate::runtime) fn evaluate_part_selectors_owned(
        &mut self,
        selector_exprs: Vec<Expr>,
    ) -> Result<EvaluatedPartSelectors, EvalError> {
        let evaluated_result = self.resolve_argument_exprs_owned(selector_exprs);
        crate::resume_typed_or_suspend!(
            self,
            evaluated_result,
            ok(values) => Ok(EvaluatedPartSelectors::from_evaluated(values)),
            suspend => FinishEvaluatedPartSelectorsSuspensionFrame,
        )
    }

    /// Evaluates Part selectors exactly once for a downstream builtin family.
    #[doc(hidden)]
    pub fn evaluate_part_selectors_owned_for_builtin(
        &mut self,
        selector_exprs: Vec<Expr>,
    ) -> Result<EvaluatedPartSelectors, EvalError> {
        self.evaluate_part_selectors_owned(selector_exprs)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishEvaluatedPartSelectorsSuspensionFrame;

impl TypedFrame for FinishEvaluatedPartSelectorsSuspensionFrame {
    type Input = Vec<Expr>;
    type Output = EvaluatedPartSelectors;

    fn resume<'resume>(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Vec<Expr>, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<EvaluatedPartSelectors> {
        TypedAction::complete(outcome.map(EvaluatedPartSelectors::from_evaluated))
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<EvaluatedPartSelectors>();
    assert_traces::<EvaluatedPartSelectorStorage>();
    assert_traces::<FinishEvaluatedPartSelectorsSuspensionFrame>();
};
