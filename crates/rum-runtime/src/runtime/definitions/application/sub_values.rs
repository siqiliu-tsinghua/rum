use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
struct SuspendedSubValueApplication {
    candidate: Expr,
    normalized_candidate: Expr,
    symbol: Symbol,
    index: usize,
    lhs: Expr,
    rhs: Expr,
    rhs_may_splice_without_bindings: bool,
}

impl Session {
    pub(in crate::runtime) fn try_apply_sub_values(
        &mut self,
        identity: PreparedDefinitionLookup<'_>,
    ) -> Result<Option<Expr>, EvalError> {
        let head = identity.head();
        let args = identity.args();
        let attributes = identity.attributes();
        let runtime_profile_head = runtime_path_timing_start("sub_values.base_head_lookup");
        let Some(symbol) = base_head_symbol_of_nested_call(head).cloned() else {
            record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::None);
            return Ok(None);
        };
        record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::Some);

        let runtime_profile_definition = runtime_path_timing_start("sub_values.definition_lookup");
        let Some(definition) = self.definitions.get(&symbol) else {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        };

        if definition.sub_values.is_empty() {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        }
        record_profile_runtime_path_timing(runtime_profile_definition, RuntimeProfileOutcome::Some);

        let runtime_profile_candidate = runtime_path_timing_start("sub_values.candidate_build");
        let candidate = Expr::call_with_shared_args(head.clone(), args.clone());
        record_profile_runtime_path_timing(runtime_profile_candidate, RuntimeProfileOutcome::Ok);
        let runtime_profile_normalize = runtime_path_timing_start("sub_values.normalize_candidate");
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(&candidate, attributes);
        record_profile_runtime_path_timing(runtime_profile_normalize, RuntimeProfileOutcome::Ok);

        self.try_apply_sub_values_from_symbol(candidate, normalized_candidate, symbol, 0, None)
    }

    /// Walks the sub-value candidates one per loop iteration instead of
    /// recursing per candidate, so a wide sub-value list no longer grows the
    /// native stack. `pending` carries an already-computed matcher outcome
    /// across a suspension resume, so the outcome is merged exactly once per
    /// candidate on both the live and the resumed paths.
    fn try_apply_sub_values_from_symbol(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        symbol: Symbol,
        mut index: usize,
        mut pending: Option<CompiledMatchOutcome>,
    ) -> Result<Option<Expr>, EvalError> {
        loop {
            let runtime_profile_value = runtime_path_timing_start("sub_values.value_lookup");
            let Some(sub_value) = self
                .definitions
                .get(&symbol)
                .and_then(|definition| definition.sub_values.get(index))
            else {
                record_profile_runtime_path_timing(
                    runtime_profile_value,
                    RuntimeProfileOutcome::None,
                );
                return Ok(None);
            };
            record_profile_runtime_path_timing(runtime_profile_value, RuntimeProfileOutcome::Some);

            if let Some(outcome) = pending.take() {
                record_profile_definition_dispatch_matcher_outcome(
                    "sub_value",
                    &symbol,
                    outcome.matched,
                );
                if !outcome.matched {
                    index += 1;
                    continue;
                }

                let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
                let runtime_profile_rhs_substitution =
                    runtime_path_timing_start("sub_values.rhs_substitution");
                let rhs_substitution_start = definition_dispatch_timing_start();
                let lhs = sub_value.lhs.clone();
                let rhs = substitute_rule_bindings_for_rhs(
                    &sub_value.rhs,
                    &outcome.bindings,
                    sub_value.rhs_may_splice_without_bindings,
                );
                record_profile_definition_dispatch_phase_timing(
                    "sub_value",
                    &symbol,
                    "rhs_substitution",
                    rhs_substitution_start,
                );
                record_profile_runtime_path_timing(
                    runtime_profile_rhs_substitution,
                    RuntimeProfileOutcome::Ok,
                );
                let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
                let trace_rhs = rhs.clone();
                self.push_rule_applied_trace(
                    RuleApplicationSource::SubValue,
                    &candidate,
                    &lhs,
                    &trace_rhs,
                );
                self.record_profile_sub_value_application(&symbol);
                let _profile_definition_source = evaluation_profile_definition_source(symbol);
                let profile_rule_start = definition_rule_timing_start();
                let result = self.evaluate_definition_rhs(option_scope, rhs);
                record_profile_definition_rule_timing(
                    "sub_value_rhs_eval",
                    &symbol,
                    index,
                    profile_rule_start,
                    result.is_ok(),
                );
                return result.map(Some);
            }

            record_profile_definition_dispatch_candidate("sub_value", &symbol);
            let runtime_profile_prefilter = runtime_path_timing_start("sub_values.prefilter");
            let prefilter_start = definition_dispatch_timing_start();
            let could_match = self.definition_lhs_could_match_candidate(
                &sub_value.lhs,
                &sub_value.matcher,
                &normalized_candidate,
            );
            record_profile_definition_dispatch_phase_timing(
                "sub_value",
                &symbol,
                "prefilter",
                prefilter_start,
            );
            record_profile_runtime_path_timing(
                runtime_profile_prefilter,
                RuntimeProfileOutcome::Ok,
            );
            if !could_match {
                record_profile_definition_dispatch_prefilter_skip("sub_value", &symbol);
                index += 1;
                continue;
            }

            let runtime_profile_fast_match = runtime_path_timing_start("sub_values.fast_match");
            let fast_match_start = definition_dispatch_timing_start();
            let fast_outcome = if sub_value.matcher_attribute_revision == self.attribute_revision {
                self.try_fast_definition_match_compiled(&sub_value.matcher, &normalized_candidate)
            } else {
                self.try_fast_definition_match(
                    &sub_value.lhs,
                    &sub_value.guards,
                    &normalized_candidate,
                )
            };
            record_profile_definition_dispatch_phase_timing(
                "sub_value",
                &symbol,
                "fast_match",
                fast_match_start,
            );
            record_profile_runtime_path_timing(
                runtime_profile_fast_match,
                runtime_profile_outcome_for_optional_fast_match(&fast_outcome),
            );
            if let Some(outcome) = fast_outcome {
                record_profile_definition_dispatch_fast_outcome(
                    "sub_value",
                    &symbol,
                    outcome.matched,
                );
                if !outcome.matched {
                    index += 1;
                    continue;
                }

                let lhs = sub_value.lhs.clone();
                let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
                let rhs = {
                    let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
                    let runtime_profile_rhs_substitution =
                        runtime_path_timing_start("sub_values.rhs_substitution");
                    let rhs_substitution_start = definition_dispatch_timing_start();
                    let rhs = substitute_rule_bindings_for_rhs(
                        &sub_value.rhs,
                        &outcome.bindings,
                        sub_value.rhs_may_splice_without_bindings,
                    );
                    record_profile_definition_dispatch_phase_timing(
                        "sub_value",
                        &symbol,
                        "rhs_substitution",
                        rhs_substitution_start,
                    );
                    record_profile_runtime_path_timing(
                        runtime_profile_rhs_substitution,
                        RuntimeProfileOutcome::Ok,
                    );
                    rhs
                };
                let trace_rhs = rhs.clone();
                self.push_rule_applied_trace(
                    RuleApplicationSource::SubValue,
                    &candidate,
                    &lhs,
                    &trace_rhs,
                );
                self.record_profile_sub_value_application(&symbol);
                let _profile_definition_source = evaluation_profile_definition_source(symbol);
                let profile_rule_start = definition_rule_timing_start();
                let result = self.evaluate_definition_rhs(option_scope, rhs);
                record_profile_definition_rule_timing(
                    "sub_value_rhs_eval",
                    &symbol,
                    index,
                    profile_rule_start,
                    result.is_ok(),
                );
                return result.map(Some);
            }

            let matcher_is_current =
                sub_value.matcher_attribute_revision == self.attribute_revision;
            let cached_matcher = matcher_is_current.then(|| sub_value.matcher.clone());
            let application = SuspendedSubValueApplication {
                candidate,
                normalized_candidate,
                symbol,
                index,
                lhs: sub_value.lhs.clone(),
                rhs: sub_value.rhs.clone(),
                rhs_may_splice_without_bindings: sub_value.rhs_may_splice_without_bindings,
            };
            let matcher = cached_matcher.unwrap_or_else(|| {
                self.compile_down_value_matcher(&application.lhs, &sub_value.guards)
            });
            let runtime_profile_compiled_matcher =
                runtime_path_timing_start("sub_values.compiled_matcher");
            let compiled_matcher_start = definition_dispatch_timing_start();
            let match_result = self.match_compiled_owned(
                matcher,
                application.normalized_candidate.clone(),
                SymbolExprMap::default(),
            );
            record_profile_definition_dispatch_phase_timing(
                "sub_value",
                &symbol,
                "compiled_matcher",
                compiled_matcher_start,
            );
            record_profile_runtime_path_timing(
                runtime_profile_compiled_matcher,
                runtime_profile_outcome_for_internal_result(&match_result),
            );

            match match_result {
                Ok(outcome) => {
                    return self.finish_suspended_sub_value_application(application, outcome);
                }
                Err(EvalError::InputRequired { request }) => {
                    return self.suspend_sub_value_application(application, request);
                }
                Err(error) => return Err(error),
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_sub_value_application(
        &mut self,
        application: SuspendedSubValueApplication,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        crate::resume_or_suspend!(insert self, TryApplySubValuesFromSymbolSuspensionFrame { application });
        Err(EvalError::InputRequired { request })
    }

    fn finish_suspended_sub_value_application(
        &mut self,
        application: SuspendedSubValueApplication,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        record_profile_definition_dispatch_matcher_outcome(
            "sub_value",
            &application.symbol,
            outcome.matched,
        );
        if !outcome.matched {
            return self.try_apply_sub_values_from_symbol(
                application.candidate,
                application.normalized_candidate,
                application.symbol,
                application.index + 1,
                None,
            );
        }

        let _profile_rhs_source = expression_rebuild_source_symbol(application.symbol);
        let runtime_profile_rhs_substitution =
            runtime_path_timing_start("sub_values.rhs_substitution");
        let rhs_substitution_start = definition_dispatch_timing_start();
        let rhs = substitute_rule_bindings_for_rhs(
            &application.rhs,
            &outcome.bindings,
            application.rhs_may_splice_without_bindings,
        );
        record_profile_definition_dispatch_phase_timing(
            "sub_value",
            &application.symbol,
            "rhs_substitution",
            rhs_substitution_start,
        );
        record_profile_runtime_path_timing(
            runtime_profile_rhs_substitution,
            RuntimeProfileOutcome::Ok,
        );
        let option_scope =
            self.option_value_scope_for_definition(&application.lhs, &application.candidate);
        let trace_rhs = rhs.clone();
        self.push_rule_applied_trace(
            RuleApplicationSource::SubValue,
            &application.candidate,
            &application.lhs,
            &trace_rhs,
        );
        self.record_profile_sub_value_application(&application.symbol);
        let _profile_definition_source = evaluation_profile_definition_source(application.symbol);
        let profile_rule_start = definition_rule_timing_start();
        let result = self.evaluate_definition_rhs(option_scope, rhs);
        record_profile_definition_rule_timing(
            "sub_value_rhs_eval",
            &application.symbol,
            application.index,
            profile_rule_start,
            result.is_ok(),
        );
        result.map(Some)
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedSubValueApplication>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct TryApplySubValuesFromSymbolSuspensionFrame {
    pub(super) application:
        crate::runtime::definitions::application::sub_values::SuspendedSubValueApplication,
}

impl crate::runtime::suspension::SuspensionFrame for TryApplySubValuesFromSymbolSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut application = self.application.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .finish_suspended_sub_value_application(application, outcome)
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
