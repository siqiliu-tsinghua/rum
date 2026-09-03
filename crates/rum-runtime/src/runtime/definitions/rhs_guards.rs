mod resume_frames;
use resume_frames::*;

use super::super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[doc(hidden)]
#[inline]
pub fn split_definition_rhs_guards(rhs: Expr) -> (Expr, Box<[Expr]>) {
    if pattern_condition(&rhs).is_none() {
        return (rhs, Box::default());
    }

    let mut terms = Vec::new();
    flatten_condition_terms(rhs, &mut terms);
    let mut terms = terms.into_iter();
    let body = terms
        .next()
        .expect("a Condition expression always has a body");
    let guards = terms.collect::<Vec<_>>().into_boxed_slice();
    (body, guards)
}

fn flatten_condition_terms(expr: Expr, terms: &mut Vec<Expr>) {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(&head, heads::patterns::CONDITION) && args.len() == 2 =>
        {
            let mut args = args.into_iter();
            flatten_condition_terms(args.next().expect("Condition has a body"), terms);
            flatten_condition_terms(args.next().expect("Condition has a test"), terms);
        }
        other => terms.push(other),
    }
}

pub(in crate::runtime) fn rebuild_definition_rhs_guards(body: Expr, guards: &[Expr]) -> Expr {
    let Some((last, preceding)) = guards.split_last() else {
        return body;
    };

    let condition = preceding.iter().rev().fold(last.clone(), |nested, guard| {
        condition_expr(guard.clone(), nested)
    });
    condition_expr(body, condition)
}

fn condition_expr(body: Expr, test: Expr) -> Expr {
    Expr::call(system_expr(heads::patterns::CONDITION), vec![body, test])
}

pub(in crate::runtime) fn down_value_patterns_equivalent(
    left_lhs: &Expr,
    left_guards: &[Expr],
    right_lhs: &Expr,
    right_guards: &[Expr],
) -> bool {
    if left_guards.len() != right_guards.len() {
        return false;
    }

    let left_mapping = pattern_name_mapping(left_lhs);
    let right_mapping = pattern_name_mapping(right_lhs);
    substitute_pattern_name_mapping(left_lhs, &left_mapping)
        == substitute_pattern_name_mapping(right_lhs, &right_mapping)
        && left_guards.iter().zip(right_guards).all(|(left, right)| {
            substitute_pattern_name_mapping(left, &left_mapping)
                == substitute_pattern_name_mapping(right, &right_mapping)
        })
}

fn pattern_name_mapping(lhs: &Expr) -> BTreeMap<Symbol, Symbol> {
    let mut mapping = BTreeMap::new();
    let mut next_id = 0_u64;
    collect_pattern_name_mapping(lhs, &mut mapping, &mut next_id);
    mapping
}

impl Session {
    pub(in crate::runtime) fn compile_down_value_matcher(
        &self,
        lhs: &Expr,
        guards: &[Expr],
    ) -> MatcherNode {
        let inner = self.compile_matcher_for_expr(lhs);
        if guards.is_empty() {
            return inner;
        }

        MatcherNode::DefinitionCondition(Box::new(DefinitionConditionMatcher {
            inner,
            lhs: lhs.clone(),
            guards: guards.to_vec().into_boxed_slice(),
        }))
    }

    pub(in crate::runtime) fn match_compiled_definition_condition_owned(
        &mut self,
        matcher: DefinitionConditionMatcher,
        value: Expr,
        bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let DefinitionConditionMatcher { inner, lhs, guards } = matcher;
        let option_scope = self.option_value_scope_for_definition(&lhs, &value);
        let inner_result = self.match_compiled_owned(inner, value, bindings);
        match inner_result {
            Ok(outcome) => {
                self.finish_definition_condition_match_owned(guards, option_scope, outcome)
            }
            Err(EvalError::InputRequired { request }) => {
                self.suspend_definition_condition_match(guards, option_scope, request)
            }
            Err(error) => Err(error),
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_definition_condition_match(
        &mut self,
        guards: Box<[Expr]>,
        option_scope: Option<OptionValueScope>,
        request: RuntimeInputRequest,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let guards = SharedArgs::new(guards.into_vec());
        crate::resume_or_suspend!(insert self, MatchCompiledDefinitionConditionOwnedSuspensionFrame { guards, option_scope });
        Err(EvalError::InputRequired { request })
    }

    fn finish_definition_condition_match_owned(
        &mut self,
        guards: Box<[Expr]>,
        option_scope: Option<OptionValueScope>,
        outcome: CompiledMatchOutcome,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if !outcome.matched {
            return Ok(outcome);
        }

        let false_bindings = outcome.bindings.clone();
        self.evaluate_guard_sequence_owned(guards, outcome.bindings, false_bindings, option_scope)
    }

    pub(in crate::runtime) fn evaluate_guard_owned(
        &mut self,
        guard: Expr,
        bindings: SymbolExprMap,
        false_bindings: SymbolExprMap,
        option_scope: Option<OptionValueScope>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.evaluate_owned_guards(
            OwnedGuards::One(guard),
            bindings,
            false_bindings,
            option_scope,
            0,
        )
    }

    pub(in crate::runtime) fn evaluate_guard_sequence_owned(
        &mut self,
        guards: Box<[Expr]>,
        bindings: SymbolExprMap,
        false_bindings: SymbolExprMap,
        option_scope: Option<OptionValueScope>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.evaluate_owned_guards(
            OwnedGuards::Many(guards),
            bindings,
            false_bindings,
            option_scope,
            0,
        )
    }

    pub(in crate::runtime) fn evaluate_guard_sequence_readonly_suspended(
        &mut self,
        guards: SharedArgs,
        bindings: FrozenMatchBindings,
        option_scope: Option<OptionValueScope>,
    ) -> Result<bool, EvalError> {
        self.evaluate_readonly_suspended_guards(SuspendedReadonlyGuardEvaluation {
            guards,
            bindings,
            option_scope,
            index: 0,
        })
    }

    fn evaluate_owned_guards(
        &mut self,
        guards: OwnedGuards,
        bindings: SymbolExprMap,
        false_bindings: SymbolExprMap,
        option_scope: Option<OptionValueScope>,
        index: usize,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let Some(guard) = guards.get(index) else {
            return Ok(guard_outcome(true, bindings));
        };

        let guard_expr = substitute_bindings(guard, &bindings);
        let guard_result = self.evaluate_with_option_value_scope(option_scope.clone(), guard_expr);
        match guard_result {
            Ok(result) => {
                if is_system_symbol_name(&result, heads::structure::TRUE) {
                    self.evaluate_owned_guards(
                        guards,
                        bindings,
                        false_bindings,
                        option_scope,
                        index + 1,
                    )
                } else {
                    Ok(guard_outcome(false, false_bindings))
                }
            }
            Err(EvalError::InputRequired { request }) => self.suspend_owned_guards(
                guards,
                bindings,
                false_bindings,
                option_scope,
                index,
                request,
            ),
            Err(error) => Err(error),
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_owned_guards(
        &mut self,
        guards: OwnedGuards,
        bindings: SymbolExprMap,
        false_bindings: SymbolExprMap,
        option_scope: Option<OptionValueScope>,
        index: usize,
        request: RuntimeInputRequest,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        let state = SuspendedGuardEvaluation {
            guards: guards.freeze(),
            bindings: FrozenMatchBindings::freeze(bindings),
            false_bindings: FrozenMatchBindings::freeze(false_bindings),
            option_scope,
            index,
        };
        crate::resume_or_suspend!(insert self, EvaluateOwnedGuardsSuspensionFrame { state });
        Err(EvalError::InputRequired { request })
    }

    fn resume_suspended_guard_deferred(
        &mut self,
        mut state: SuspendedGuardEvaluation,
        result: Expr,
    ) -> Result<SuspendedGuardCompletion, EvalError> {
        if !is_system_symbol_name(&result, heads::structure::TRUE) {
            return Ok(SuspendedGuardCompletion {
                matched: false,
                bindings: state.false_bindings,
            });
        }
        state.index += 1;
        self.evaluate_suspended_guards_deferred(state)
    }

    fn evaluate_suspended_guards_deferred(
        &mut self,
        state: SuspendedGuardEvaluation,
    ) -> Result<SuspendedGuardCompletion, EvalError> {
        let Some(guard) = state.guards.get(state.index) else {
            return Ok(SuspendedGuardCompletion {
                matched: true,
                bindings: state.bindings,
            });
        };

        let guard_expr = substitute_bindings(guard, state.bindings.as_map());
        let guard_result =
            self.evaluate_with_option_value_scope(state.option_scope.clone(), guard_expr);
        match guard_result {
            Ok(result) => self.resume_suspended_guard_deferred(state, result),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, EvaluateOwnedGuardsSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn evaluate_readonly_suspended_guards(
        &mut self,
        mut state: SuspendedReadonlyGuardEvaluation,
    ) -> Result<bool, EvalError> {
        let Some(guard) = state.guards.get(state.index) else {
            return Ok(true);
        };
        let guard_expr = substitute_bindings(guard, state.bindings.as_map());
        let guard_result =
            self.evaluate_with_option_value_scope(state.option_scope.clone(), guard_expr);
        match guard_result {
            Ok(result) if is_system_symbol_name(&result, heads::structure::TRUE) => {
                state.index += 1;
                self.evaluate_readonly_suspended_guards(state)
            }
            Ok(_) => Ok(false),
            Err(EvalError::InputRequired { request }) => {
                crate::resume_or_suspend!(insert self, EvaluateReadonlySuspendedGuardsSuspensionFrame { state });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(FrameRootTrace)]
enum OwnedGuards {
    One(Expr),
    Many(Box<[Expr]>),
}

impl OwnedGuards {
    fn get(&self, index: usize) -> Option<&Expr> {
        match self {
            Self::One(guard) => (index == 0).then_some(guard),
            Self::Many(guards) => guards.get(index),
        }
    }

    fn freeze(self) -> SuspendedGuards {
        match self {
            Self::One(guard) => SuspendedGuards::One(guard),
            Self::Many(guards) => SuspendedGuards::Many(SharedArgs::new(guards.into_vec())),
        }
    }
}

#[derive(Clone, FrameRootTrace)]
enum SuspendedGuards {
    One(Expr),
    Many(SharedArgs),
}

impl SuspendedGuards {
    fn get(&self, index: usize) -> Option<&Expr> {
        match self {
            Self::One(guard) => (index == 0).then_some(guard),
            Self::Many(guards) => guards.get(index),
        }
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedGuardEvaluation {
    guards: SuspendedGuards,
    bindings: FrozenMatchBindings,
    false_bindings: FrozenMatchBindings,
    option_scope: Option<OptionValueScope>,
    index: usize,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedReadonlyGuardEvaluation {
    guards: SharedArgs,
    bindings: FrozenMatchBindings,
    option_scope: Option<OptionValueScope>,
    index: usize,
}

struct SuspendedGuardCompletion {
    matched: bool,
    bindings: FrozenMatchBindings,
}

fn guard_outcome(matched: bool, bindings: SymbolExprMap) -> CompiledMatchOutcome {
    CompiledMatchOutcome { matched, bindings }
}

#[doc(hidden)]
pub fn evaluate_definition_rhs_guards_for_match(
    session: &mut Session,
    lhs: &Expr,
    candidate: &Expr,
    guards: &[Expr],
    bindings: &SymbolExprMap,
) -> Result<bool, EvalError> {
    let option_scope = session.option_value_scope_for_definition(lhs, candidate);
    let guard_result = session.evaluate_guard_sequence_owned(
        guards.to_vec().into_boxed_slice(),
        bindings.clone(),
        bindings.clone(),
        option_scope,
    );
    crate::resume_or_suspend!(
        session,
        guard_result,
        ok(outcome) => { (|| { Ok(outcome.matched) })() },
        suspend => EvaluateDefinitionRhsGuardsForMatchSuspensionFrame,
    )
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<OwnedGuards>();
    assert_traces::<SuspendedGuardEvaluation>();
    assert_traces::<SuspendedGuards>();
    assert_traces::<SuspendedReadonlyGuardEvaluation>();
};
