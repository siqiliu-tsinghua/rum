mod default_fallback;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
struct SuspendedDownValueApplication {
    candidate: Expr,
    normalized_candidate: Expr,
    symbol: Symbol,
    index: usize,
    lhs: Expr,
    rhs: Expr,
    rhs_may_splice_without_bindings: bool,
}

impl Session {
    pub(in crate::runtime) fn try_apply_down_values(
        &mut self,
        identity: PreparedDefinitionLookup<'_>,
    ) -> Result<Option<Expr>, EvalError> {
        let head = identity.head();
        let args = identity.args();
        let attributes = identity.attributes();
        let runtime_profile_head = runtime_path_timing_start("down_values.head_lookup");
        let Some(symbol) = expr_as_symbol(head) else {
            record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::None);
            return Ok(None);
        };
        record_profile_runtime_path_timing(runtime_profile_head, RuntimeProfileOutcome::Some);

        let runtime_profile_definition = runtime_path_timing_start("down_values.definition_lookup");
        let Some(definition) = self.definitions.get(symbol) else {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        };

        if definition.down_values.is_empty() {
            record_profile_runtime_path_timing(
                runtime_profile_definition,
                RuntimeProfileOutcome::None,
            );
            return Ok(None);
        }
        record_profile_runtime_path_timing(runtime_profile_definition, RuntimeProfileOutcome::Some);

        self.record_profile_down_value_scan(symbol);

        let args_slice = args.as_ref();
        let head_allows_raw_miss_cache = matches!(attributes.hold_attribute(), HoldAttribute::None);
        let down_values_cannot_match = !definition.has_ground_down_value_index()
            && head_allows_raw_miss_cache
            && definition
                .down_value_shallow_signatures()
                .is_some_and(|signatures| {
                    signatures
                        .iter()
                        .all(|signature| !signature.could_match_args(args_slice))
                });

        if is_system_symbol_name(head, heads::patterns::DEFAULT) {
            let runtime_profile_default =
                runtime_path_timing_start("down_values.direct_default_lookup");
            let has_guarded_default_values = definition.has_non_ground_down_values();
            let down_values = definition.down_values.clone();
            let direct_default = self.try_apply_direct_default_value(
                args_slice,
                &down_values,
                has_guarded_default_values,
            );
            record_profile_runtime_path_timing(
                runtime_profile_default,
                runtime_profile_outcome_for_option_result(&direct_default),
            );
            match direct_default {
                Ok(Some(value)) => return Ok(Some(value)),
                Ok(None) => {}
                Err(error) => {
                    if matches!(error, EvalError::InputRequired { .. }) {
                        default_fallback::suspend(
                            self,
                            &identity,
                            symbol,
                            down_values_cannot_match,
                        );
                    }
                    return Err(error);
                }
            }
        }

        self.try_apply_down_values_fallback(
            head,
            args,
            attributes,
            symbol,
            down_values_cannot_match,
        )
    }

    #[inline(always)]
    fn try_apply_down_values_fallback(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
        attributes: AttributeSet,
        symbol: &Symbol,
        down_values_cannot_match: bool,
    ) -> Result<Option<Expr>, EvalError> {
        if down_values_cannot_match {
            return Ok(None);
        }

        let runtime_profile_candidate = runtime_path_timing_start("down_values.candidate_build");
        let candidate = Expr::call_with_shared_args(head.clone(), args.clone());
        record_profile_runtime_path_timing(runtime_profile_candidate, RuntimeProfileOutcome::Ok);
        let runtime_profile_normalize =
            runtime_path_timing_start("down_values.normalize_candidate");
        let normalized_candidate =
            self.normalize_definition_candidate_for_matching(&candidate, attributes);
        record_profile_runtime_path_timing(runtime_profile_normalize, RuntimeProfileOutcome::Ok);

        let prior_dispatch_bytes = self.current_memory_limit.map(|_| {
            self.definitions
                .get(symbol)
                .unwrap()
                .down_value_dispatch_memory_proxy_bytes()
        });
        let indexed_scan = self
            .definitions
            .get(symbol)
            .expect("DownValue definition remains present during dispatch setup")
            .start_indexed_down_value_scan(&normalized_candidate, self.evaluation_epoch);
        if let Some(prior) = prior_dispatch_bytes {
            let current = self
                .definitions
                .get(symbol)
                .unwrap()
                .down_value_dispatch_memory_proxy_bytes();
            self.release_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::Definitions,
                prior,
            );
            self.charge_memory_owner(
                crate::runtime::debug_tools::resources::MemoryOwner::Definitions,
                current,
            );
        }
        if let Some(scan) = indexed_scan {
            self.try_apply_down_values_from_symbol(
                candidate,
                normalized_candidate,
                symbol.clone(),
                scan,
            )
        } else {
            self.try_apply_down_values_from_symbol(
                candidate,
                normalized_candidate,
                symbol.clone(),
                LinearDownValueScan::default(),
            )
        }
    }

    fn try_apply_down_values_from_symbol<Scan: DownValueScan>(
        &mut self,
        candidate: Expr,
        normalized_candidate: Expr,
        symbol: Symbol,
        mut scan: Scan,
    ) -> Result<Option<Expr>, EvalError> {
        let runtime_profile_value = runtime_path_timing_start("down_values.value_lookup");
        let Some((index, down_value)) = self.definitions.get(&symbol).and_then(|definition| {
            let index = scan.next_rule_index(definition, self.evaluation_epoch)?;
            definition
                .down_values
                .get(index)
                .map(|value| (index, value))
        }) else {
            record_profile_runtime_path_timing(runtime_profile_value, RuntimeProfileOutcome::None);
            return Ok(None);
        };
        record_profile_runtime_path_timing(runtime_profile_value, RuntimeProfileOutcome::Some);

        record_profile_definition_dispatch_candidate("down_value", &symbol);
        if down_value.lhs_is_ground {
            let matched = strip_hold_pattern(&down_value.lhs) == &normalized_candidate;
            if !matched {
                return self.try_apply_down_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    scan,
                );
            }

            let lhs = down_value.lhs.clone();
            let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
            let rhs = {
                let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
                substitute_rule_bindings_for_rhs(
                    &down_value.rhs,
                    &SymbolExprMap::default(),
                    down_value.rhs_may_splice_without_bindings,
                )
            };
            let trace_rhs = rhs.clone();
            self.push_rule_applied_trace(
                RuleApplicationSource::DownValue,
                &candidate,
                &lhs,
                &trace_rhs,
            );
            self.record_profile_down_value_application(&symbol);
            let _profile_definition_source = evaluation_profile_definition_source(symbol);
            let profile_rule_start = definition_rule_timing_start();
            let result = self.evaluate_definition_rhs(option_scope, rhs);
            record_profile_definition_rule_timing(
                "down_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            return result.map(Some);
        }

        let runtime_profile_prefilter = runtime_path_timing_start("down_values.prefilter");
        let prefilter_start = definition_dispatch_timing_start();
        let could_match = self.definition_lhs_could_match_candidate(
            &down_value.lhs,
            &down_value.matcher,
            &normalized_candidate,
        );
        record_profile_definition_dispatch_phase_timing(
            "down_value",
            &symbol,
            "prefilter",
            prefilter_start,
        );
        record_profile_runtime_path_timing(runtime_profile_prefilter, RuntimeProfileOutcome::Ok);
        if !could_match {
            record_profile_definition_dispatch_prefilter_skip("down_value", &symbol);
            return self.try_apply_down_values_from_symbol(
                candidate,
                normalized_candidate,
                symbol,
                scan,
            );
        }

        let runtime_profile_fast_match = runtime_path_timing_start("down_values.fast_match");
        let fast_match_start = definition_dispatch_timing_start();
        let fast_outcome = if down_value.matcher_attribute_revision == self.attribute_revision {
            self.try_fast_definition_match_compiled(&down_value.matcher, &normalized_candidate)
        } else {
            self.try_fast_definition_match(
                &down_value.lhs,
                &down_value.guards,
                &normalized_candidate,
            )
        };
        record_profile_definition_dispatch_phase_timing(
            "down_value",
            &symbol,
            "fast_match",
            fast_match_start,
        );
        record_profile_runtime_path_timing(
            runtime_profile_fast_match,
            runtime_profile_outcome_for_optional_fast_match(&fast_outcome),
        );
        if let Some(outcome) = fast_outcome {
            record_profile_definition_dispatch_fast_outcome("down_value", &symbol, outcome.matched);
            if !outcome.matched {
                return self.try_apply_down_values_from_symbol(
                    candidate,
                    normalized_candidate,
                    symbol,
                    scan,
                );
            }

            let lhs = down_value.lhs.clone();
            let option_scope = self.option_value_scope_for_definition(&lhs, &candidate);
            let rhs = {
                let _profile_rhs_source = expression_rebuild_source_symbol(symbol);
                let runtime_profile_rhs_substitution =
                    runtime_path_timing_start("down_values.rhs_substitution");
                let rhs_substitution_start = definition_dispatch_timing_start();
                let rhs = substitute_rule_bindings_for_rhs(
                    &down_value.rhs,
                    &outcome.bindings,
                    down_value.rhs_may_splice_without_bindings,
                );
                record_profile_definition_dispatch_phase_timing(
                    "down_value",
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
                RuleApplicationSource::DownValue,
                &candidate,
                &lhs,
                &trace_rhs,
            );
            self.record_profile_down_value_application(&symbol);
            let _profile_definition_source = evaluation_profile_definition_source(symbol);
            let profile_rule_start = definition_rule_timing_start();
            let result = self.evaluate_definition_rhs(option_scope, rhs);
            record_profile_definition_rule_timing(
                "down_value_rhs_eval",
                &symbol,
                index,
                profile_rule_start,
                result.is_ok(),
            );
            return result.map(Some);
        }

        let matcher_is_current = down_value.matcher_attribute_revision == self.attribute_revision;
        let cached_matcher = matcher_is_current.then(|| down_value.matcher.clone());
        let application = SuspendedDownValueApplication {
            candidate,
            normalized_candidate,
            symbol,
            index,
            lhs: down_value.lhs.clone(),
            rhs: down_value.rhs.clone(),
            rhs_may_splice_without_bindings: down_value.rhs_may_splice_without_bindings,
        };
        let matcher = cached_matcher.unwrap_or_else(|| {
            self.compile_down_value_matcher(&application.lhs, &down_value.guards)
        });
        let runtime_profile_compiled_matcher =
            runtime_path_timing_start("down_values.compiled_matcher");
        let compiled_matcher_start = definition_dispatch_timing_start();
        let match_result = self.match_compiled_owned(
            matcher,
            application.normalized_candidate.clone(),
            SymbolExprMap::default(),
        );
        record_profile_definition_dispatch_phase_timing(
            "down_value",
            &symbol,
            "compiled_matcher",
            compiled_matcher_start,
        );
        record_profile_runtime_path_timing(
            runtime_profile_compiled_matcher,
            runtime_profile_outcome_for_internal_result(&match_result),
        );

        match match_result {
            Ok(outcome) => self.finish_suspended_down_value_application(application, scan, outcome),
            Err(EvalError::InputRequired { request }) => {
                self.suspend_down_value_application(application, scan, request)
            }
            Err(error) => Err(error),
        }
    }

    #[cold]
    #[inline(never)]
    fn suspend_down_value_application<Scan: DownValueScan>(
        &mut self,
        application: SuspendedDownValueApplication,
        scan: Scan,
        request: RuntimeInputRequest,
    ) -> Result<Option<Expr>, EvalError> {
        crate::resume_or_suspend!(insert self, TryApplyDownValuesFromSymbolSuspensionFrame { application, scan });
        Err(EvalError::InputRequired { request })
    }

    fn finish_suspended_down_value_application<Scan: DownValueScan>(
        &mut self,
        application: SuspendedDownValueApplication,
        scan: Scan,
        outcome: CompiledMatchOutcome,
    ) -> Result<Option<Expr>, EvalError> {
        record_profile_definition_dispatch_matcher_outcome(
            "down_value",
            &application.symbol,
            outcome.matched,
        );
        if !outcome.matched {
            return self.try_apply_down_values_from_symbol(
                application.candidate,
                application.normalized_candidate,
                application.symbol,
                scan,
            );
        }

        let _profile_rhs_source = expression_rebuild_source_symbol(application.symbol);
        let runtime_profile_rhs_substitution =
            runtime_path_timing_start("down_values.rhs_substitution");
        let rhs_substitution_start = definition_dispatch_timing_start();
        let rhs = substitute_rule_bindings_for_rhs(
            &application.rhs,
            &outcome.bindings,
            application.rhs_may_splice_without_bindings,
        );
        record_profile_definition_dispatch_phase_timing(
            "down_value",
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
            RuleApplicationSource::DownValue,
            &application.candidate,
            &application.lhs,
            &trace_rhs,
        );
        self.record_profile_down_value_application(&application.symbol);
        let _profile_definition_source = evaluation_profile_definition_source(application.symbol);
        let profile_rule_start = definition_rule_timing_start();
        let result = self.evaluate_definition_rhs(option_scope, rhs);
        record_profile_definition_rule_timing(
            "down_value_rhs_eval",
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
    assert_traces::<SuspendedDownValueApplication>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct TryApplyDownValuesFromSymbolSuspensionFrame<Scan> {
    pub(super) application:
        crate::runtime::definitions::application::down_values::SuspendedDownValueApplication,
    pub(super) scan: Scan,
}

impl<Scan> crate::runtime::suspension::SuspensionFrame
    for TryApplyDownValuesFromSymbolSuspensionFrame<Scan>
where
    Scan: DownValueScan,
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut application = self.application.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .finish_suspended_down_value_application(application, scan, outcome)
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
