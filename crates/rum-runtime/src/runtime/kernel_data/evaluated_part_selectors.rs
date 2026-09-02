use super::super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::{Arc, OnceLock};

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

/// The completion capability for an actually suspended selector evaluation.
///
/// Only the runtime's private finishing frame can populate the shared slot.
/// Supplying an arbitrary continuation expression therefore cannot construct
/// an [`EvaluatedPartSelectors`].
#[doc(hidden)]
#[derive(Clone, FrameRootTrace)]
pub struct EvaluatedPartSelectorsContinuation {
    completed: Arc<OnceLock<EvaluatedPartSelectors>>,
}

impl EvaluatedPartSelectorsContinuation {
    #[doc(hidden)]
    pub fn finish(
        &self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<EvaluatedPartSelectors, EvalError> {
        outcome?;
        Ok(self
            .completed
            .get()
            .expect("the runtime selector frame completes before its caller")
            .clone())
    }
}

/// The result of beginning exactly-once Part selector evaluation.
#[doc(hidden)]
pub enum EvaluatedPartSelectorsEvaluation {
    Complete(Result<EvaluatedPartSelectors, EvalError>),
    Suspended {
        error: EvalError,
        continuation: EvaluatedPartSelectorsContinuation,
    },
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
        crate::resume_or_suspend!(
            self,
            evaluated_result,
            ok(values) => Ok(EvaluatedPartSelectors::from_evaluated(values)),
            suspend => FinishEvaluatedPartSelectorsSuspensionFrame,
        )
    }

    /// Evaluates Part selectors for a downstream builtin family without
    /// publishing the raw continuation decoder.
    #[doc(hidden)]
    pub fn evaluate_part_selectors_owned_for_builtin(
        &mut self,
        selector_exprs: Vec<Expr>,
    ) -> EvaluatedPartSelectorsEvaluation {
        match self.resolve_argument_exprs_owned(selector_exprs) {
            Ok(values) => EvaluatedPartSelectorsEvaluation::Complete(Ok(
                EvaluatedPartSelectors::from_evaluated(values),
            )),
            Err(error @ EvalError::InputRequired { .. }) => {
                let completed = Arc::new(OnceLock::new());
                crate::resume_or_suspend!(
                    insert self,
                    FinishPublishedEvaluatedPartSelectorsSuspensionFrame {
                        completed: completed.clone(),
                    },
                );
                EvaluatedPartSelectorsEvaluation::Suspended {
                    error,
                    continuation: EvaluatedPartSelectorsContinuation { completed },
                }
            }
            Err(error) => EvaluatedPartSelectorsEvaluation::Complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishEvaluatedPartSelectorsSuspensionFrame;

impl crate::runtime::suspension::SuspensionFrame for FinishEvaluatedPartSelectorsSuspensionFrame {
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.map(|value| {
            let values = <Vec<Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(value);
            EvaluatedPartSelectors::from_evaluated(values).into_continuation_expr()
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct FinishPublishedEvaluatedPartSelectorsSuspensionFrame {
    completed: Arc<OnceLock<EvaluatedPartSelectors>>,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishPublishedEvaluatedPartSelectorsSuspensionFrame
{
    fn resume(
        self: Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.map(|value| {
            let values = <Vec<Expr> as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(value);
            assert!(
                self.completed
                    .set(EvaluatedPartSelectors::from_evaluated(values))
                    .is_ok(),
                "a selector continuation completes exactly once",
            );
            system_expr("Null")
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<EvaluatedPartSelectors>();
    assert_traces::<EvaluatedPartSelectorStorage>();
    assert_traces::<EvaluatedPartSelectorsContinuation>();
    assert_traces::<FinishEvaluatedPartSelectorsSuspensionFrame>();
    assert_traces::<FinishPublishedEvaluatedPartSelectorsSuspensionFrame>();
};
