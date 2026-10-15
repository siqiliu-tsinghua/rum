use super::*;
use crate::runtime::kernel_api::{
    ContinueToken, ResumeStop, SlotWitness, TypedAction, TypedFrame, slot_witness,
};
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct SuspendedUpValueScan {
    candidate: Expr,
    normalized_candidate: Expr,
    up_values: Arc<[(Symbol, UpValue)]>,
    index: usize,
}

impl Session {
    pub(in crate::runtime) fn try_apply_up_values(
        &mut self,
        identity: PreparedDefinitionLookup<'_>,
    ) -> Result<Option<Expr>, EvalError> {
        let head = identity.head();
        let args = identity.args();
        let attributes = identity.attributes();
        if !self.has_up_value_definitions {
            return Ok(None);
        }

        let mut up_values = Vec::new();
        let args_slice = args.as_ref();

        for symbol in collect_upvalue_candidate_symbols(args_slice) {
            let Some(definition) = self.definitions.get(&symbol) else {
                continue;
            };
            up_values.extend(
                definition
                    .up_values
                    .iter()
                    .cloned()
                    .map(|up_value| (symbol, up_value)),
            );
        }

        if up_values.is_empty() {
            return Ok(None);
        }

        let candidate = Expr::call_with_shared_args(head.clone(), args.clone());
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(&candidate, attributes);

        self.try_apply_up_values_from_list(candidate, normalized_candidate, up_values, 0, None)
    }

    /// Walks the up-value candidates one per loop iteration instead of
    /// recursing per candidate, so a wide up-value list no longer grows the
    /// native stack. `pending` carries an already-computed matcher outcome
    /// across a suspension resume, so the outcome is merged exactly once per
    /// candidate on both the live and the resumed paths.
    fn try_apply_up_values_from_list(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        up_values: Vec<(Symbol, UpValue)>,
        mut index: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Option<Expr>, EvalError> {
        loop {
            let Some((symbol, up_value)) = up_values.get(index) else {
                return Ok(None);
            };
            let symbol = *symbol;

            if let Some(outcome) = pending.take() {
                record_profile_definition_dispatch_matcher_outcome(
                    "up_value",
                    &symbol,
                    outcome.matched,
                );
                if !outcome.matched {
                    index += 1;
                    continue;
                }

                let rhs_substitution_start = definition_dispatch_timing_start();
                let substituted = substitute_rule_bindings_for_rhs(
                    &up_value.rhs,
                    &outcome.bindings,
                    up_value.rhs_may_splice_without_bindings,
                );
                record_profile_definition_dispatch_phase_timing(
                    "up_value",
                    &symbol,
                    "rhs_substitution",
                    rhs_substitution_start,
                );
                let option_scope =
                    self.option_value_scope_for_definition(&up_value.lhs, &candidate);
                self.push_rule_applied_trace(
                    RuleApplicationSource::UpValue,
                    &candidate,
                    &up_value.lhs,
                    &substituted,
                );
                let profile_rule_start = definition_rule_timing_start();
                let result = self.evaluate_definition_application_rhs(option_scope, substituted);
                record_profile_definition_rule_timing(
                    "up_value_rhs_eval",
                    &symbol,
                    index,
                    profile_rule_start,
                    result.is_ok(),
                );
                return result;
            }

            record_profile_definition_dispatch_candidate("up_value", &symbol);
            let prefilter_start = definition_dispatch_timing_start();
            let could_match = self.definition_lhs_could_match_candidate(
                &up_value.lhs,
                &up_value.matcher,
                &normalized_candidate,
            );
            record_profile_definition_dispatch_phase_timing(
                "up_value",
                &symbol,
                "prefilter",
                prefilter_start,
            );
            if !could_match {
                record_profile_definition_dispatch_prefilter_skip("up_value", &symbol);
                index += 1;
                continue;
            }

            let fast_match_start = definition_dispatch_timing_start();
            let fast_outcome = if up_value.matcher_attribute_revision == self.attribute_revision {
                self.try_fast_definition_match_compiled(&up_value.matcher, &normalized_candidate)
            } else {
                self.try_fast_definition_match(
                    &up_value.lhs,
                    &up_value.guards,
                    &normalized_candidate,
                )
            };
            record_profile_definition_dispatch_phase_timing(
                "up_value",
                &symbol,
                "fast_match",
                fast_match_start,
            );
            if let Some(outcome) = fast_outcome {
                record_profile_definition_dispatch_fast_outcome(
                    "up_value",
                    &symbol,
                    outcome.matched,
                );
                if !outcome.matched {
                    index += 1;
                    continue;
                }

                let (lhs, substituted) = {
                    let rhs_substitution_start = definition_dispatch_timing_start();
                    let substituted = substitute_rule_bindings_for_rhs(
                        &up_value.rhs,
                        &outcome.bindings,
                        up_value.rhs_may_splice_without_bindings,
                    );
                    record_profile_definition_dispatch_phase_timing(
                        "up_value",
                        &symbol,
                        "rhs_substitution",
                        rhs_substitution_start,
                    );
                    (up_value.lhs.clone(), substituted)
                };
                let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
                self.push_rule_applied_trace(
                    RuleApplicationSource::UpValue,
                    &candidate,
                    &lhs,
                    &substituted,
                );
                let profile_rule_start = definition_rule_timing_start();
                let result = self.evaluate_definition_application_rhs(option_scope, substituted);
                record_profile_definition_rule_timing(
                    "up_value_rhs_eval",
                    &symbol,
                    index,
                    profile_rule_start,
                    result.is_ok(),
                );
                return result;
            }

            let up_value = up_value.clone();
            let matcher = if up_value.matcher_attribute_revision == self.attribute_revision {
                up_value.matcher.clone()
            } else {
                self.compile_down_value_matcher(&up_value.lhs, &up_value.guards)
            };
            let compiled_matcher_start = definition_dispatch_timing_start();
            let match_result = self.match_compiled_owned(
                matcher,
                normalized_candidate.clone(),
                SymbolExprMap::default(),
            );
            let witness = slot_witness(&match_result);
            record_profile_definition_dispatch_phase_timing(
                "up_value",
                &symbol,
                "compiled_matcher",
                compiled_matcher_start,
            );

            match match_result {
                Ok(outcome) => pending = Some(outcome),
                Err(EvalError::InputRequired { request }) => {
                    return self.suspend_up_value_scan_from_owned(
                        candidate,
                        normalized_candidate,
                        up_values,
                        index,
                        witness,
                        request,
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_up_value_scan_from_owned(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        up_values: Vec<(Symbol, UpValue)>,
        index: usize,
        witness: SlotWitness<CompiledMatchOutcome>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        self.suspend_up_value_scan(
            SuspendedUpValueScan {
                candidate,
                normalized_candidate,
                up_values: Arc::from(up_values),
                index,
            },
            witness,
            request,
        )
    }

    #[cold]
    #[inline(never)]
    fn suspend_up_value_scan(
        &mut self,
        state: SuspendedUpValueScan,
        witness: SlotWitness<CompiledMatchOutcome>,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        witness.suspend(
            self,
            request,
            TryApplyUpValuesFromListSuspensionFrame { state },
        )
    }

    fn resume_suspended_up_value_scan(
        &mut self,
        mut state: SuspendedUpValueScan,
        mut outcome: CompiledMatchOutcome,
        token: ContinueToken<'_, TryApplyUpValuesFromListSuspensionFrame>,
    ) -> Result<Option<Expr>, ResumeStop<Option<Expr>>> {
        loop {
            let Some((symbol, up_value)) = state.up_values.get(state.index) else {
                return Ok(None);
            };
            let symbol = *symbol;

            record_profile_definition_dispatch_matcher_outcome(
                "up_value",
                &symbol,
                outcome.matched,
            );
            if outcome.matched {
                let rhs_substitution_start = definition_dispatch_timing_start();
                let substituted = substitute_rule_bindings_for_rhs(
                    &up_value.rhs,
                    &outcome.bindings,
                    up_value.rhs_may_splice_without_bindings,
                );
                record_profile_definition_dispatch_phase_timing(
                    "up_value",
                    &symbol,
                    "rhs_substitution",
                    rhs_substitution_start,
                );
                let option_scope =
                    self.option_value_scope_for_definition(&up_value.lhs, &state.candidate);
                self.push_rule_applied_trace(
                    RuleApplicationSource::UpValue,
                    &state.candidate,
                    &up_value.lhs,
                    &substituted,
                );
                let profile_rule_start = definition_rule_timing_start();
                let result = self.evaluate_definition_application_rhs(option_scope, substituted);
                record_profile_definition_rule_timing(
                    "up_value_rhs_eval",
                    &symbol,
                    state.index,
                    profile_rule_start,
                    result.is_ok(),
                );
                return result.map_err(ResumeStop::failure);
            }
            state.index += 1;

            loop {
                let Some((symbol, up_value)) = state.up_values.get(state.index) else {
                    return Ok(None);
                };
                let symbol = *symbol;

                record_profile_definition_dispatch_candidate("up_value", &symbol);
                let prefilter_start = definition_dispatch_timing_start();
                let could_match = self.definition_lhs_could_match_candidate(
                    &up_value.lhs,
                    &up_value.matcher,
                    &state.normalized_candidate,
                );
                record_profile_definition_dispatch_phase_timing(
                    "up_value",
                    &symbol,
                    "prefilter",
                    prefilter_start,
                );
                if !could_match {
                    record_profile_definition_dispatch_prefilter_skip("up_value", &symbol);
                    state.index += 1;
                    continue;
                }

                let fast_match_start = definition_dispatch_timing_start();
                let fast_outcome = if up_value.matcher_attribute_revision == self.attribute_revision
                {
                    self.try_fast_definition_match_compiled(
                        &up_value.matcher,
                        &state.normalized_candidate,
                    )
                } else {
                    self.try_fast_definition_match(
                        &up_value.lhs,
                        &up_value.guards,
                        &state.normalized_candidate,
                    )
                };
                record_profile_definition_dispatch_phase_timing(
                    "up_value",
                    &symbol,
                    "fast_match",
                    fast_match_start,
                );
                if let Some(fast_outcome) = fast_outcome {
                    record_profile_definition_dispatch_fast_outcome(
                        "up_value",
                        &symbol,
                        fast_outcome.matched,
                    );
                    outcome = fast_outcome;
                    break;
                }

                let matcher = if up_value.matcher_attribute_revision == self.attribute_revision {
                    up_value.matcher.clone()
                } else {
                    self.compile_down_value_matcher(&up_value.lhs, &up_value.guards)
                };
                let compiled_matcher_start = definition_dispatch_timing_start();
                let match_result = self.match_compiled_owned(
                    matcher,
                    state.normalized_candidate.clone(),
                    SymbolExprMap::default(),
                );
                let witness = slot_witness(&match_result);
                record_profile_definition_dispatch_phase_timing(
                    "up_value",
                    &symbol,
                    "compiled_matcher",
                    compiled_matcher_start,
                );

                match match_result {
                    Ok(next_outcome) => {
                        outcome = next_outcome;
                        break;
                    }
                    Err(EvalError::InputRequired { request }) => {
                        return Err(token.suspend(
                            self,
                            witness,
                            request,
                            TryApplyUpValuesFromListSuspensionFrame { state },
                        ));
                    }
                    Err(error) => return Err(ResumeStop::failure(error)),
                }
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedUpValueScan>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct TryApplyUpValuesFromListSuspensionFrame {
    pub(super) state: crate::runtime::definitions::application::up_values::SuspendedUpValueScan,
}

impl TypedFrame for TryApplyUpValuesFromListSuspensionFrame {
    type Input = CompiledMatchOutcome;
    type Output = Option<Expr>;

    fn resume<'resume>(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<CompiledMatchOutcome, EvalError>,
        token: ContinueToken<'resume, Self>,
    ) -> TypedAction<Option<Expr>> {
        let state = self.state.clone();
        let resumed = (|| {
            let outcome = outcome.map_err(ResumeStop::failure)?;
            session.resume_suspended_up_value_scan(state, outcome, token)
        })();
        TypedAction::complete_work(resumed)
    }
}
