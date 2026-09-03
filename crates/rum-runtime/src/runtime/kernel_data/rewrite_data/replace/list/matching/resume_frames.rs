use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

fn option_scope_into_continuation_expr(
    scope: Option<FrozenRewriteValue<OptionValueScope>>,
) -> Expr {
    scope
        .map(|scope| scope.into_inner().into_continuation_expr())
        .into_continuation_expr()
}

fn option_scope_from_continuation_expr(expr: Expr) -> Option<OptionValueScope> {
    Option::<Expr>::from_continuation_expr(expr).map(OptionValueScope::from_continuation_expr)
}

fn continuation_args(expr: Expr, tag: &'static str) -> std::vec::IntoIter<Expr> {
    let Expr::Call { head, args, .. } = expr else {
        panic!("unexpected {tag} continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, tag),
        "unexpected {tag} continuation tag: {head:?}"
    );
    args.into_iter()
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectReplaceListRuleBindingsOwnedSuspensionFrame1 {
    pub(super) guards: SharedArgs,
    pub(super) option_scope: Option<FrozenRewriteValue<OptionValueScope>>,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectReplaceListRuleBindingsOwnedSuspensionFrame1
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(bindings) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceListInitialGuardFilterFinalizer {
                    guards: self.guards.clone(),
                    option_scope: self.option_scope.clone(),
                    bindings,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareReplaceListInitialGuardFilterFinalizer {
    guards: SharedArgs,
    option_scope: Option<FrozenRewriteValue<OptionValueScope>>,
    bindings: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer
    for PrepareReplaceListInitialGuardFilterFinalizer
{
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationReplaceListInitialGuardFilter"),
            vec![
                self.guards.into_vec().into_continuation_expr(),
                self.bindings,
                option_scope_into_continuation_expr(self.option_scope),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueReplaceListInitialGuardFilterFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceListInitialGuardFilterFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args =
                continuation_args(payload, "InternalContinuationReplaceListInitialGuardFilter");
            let guards = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard continuation stores guards"),
            )
            .into_boxed_slice();
            let bindings = Vec::<SymbolExprMap>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard continuation stores bindings"),
            );
            let option_scope = option_scope_from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard continuation stores its option scope"),
            );
            debug_assert!(args.next().is_none());
            session
                .filter_replace_list_guard_bindings_owned(
                    guards,
                    bindings,
                    option_scope,
                    Vec::new(),
                    0,
                    None,
                )
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectReplaceListRuleBindingsOwnedSuspensionFrame2 {
    pub(super) state: SuspendedReplaceListCallMatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectReplaceListRuleBindingsOwnedSuspensionFrame2
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(match_outcome) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceListCallMatchFinalizer {
                    state: self.state.clone(),
                    match_outcome,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareReplaceListCallMatchFinalizer {
    state: SuspendedReplaceListCallMatch,
    match_outcome: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareReplaceListCallMatchFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationReplaceListCallMatchResult"),
            vec![self.state.into_continuation_expr(), self.match_outcome],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueReplaceListCallMatchFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceListCallMatchFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args =
                continuation_args(payload, "InternalContinuationReplaceListCallMatchResult");
            let state_expr = args
                .next()
                .expect("ReplaceList call match stores its suspended state");
            let mut state_args =
                continuation_args(state_expr, "InternalContinuationReplaceListCallMatch");
            let matcher_args = matcher_args_from_continuation_expr(
                state_args
                    .next()
                    .expect("ReplaceList call match stores matcher arguments"),
            );
            let value_args = Vec::<Expr>::from_continuation_expr(
                state_args
                    .next()
                    .expect("ReplaceList call match stores value arguments"),
            );
            let flat_head = state_args
                .next()
                .expect("ReplaceList call match stores its flat head");
            let orderless = bool::from_continuation_expr(
                state_args
                    .next()
                    .expect("ReplaceList call match stores its Orderless flag"),
            );
            let flat = bool::from_continuation_expr(
                state_args
                    .next()
                    .expect("ReplaceList call match stores its Flat flag"),
            );
            debug_assert!(state_args.next().is_none());
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("ReplaceList call match stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_replace_list_call_head_match(
                    matcher_args,
                    value_args,
                    flat_head,
                    orderless,
                    flat,
                    match_outcome,
                )
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectReplaceListRuleBindingsOwnedSuspensionFrame3;

impl crate::runtime::suspension::SuspensionFrame
    for CollectReplaceListRuleBindingsOwnedSuspensionFrame3
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.map(|payload| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(payload);
            let bindings = if outcome.matched {
                vec![outcome.bindings]
            } else {
                Vec::new()
            };
            bindings.into_continuation_expr()
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct FilterReplaceListGuardBindingsOwnedSuspensionFrame {
    pub(super) state: SuspendedReplaceListGuardFilter,
}

impl crate::runtime::suspension::SuspensionFrame
    for FilterReplaceListGuardBindingsOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(match_outcome) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceListGuardFilterFinalizer {
                    state: self.state.clone(),
                    match_outcome,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareReplaceListGuardFilterFinalizer {
    state: SuspendedReplaceListGuardFilter,
    match_outcome: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareReplaceListGuardFilterFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationReplaceListGuardFilter"),
            vec![
                self.state.guards.into_vec().into_continuation_expr(),
                self.state.bindings.into_vec().into_continuation_expr(),
                option_scope_into_continuation_expr(self.state.option_scope),
                self.state.accepted.into_vec().into_continuation_expr(),
                self.match_outcome,
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueReplaceListGuardFilterFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceListGuardFilterFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args = continuation_args(payload, "InternalContinuationReplaceListGuardFilter");
            let guards = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard filter stores its guards"),
            )
            .into_boxed_slice();
            let bindings = Vec::<SymbolExprMap>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard filter stores remaining bindings"),
            );
            let option_scope = option_scope_from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard filter stores its option scope"),
            );
            let mut accepted = Vec::<SymbolExprMap>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard filter stores accepted bindings"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("ReplaceList guard filter stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            if match_outcome.matched {
                accepted.push(match_outcome.bindings);
            }
            session
                .filter_replace_list_guard_bindings_owned(
                    guards,
                    bindings,
                    option_scope,
                    accepted,
                    0,
                    None,
                )
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct CollectReplaceListRuleResultsOwnedSuspensionFrame {
    pub(super) state: SuspendedReplaceListRuleResults,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectReplaceListRuleResultsOwnedSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(value) => crate::runtime::suspension::FrameAction::finalize(
                PrepareReplaceListRuleResultsFinalizer {
                    state: self.state.clone(),
                    value,
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareReplaceListRuleResultsFinalizer {
    state: SuspendedReplaceListRuleResults,
    value: Expr,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareReplaceListRuleResultsFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        let mut collected = self.state.scan.collected.into_vec();
        collected.push(self.value);
        Ok(Expr::call(
            system_expr("InternalContinuationReplaceListRuleResults"),
            vec![
                collected.into_continuation_expr(),
                self.state.bindings.into_vec().into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ContinueReplaceListRuleResultsFrame {
    pub(super) target: Expr,
    pub(super) normalized_target: Expr,
    pub(super) rules: RewriteRulePlan,
    pub(super) max_results: Option<usize>,
    pub(super) rule_index: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ContinueReplaceListRuleResultsFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args = continuation_args(payload, "InternalContinuationReplaceListRuleResults");
            let collected = Vec::<Expr>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList rule results store collected values"),
            );
            let bindings = Vec::<SymbolExprMap>::from_continuation_expr(
                args.next()
                    .expect("ReplaceList rule results store remaining bindings"),
            );
            debug_assert!(args.next().is_none());
            session
                .collect_replace_list_rule_results_owned(
                    self.target.clone(),
                    self.normalized_target.clone(),
                    self.rules.clone(),
                    self.max_results,
                    collected,
                    self.rule_index,
                    bindings,
                )
                .map(Vec::<Expr>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
