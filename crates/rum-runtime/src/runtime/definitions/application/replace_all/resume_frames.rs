use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllSuspensionFrame;

impl TypedFrame for ApplyReplaceAllSuspensionFrame {
    type Input = ReplaceAllTraversalOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<ReplaceAllTraversalOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let resumed = match outcome {
            Ok(outcome) => Ok(outcome.expr),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllWithPlanSuspensionFrame;

impl TypedFrame for ApplyReplaceAllWithPlanSuspensionFrame {
    type Input = ReplaceAllTraversalOutcome;
    type Output = Expr;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<ReplaceAllTraversalOutcome, EvalError>,
        _token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Expr> {
        let resumed = match outcome {
            Ok(outcome) => Ok(outcome.expr),
            Err(error) => Err(error),
        };
        TypedAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyReplaceAllOwnedSuspensionFrame {
    pub(super) expr: rum_core::Expr,
    pub(super) state:
        crate::runtime::definitions::application::replace_all::ReplaceAllTraversalState,
}

impl TypedFrame for ApplyReplaceAllOwnedSuspensionFrame {
    type Input = Option<Expr>;
    type Output = ReplaceAllTraversalOutcome;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Option<Expr>, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<ReplaceAllTraversalOutcome> {
        let expr = self.expr.clone();
        let state = self.state.clone();
        let resumed = match outcome {
            Ok(outcome) => session.resume_replace_all_traversal(state, expr, outcome, token),
            Err(error) => Err(ResumeStop::failure(error)),
        };
        TypedAction::complete_work(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct ApplyRulesOnceFromListSuspensionFrame {
    pub(super) state:
        crate::runtime::definitions::application::replace_all::SuspendedRewriteRuleMatch,
    pub(super) source: crate::runtime::RuleApplicationSource,
    pub(super) cursor: RewriteRuleScanCursor,
    pub(super) index: usize,
    pub(super) would_reject: bool,
}

impl TypedFrame for ApplyRulesOnceFromListSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = Option<Expr>;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        let state = self.state.clone();
        let source = self.source;
        let cursor = self.cursor.clone();
        let index = self.index;
        let would_reject = self.would_reject;
        let resumed = match outcome {
            Ok(outcome) => session.finish_suspended_rewrite_rule_match(
                state,
                source,
                cursor,
                index,
                would_reject,
                outcome,
                token,
            ),
            Err(error) => Err(ResumeStop::failure(error)),
        };
        TypedAction::complete_work(resumed)
    }
}

#[cold]
#[inline(never)]
pub(super) fn suspend_replace_all_atom(
    session: &mut Session,
    witness: SlotWitness<Option<Expr>>,
    request: RuntimeInputRequest,
    expr: Expr,
    rules: RewriteRulePlan,
    source: RuleApplicationSource,
) -> Result<ReplaceAllTraversalOutcome, EvalError> {
    let state = ReplaceAllTraversalState {
        work: Vec::new(),
        completed: Vec::new(),
        rules,
        source,
    };
    witness.suspend(
        session,
        request,
        ApplyReplaceAllOwnedSuspensionFrame { state, expr },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_replace_all_traversal<
    Route: DefinitionResumeRoute<ApplyReplaceAllOwnedSuspensionFrame>,
>(
    session: &mut Session,
    witness: SlotWitness<Option<Expr>>,
    request: RuntimeInputRequest,
    state: ReplaceAllTraversalState,
    expr: Expr,
    route: Route,
) -> Result<ReplaceAllTraversalOutcome, Route::Error> {
    route.suspend(
        session,
        witness,
        request,
        ApplyReplaceAllOwnedSuspensionFrame { state, expr },
    )
}

#[cold]
#[inline(never)]
pub(super) fn suspend_rewrite_rule_match<
    Route: DefinitionResumeRoute<ApplyRulesOnceFromListSuspensionFrame>,
>(
    session: &mut Session,
    witness: SlotWitness<CompiledMatchOutcome>,
    request: RuntimeInputRequest,
    expr: Expr,
    normalized_expr: Expr,
    rules: RewriteRulePlan,
    source: RuleApplicationSource,
    cursor: RewriteRuleScanCursor,
    index: usize,
    would_reject: bool,
    route: Route,
) -> Result<Option<Expr>, Route::Error> {
    let state = SuspendedRewriteRuleMatch {
        expr,
        normalized_expr,
        rules,
    };
    route.suspend(
        session,
        witness,
        request,
        ApplyRulesOnceFromListSuspensionFrame {
            state,
            source,
            cursor,
            index,
            would_reject,
        },
    )
}
