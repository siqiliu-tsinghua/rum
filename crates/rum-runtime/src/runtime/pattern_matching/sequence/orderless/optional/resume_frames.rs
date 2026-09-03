use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy)]
enum OrderlessOptionalContinuationKind {
    Present,
    Omission,
    DefaultMatch,
    PresentCollection,
    CollectionOmission,
    CollectionMatch,
}

impl OrderlessOptionalContinuationKind {
    fn tag(self) -> &'static str {
        match self {
            Self::Present => "InternalContinuationOrderlessOptionalPresent",
            Self::Omission => "InternalContinuationOrderlessOptionalOmission",
            Self::DefaultMatch => "InternalContinuationOrderlessOptionalDefaultMatch",
            Self::PresentCollection => "InternalContinuationOrderlessOptionalPresentCollection",
            Self::CollectionOmission => "InternalContinuationOrderlessOptionalCollectionOmission",
            Self::CollectionMatch => "InternalContinuationOrderlessOptionalCollectionMatch",
        }
    }
}

fn prepare_orderless_optional_action(
    state: SuspendedOrderlessOptionalState,
    outcome: Result<Expr, EvalError>,
    kind: OrderlessOptionalContinuationKind,
) -> crate::runtime::suspension::FrameAction {
    match outcome {
        Ok(result) => {
            crate::runtime::suspension::FrameAction::finalize(PrepareOrderlessOptionalFinalizer {
                state,
                result,
                kind,
            })
        }
        Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
    }
}

struct PrepareOrderlessOptionalFinalizer {
    state: SuspendedOrderlessOptionalState,
    result: Expr,
    kind: OrderlessOptionalContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOrderlessOptionalFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr(self.kind.tag()),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.result,
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchOrderlessOptionalChoicesSuspensionFrame {
    pub(super) state: SuspendedOrderlessOptionalState,
}

impl crate::runtime::suspension::SuspensionFrame for MatchOrderlessOptionalChoicesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        prepare_orderless_optional_action(
            self.state.clone(),
            outcome,
            OrderlessOptionalContinuationKind::Present,
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct MatchOrderlessOptionalOmissionSuspensionFrame {
    pub(super) state: SuspendedOrderlessOptionalState,
}

impl crate::runtime::suspension::SuspensionFrame for MatchOrderlessOptionalOmissionSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        prepare_orderless_optional_action(
            self.state.clone(),
            outcome,
            OrderlessOptionalContinuationKind::Omission,
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishOrderlessOptionalDefaultSuspensionFrame {
    pub(super) state: SuspendedOrderlessOptionalState,
}

impl crate::runtime::suspension::SuspensionFrame for FinishOrderlessOptionalDefaultSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        prepare_orderless_optional_action(
            self.state.clone(),
            outcome,
            OrderlessOptionalContinuationKind::DefaultMatch,
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectOrderlessOptionalMatchesSuspensionFrame {
    pub(super) state: SuspendedOrderlessOptionalState,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectOrderlessOptionalMatchesSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        prepare_orderless_optional_action(
            self.state.clone(),
            outcome,
            OrderlessOptionalContinuationKind::PresentCollection,
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishOrderlessOptionalPresentCollectionSuspensionFrame {
    pub(super) present: PersistentMatchResults,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishOrderlessOptionalPresentCollectionSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(omitted) => crate::runtime::suspension::FrameAction::finalize(
                FinishOrderlessOptionalCollectionFinalizer {
                    present: self.present.clone(),
                    omitted,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct FinishOrderlessOptionalCollectionFinalizer {
    present: PersistentMatchResults,
    omitted: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishOrderlessOptionalCollectionFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let omitted = Vec::<SymbolExprMap>::from_continuation_expr(self.omitted);
        Ok(append_match_results(self.present.into_vec(), omitted).into_continuation_expr())
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectOrderlessOptionalOmissionSuspensionFrame {
    pub(super) state: SuspendedOrderlessOptionalState,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectOrderlessOptionalOmissionSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        prepare_orderless_optional_action(
            self.state.clone(),
            outcome,
            OrderlessOptionalContinuationKind::CollectionOmission,
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FinishOrderlessOptionalCollectionDefaultSuspensionFrame {
    pub(super) state: SuspendedOrderlessOptionalState,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishOrderlessOptionalCollectionDefaultSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        prepare_orderless_optional_action(
            self.state.clone(),
            outcome,
            OrderlessOptionalContinuationKind::CollectionMatch,
        )
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueOrderlessOptionalFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOrderlessOptionalFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args, .. } = payload else {
                panic!("unexpected orderless-optional continuation payload")
            };
            let mut args = args.into_iter();
            let state = OrderlessOptionalState::from_continuation_expr(
                args.next().expect("orderless optional stores its state"),
            );
            let result = args
                .next()
                .expect("orderless optional stores its suspended result");
            debug_assert!(args.next().is_none());

            if is_system_symbol_name(&head, OrderlessOptionalContinuationKind::Present.tag()) {
                session
                    .finish_orderless_optional_present(
                        state,
                        CompiledMatchOutcome::from_continuation_expr(result),
                    )
                    .map(CompiledMatchOutcome::into_continuation_expr)
            } else if is_system_symbol_name(
                &head,
                OrderlessOptionalContinuationKind::Omission.tag(),
            ) {
                session
                    .finish_orderless_optional_default(
                        state,
                        Option::<Expr>::from_continuation_expr(result),
                    )
                    .map(CompiledMatchOutcome::into_continuation_expr)
            } else if is_system_symbol_name(
                &head,
                OrderlessOptionalContinuationKind::DefaultMatch.tag(),
            ) {
                session
                    .finish_orderless_optional_default_match(
                        state,
                        CompiledMatchOutcome::from_continuation_expr(result),
                    )
                    .map(CompiledMatchOutcome::into_continuation_expr)
            } else if is_system_symbol_name(
                &head,
                OrderlessOptionalContinuationKind::PresentCollection.tag(),
            ) {
                session
                    .finish_orderless_optional_present_collection(
                        state,
                        Vec::<SymbolExprMap>::from_continuation_expr(result),
                    )
                    .map(Vec::<SymbolExprMap>::into_continuation_expr)
            } else if is_system_symbol_name(
                &head,
                OrderlessOptionalContinuationKind::CollectionOmission.tag(),
            ) {
                session
                    .finish_orderless_optional_collection_default(
                        state,
                        Option::<Expr>::from_continuation_expr(result),
                    )
                    .map(Vec::<SymbolExprMap>::into_continuation_expr)
            } else if is_system_symbol_name(
                &head,
                OrderlessOptionalContinuationKind::CollectionMatch.tag(),
            ) {
                session
                    .finish_orderless_optional_collection_match(
                        state,
                        CompiledMatchOutcome::from_continuation_expr(result),
                    )
                    .map(Vec::<SymbolExprMap>::into_continuation_expr)
            } else {
                panic!("unexpected orderless-optional continuation tag: {head:?}")
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
