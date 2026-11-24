//! Host observations retain every public channel and every ten-field native row.
use super::native::{self, NativeObservation, ObserverScope};
use crate::oracle::obligations::{
    bindings::{BindingError, BindingResult, TupleKey},
    content_digest,
};
use crate::{
    HostEvalOutcome, HostInputKind, HostInputRequest, HostMessage, HostSession, HostTraceEvent,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::{Arc, OnceLock},
};

#[derive(Clone, Debug)]
pub(crate) struct Program {
    pub id: String,
    pub family: String,
    pub key: TupleKey,
    pub source: String,
    pub setup: String,
    pub source_hash: String,
    pub setup_hash: String,
    pub answers_hash: String,
    pub plan_hash: String,
    pub pauses: Vec<(HostInputRequest, String)>,
    pub trace: bool,
    pub producer_route: String,
    pub phases: Vec<String>,
    pub canonical: Option<super::CanonicalFixture>,
    pub paired: Option<Box<Program>>,
}

impl Program {
    pub fn total_requests(&self) -> usize {
        self.pauses.len() + self.paired.as_ref().map_or(0, |p| p.total_requests())
    }

    pub fn subject_requests(&self) -> usize {
        self.phases
            .iter()
            .filter(|phase| credited_phase(phase))
            .count()
    }

    fn native_entries(&self) -> usize {
        usize::from(matches!(self.key.row(), "P0001" | "P0002"))
    }
}

pub(crate) fn credited_phase(phase: &str) -> bool {
    !matches!(phase, "prior-uncredited" | "other-producer-uncredited")
}

fn program(hash: &str) -> BindingResult<String> {
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(BindingError::ProgramIdentity);
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/suspension_oracle/fixtures/obligations/programs")
        .join(format!("{hash}.rum"));
    let bytes = std::fs::read(path).map_err(|_| BindingError::ProgramIdentity)?;
    if content_digest(&bytes) != hash {
        return Err(BindingError::ProgramIdentity);
    }
    String::from_utf8(bytes).map_err(|_| BindingError::ProgramIdentity)
}

fn unhex(text: &str) -> BindingResult<String> {
    if text.len() % 2 != 0 || !text.is_ascii() {
        return Err(BindingError::ProgramIdentity);
    }
    let bytes = (0..text.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&text[index..index + 2], 16)
                .map_err(|_| BindingError::ProgramIdentity)
        })
        .collect::<BindingResult<Vec<_>>>()?;
    String::from_utf8(bytes).map_err(|_| BindingError::ProgramIdentity)
}

pub(crate) fn programs() -> BindingResult<Arc<BTreeMap<String, Program>>> {
    static PLANS: OnceLock<BindingResult<Arc<BTreeMap<String, Program>>>> = OnceLock::new();
    PLANS.get_or_init(|| load_programs().map(Arc::new)).clone()
}

fn guest_route(row: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match row {
        "P2611" | "P2612" => Some((
            "crates/rum-jupyter/src/adapter/worker.rs",
            "Print[\"c52.cancel\"]",
            "observed_worker::observe_segment>run_segment.Eval-or-Resume>InputString;tail=host_outcome_to_kernel;second=run_segment.Resume;terminal=logical_profile_test_rows;handoff=outside-counted-epoch",
        )),
        "P1470" | "P1471" => Some((
            "crates/rum-runtime/src/runtime/resolution/resume_frames.rs",
            "C8FixedPointStep[",
            "C8FixedPointStep>Input.c8step1>EvaluateProfiledFixedPointSuspensionFrame::resume>resume_fixed_point_from>Input.c8step2;tail=publish-own-fixed-point-pass;second=ResumeFixedPointFromSuspensionFrame::resume>Input.c8step3",
        )),
        "P0813" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "Catch[",
            "HostSession::eval>Catch>eval_catch>InputString;tail=resume_catch_after_body;second=resumed-catch-body-continuation;terminal=actual_owns_eval_timeout",
        )),
        "P2600" => Some((
            "crates/rum-host/src/session.rs",
            "native623.one",
            "HostSession::call.c47driver>call_unprotected>eval_request>InputString;tail=structured-result-output;second=HostSession::resume_with_input;terminal=actual_owns_eval_timeout",
        )),
        "P0267" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values.rs",
            "c7f[2]",
            "down-value-guard>Input.c7guard>TryApplyDownValuesFromSymbolSuspensionFrame::resume>finish_suspended_down_value_application>Input.c7tail;tail=TypedAction::complete_work;second=rhs-expression-continuation",
        )),
        "P0312" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/sub_values.rs",
            "",
            "sub-value-guard>Input.c7guard>TryApplySubValuesFromSymbolSuspensionFrame::resume>finish_suspended_sub_value_application>Input.c7tail;tail=TypedAction::complete_work;second=rhs-expression-continuation",
        )),
        "P0323" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/up_values.rs",
            "",
            "up-value-guard>Input.c7guard>TryApplyUpValuesFromListSuspensionFrame::resume>resume_suspended_up_value_scan>Input.c7tail;tail=TypedAction::complete_work;second=rhs-expression-continuation",
        )),
        "P0410" | "P0416" => Some((
            "crates/rum-runtime/src/runtime/definitions/definitions.rs",
            "",
            "Set-or-Rule>eval_set-or-eval_rule>Input;tail=definition-assignment-or-rule-lhs;second=rhs-or-lhs-expression-continuation",
        )),
        "P0492" => Some((
            "crates/rum-runtime/src/runtime/eval_core/call.rs",
            "",
            "prepared-call>resume_call_with_dispatch_plan>Input;tail=prepared-argument-status;second=prepared-call-expression-continuation",
        )),
        "P0820" | "P0825" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "",
            "Catch>resume_catch_for_throw-or-resume_catch_for_bypassing_throw>Input;tail=catch-pattern-result;second=pattern-expression-continuation",
        )),
        "P1477" => Some((
            "crates/rum-runtime/src/runtime/scoping/callable/status_application.rs",
            "",
            "Failure-property>apply_status_function>Input;tail=status-function-result;second=property-expression-continuation",
        )),
        "P2593" => Some((
            "crates/rum-stdlib/src/structure_ops/surface.rs",
            "First[",
            "First>eval_first>Input;tail=lazy-first-result;second=lazy-expression-continuation",
        )),
        "P1566" => Some((
            "crates/rum-runtime/src/runtime/session/session_requests.rs",
            "Input[\"c9native1\"]",
            "HostSession::eval>eval_expr_request_from_checkpoint>evaluate;tail=message-checkpoint-restoration;second=HostSession::resume_with_input>resume_eval_request_from_checkpoint;terminal=actual_message_checkpoint_state",
        )),
        "P1568" => Some((
            "crates/rum-runtime/src/runtime/session/session_requests.rs",
            "Input[\"c9native2\"]",
            "HostSession::eval>Input.c9native1>HostSession::resume_with_input>resume_eval_request_from_checkpoint>resume_suspended_evaluation;tail=message-checkpoint-restoration;second=HostSession::resume_with_input>Input.c9native3;terminal=actual_message_checkpoint_state",
        )),
        "P0369" | "P0372" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_tags.rs",
            "",
            "UpSet-or-TagSet>eval_up_set-or-eval_tag_set>Input.c6rhs;tail=tagged-definition-publication;second=rhs-expression-continuation",
        )),
        "P0408" => Some((
            "crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs",
            "Set[c7f[",
            "Set>Input.c7rhs>EvalSetSuspensionFrame::resume>resume_set_with_rhs>Input.c7tail;tail=resumed-definition-assignment;second=lhs-expression-continuation",
        )),
        "P0498" => Some((
            "crates/rum-runtime/src/runtime/eval_core/call.rs",
            "AssociateTo[",
            "AssociateTo>resume_suspended_call_with_dispatch_plan>Input;tail=resumed-call-dispatch;second=target-expression-continuation",
        )),
        "P0817" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "WithCleanup[Throw[",
            "Catch>WithCleanup.Throw>resume_catch_after_body>Input;tail=bypassing-throw-catch-dispatch;second=tag-pattern-continuation",
        )),
        "P1708" => Some((
            "crates/rum-stdlib/src/associations/basic.rs",
            "Normal[",
            "Normal>eval_normal>Input;tail=lazy-consumer-completion;second=source-expression-continuation",
        )),
        "P1814" | "P1817" => Some((
            "crates/rum-stdlib/src/associations/mutation.rs",
            "",
            "held-association-mutation>resume-after-target-preparation>Input;tail=association-target-adoption;second=target-expression-continuation",
        )),
        "P2591" => Some((
            "crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs",
            "Take[",
            "Take>eval_take_lazy_list>Input;tail=lazy-take-completion;second=source-expression-continuation",
        )),
        "P0287" | "P0292" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/rhs.rs",
            "",
            "definition-rhs>evaluate_definition_rhs-or-application_rhs>Input;tail=definition-rhs-accounting;second=rhs-expression-continuation",
        )),
        "P0426" | "P0428" => Some((
            "crates/rum-runtime/src/runtime/definitions/definitions.rs",
            "",
            "Set>finish_definition_assignment>Input;tail=definition-assignment;second=assignment-expression-continuation",
        )),
        "P0485" | "P0494" => Some((
            "crates/rum-runtime/src/runtime/eval_core/call.rs",
            "",
            "evaluate_call>resume_call_with_dispatch_plan>Input;tail=call-dispatch;second=body-expression-continuation",
        )),
        "P0818" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "",
            "Catch>resume_catch_after_body>Input;tail=catch-body-completion;second=body-expression-continuation",
        )),
        "P1937" => Some((
            "crates/rum-stdlib/src/functional_ops/arrays/mod.rs",
            "",
            "Dot>eval_dot>Input;tail=dot-evaluation;second=argument-expression-continuation",
        )),
        "P2137" => Some((
            "crates/rum-stdlib/src/lazy/source.rs",
            "",
            "lazy-source>pull_source>Input;tail=lazy-source-pull;second=source-expression-continuation",
        )),
        "P0470" => Some((
            "crates/rum-runtime/src/runtime/definitions/rule_lhs.rs",
            "Rule[c21rulelhs[",
            "Rule>drive_rule_lhs>Input.finish;tail=rule-lhs-child-parent-acceptance;second=lhs-expression-continuation",
        )),
        "P0936" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs",
            "MatchQ[",
            "MatchQ>drive_matcher_pattern>Input.finish;tail=matcher-pattern-adoption;second=pattern-expression-continuation",
        )),
        "P1070" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs",
            "ReplacePart[",
            "ReplacePart>select_guarded_replace_part_targets_owned>Input.finish;tail=guarded-target-selection;second=guard-expression-continuation",
        )),
        "P1610" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/count_like.rs",
            "Count[",
            "Count>finish_count_like_after_pattern>Input.finish;tail=traversal-settings-adoption;second=level-expression-continuation",
        )),
        "P1613" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/count_like.rs",
            "x_?c19test",
            "Count-or-MemberQ-or-FreeQ>finish_count_like_after_traversal>Input.finish;tail=count-like-traversal-completion;second=predicate-expression-continuation",
        )),
        "P1621" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/position/options.rs",
            "Position[",
            "Position>finish_position_option>Input.finish;tail=position-option-adoption;second=level-expression-continuation",
        )),
        "P1636" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/position.rs",
            "Position[",
            "Position>finish_position_after_resolved_traversal>Input.finish;tail=position-traversal-completion;second=guard-expression-continuation",
        )),
        "P0450" => Some((
            "crates/rum-runtime/src/runtime/definitions/rhs_guards.rs",
            "c19suffix[17]",
            "guarded-definition>evaluate_owned_guards>evaluate_with_option_value_scope>Input;tail=owned-guard-suffix;second=EvaluateOwnedGuardsSuspensionFrame>resume_suspended_guard_deferred>evaluate_suspended_guards_deferred",
        )),
        "P0455" => Some((
            "crates/rum-runtime/src/runtime/definitions/rhs_guards.rs",
            "c19suffix[17]",
            "guarded-definition>evaluate_owned_guards>InputString(first)>EvaluateOwnedGuardsSuspensionFrame>resume_suspended_guard_deferred>evaluate_suspended_guards_deferred>evaluate_with_option_value_scope>Input;tail=deferred-guard-suffix;second=EvaluateOwnedGuardsSuspensionFrame>evaluate_suspended_guards_deferred",
        )),
        "P0256" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values.rs",
            "",
            "ground-down-value>try_apply_down_values_from_symbol>evaluate_definition_application_rhs>Input;tail=definition-rule.down_value_rhs_eval;second=EvaluateDefinitionApplicationRhsSuspensionFrame>rhs-expression-continuation",
        )),
        "P0259" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values.rs",
            "",
            "fast-down-value>try_apply_down_values_from_symbol>evaluate_definition_application_rhs>Input;tail=definition-rule.down_value_rhs_eval;second=rhs-expression-continuation",
        )),
        "P0266" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values.rs",
            "",
            "compiled-down-value>finish_suspended_down_value_application>evaluate_definition_application_rhs>Input;tail=definition-rule.down_value_rhs_eval;second=rhs-expression-continuation",
        )),
        "P0314" | "P0315" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/up_values.rs",
            "",
            "up-value>try_apply_up_values_from_list>evaluate_definition_application_rhs>Input;tail=definition-rule.up_value_rhs_eval;second=rhs-expression-continuation",
        )),
        "P0413" => Some((
            "crates/rum-runtime/src/runtime/definitions/definitions.rs",
            "",
            "Set>eval_set>resume_set_with_rhs>Input;tail=definition-assignment;second=lhs-expression-continuation",
        )),
        "P0431" => Some((
            "crates/rum-runtime/src/runtime/definitions/definitions.rs",
            "",
            "Set>finish_definition_assignment>prepare_definition_lhs>Input;tail=definition-assignment;second=lhs-expression-continuation",
        )),
        "P0484" => Some((
            "crates/rum-runtime/src/runtime/eval_core/call.rs",
            "",
            "evaluate_call>resume_call_after_head>Input;tail=call-head-continuation;second=body-expression-continuation",
        )),
        "P0489" => Some((
            "crates/rum-runtime/src/runtime/eval_core/call.rs",
            "",
            "resume_call_after_head>resume_call_with_dispatch_plan>Input;tail=call-dispatch-continuation;second=body-expression-continuation",
        )),
        "P0509" => Some((
            "crates/rum-runtime/src/runtime/eval_core/mod.rs",
            "",
            "evaluate_inner_unstable>evaluate_symbol>Input;tail=evaluate-symbol;second=own-value-continuation",
        )),
        "P0511" => Some((
            "crates/rum-runtime/src/runtime/eval_core/mod.rs",
            "",
            "evaluate_inner_unstable>evaluate_call>Input;tail=evaluate-call;second=body-expression-continuation",
        )),
        "P0515" => Some((
            "crates/rum-runtime/src/runtime/eval_core/mod.rs",
            "",
            "evaluate_atom_fast_path>evaluate>Input;tail=evaluate-atom-fallback;second=body-expression-continuation",
        )),
        "P0814" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "",
            "Catch>eval_catch>resume_catch_after_body>Input;tail=catch-error-continuation;second=body-expression-continuation",
        )),
        "P1472" => Some((
            "crates/rum-runtime/src/runtime/resolution.rs",
            "",
            "evaluate>evaluate_profiled_fixed_point>Input;tail=profiled-fixed-point;second=body-expression-continuation",
        )),
        "P1507" => Some((
            "crates/rum-runtime/src/runtime/scoping/lexical/module.rs",
            "",
            "Module>eval_module>resume_module_initializers>Input;tail=module-cleanup;second=initializer-or-body-expression-continuation",
        )),
        "C5.default.profiled.finish" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs",
            "Default[c12default]",
            "Default>exact-guard>Input.guard>try_apply_direct_default_value>suspend_profiled>suspend_frame.true;tail=pause_call_stage_at_finish.down_values.direct_default_lookup;second=DirectDefaultFallbackFrame::resume>try_apply_down_values_fallback>Input.fallback",
        )),
        "C3.evaluate_call.unstable_head" => Some((
            "crates/rum-runtime/src/runtime/eval_core/call.rs",
            "AllTrue[",
            "AllTrue.predicate>evaluate_call.unstable_head>evaluate_unstable_call_head>Input;tail=record_profile_runtime_path_timing_adopted.evaluate_call.total;second=predicate-expression-continuation>evaluate_call.unstable_head",
        )),
        "P0459" => Some((
            "crates/rum-runtime/src/runtime/definitions/rhs_guards.rs",
            "ReplacePart[{1,2},{i_}",
            "ReplacePart>readonly-guards.first-target>second-target.first-guard>evaluate_readonly_suspended_guards.index-1>evaluate_with_option_value_scope>Input;tail=EvaluateReadonlySuspendedGuardsSuspensionFrame>remaining-suffix-guard;second=guard-expression-continuation",
        )),
        "P1238" => Some((
            "crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs",
            "MatchQ[c14seq[",
            "MatchQ>match_compiled_sequence_choices>match_compiled_sequence_pattern_owned>c14pred>Input;tail=DriveSequenceChoiceResultSuspensionFrame>ContinueSequenceChoiceResultFrame>finish_sequence_choice;second=predicate-expression-continuation",
        )),
        "P1240" => Some((
            "crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs",
            "MatchQ[c14seq[",
            "MatchQ>Flat.c14seq>match_compiled_flat_sequence_choices>match_compiled_sequence_pattern_owned>c14pred>Input;tail=DriveSequenceChoiceResultSuspensionFrame>ContinueSequenceChoiceResultFrame>finish_sequence_choice;second=predicate-expression-continuation",
        )),
        "P0278" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/replace_all.rs",
            "ReplaceAll[17,x_Integer?c20test",
            "ReplaceAll.atom>apply_replace_all_owned>apply_rewrite_rule_plan_once>c20test>Input;tail=suspend_replace_all_atom>ReplaceAllTraversalOutcome;second=predicate-expression-continuation",
        )),
        "P0281" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/replace_all.rs",
            "ReplaceAll[{17,23},x_Integer?c20test",
            "ReplaceAll.list>drive_replace_all_traversal>apply_rewrite_rule_plan_once>c20test>Input;tail=ApplyReplaceAllOwnedSuspensionFrame>finish_replace_all_visit>resume_replace_all_traversal;second=predicate-expression-continuation",
        )),
        "P0284" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/replace_all.rs",
            "ReplaceAll[17,x_Integer?c20test",
            "ReplaceAll>apply_rules_once_from_list>match_compiled_owned>c20test>Input;tail=ApplyRulesOnceFromListSuspensionFrame>substitute_rule_bindings_for_rhs;second=predicate-expression-continuation",
        )),
        "P0316" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/up_values.rs",
            "c22up[c22tag,23]",
            "up-value>try_apply_up_values_from_list>match_compiled_owned>c22accept>Input;tail=TryApplyUpValuesFromListSuspensionFrame>resume_suspended_up_value_scan;second=predicate-expression-continuation",
        )),
        "P0321" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/up_values.rs",
            "c22up[c22tag,23]",
            "up-value>c22reject-uncredited>resume_suspended_up_value_scan>match_compiled_owned>c22accept>Input;tail=TryApplyUpValuesFromListSuspensionFrame>resume_suspended_up_value_scan;second=predicate-expression-continuation",
        )),
        "P0260" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values.rs",
            "c18match[17]",
            "down-value>try_apply_down_values_from_symbol>match_compiled_owned>predicate>Input;tail=record_profile_runtime_path_timing_adopted.down_values.compiled_matcher>TryApplyDownValuesFromSymbolSuspensionFrame>finish_suspended_down_value_application;second=predicate-expression-continuation",
        )),
        "P0305" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/sub_values.rs",
            "c18match[17][23]",
            "sub-value>try_apply_sub_values_from_symbol>match_compiled_owned>predicate>Input;tail=record_profile_runtime_path_timing_adopted.sub_values.compiled_matcher>TryApplySubValuesFromSymbolSuspensionFrame>finish_suspended_sub_value_application;second=predicate-expression-continuation",
        )),
        "P0527" => Some((
            "crates/rum-runtime/src/runtime/eval_core/symbol.rs",
            "c25owned",
            "evaluate_symbol_after_autoload>evaluate_stored_own_value>evaluate_guard_sequence_owned>Input;tail=EvaluateStoredOwnValueSuspensionFrame>evaluate_accepted_own_value;second=guard-expression-continuation",
        )),
        "P0338" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs",
            "NormalizeRules[c25rules]",
            "NormalizeRules>resolve_rewrite_rule_exprs_owned>evaluate>Input;tail=ResolveRewriteRuleExprsOwnedSuspensionFrame>normalize_rewrite_rule_exprs_from_resolved;second=source-expression-continuation",
        )),
        "P1155" => Some((
            "crates/rum-runtime/src/runtime/options/read_phase.rs",
            "Monitor[",
            "Monitor>eval_option_syntax>evaluate>Input;tail=OptionSyntaxDecode>MonitorOptionWorking.syntax.finish;second=OptionReadSuspensionFrame::resume>callback-expression-continuation",
        )),
        "P1139" => Some((
            "crates/rum-runtime/src/runtime/options/core.rs",
            "",
            "system_option_bool>option_value_for_symbol>resolve_option_entry_value>Input;tail=BoolOptionDecode>OptionReadContinuation::finish;second=OptionReadSuspensionFrame::resume>callback-expression-continuation",
        )),
        "P1142" => Some((
            "crates/rum-runtime/src/runtime/options/core.rs",
            "",
            "option_rule_expr_for_symbol>resolve_option_entry_value>Input;tail=RuleOptionDecode>OptionReadContinuation::finish;second=OptionReadSuspensionFrame::resume>callback-expression-continuation",
        )),
        "P1147" => Some((
            "crates/rum-runtime/src/runtime/options/core.rs",
            "",
            "runtime_option_expr_for_symbol>evaluate>Input;tail=RuntimeOptionDecode>OptionReadContinuation::finish;second=OptionReadSuspensionFrame::resume>callback-expression-continuation",
        )),
        "C10.prepared.profiled_consumer" => Some((
            "crates/rum-runtime/src/runtime/eval_core/call.rs",
            "c12f[Input[\"rule\"]]",
            "evaluate_call>prepared-argument-Input.rule>ProfiledPreparedDispatchFrame::resume>resume_prepared_call_dispatch_frame::<true>>c12f.downvalue>Input.finish;second=downvalue-body-continuation;tail=completed-generation-inert-pause",
        )),
        "P0544" => Some((
            "crates/rum-runtime/src/runtime/eval_dispatch/mod.rs",
            "Input[\"finish\"]",
            "try_dispatch_builtin_call.checked>run_builtin_handler>Input;tail=finish_builtin_dispatch;second=resume_eval_request_with_input",
        )),
        "P0508" => Some((
            "crates/rum-runtime/src/runtime/eval_core/mod.rs",
            "Input[\"finish\"]",
            "eval_request>evaluate_once_in_accounting_mode>evaluate_inner_unstable>Input;tail=leave_level-and-evaluation-stack-pop;second=resume_eval_request_with_input;request-return=actual_evaluation_state",
        )),
        "P0543" => Some((
            "crates/rum-runtime/src/runtime/eval_dispatch/mod.rs",
            "Input[\"finish\"]",
            "try_dispatch_builtin_call.unchecked>run_builtin_handler>Input;tail=finish_builtin_dispatch;second=resume_eval_request_with_input;configuration=NonAmplifying-enforcement-disabled",
        )),
        "P0145" => Some((
            "crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs",
            "Reap[Sow[83,71],_?c15pred]",
            "Reap.recipientPattern>Sow>drive_sow_matches>match_expr_owned>c15pred",
        )),
        "P0214" => Some((
            "crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs",
            "Trace[1+2,_?c15pred]",
            "Trace>drive_trace_filter>match_trace_leaf>match_expr_owned>c15pred",
        )),
        "P0216" => Some((
            "crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs",
            "Trace[1+2,_?c15pred]",
            "Trace>drive_trace_filter>drive_trace_filter_action>match_trace_leaf>c15pred",
        )),
        "P0239" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/default_values.rs",
            "Default[c26d]",
            "Default>evaluate_first_exact_default_value_from_cursor>evaluate_guard_sequence_owned>c26guard",
        )),
        "P0245" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/default_values.rs",
            "Default[c26d]",
            "Default>c26first.entry>resume_suspended_exact_default_value_scan>evaluate_guard_sequence_owned>c26guard",
        )),
        "P0248" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs",
            "Default[d]",
            "Default>guard>DirectDefaultFallbackFrame::resume>try_apply_down_values_fallback>rhs",
        )),
        "P0249" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/down_values.rs",
            "Default[",
            "Default>try_apply_down_values>try_apply_direct_default_value>exact-default-guard>Input",
        )),
        "P2348" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs",
            "StringReplace[\"ab\",x_:>",
            "StringReplace>resume_replace_string_value_from_candidate>evaluate_string_rule_guard_result>Input;second=guard-expression-continuation",
        )),
        "P2354" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs",
            "StringReplace[\"ab\",x_:>",
            "StringReplace>resume_replace_string_value_after_guard>evaluate_replace_string_output>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2379" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            "StringCases[\"ab\",x_:>",
            "StringCases>resume_collect_string_cases_from_index>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2387" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            "StringCases[\"ab\",x_:>",
            "StringCases>first-uncredited-output>ResumeCollectStringCasesFromIndexSuspensionFrame>resume_suspended_collect_string_cases_with_output>resume_suspended_collect_string_cases_from_index>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2409" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            "StringReplace[\"ab\",x_:>",
            "StringReplace>resume_replace_string_value_from_index>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2418" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            "StringReplace[\"ab\",x_:>",
            "StringReplace>first-uncredited-output>ResumeReplaceStringValueFromIndexSuspensionFrame>resume_suspended_replace_string_value_with_output>resume_suspended_replace_string_value_from_index>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2381" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            "\"abcdefghijklmnop\":>",
            "StringCases>linear_literal_match_ranges>resume_collect_literal_string_cases>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2385" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules/cases.rs",
            "\"abcdefghijklmnop\":>",
            "StringCases>first-uncredited-output>ResumeCollectStringCasesFromIndexSuspensionFrame>resume_suspended_collect_string_cases_from_index.literal_matches>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2411" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            "\"abcdefghijklmnop\":>",
            "StringReplace>linear_literal_match_ranges>resume_replace_literal_string_from_index>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P2416" => Some((
            "crates/rum-stdlib/src/strings/patterns/search/rules.rs",
            "\"abcdefghijklmnop\":>",
            "StringReplace>first-uncredited-output>ResumeReplaceStringValueFromIndexSuspensionFrame>resume_suspended_replace_string_value_from_index.literal_matches>evaluate_string_rule_output_expr_result>Input;second=output-expression-continuation",
        )),
        "P1804" => Some((
            "crates/rum-stdlib/src/associations/missing/target.rs",
            "DeleteMissing[{<|",
            "DeleteMissing>drive_owned_list_cursor>drive_owned_target_cursor>drive_owned_association_cursor>evaluate>Input;second=association-value-continuation",
        )),
        "P1805" => Some((
            "crates/rum-stdlib/src/associations/missing/target.rs",
            "DeleteMissing[<|\"a\":>",
            "DeleteMissing>drive_owned_association_cursor>evaluate>Input;second=association-value-continuation",
        )),
        "P1806" => Some((
            "crates/rum-stdlib/src/associations/missing/target.rs",
            "DeleteMissing[<|\"outer\"-><|",
            "DeleteMissing>drive_owned_association_cursor>drive_owned_target_cursor>drive_owned_association_cursor>evaluate>Input;second=association-value-continuation",
        )),
        "P1615" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/count_like.rs",
            "Count[{71,73},_?c15pred,{1},Heads->False]",
            "Count>count_pattern_matches_owned>drive_count_pattern_matches>match_expr_owned>c15pred;later=ContinueCountTraversalSuspensionFrame>resume_count_traversal>drive_count_pattern_matches",
        )),
        "P1637" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/position.rs",
            "Position[{71,73},_?c15pred,{1},Heads->False]",
            "Position>collect_positions_owned>match_expr_owned>c15pred;later=ContinuePositionTraversalSuspensionFrame>resume_position_traversal>drive_suspended_position_traversal",
        )),
        "P1640" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/position.rs",
            "Position[{71,73},_?c15pred,{1},Heads->False]",
            "Position>first-uncredited-match>ContinuePositionTraversalSuspensionFrame>resume_position_traversal>drive_suspended_position_traversal>match_expr_owned>c15pred;second=predicate-expression-continuation",
        )),
        "P0554" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/association_data.rs",
            "c14assoc[[Input[\"selector\"],",
            "Set>try_set_association_lvalue.Part>evaluate_part_selectors_owned>selector;later=selector-traversal-continuation>finish;tail=finish_association_set_part_lvalue",
        )),
        "P0558" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/association_data.rs",
            "c14assoc[Input[\"selector\"],",
            "Set>first-uncredited-selector>ResumeSuspendedAssociationSetLvalueSuspensionFrame>resume_suspended_association_set_lvalue>evaluate>finish;second=selector-expression-continuation;tail=finish_association_set_function_lvalue",
        )),
        "P1595" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/count_like.rs",
            "Count[",
            "Count>eval_count>evaluate.target>Input;second=target-expression-continuation;tail=EvalCountSuspensionFrame>resume_count_after_target",
        )),
        "P1597" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/count_like.rs",
            "MemberQ[",
            "MemberQ>eval_member_q>evaluate.target>Input;second=target-expression-continuation;tail=EvalMemberQSuspensionFrame>resume_member_q_after_target",
        )),
        "P1599" => Some((
            "crates/rum-runtime/src/runtime/tree_walkers/count_like.rs",
            "FreeQ[",
            "FreeQ>eval_free_q>evaluate.target>Input;second=target-expression-continuation;tail=EvalFreeQSuspensionFrame>resume_free_q_after_target",
        )),
        "P2544" => Some((
            "crates/rum-stdlib/src/structure_ops/list_plumbing.rs",
            "Differences[{c16x,19,23},2]",
            "Differences>eval_differences>successive_differences>difference_expr>Times.upvalue>Input;second=upvalue-expression-continuation;tail=DifferencesOrderSuspensionFrame>differences_from_order",
        )),
        "P2547" => Some((
            "crates/rum-stdlib/src/structure_ops/list_plumbing.rs",
            "Differences[{c16x,19,23},2]",
            "Differences>successive_differences>difference_expr>Times.upvalue>Input;second=upvalue-expression-continuation;tail=DifferencesRowSuspensionFrame>differences_from_pair",
        )),
        "P2132" => Some((
            "crates/rum-stdlib/src/lazy/machine.rs",
            "LazyNestList[",
            "Normal>drive_machine>pull_source.NestList>evaluate>Input;second=callback-expression-continuation;tail=LazyRealizeSuspensionFrame>AwaitCursor.Nest",
        )),
        "P2133" => Some((
            "crates/rum-stdlib/src/lazy/machine.rs",
            "LazyMap[",
            "Normal>drive_machine.PullStage.Map>evaluate>Input;second=callback-expression-continuation;tail=LazyRealizeSuspensionFrame>AwaitCursor.Map",
        )),
        "P2134" => Some((
            "crates/rum-stdlib/src/lazy/machine.rs",
            "LazySelect[",
            "Normal>drive_machine.PullStage.Select>evaluate>Input;second=predicate-expression-continuation;tail=LazyRealizeSuspensionFrame>AwaitCursor.Select",
        )),
        "P2135" => Some((
            "crates/rum-stdlib/src/lazy/machine.rs",
            "LazyFoldList[",
            "Normal>drive_machine.PullStage.FoldList>evaluate>Input;second=callback-expression-continuation;tail=LazyRealizeSuspensionFrame>AwaitCursor.Fold",
        )),
        "P1193" => Some((
            "crates/rum-runtime/src/runtime/pattern_matching/core.rs",
            "Replace[17,(x:",
            "Replace>match_compiled_named_owned>match_compiled_owned>predicate>Input;second=predicate-expression-continuation;tail=MatchCompiledNamedOwnedSuspensionFrame>bind_pattern_name",
        )),
        "P2299" => Some((
            "crates/rum-stdlib/src/schema.rs",
            "_?Function[x,",
            "SchemaMode.Errors-or-Conform>start_validation>schema_drive>predicate>Input;second=predicate-expression-continuation;tail=SchemaValidationSuspensionFrame>resume_validation>finish_validation",
        )),
        "P0442" => Some((
            "crates/rum-runtime/src/runtime/definitions/rhs_guards.rs",
            "c17guarded[17]",
            "definition-condition>match_compiled_definition_condition_owned>match_compiled_owned>c17pred;tail=MatchCompiledDefinitionConditionOwnedSuspensionFrame>finish_definition_condition_match_owned;second=callback-expression-continuation",
        )),
        "P0765" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs",
            "CatchExceptions[",
            "CatchExceptions>dispatch_exception_handlers_owned>match_expr_owned>handler-predicate;tail=suspend_exception_handlers_owned_match;second=callback-expression-continuation",
        )),
        "P0769" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs",
            "CatchExceptions[",
            "CatchExceptions>body-uncredited>dispatch_suspended_exception_handlers>match_expr_owned>handler-predicate;tail=suspend_suspended_exception_handlers;second=callback-expression-continuation",
        )),
        "P0394" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_updates.rs",
            "AddTo[c17target[[",
            "AddTo>eval_held_update_operation>current_held_update_value>evaluate_part_selectors_owned;tail=suspend_held_part_selectors>finish_held_update_operation;second=callback-expression-continuation",
        )),
        "P0396" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_updates.rs",
            "AddTo[c17target[[",
            "AddTo>current_held_update_value>evaluate_part_selectors_owned;tail=suspend_held_part_selectors>finish_held_update_operation;second=callback-expression-continuation",
        )),
        "P0403" => Some((
            "crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs",
            "c17suffix}={89,97}",
            "Set.List>drive_list_destructuring>finish_definition_assignment>Part.selector;tail=DestructuringSuspensionFrame>drive_list_destructuring;second=callback-expression-continuation",
        )),
        "P0435" => Some((
            "crates/rum-runtime/src/runtime/definitions/part_assignment.rs",
            "c17target[[",
            "Set.Part>try_set_positional_part_lvalue>evaluate_part_selectors_owned;tail=suspend_positional_part_selectors>finish_positional_part_assignment;second=callback-expression-continuation",
        )),
        "P1818" => Some((
            "crates/rum-stdlib/src/associations/mutation.rs",
            "AssociateTo[c17target[[",
            "AssociateTo>eval_held_association_part_mutation>evaluate_part_selectors_owned_for_builtin;tail=FinishHeldAssociationPartSelectorEvaluationSuspensionFrame>finish_held_association_part_mutation;second=callback-expression-continuation",
        )),
        "P1132" => Some((
            "crates/rum-runtime/src/runtime/options/builtins.rs",
            "OptionValue[c17opts,{a,b}]",
            "OptionValue>resolve_option_value_query.Many>resolve_option_value_scalar>delayed-option;tail=suspend_option_values;second=callback-expression-continuation",
        )),
        "P1037" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs",
            "ReplaceRepeated[17,",
            "ReplaceRepeated>replace_repeated_from_start>apply_replace_all_with_plan>rule-guard;tail=suspend_replace_repeated_pass;second=callback-expression-continuation",
        )),
        "P1038" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs",
            "ReplaceRepeated[17,",
            "ReplaceRepeated>replace_repeated_from_start>evaluate.replaced>c17gate;tail=suspend_replace_repeated_evaluation;second=callback-expression-continuation",
        )),
        "P0289" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/rhs.rs",
            "c17rhs",
            "own-value>evaluate_definition_rhs>suspend_definition_rhs_evaluation;tail=EvaluateDefinitionRhsSuspensionFrame>finish_definition_rhs_evaluation;second=callback-expression-continuation",
        )),
        "P0294" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/rhs.rs",
            "c17f[17]",
            "down-value>evaluate_definition_application_rhs>suspend_definition_application_rhs_evaluation;tail=EvaluateDefinitionApplicationRhsSuspensionFrame>finish_definition_rhs_evaluation;second=callback-expression-continuation",
        )),
        "P0767" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs",
            "CatchExceptions[",
            "CatchExceptions>dispatch_exception_handlers_owned>evaluate_guard_sequence_owned;tail=suspend_exception_handlers_owned_guard;second=callback-expression-continuation",
        )),
        "P0773" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs",
            "CatchExceptions[",
            "CatchExceptions>match-uncredited>resume_suspended_exception_match_outcome>evaluate_guard_sequence_owned;tail=suspend_suspended_exception_match_outcome;second=callback-expression-continuation",
        )),
        "P0304" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/sub_values.rs",
            "c18sub[17][23]",
            "sub-value>try_apply_sub_values_from_symbol>evaluate_definition_application_rhs;tail=EvaluateDefinitionApplicationRhsSuspensionFrame;second=callback-expression-continuation",
        )),
        "P0311" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/sub_values.rs",
            "c18sub[17][23]",
            "sub-value>match-uncredited>finish_suspended_sub_value_application>evaluate_definition_application_rhs;tail=EvaluateDefinitionApplicationRhsSuspensionFrame;second=callback-expression-continuation",
        )),
        "P0320" => Some((
            "crates/rum-runtime/src/runtime/definitions/application/up_values.rs",
            "c18up[c18a,23]",
            "up-value>match-uncredited>resume_suspended_up_value_scan>evaluate_definition_application_rhs;tail=EvaluateDefinitionApplicationRhsSuspensionFrame;second=callback-expression-continuation",
        )),
        "P0812" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "Catch[",
            "Catch>eval_catch>suspend_catch;tail=resume_catch_after_body;second=callback-expression-continuation",
        )),
        "P1480" => Some((
            "crates/rum-runtime/src/runtime/scoping/callable/status_application.rs",
            "Failure[\"tag\",",
            "Failure.Payload>evaluate_status_function_value>evaluate.applied;tail=StatusFunctionValueFrame>finish_function_application;second=callback-expression-continuation",
        )),
        "P1562" => Some((
            "crates/rum-runtime/src/runtime/session/session_requests.rs",
            "7+(",
            "accept_cancellable_request>eval_accepted_request>eval_request_admitted>eval_expr_request_from_checkpoint;tail=finish_accepted_value_request;second=resume_eval_request_with_input;terminal=actual_accepted_request_state",
        )),
        "P1563" => Some((
            "crates/rum-runtime/src/runtime/session/session_requests.rs",
            "7+(",
            "lower_request>accept_cancellable_request>eval_accepted_expr_request>eval_expr_request_admitted>eval_expr_request_from_checkpoint;tail=finish_accepted_value_request;second=resume_eval_request_with_input;terminal=actual_accepted_request_state",
        )),
        "P0339" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs",
            "NormalizeRules[c23rules]",
            "NormalizeRules>drive_rewrite_rule_exprs_owned>evaluate.c23rules>Input.finish;tail=normalized-rule-list-emission;second=rule-expression-continuation",
        )),
        "P0348" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs",
            "Cases[",
            "Cases>resolve_cases_traversal_owned>evaluate.c21level>Input.finish;tail=resolved-traversal-configuration-adoption;second=level-expression-continuation",
        )),
        "P0349" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs",
            "Cases[",
            "Cases>InputString.target>resolve_cases_traversal_shared_from>evaluate.c21level>Input.finish;tail=resolved-traversal-configuration-adoption;second=level-expression-continuation",
        )),
        "P0350" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs",
            "Position[",
            "Position>resolve_position_traversal_owned>evaluate.c21level>Input.finish;tail=resolved-traversal-configuration-adoption;second=level-expression-continuation",
        )),
        "P0351" => Some((
            "crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs",
            "Count[",
            "Count>resolve_traversal_without_limit_owned>evaluate.c21level>Input.finish;tail=resolved-traversal-configuration-adoption;second=level-expression-continuation",
        )),
        "P0823" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "Catch[",
            "Catch>Throw>finish_catch_for_throw_with_form>match_compiled_pattern>Input.finish;tail=catch-tag-match-and-cleanup-carriers;second=guard-expression-continuation",
        )),
        "P0827" => Some((
            "crates/rum-runtime/src/runtime/kernel_data/control_data/nonlocal.rs",
            "WithCleanup[",
            "Catch>WithCleanup>Throw>finish_catch_for_bypassing_throw_with_form>match_compiled_pattern>Input.finish;tail=catch-bypass-tag-match-and-cleanup-carriers;second=guard-expression-continuation",
        )),
        "P1476" => Some((
            "crates/rum-runtime/src/runtime/resolution.rs",
            "C8FixedPointStep[",
            "C8FixedPointStep>Input.c8step1>resume_fixed_point_from>evaluate_once_unstable_after_input_check_with_cycle_tracking>Input.c8step2;tail=evaluate.fixed_point_equal;second=resume_fixed_point_from>Input.c8step3;terminal=c31_fixed_point_equality_events",
        )),
        "P1577" => Some((
            "crates/rum-runtime/src/runtime/suspension/core.rs",
            "InputString[\"enter-resume\"];7+(",
            "eval_request>InputString.enter-resume>resume_eval_request_with_input>resume_suspended_slot>drive_resuming_frames>Input.finish;tail=saved_stack-and-saved_depth-restoration;second=drive_resuming_frames>Input.finish2;each-request-return=actual_evaluation_state",
        )),
        _ => None,
    }
}

fn phases(
    key: &TupleKey,
    family: &str,
    pauses: &[(HostInputRequest, String)],
    canonical_id: Option<&str>,
) -> BindingResult<Vec<String>> {
    if family.starts_with("cm3-") {
        let mut phases = Vec::new();
        for (ordinal, (request, _)) in pauses.iter().enumerate() {
            let owned = native::late_guest_pause(
                key.row(),
                &request.prompt,
                pauses.iter().any(|(r, _)| r.prompt == "control caller"),
            )
            .ok_or(BindingError::SourceIdentity)?;
            let string = request.prompt.ends_with(" caller")
                || matches!(request.prompt.as_str(), "c7prior" | "c10setPrior");
            if request.kind
                != if string {
                    HostInputKind::InputString
                } else {
                    HostInputKind::Input
                }
            {
                return Err(BindingError::SourceIdentity);
            }
            phases.push(if owned {
                format!("{}:{family}:producer-pause-{ordinal}", key.row())
            } else {
                "other-producer-uncredited".into()
            });
        }
        if !phases.iter().any(|phase| credited_phase(phase)) {
            return Err(BindingError::SourceIdentity);
        }
        return Ok(phases);
    }
    if matches!(key.row(), "P0813" | "P2600" | "P2611" | "P2612") {
        let width = if matches!(key.variant(), "AlreadyResumed" | "Twice") {
            2
        } else {
            1
        };
        let loops = if key.context() == "return_boundary.loop-store-reset" {
            2
        } else {
            1
        };
        if pauses.len() != width * loops {
            return Err(BindingError::ProgramIdentity);
        }
        let mut phases = Vec::new();
        for (ordinal, (request, _)) in pauses.iter().enumerate() {
            let prompt = match (key.variant(), ordinal % width) {
                ("AlreadyResumed", 0) => "native623.prior",
                ("Twice", 1) => "native623.two",
                _ => "native623.one",
            };
            if request.kind != HostInputKind::InputString || request.prompt != prompt {
                return Err(BindingError::SourceIdentity);
            }
            // The sealed disabled stacks attribute only each Catch body's
            // first pause, and only the Host call's first request. Later
            // pauses retain their continuation provenance without re-credit.
            let owned = ordinal == 0
                || key.row() == "P2612"
                || (key.row() == "P0813" && ordinal % width == 0);
            phases.push(if owned {
                format!("{}:{family}:request-return-{ordinal}", key.row())
            } else {
                "other-producer-uncredited".into()
            });
        }
        return Ok(phases);
    }
    if matches!(key.row(), "P1566" | "P1568") {
        let prior = usize::from(key.variant() == "AlreadyResumed");
        let native = if key.variant() == "Twice" { 3 } else { 2 };
        if pauses.len() != prior + native {
            return Err(BindingError::ProgramIdentity);
        }
        let mut phases = Vec::new();
        for (ordinal, (request, _)) in pauses.iter().enumerate() {
            let (kind, prompt) = if ordinal < prior {
                (HostInputKind::InputString, "c9prior".to_string())
            } else {
                (
                    HostInputKind::Input,
                    format!("c9native{}", ordinal + 1 - prior),
                )
            };
            if request.kind != kind || request.prompt != prompt {
                return Err(BindingError::SourceIdentity);
            }
            // The sealed disabled stack attributes the first actual pause to
            // P1566 and every subsequent pause to P1568, including c9prior.
            let producer = if ordinal == 0 { "P1566" } else { "P1568" };
            phases.push(if key.row() == producer {
                format!("{producer}:{family}:request-return-{ordinal}")
            } else {
                "other-producer-uncredited".into()
            });
        }
        return Ok(phases);
    }
    if matches!(key.row(), "P1470" | "P1471" | "P1476") {
        let prior = usize::from(key.variant() == "AlreadyResumed");
        let steps = if key.variant() == "Twice" { 4 } else { 3 };
        if pauses.len() != prior + steps {
            return Err(BindingError::ProgramIdentity);
        }
        let mut phases = Vec::new();
        for (ordinal, (request, _)) in pauses.iter().enumerate() {
            let (kind, prompt) = if ordinal < prior {
                (
                    HostInputKind::InputString,
                    if key.row() == "P1476" {
                        "prior"
                    } else {
                        "c8prior"
                    }
                    .to_string(),
                )
            } else {
                (
                    HostInputKind::Input,
                    format!("c8step{}", ordinal - prior + 1),
                )
            };
            if request.kind != kind || request.prompt != prompt {
                return Err(BindingError::SourceIdentity);
            }
            let owned = match key.row() {
                "P1470" => ordinal == prior + 1,
                "P1471" => ordinal >= prior + 2,
                _ => ordinal > prior,
            };
            phases.push(if !owned {
                "prior-uncredited".into()
            } else {
                format!(
                    "{}:{family}:resume-fixed-point-pass-{}",
                    key.row(),
                    ordinal - prior + 1
                )
            });
        }
        return Ok(phases);
    }
    if family == "diagnostic-native" {
        let id = canonical_id.ok_or(BindingError::ProgramIdentity)?;
        let caller = format!("{id}:caller");
        let mut result = Vec::new();
        for (ordinal, (request, _)) in pauses.iter().enumerate() {
            let caller = request.kind == HostInputKind::InputString
                && (request.prompt == caller
                    || matches!(
                        request.prompt.as_str(),
                        "associations caller"
                            | "control caller"
                            | "rules caller"
                            | "debug caller"
                            | "functional caller"
                            | "predicate caller"
                            | "completion caller"
                    )
                    || (matches!(key.row(), "P0369" | "P0372") && request.prompt == "c6prior")
                    || (key.row() == "P0408" && request.prompt == "c7prior")
                    || (key.row() == "P0817" && request.prompt == "prior"));
            if caller && key.variant() != "AlreadyResumed" {
                return Err(BindingError::SourceIdentity);
            }
            let prior = caller
                || (key.row() == "P0408"
                    && request.kind == HostInputKind::Input
                    && request.prompt == "c7rhs");
            result.push(if prior {
                "prior-uncredited".into()
            } else {
                format!("{}:{family}:{id}:operand-{}", key.row(), ordinal + 1)
            });
        }
        if result.iter().all(|phase| phase == "prior-uncredited") {
            return Err(BindingError::SourceIdentity);
        }
        return Ok(result);
    }
    if key.row() == "C5.default.profiled.finish" {
        let prior = usize::from(key.variant() == "AlreadyResumed");
        let invocations = if key.variant() == "Twice" { 2 } else { 1 };
        if pauses.len() != prior + 2 * invocations
            || (prior == 1
                && (pauses[0].0.kind != HostInputKind::InputString
                    || pauses[0].0.prompt != "prior"))
            || pauses[prior..].iter().enumerate().any(|(i, (request, _))| {
                request.kind != HostInputKind::Input
                    || request.prompt != if i % 2 == 0 { "guard" } else { "fallback" }
            })
        {
            return Err(BindingError::SourceIdentity);
        }
        return Ok((0..pauses.len())
            .map(|ordinal| {
                if ordinal < prior {
                    "prior-uncredited".into()
                } else {
                    format!(
                        "C5.default.profiled.finish:{family}:invocation-{}:{}",
                        (ordinal - prior) / 2 + 1,
                        if (ordinal - prior) % 2 == 0 {
                            "returned-guard-request"
                        } else {
                            "resumed-fallback-request"
                        }
                    )
                }
            })
            .collect());
    }
    if key.row() == "C3.evaluate_call.unstable_head" {
        let mut invocation = 0;
        let mut result = Vec::new();
        for (request, _) in pauses {
            let operand = match (request.kind, request.prompt.as_str()) {
                (HostInputKind::InputString, "predicate caller")
                    if key.variant() == "AlreadyResumed" =>
                {
                    result.push("prior-uncredited".into());
                    continue;
                }
                (HostInputKind::Input, "AllTrue.predicate:2") => {
                    invocation += 1;
                    1
                }
                (HostInputKind::Input, "AllTrue.predicate:4") if invocation > 0 => 2,
                _ => return Err(BindingError::SourceIdentity),
            };
            result.push(format!(
                "C3.evaluate_call.unstable_head:{family}:invocation-{invocation}:operand-{operand}"
            ));
        }
        let expected =
            if key.context() == "return_boundary.loop-store-reset" && key.variant() != "Error" {
                2
            } else {
                1
            };
        if invocation != expected {
            return Err(BindingError::SourceIdentity);
        }
        return Ok(result);
    }
    if matches!(key.row(), "P1139" | "P1142" | "P1147" | "P1155") {
        let mut phases = Vec::new();
        for (request, _) in pauses {
            let prior = matches!(
                request.prompt.as_str(),
                "c7prior" | "c7target" | "rewrite caller" | "functional caller" | "debug caller"
            );
            let producer = request.kind == HostInputKind::Input
                && (matches!(request.prompt.as_str(), "c7option" | "c7option2")
                    || request.prompt.starts_with("Cases.defaultOption:")
                    || request.prompt.starts_with("ArrayFilter.defaultOption:")
                    || request.prompt.starts_with("Monitor.explicitOption:")
                    || request.prompt.starts_with("DumpSave.defaultOption:")
                    || request.prompt.starts_with("DumpSave.explicitOption:"));
            let historical = request.kind == HostInputKind::InputString
                && request.prompt == "p: "
                && key.0[4].starts_with("historical:");
            if !prior && !producer && !historical {
                return Err(BindingError::SourceIdentity);
            }
            phases.push(if prior {
                "prior-uncredited".into()
            } else {
                format!(
                    "{}:{family}:{}:option-operand-{}",
                    key.row(),
                    key.0[3],
                    phases.len() + 1
                )
            });
        }
        if phases.iter().all(|phase| phase == "prior-uncredited") {
            return Err(BindingError::SourceIdentity);
        }
        return Ok(phases);
    }
    if key.row() == "P0554" {
        let prior = usize::from(key.variant() == "AlreadyResumed");
        if pauses.len() != prior + if key.variant() == "Twice" { 3 } else { 2 }
            || pauses[prior].0.kind != HostInputKind::Input
            || pauses[prior].0.prompt != "selector"
        {
            return Err(BindingError::SourceIdentity);
        }
        return Ok((0..pauses.len())
            .map(|ordinal| {
                if ordinal < prior {
                    "prior-uncredited".into()
                } else {
                    format!(
                        "P0554:{family}:selector-traversal:operand-{}",
                        ordinal + 1 - prior
                    )
                }
            })
            .collect());
    }
    if matches!(key.row(), "P1615" | "P1637" | "P1640") {
        let prior = usize::from(key.variant() == "AlreadyResumed");
        if pauses.len() != prior + if key.variant() == "Twice" { 3 } else { 2 }
            || pauses[prior].0.kind != HostInputKind::InputString
            || pauses[prior].0.prompt != "first"
            || pauses[prior + 1].0.kind != HostInputKind::Input
            || pauses[prior + 1].0.prompt != "finish"
            || (key.variant() == "Twice"
                && (pauses[prior + 2].0.kind != HostInputKind::Input
                    || pauses[prior + 2].0.prompt != "finish2"))
        {
            return Err(BindingError::SourceIdentity);
        }
        return Ok((0..pauses.len())
            .map(|ordinal| {
                if ordinal < prior || (key.row() == "P1640" && ordinal == prior) {
                    "prior-uncredited".into()
                } else {
                    let site = match (key.row(), ordinal == prior) {
                        ("P1615", _) => "drive_count_pattern_matches",
                        ("P1637", true) => "collect_positions_owned",
                        _ => "drive_suspended_position_traversal",
                    };
                    format!(
                        "{}:{family}:{site}:operand-{}",
                        key.row(),
                        ordinal + 1 - prior
                    )
                }
            })
            .collect());
    }
    if key.row() == "P0249" {
        let mut invocation = 0;
        let mut result = Vec::new();
        for (request, _) in pauses {
            let operand = match (request.kind, request.prompt.as_str()) {
                (HostInputKind::InputString, "associations caller") => {
                    if key.variant() != "AlreadyResumed" {
                        return Err(BindingError::SourceIdentity);
                    }
                    result.push("prior-uncredited".into());
                    continue;
                }
                (HostInputKind::Input, "Evaluator.defaultFallback:2") => {
                    invocation += 1;
                    1
                }
                (HostInputKind::Input, "Evaluator.defaultFallback:4") if invocation > 0 => 2,
                (HostInputKind::InputString, "p: ") if key.0[4] == "historical:207" => {
                    invocation += 1;
                    1
                }
                _ => return Err(BindingError::SourceIdentity),
            };
            result.push(format!(
                "P0249:{family}:invocation-{invocation}:operand-{operand}"
            ));
        }
        let expected =
            if key.context() == "return_boundary.loop-store-reset" && key.variant() != "Error" {
                2
            } else {
                1
            };
        if invocation != expected {
            return Err(BindingError::SourceIdentity);
        }
        return Ok(result);
    }
    let start = usize::from(key.variant() == "AlreadyResumed")
        + usize::from(matches!(
            key.row(),
            "P0245"
                | "P0248"
                | "P0558"
                | "P0769"
                | "P0773"
                | "P0311"
                | "P0320"
                | "P0321"
                | "P2385"
                | "P2387"
                | "P2416"
                | "P2418"
                | "P1577"
                | "P0455"
                | "P0349"
                | "C10.prepared.profiled_consumer"
        ))
        + if key.row() == "P0459" { 3 } else { 0 };
    if pauses.len().checked_sub(start) != Some(if key.variant() == "Twice" { 2 } else { 1 }) {
        return Err(BindingError::ProgramIdentity);
    }
    if matches!(
        key.row(),
        "P0339"
            | "P0348"
            | "P0349"
            | "P0350"
            | "P0351"
            | "P0823"
            | "P0827"
            | "P0470"
            | "P0936"
            | "P1070"
            | "P1610"
            | "P1613"
            | "P1621"
            | "P1636"
    ) && ((key.variant() == "AlreadyResumed"
        && (pauses[0].0.kind != HostInputKind::InputString || pauses[0].0.prompt != "prior"))
        || (key.row() == "P0349"
            && (pauses[start - 1].0.kind != HostInputKind::InputString
                || pauses[start - 1].0.prompt != "target"))
        || pauses[start..]
            .iter()
            .enumerate()
            .any(|(index, (request, _))| {
                request.kind != HostInputKind::Input
                    || request.prompt != if index == 0 { "finish" } else { "finish2" }
            }))
    {
        return Err(BindingError::SourceIdentity);
    }
    if matches!(key.row(), "P0245" | "P0248")
        && (pauses[start - 1].0.kind != HostInputKind::InputString
            || pauses[start - 1].0.prompt
                != if key.row() == "P0245" {
                    "entry"
                } else {
                    "guard"
                })
    {
        return Err(BindingError::SourceIdentity);
    }
    if matches!(key.row(), "P2385" | "P2387" | "P2416" | "P2418")
        && (pauses[start - 1].0.kind != HostInputKind::InputString
            || pauses[start - 1].0.prompt != "first")
    {
        return Err(BindingError::SourceIdentity);
    }
    if key.row() == "P0459"
        && pauses[start - 3..start]
            .iter()
            .zip(["first", "later", "first"])
            .any(|((request, _), prompt)| {
                request.kind != HostInputKind::InputString || request.prompt != prompt
            })
    {
        return Err(BindingError::SourceIdentity);
    }
    if matches!(key.row(), "P0450" | "P0455")
        && ((key.variant() == "AlreadyResumed"
            && (pauses[0].0.kind != HostInputKind::InputString || pauses[0].0.prompt != "prior"))
            || (key.row() == "P0455"
                && (pauses[start - 1].0.kind != HostInputKind::InputString
                    || pauses[start - 1].0.prompt != "first"))
            || pauses[start].0.kind != HostInputKind::Input
            || pauses[start].0.prompt != "finish"
            || (key.variant() == "Twice"
                && (pauses[start + 1].0.kind != HostInputKind::Input
                    || pauses[start + 1].0.prompt != "finish2")))
    {
        return Err(BindingError::SourceIdentity);
    }
    if key.row() == "P0321"
        && (pauses[start - 1].0.kind != HostInputKind::InputString
            || pauses[start - 1].0.prompt != "reject")
    {
        return Err(BindingError::SourceIdentity);
    }
    if key.row() == "P0558"
        && (pauses[start - 1].0.kind != HostInputKind::Input
            || pauses[start - 1].0.prompt != "selector")
    {
        return Err(BindingError::SourceIdentity);
    }
    if matches!(key.row(), "P0769" | "P0773" | "P0311" | "P0320")
        && (pauses[start - 1].0.kind != HostInputKind::InputString
            || pauses[start - 1].0.prompt
                != if key.row() == "P0769" {
                    "body"
                } else {
                    "match"
                })
    {
        return Err(BindingError::SourceIdentity);
    }
    if key.row() == "P1577"
        && (pauses[start - 1].0.kind != HostInputKind::InputString
            || pauses[start - 1].0.prompt != "enter-resume"
            || pauses[start].0.kind != HostInputKind::Input
            || pauses[start].0.prompt != "finish"
            || (key.variant() == "Twice"
                && (pauses[start + 1].0.kind != HostInputKind::Input
                    || pauses[start + 1].0.prompt != "finish2")))
    {
        return Err(BindingError::SourceIdentity);
    }
    if key.row() == "C10.prepared.profiled_consumer"
        && (key.0[3] != "C10.ProfiledPreparedDispatchFrame.true"
            || pauses[start - 1].0.kind != HostInputKind::Input
            || pauses[start - 1].0.prompt != "rule"
            || pauses[start].0.kind != HostInputKind::Input
            || pauses[start].0.prompt != "finish")
    {
        return Err(BindingError::SourceIdentity);
    }
    Ok((0..pauses.len())
        .map(|ordinal| {
            if ordinal < start {
                "prior-uncredited".into()
            } else {
                format!("{}:{family}:operand-{}", key.row(), ordinal + 1 - start)
            }
        })
        .collect())
}

fn load_programs() -> BindingResult<BTreeMap<String, Program>> {
    let seals = crate::oracle::obligations::bindings::class_provenance()?;
    load_programs_with(&seals, &|path| {
        std::fs::read(crate::oracle::coverage::source::workspace().join(path))
            .map_err(|_| BindingError::SourceIdentity)
    })
}

pub(crate) fn programs_with_mutation<A: crate::oracle::mutations::source_auth::Arm>(
    source: &crate::oracle::mutations::source_auth::AuthenticatedSource<A>,
) -> BindingResult<Arc<BTreeMap<String, Program>>> {
    let seals = crate::oracle::obligations::bindings::provenance()?;
    load_programs_with(&seals, &|path| source.source(path)).map(Arc::new)
}

fn load_programs_with(
    seals: &[Vec<&'static str>],
    source_bytes: &impl Fn(&str) -> BindingResult<Vec<u8>>,
) -> BindingResult<BTreeMap<String, Program>> {
    let table = include_str!("../../inventory/obligations/programs.tsv");
    if table.lines().next()
        != Some(
            "program\tfamily\trow\tvariant\tcontext\tconsumer_route\tdescriptor\tsource_sha256\tsetup_sha256\tpauses\ttrace\tcanonical_input\tcanonical_fingerprint",
        )
    {
        return Err(BindingError::Inventory);
    }
    let seal: Vec<_> = seals
        .iter()
        .filter(|r| r[0] == "program-table" && r[1] == "programs")
        .collect();
    if seal.len() != 1 || seal[0][3] != content_digest(table.as_bytes()) {
        return Err(BindingError::FrozenSeal);
    }
    for source in seals.iter().filter(|r| r[0] == "canonical-source") {
        if source[5] != crate::oracle::obligations::bindings::LEDGER_PIN
            || content_digest(&source_bytes(source[2])?) != source[3]
        {
            return Err(BindingError::SourceIdentity);
        }
    }
    let canonical_rows = table
        .lines()
        .skip(1)
        .map(|line| {
            let cells: Vec<_> = line.split('\t').collect();
            if cells.len() != 13 {
                return Err(BindingError::Inventory);
            }
            if cells[11] == "-" {
                Ok(None)
            } else {
                unhex(cells[11]).map(Some)
            }
        })
        .collect::<BindingResult<Vec<_>>>()?;
    let text = canonical_rows
        .iter()
        .flatten()
        .flat_map(|rows| rows.lines())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n");
    let canonical = super::canonical_plans::from_rows(&text)
        .into_iter()
        .map(|plan| (plan.id, super::CanonicalFixture(plan.fixture)))
        .collect::<BTreeMap<_, _>>();
    let mut result = BTreeMap::new();
    for (line, canonical_row) in table.lines().skip(1).zip(canonical_rows) {
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 13 {
            return Err(BindingError::Inventory);
        }
        let key = TupleKey(std::array::from_fn(|index| row[index + 2].to_string()));
        if key.context().starts_with("guest_heads.") {
            return Err(BindingError::Encoding);
        }
        let pauses = row[9]
            .split(';')
            .map(|cell| {
                let cells: Vec<_> = cell.split(':').collect();
                if cells.len() != 3 {
                    return Err(BindingError::ProgramIdentity);
                }
                let kind = match cells[0] {
                    "Input" => HostInputKind::Input,
                    "InputString" => HostInputKind::InputString,
                    _ => return Err(BindingError::ProgramIdentity),
                };
                Ok((
                    HostInputRequest {
                        kind,
                        prompt: unhex(cells[1])?,
                        password: false,
                    },
                    unhex(cells[2])?,
                ))
            })
            .collect::<BindingResult<Vec<_>>>()?;
        let canonical_id = canonical_row
            .as_ref()
            .map(|line| unhex(line.split('\t').next().unwrap()))
            .transpose()?;
        let phases = phases(&key, row[1], &pauses, canonical_id.as_deref())?;
        let subject_requests = phases.iter().filter(|phase| credited_phase(phase)).count();
        // A guest recipe uses its authenticated source chain and immutable
        // program/answer plan, including the later resumed predicate pause.
        // Native fixtures instead observe their concrete retention boundary.
        let producer_route = if let Some((path, recipe, chain)) = guest_route(row[2]) {
            let route: Vec<_> = seals
                .iter()
                .filter(|r| r[0] == "guest-route" && r[1] == row[0])
                .collect();
            if route.len() != 1
                || route[0][2] != path
                || route[0][4] != subject_requests.to_string()
                || route[0][5] != crate::oracle::obligations::bindings::LEDGER_PIN
                || !program(row[7])?.contains(recipe)
            {
                return Err(BindingError::SourceIdentity);
            }
            if content_digest(&source_bytes(route[0][2])?) != route[0][3] {
                return Err(BindingError::SourceIdentity);
            }
            format!("{chain};{}@{}", route[0][2], route[0][3])
        } else {
            "native-retention-observer".to_string()
        };
        let fixture = canonical_row
            .as_ref()
            .map(|line| {
                let id = unhex(
                    line.split('\t')
                        .next()
                        .ok_or(BindingError::ProgramIdentity)?,
                )?;
                canonical
                    .get(&id)
                    .cloned()
                    .ok_or(BindingError::ProgramIdentity)
            })
            .transpose()?;
        let mut p = Program {
            id: row[0].to_string(),
            family: row[1].to_string(),
            key,
            source: program(row[7])?,
            setup: program(row[8])?,
            source_hash: row[7].to_string(),
            setup_hash: row[8].to_string(),
            answers_hash: content_digest(row[9].as_bytes()),
            // Keep prior plans' identities unchanged when their optional
            // canonical-input cell is absent.
            plan_hash: content_digest(
                if fixture.is_some() {
                    line.to_string()
                } else {
                    row[..11].join("\t")
                }
                .as_bytes(),
            ),
            pauses,
            trace: match row[10] {
                "true" => true,
                "false" => false,
                _ => return Err(BindingError::ProgramIdentity),
            },
            producer_route,
            phases,
            canonical: fixture,
            paired: None,
        };
        let canonical_inputs = canonical_row
            .as_deref()
            .map_or_else(Vec::new, |text| text.lines().collect::<Vec<_>>());
        let fingerprints = row[12].split(';').collect::<Vec<_>>();
        let constituents = constituent_count(&p.key);
        if constituents > 1 {
            if canonical_inputs.len() != constituents || fingerprints.len() != constituents {
                return Err(BindingError::ProgramIdentity);
            }
            let mut ids = BTreeSet::new();
            for input in &canonical_inputs {
                if !ids.insert(input.split('\t').next().unwrap()) {
                    return Err(BindingError::ProgramIdentity);
                }
            }
            // Retain the sealed order and every constituent. Each link is
            // executed in fresh sessions, and only the root credits a key.
            let mut paired = None;
            for index in (1..constituents).rev() {
                let mut program =
                    paired_program(&p, canonical_inputs[index], fingerprints[index], &canonical)?;
                program.paired = paired;
                paired = Some(Box::new(program));
            }
            p.paired = paired;
        } else if canonical_inputs.len() > 1 || fingerprints.len() != 1 {
            return Err(BindingError::ProgramIdentity);
        }
        if matches!(p.key.row(), "P1139" | "P1142" | "P1147" | "P1155") && p.canonical.is_none() {
            return Err(BindingError::ProgramIdentity);
        }
        if let Some(canonical) = &p.canonical {
            let f = &canonical.0;
            if f.source != p.source
                || f.setup != p.setup
                || f.profile_trace != p.trace
                || super::canonical_plans::fixture_fingerprint(f) != fingerprints[0]
                || (!f.duration_paths.is_empty() && !declared_duration(&p.key, f))
                || f.pauses.len() != p.pauses.len()
                || f.pauses.iter().zip(&p.pauses).any(|(a, (b, answer))| {
                    (match a.kind {
                        rum_runtime::RuntimeInputKind::Input => HostInputKind::Input,
                        rum_runtime::RuntimeInputKind::InputString => HostInputKind::InputString,
                    }) != b.kind
                        || a.prompt != b.prompt
                        || a.answer != *answer
                })
            {
                return Err(BindingError::ProgramIdentity);
            }
        } else if row[12] != "-" {
            return Err(BindingError::ProgramIdentity);
        }
        if p.key.row() == "C10.prepared.profiled_consumer" {
            let dependencies: Vec<_> = seals
                .iter()
                .filter(|r| r[0] == "guest-dependency" && r[1] == p.id)
                .collect();
            if dependencies.len() != 1
                || dependencies[0][2]
                    != "crates/rum-runtime/src/runtime/profiling/logical/segments.rs"
                || dependencies[0][5] != crate::oracle::obligations::bindings::LEDGER_PIN
                || content_digest(&source_bytes(dependencies[0][2])?) != dependencies[0][3]
            {
                return Err(BindingError::SourceIdentity);
            }
        }
        if result.insert(p.id.clone(), p).is_some() {
            return Err(BindingError::DuplicateBinding);
        }
    }
    Ok(result)
}

fn constituent_count(key: &TupleKey) -> usize {
    match key.row() {
        "P0492"
            if key.variant() == "Middle"
                && key.context() == "phase_observation.trace"
                && key.0[4] == "Accumulate.arithmetic" =>
        {
            2
        }
        "P1613" => 3,
        "P0348" | "P0349" | "P0350" | "P0351" => {
            if matches!(key.variant(), "AlreadyResumed" | "Nested")
                && matches!(
                    key.context(),
                    "held_and_splice.tail-hold"
                        | "held_and_splice.tail-nothing"
                        | "held_and_splice.tail-sequence"
                )
            {
                1
            } else {
                2
            }
        }
        "P0256" | "P0259" | "P0266"
            if key.context() == "same_type_error_finish.explicit-cycle-controls-required" =>
        {
            2
        }
        "P0450" => 2,
        "P0455" => {
            // The sealed union has six two-record cells; the other 49
            // additionally retain the registered-owner program.
            if matches!(key.variant(), "AlreadyResumed" | "Nested")
                && matches!(
                    key.context(),
                    "held_and_splice.tail-hold"
                        | "held_and_splice.tail-nothing"
                        | "held_and_splice.tail-sequence"
                )
            {
                2
            } else {
                3
            }
        }
        _ => 1,
    }
}

fn declared_duration(key: &TupleKey, fixture: &crate::oracle::descriptors::Fixture) -> bool {
    fixture.duration_paths == vec![vec![0]]
        && match key.row() {
            "P0515" => matches!(
                fixture.id.as_str(),
                "E1.2079.phase_observation.trace"
                    | "E1.2079.prefix_and_tail.log"
                    | "E1.2079.value.base"
            ),
            "P0492" => matches!(
                fixture.id.as_str(),
                "E1.2079.consumer_routes.route-original"
                    | "E1.2079.phase_observation.trace"
                    | "E1.2079.prefix_and_tail.log"
                    | "E1.2079.value.base"
                    | "E1T.2079.held_and_splice.tail-hold"
                    | "E1T.2079.held_and_splice.tail-nothing"
                    | "E1T.2079.held_and_splice.tail-sequence"
            ),
            _ => false,
        }
}

fn paired_program(
    primary: &Program,
    input: &str,
    fingerprint: &str,
    canonical: &BTreeMap<String, super::CanonicalFixture>,
) -> BindingResult<Program> {
    let id = unhex(
        input
            .split('\t')
            .next()
            .ok_or(BindingError::ProgramIdentity)?,
    )?;
    let fixture = canonical.get(&id).ok_or(BindingError::ProgramIdentity)?;
    let f = &fixture.0;
    if super::canonical_plans::fixture_fingerprint(f) != fingerprint
        || !f.duration_paths.is_empty()
        || primary.canonical.as_ref().is_some_and(|p| p.0.id == f.id)
    {
        return Err(BindingError::ProgramIdentity);
    }
    let mut paired = primary.clone();
    paired.paired = None;
    paired.id = format!("{}:paired:{id}", primary.id);
    paired.source_hash = content_digest(f.source.as_bytes());
    paired.setup_hash = content_digest(f.setup.as_bytes());
    paired.source = program(&paired.source_hash)?;
    paired.setup = program(&paired.setup_hash)?;
    paired.pauses = f
        .pauses
        .iter()
        .map(|pause| {
            (
                HostInputRequest {
                    kind: match pause.kind {
                        rum_runtime::RuntimeInputKind::Input => HostInputKind::Input,
                        rum_runtime::RuntimeInputKind::InputString => HostInputKind::InputString,
                    },
                    prompt: pause.prompt.clone(),
                    password: false,
                },
                pause.answer.clone(),
            )
        })
        .collect();
    paired.answers_hash = content_digest(format!("{:?}", paired.pauses).as_bytes());
    paired.plan_hash = content_digest(format!("{input}\n{fingerprint}").as_bytes());
    paired.trace = f.profile_trace;
    paired.phases = phases(&paired.key, &paired.family, &paired.pauses, Some(&id))?;
    paired.canonical = Some(fixture.clone());
    Ok(paired)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PauseObservation {
    pub ordinal: usize,
    pub phase: String,
    pub producer_route: String,
    pub output: HostEvalOutcome,
    pub native: NativeObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunObservation {
    pub setup: SetupObservation,
    pub setup_native: NativeObservation,
    pub pauses: Vec<PauseObservation>,
    pub output: HostEvalOutcome,
    pub projections: Vec<HostEvalOutcome>,
    pub checkpoint_snapshot: Option<Result<Vec<u8>, crate::HostFailure>>,
    pub native: NativeObservation,
    pub counters: Vec<(String, [u64; 10])>,
    pub evaluation_states: Vec<(usize, usize)>,
    pub pending: Option<HostInputRequest>,
    pub queue_probe: HostEvalOutcome,
    pub queue_finish: HostEvalOutcome,
    pub fresh: HostEvalOutcome,
    pub adapter: Option<AdapterObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdapterObservation {
    pub handoff: Option<HostEvalOutcome>,
    pub terminal: crate::KernelExecuteOutcome,
    pub delivered: Vec<String>,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) enum SetupObservation {
    Absent,
    Executed(HostEvalOutcome),
    CallDefinition {
        setup: HostEvalOutcome,
        definition: HostEvalOutcome,
    },
}

impl std::fmt::Debug for SetupObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absent => f.write_str("Absent"),
            // Preserve the existing raw identity for nonempty setup programs.
            Self::Executed(outcome) => std::fmt::Debug::fmt(outcome, f),
            Self::CallDefinition { setup, definition } => f
                .debug_struct("CallDefinition")
                .field("setup", setup)
                .field("definition", definition)
                .finish(),
        }
    }
}

pub(crate) fn setup(program: &Program, session: &mut HostSession) -> SetupObservation {
    setup_for_source(program, session, &program.source)
}

pub(crate) fn setup_for_source(
    program: &Program,
    session: &mut HostSession,
    source: &str,
) -> SetupObservation {
    if program.key.row() == "P2600" {
        return SetupObservation::CallDefinition {
            setup: session.eval(program.setup.as_str()),
            definition: session.eval(format!("c47driver[]:=({source})")),
        };
    }
    if program.setup.is_empty() {
        SetupObservation::Absent
    } else {
        SetupObservation::Executed(session.eval(program.setup.as_str()))
    }
}

pub(crate) fn setup_matches(program: &Program, observed: &SetupObservation) -> bool {
    match observed {
        SetupObservation::Absent => program.canonical.is_some() && program.setup.is_empty(),
        SetupObservation::Executed(outcome) => {
            !program.setup.is_empty() && matches!(outcome, HostEvalOutcome::Success(_))
        }
        SetupObservation::CallDefinition { setup, definition } => {
            program.key.row() == "P2600"
                && matches!(setup, HostEvalOutcome::Success(_))
                && matches!(definition, HostEvalOutcome::Success(_))
        }
    }
}

pub(crate) fn start(program: &Program, session: &mut HostSession, source: &str) -> HostEvalOutcome {
    if adapter_program(program) {
        let code = if program.key.row() == "P2612" {
            let handoff = session.eval(source);
            assert!(matches!(&handoff, HostEvalOutcome::InputRequired(p)
                if p.request.kind == HostInputKind::InputString && p.request.prompt == "c52.handoff"));
            None
        } else {
            Some(source)
        };
        let cancel = session.cancel_handle();
        adapter_output(
            crate::adapter_facade::observed_worker::observe_segment(
                session, code, "handoff", &cancel,
            )
            .0,
        )
    } else if program.key.row() == "P2600" {
        session.call("c47driver", &[])
    } else {
        session.eval(source)
    }
}

pub(crate) fn adapter_program(program: &Program) -> bool {
    matches!(program.key.row(), "P2611" | "P2612")
}

fn adapter_output(outcome: crate::KernelExecuteOutcome) -> HostEvalOutcome {
    // Lossless channel projection only; the included adapter already performed
    // cancellation normalization. WireDecoded is never a segment outcome.
    match outcome {
        crate::KernelExecuteOutcome::Success(out) => HostEvalOutcome::Success(out),
        crate::KernelExecuteOutcome::InputRequired(out) => HostEvalOutcome::InputRequired(out),
        crate::KernelExecuteOutcome::Failure(out) => HostEvalOutcome::Failure(out),
        crate::KernelExecuteOutcome::WireDecoded => panic!("unexpected wire outcome in segment"),
    }
}

pub(crate) fn resume(program: &Program, session: &mut HostSession, input: &str) -> HostEvalOutcome {
    if adapter_program(program) {
        let cancel = session.cancel_handle();
        adapter_output(
            crate::adapter_facade::observed_worker::observe_segment(session, None, input, &cancel)
                .0,
        )
    } else {
        session.resume_with_input(input)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunFailure {
    Setup,
    Producer,
    Phase,
    PauseCount,
    Prompt,
    Kind,
    Ordinal,
    PrintPrefix,
    MessagePrefix,
    TracePrefix,
    Output,
    Projection,
    Native,
    Pending,
    Queue,
    Fresh,
    Lifecycle,
    Program,
}

pub(crate) fn channels(out: &HostEvalOutcome) -> (&[String], &[HostMessage], &[HostTraceEvent]) {
    match out {
        HostEvalOutcome::Success(out) => (&out.prints, &out.messages, &out.trace),
        HostEvalOutcome::InputRequired(out) => (&out.prints, &out.messages, &out.trace),
        HostEvalOutcome::Failure(out) => (&out.prints, &out.messages, &out.trace),
    }
}

pub(crate) fn prefix(a: &HostEvalOutcome, b: &HostEvalOutcome) -> Result<(), RunFailure> {
    let (a_prints, a_messages, a_trace) = channels(a);
    let (b_prints, b_messages, b_trace) = channels(b);
    if !b_prints.starts_with(a_prints) {
        return Err(RunFailure::PrintPrefix);
    }
    if !b_messages.starts_with(a_messages) {
        return Err(RunFailure::MessagePrefix);
    }
    if !b_trace.starts_with(a_trace) {
        return Err(RunFailure::TracePrefix);
    }
    Ok(())
}

pub(crate) fn phase(program: &Program, ordinal: usize) -> String {
    program.phases[ordinal].clone()
}

pub(crate) fn session(program: &Program) -> HostSession {
    if adapter_program(program) {
        return HostSession::builder().with_tracing(program.trace).build();
    }
    if let Some(canonical) = &program.canonical {
        let f = canonical.0.clone();
        let c31 = program.key.row() == "P1476";
        return HostSession::builder()
            .with_builtin_installer(move |session| {
                native::install(session);
                if c31 {
                    native::c8_fixedpoint_configure(session);
                }
                session
                    .eval_request("Global`log={};Global`probe::tag=\"oracle message\"")
                    .unwrap();
                (f.configure)(session);
                if let Some(files) = &f.files {
                    crate::oracle::support::files::configure_memory(session, files);
                }
            })
            .with_tracing(program.trace)
            .build();
    }
    let fixedpoint = matches!(program.key.row(), "P1470" | "P1471");
    HostSession::builder()
        .with_builtin_installer(move |session| {
            native::install(session);
            if fixedpoint {
                native::c8_fixedpoint_configure(session);
            }
        })
        .with_tracing(program.trace)
        .build()
}

pub(crate) fn run(program: &Program, queued: bool) -> Result<RunObservation, RunFailure> {
    run_observed(program, queued, None)
}

/// Alternative observation requires a schema-admitted, authenticated armed row.
/// Ordinary callers always execute the original quiescence assertion below.
pub(crate) fn run_with_mutation_observer(
    program: &Program,
    queued: bool,
    observer: &crate::oracle::mutations::source_auth::MutationObserver<'_>,
) -> Result<RunObservation, RunFailure> {
    observer.check(program).map_err(|_| RunFailure::Program)?;
    run_observed(program, queued, Some(observer))
}

fn run_observed(
    program: &Program,
    queued: bool,
    mutation: Option<&crate::oracle::mutations::source_auth::MutationObserver<'_>>,
) -> Result<RunObservation, RunFailure> {
    if adapter_program(program) {
        return run_adapter(program, queued);
    }
    if matches!(program.key.row(), "P1562" | "P1563") {
        return native::run_accepted_request(program, queued);
    }
    if matches!(program.key.row(), "P0508" | "P1577") {
        return native::run_resuming_driver(program, queued);
    }
    if (!matches!(program.key.row(), "P0001" | "P0002") && guest_route(program.key.row()).is_none())
        || program.key.context().starts_with("guest_heads.")
    {
        return Err(RunFailure::Program);
    }
    let checked_dispatch = (program.key.row() == "P0543").then(native::checked_dispatch_enabled);
    if checked_dispatch == Some(true) {
        return Err(RunFailure::Native);
    }
    let lease = rum_runtime::Session::logical_profile_test_acquire();
    let observer = ObserverScope::enter();
    let mut session = session(program);
    let setup = setup(program, &mut session);
    let setup_native = observer.snapshot();
    if !setup_matches(program, &setup) || setup_native.entries != 0 {
        return Err(RunFailure::Setup);
    }
    if queued {
        for (_, answer) in &program.pauses {
            session.queue_input_response(answer);
        }
    }
    rum_runtime::test_logical_profile_reset(&lease);
    rum_runtime::cm740_probe::begin(queued, "CM740-GUEST");
    let mut output = start(program, &mut session, &program.source);
    let mut pauses = Vec::new();
    if !queued {
        for (ordinal, (_, answer)) in program.pauses.iter().enumerate() {
            if !matches!(output, HostEvalOutcome::InputRequired(_)) {
                return Err(RunFailure::PauseCount);
            }
            pauses.push(PauseObservation {
                ordinal,
                phase: phase(program, ordinal),
                producer_route: program.producer_route.clone(),
                output: output.clone(),
                native: observer.snapshot(),
            });
            output = session.resume_with_input(answer);
        }
    }
    if matches!(output, HostEvalOutcome::InputRequired(_)) {
        return Err(RunFailure::PauseCount);
    }
    rum_runtime::cm740_probe::end();
    let timeout = native::timeout_event(program.key.row())
        .map(|event| (event, session.actual_owns_eval_timeout()));
    let native = observer.snapshot();
    let checkpoint = matches!(program.key.row(), "P1566" | "P1568")
        .then(|| session.actual_message_checkpoint_state());
    let mut counters = rum_runtime::Session::logical_profile_test_rows();
    if let Some((event, owned)) = timeout {
        counters.push((event.into(), [u64::from(owned), 0, 0, 0, 0, 0, 0, 0, 0, 0]));
    }
    if let Some((present, dirty)) = checkpoint {
        counters.push((
            native::CHECKPOINT_EVENT.into(),
            [u64::from(present), u64::from(dirty), 0, 0, 0, 0, 0, 0, 0, 0],
        ));
    }
    if program.key.row() == "P1476" {
        counters.extend(rum_runtime::Session::c31_fixed_point_equality_events());
    }
    if let Some(enabled) = checked_dispatch {
        counters.push((
            "CM739UncheckedDispatchBuildConfiguration".into(),
            [u64::from(enabled), 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ));
    }
    for (index, (_, bytes)) in native.charges.iter().enumerate() {
        counters.push((
            format!("C34ActualCallbackReceiptBytes.{index}"),
            [*bytes as u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ));
    }
    if let Some(permission) = mutation {
        permission.check(program).map_err(|_| RunFailure::Program)?;
        // All values above were captured from this execution, at the ordinary
        // capture point. The epoch fault is never the row's predicate.
        println!(
            "CM740_OBSERVER_CAPTURE row={} queued={queued} native_rows={} pauses={} before_quiescence=true",
            program.key.row(),
            counters.len(),
            pauses.len()
        );
    } else {
        rum_runtime::Session::logical_profile_test_assert_quiescent();
    }
    let checkpoint_snapshot =
        (checkpoint.is_some() || timeout.is_some()).then(|| session.snapshot_to_bytes());
    let checkpoint_fresh = checkpoint.map(|_| session.eval("1+1"));
    let mut projections = if timeout.is_some() {
        ["Global`log", "Global`scope"]
            .into_iter()
            .map(|source| session.eval(source))
            .collect()
    } else if checkpoint.is_some() {
        ["log", "$MessageList", "c9scope"]
            .into_iter()
            .map(|source| session.eval(source))
            .collect()
    } else {
        vec![
            session.eval("{log,scope}"),
            session.eval("DownValues[f]"),
            session.eval("UpValues[f]"),
        ]
    };
    if let Some(canonical) = &program.canonical {
        for source in &canonical.0.projections {
            projections.push(session.eval(source.as_str()));
        }
        for source in [
            "Global`state",
            "Global`scope",
            "Global`local",
            "Global`other",
            "Global`count",
            "Global`c7reads",
            "Global`c7scope",
            "Global`c7suffix",
            "Global`answers",
            "Global`seen",
            "Global`suffix",
        ] {
            projections.push(session.eval(source));
        }
    }
    if program.key.row() == "P0249" {
        for source in [
            "Global`state",
            "Global`answers",
            "Global`scope",
            "Global`seen",
            "Global`suffix",
            "n",
            "DefaultValues[d]",
            "DownValues[d]",
        ] {
            projections.push(session.eval(source));
        }
    }
    if matches!(program.key.row(), "P0527" | "P0338") {
        for source in [
            "c25guardonce",
            "c25rulesonce",
            "c25answer",
            "OwnValues[c25owned]",
            "OwnValues[c25rules]",
        ] {
            projections.push(session.eval(source));
        }
    }
    if matches!(program.key.row(), "P1470" | "P1471") {
        projections.push(session.eval("Global`c7scope"));
    }
    if matches!(
        program.family.as_str(),
        "string-result-append"
            | "linear-literal-output"
            | "delete-missing-target-tails"
            | "traversal-accumulation"
    ) {
        for source in ["c15index", "c15once"] {
            projections.push(session.eval(source));
        }
    }
    if program.family == "association-store" {
        projections.push(session.eval("c14assoc"));
    }
    if matches!(
        program.key.row(),
        "P0442"
            | "P0394"
            | "P0396"
            | "P0403"
            | "P0435"
            | "P1818"
            | "P1132"
            | "P1037"
            | "P1038"
            | "P0289"
            | "P0294"
    ) {
        for source in [
            "c17target",
            "c17suffix",
            "c17result",
            "c17visits",
            "c17count",
            "c17assoc",
            "OwnValues[c17rhs]",
            "DownValues[c17f]",
            "DownValues[c17guarded]",
            "Options[c17opts]",
        ] {
            projections.push(session.eval(source));
        }
    }
    if matches!(program.key.row(), "P0304" | "P0311" | "P0320") {
        for source in ["SubValues[c18sub]", "UpValues[c18a]"] {
            projections.push(session.eval(source));
        }
    }
    if matches!(program.key.row(), "P0260" | "P0305") {
        for source in [
            "c18counter",
            "c18answer",
            "DownValues[c18match]",
            "SubValues[c18match]",
        ] {
            projections.push(session.eval(source));
        }
    }
    if matches!(program.key.row(), "P0278" | "P0281" | "P0284") {
        for source in ["c20matchn", "c20answer"] {
            projections.push(session.eval(source));
        }
    }
    if matches!(program.key.row(), "P0316" | "P0321") {
        for source in ["c22acceptonce", "c22rejectonce", "UpValues[c22tag]"] {
            projections.push(session.eval(source));
        }
    }
    if matches!(program.key.row(), "P1238" | "P1240") {
        projections.push(session.eval("Attributes[c14seq]"));
    }
    if program.key.row() == "C10.prepared.profiled_consumer" {
        for source in ["DownValues[c12f]", "c12answer"] {
            projections.push(session.eval(source));
        }
    }
    if program.key.row() == "C5.default.profiled.finish" {
        for source in [
            "DefaultValues[c12default]",
            "DownValues[Default]",
            "c12answer",
        ] {
            projections.push(session.eval(source));
        }
    }
    let terminal_fresh = checkpoint_fresh.or_else(|| timeout.map(|_| session.eval("1+1")));
    let pending = session.pending_input_request();
    let queue_probe = session.eval("InputString[\"cm-queue\"]");
    let queue_finish = session.resume_with_input("cm-drained");
    let fresh = terminal_fresh.unwrap_or_else(|| session.eval("1+1"));
    observer.assert_clean();
    Ok(RunObservation {
        setup,
        setup_native,
        pauses,
        output,
        projections,
        checkpoint_snapshot,
        native,
        counters,
        evaluation_states: Vec::new(),
        pending,
        queue_probe,
        queue_finish,
        fresh,
        adapter: None,
    })
}

fn run_adapter(program: &Program, queued: bool) -> Result<RunObservation, RunFailure> {
    let lease = rum_runtime::Session::logical_profile_test_acquire();
    let observer = ObserverScope::enter();
    let mut session = session(program);
    let setup = setup(program, &mut session);
    let setup_native = observer.snapshot();
    if !setup_matches(program, &setup) || setup_native.entries != 0 {
        return Err(RunFailure::Setup);
    }
    let cancel = session.cancel_handle();
    let callback_cancel = cancel.clone();
    let delivered = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&delivered);
    session.set_print_sink(crate::ClosurePrintSink::assert_bounded_destruction(
        move |text| {
            sink.lock().unwrap().push(text.to_string());
            if text == "c52.cancel" {
                callback_cancel.cancel();
            }
        },
    ));
    let handoff = if program.key.row() == "P2612" {
        let out = session.eval(program.source.as_str());
        if !matches!(&out, HostEvalOutcome::InputRequired(p)
            if p.request.kind == HostInputKind::InputString && p.request.prompt == "c52.handoff")
        {
            return Err(RunFailure::Producer);
        }
        Some(out)
    } else {
        None
    };
    if queued {
        for (_, answer) in &program.pauses {
            session.queue_input_response(answer);
        }
    }
    // The sealed handoff is outside this epoch. No historical probe is imported.
    rum_runtime::Session::logical_profile_test_reset(&lease);
    rum_runtime::cm740_probe::begin(queued, "CM740-GUEST");
    let code = handoff.is_none().then_some(program.source.as_str());
    let (mut terminal, mut sample) = crate::adapter_facade::observed_worker::observe_segment(
        &mut session,
        code,
        "handoff",
        &cancel,
    );
    let mut pauses = Vec::new();
    if !queued {
        for (ordinal, (request, answer)) in program.pauses.iter().enumerate() {
            if !matches!(&terminal, crate::KernelExecuteOutcome::InputRequired(p) if &p.request == request)
                || session.pending_input_request().as_ref() != Some(request)
                || sample.is_some()
            {
                return Err(RunFailure::PauseCount);
            }
            pauses.push(PauseObservation {
                ordinal,
                phase: phase(program, ordinal),
                producer_route: program.producer_route.clone(),
                output: adapter_output(terminal.clone()),
                native: observer.snapshot(),
            });
            (terminal, sample) = crate::adapter_facade::observed_worker::observe_segment(
                &mut session,
                None,
                answer,
                &cancel,
            );
        }
    }
    rum_runtime::cm740_probe::end();
    let counters = sample.ok_or(RunFailure::Native)?;
    let output = adapter_output(terminal.clone());
    let native = observer.snapshot();
    let delivered = delivered.lock().unwrap().clone();
    let pending = session.pending_input_request();
    rum_runtime::Session::logical_profile_test_assert_quiescent();
    let checkpoint_snapshot = Some(session.snapshot_to_bytes());
    let projections = ["Global`log", "Global`saved", "Global`scope"]
        .into_iter()
        .map(|source| session.eval(source))
        .collect();
    let fresh = session.eval("1+1");
    let queue_probe = session.eval("InputString[\"cm-queue\"]");
    let queue_finish = session.resume_with_input("cm-drained");
    observer.assert_clean();
    Ok(RunObservation {
        setup,
        setup_native,
        pauses,
        output,
        projections,
        checkpoint_snapshot,
        native,
        counters,
        evaluation_states: Vec::new(),
        pending,
        queue_probe,
        queue_finish,
        fresh,
        adapter: Some(AdapterObservation {
            handoff,
            terminal,
            delivered,
        }),
    })
}

pub(crate) fn validate(
    program: &Program,
    resumed: &RunObservation,
    queued: &RunObservation,
) -> Result<(), RunFailure> {
    if !setup_matches(program, &resumed.setup)
        || !setup_matches(program, &queued.setup)
        || resumed.setup_native.entries != 0
        || queued.setup_native.entries != 0
    {
        return Err(RunFailure::Setup);
    }
    if resumed.native.entries != program.native_entries()
        || queued.native.entries != program.native_entries()
    {
        return Err(RunFailure::Producer);
    }
    if resumed.pauses.len() != program.pauses.len() || !queued.pauses.is_empty() {
        return Err(RunFailure::PauseCount);
    }
    for (index, (pause, (planned, _))) in resumed.pauses.iter().zip(&program.pauses).enumerate() {
        if pause.ordinal != index {
            return Err(RunFailure::Ordinal);
        }
        let HostEvalOutcome::InputRequired(actual) = &pause.output else {
            return Err(RunFailure::PauseCount);
        };
        if actual.request.kind != planned.kind {
            return Err(RunFailure::Kind);
        }
        if actual.request.prompt != planned.prompt || actual.request.password != planned.password {
            return Err(RunFailure::Prompt);
        }
        if pause.phase != phase(program, index) || pause.producer_route != program.producer_route {
            return Err(RunFailure::Phase);
        }
        let entered = usize::from(credited_phase(&pause.phase)) * program.native_entries();
        if pause.native.entries != entered || pause.native.resumes != 0 {
            return Err(RunFailure::Producer);
        }
        prefix(&pause.output, &resumed.output)?;
        if let Some(next) = resumed.pauses.get(index + 1) {
            prefix(&pause.output, &next.output)?;
        }
    }
    let duration_paths = program
        .canonical
        .as_ref()
        .map_or(&[][..], |f| f.0.duration_paths.as_slice());
    let same_output = if duration_paths.is_empty() {
        resumed.output == queued.output
    } else {
        let comparison = host_duration_comparison;
        comparison(&resumed.output, duration_paths) == comparison(&queued.output, duration_paths)
    };
    if !same_output {
        return Err(RunFailure::Output);
    }
    if adapter_program(program) {
        if resumed.adapter != queued.adapter {
            return Err(RunFailure::Output);
        }
        let seals =
            crate::oracle::obligations::bindings::provenance().map_err(|_| RunFailure::Native)?;
        let seals: Vec<_> = seals
            .iter()
            .filter(|r| r[0] == "c52-native-values" && r[1] == program.id)
            .collect();
        if seals.len() != 1 {
            return Err(RunFailure::Native);
        }
        for side in [resumed, queued] {
            let a = side.adapter.as_ref().ok_or(RunFailure::Output)?;
            if !matches!(&a.terminal, crate::KernelExecuteOutcome::Failure(f)
                if f.code == crate::HostFailureCode::Cancelled)
                || adapter_output(a.terminal.clone()) != side.output
                || a.delivered != ["c52.cancel"]
                || (program.key.row() == "P2612") != a.handoff.is_some()
            {
                return Err(RunFailure::Output);
            }
            if !matches!(&side.checkpoint_snapshot, Some(Ok(_))) || side.projections.len() != 3 {
                return Err(RunFailure::Projection);
            }
            // The seal fixes all ten counter fields and their row order.
            // Symbol allocation IDs belong to their process; the complete
            // current row labels and values are compared exactly below.
            // Historical full-bag pins remain unchanged in provenance.
            let text = side
                .counters
                .iter()
                .map(|(_, values)| {
                    format!(
                        "{}\n",
                        values
                            .iter()
                            .map(u64::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
                .collect::<String>();
            if content_digest(text.as_bytes()) != seals[0][3]
                || side.counters.len().to_string() != seals[0][4]
                || side
                    .counters
                    .iter()
                    .map(|(name, _)| name)
                    .collect::<BTreeSet<_>>()
                    .len()
                    != side.counters.len()
                || seals[0][5] != crate::oracle::obligations::bindings::LEDGER_PIN
            {
                return Err(RunFailure::Native);
            }
        }
    }
    if resumed.projections != queued.projections
        || resumed.checkpoint_snapshot != queued.checkpoint_snapshot
    {
        return Err(RunFailure::Projection);
    }
    if resumed.counters != queued.counters {
        return Err(RunFailure::Native);
    }
    if let Some(event) = native::timeout_event(program.key.row()) {
        for side in [resumed, queued] {
            let rows: Vec<_> = side
                .counters
                .iter()
                .filter(|(name, _)| name == event)
                .collect();
            if rows.len() != 1 || rows[0].1 != [0; 10] {
                return Err(RunFailure::Native);
            }
            if !matches!(side.checkpoint_snapshot, Some(Ok(_))) || side.projections.len() != 2 {
                return Err(RunFailure::Projection);
            }
        }
    }
    if matches!(program.key.row(), "P1566" | "P1568") {
        let expected = native::checkpoint_expected(program.key.variant(), program.key.context())
            .ok_or(RunFailure::Native)?;
        for side in [resumed, queued] {
            let rows: Vec<_> = side
                .counters
                .iter()
                .filter(|(name, _)| name == native::CHECKPOINT_EVENT)
                .collect();
            if rows.len() != 1 || rows[0].1 != expected {
                return Err(RunFailure::Native);
            }
            if !matches!(side.checkpoint_snapshot, Some(Ok(_))) || side.projections.len() != 3 {
                return Err(RunFailure::Projection);
            }
        }
    }
    if program.key.row() == "P1476" {
        let count = native::c31_expected_count(program.key.variant(), program.key.context())
            .ok_or(RunFailure::Native)?;
        for side in [resumed, queued] {
            let events: Vec<_> = side
                .counters
                .iter()
                .filter(|(name, _)| name.starts_with("C31Synchronous "))
                .collect();
            if events.len() != 1
                || events[0].0 != native::C31_EVENT
                || events[0].1 != [count, count, 0, 0, 0, 0, 0, 0, 0, 0]
            {
                return Err(RunFailure::Native);
            }
        }
    }
    if program.key.row() == "P0543" {
        for side in [resumed, queued] {
            let configuration: Vec<_> = side
                .counters
                .iter()
                .filter(|(name, _)| name == "CM739UncheckedDispatchBuildConfiguration")
                .collect();
            if configuration.len() != 1 || configuration[0].1 != [0; 10] {
                return Err(RunFailure::Native);
            }
        }
    }
    if matches!(program.key.row(), "P1562" | "P1563") {
        for side in [resumed, queued] {
            let terminal: Vec<_> = side
                .counters
                .iter()
                .filter(|(name, _)| name == "C29TerminalAcceptedRequestState")
                .collect();
            if terminal.len() != 1 || terminal[0].1 != [0; 10] {
                return Err(RunFailure::Native);
            }
        }
    }
    if matches!(program.key.row(), "P0508" | "P1577") {
        let native_name = if program.key.row() == "P0508" {
            "C28BetweenRequestsEvaluationStateMax"
        } else {
            "C29ResumingDriverEvaluationStateMax"
        };
        for (side, expected) in [(resumed, program.pauses.len() + 1), (queued, 1)] {
            let rows: Vec<_> = side
                .counters
                .iter()
                .filter(|(name, _)| name == native_name)
                .collect();
            if side.evaluation_states.len() != expected
                || side.evaluation_states.iter().any(|state| *state != (0, 0))
                || rows.len() != 1
                || rows[0].1 != [0; 10]
            {
                return Err(RunFailure::Native);
            }
        }
    }
    for side in [resumed, queued] {
        if side.pending.is_some() {
            return Err(RunFailure::Pending);
        }
        if !matches!(&side.queue_probe, HostEvalOutcome::InputRequired(p) if p.request.prompt == "cm-queue" && p.request.kind == HostInputKind::InputString)
        {
            return Err(RunFailure::Queue);
        }
        if !matches!(&side.queue_finish, HostEvalOutcome::Success(s) if s.value.text == "\"cm-drained\"")
        {
            return Err(RunFailure::Queue);
        }
        if !matches!(&side.fresh, HostEvalOutcome::Success(s) if s.value.text == "2") {
            return Err(RunFailure::Fresh);
        }
    }
    Ok(())
}

#[test]
fn cm_receipt_observers_callable() {
    let programs = programs().unwrap();
    let mut seen = BTreeSet::new();
    let mut pauses = 0;
    for p in programs.values().filter(|p| p.key.row() == "P0001") {
        let resumed = run(p, false).unwrap();
        let queued = run(p, true).unwrap();
        validate(p, &resumed, &queued).unwrap();
        assert!(seen.insert(&p.id));
        pauses += resumed.pauses.len();
    }
    assert_eq!((seen.len(), pauses), (60, 84));
    println!("CM P0001 ACCESSOR PASS plans=60 actual_pauses=84 current_receipts=0 encoding=0");
}

/// Comparison view only: receipts keep the original host observations. The
/// frozen fixture supplies the paths; every other field and event stays exact.
pub(crate) fn mutation_disabled_duration_comparison(
    program: &Program,
    output: &HostEvalOutcome,
) -> Result<HostEvalOutcome, RunFailure> {
    if !matches!(program.key.row(), "P0492" | "P0515") {
        return Err(RunFailure::Program);
    }
    let Some(fixture) = program.canonical.as_ref().map(|f| &f.0) else {
        return Ok(output.clone());
    };
    if !fixture.duration_paths.is_empty() && !declared_duration(&program.key, fixture) {
        return Err(RunFailure::Program);
    }
    Ok(host_duration_comparison(output, &fixture.duration_paths))
}

fn host_duration_comparison(
    output: &crate::HostEvalOutcome,
    paths: &[Vec<usize>],
) -> crate::HostEvalOutcome {
    use crate::{HostEvalOutcome, HostStructuredOutputKind, HostTraceEvent, HostTyped};

    fn typed_leaf(value: &mut HostTyped, path: &[usize]) {
        if let Some((&index, rest)) = path.split_first() {
            let HostTyped::List(items) = value else {
                panic!("duration typed path crosses non-list")
            };
            typed_leaf(&mut items[index], rest);
        } else {
            // Numeric shape and sign were checked on the corresponding Expr.
            assert!(matches!(
                value,
                HostTyped::Integer(_) | HostTyped::BigInteger(_) | HostTyped::Rational { .. }
            ));
            *value = HostTyped::Integer(0);
        }
    }

    let mut comparison = output.clone();
    if paths.is_empty() {
        return comparison;
    }
    let HostEvalOutcome::Success(success) = &mut comparison else {
        return comparison;
    };
    let parser = rum_runtime::Session::new();
    let original_text = success.value.text.clone();
    let original_value = success.value.typed.to_expr(&parser).unwrap();
    assert_eq!(
        original_value.to_string(),
        original_text,
        "duration rendered/typed identity"
    );
    let mut masked = original_value;
    for path in paths {
        masked = crate::oracle::support::normalize::duration(&masked, path);
        typed_leaf(&mut success.value.typed, path);
    }
    success.value.text = masked.to_string();
    // Parse, never evaluate, the rendered trace. Mapping complete timing values
    // preserves sibling numbers and string contents, including equal numbers.
    let replacements = [(
        parser.lower_text(&original_text).unwrap(),
        parser.lower_text(&success.value.text).unwrap(),
    )];
    let mask_text = |text: &mut String| {
        let expression = parser.lower_text(text).unwrap();
        assert_eq!(expression.to_string(), *text, "duration trace round trip");
        *text = crate::oracle::support::observe::duration_comparison_expression(
            &expression,
            &replacements,
        )
        .to_string();
    };
    let original_trace = success.trace.clone();
    for event in &mut success.trace {
        match event {
            HostTraceEvent::EvaluationPass { before, after } => {
                mask_text(before);
                mask_text(after);
            }
            HostTraceEvent::DefinitionStored { value, .. } => mask_text(value),
            HostTraceEvent::RuleApplied {
                input,
                rule,
                result,
                ..
            } => {
                mask_text(input);
                mask_text(rule);
                mask_text(result);
            }
        }
    }
    let mut trace_index = 0;
    let mut results = 0;
    for item in &mut success.outputs {
        match item.kind {
            HostStructuredOutputKind::Trace => {
                assert_eq!(
                    item.text,
                    format!("{:?}", original_trace[trace_index]),
                    "duration structured trace identity"
                );
                item.text = format!("{:?}", success.trace[trace_index]);
                trace_index += 1;
            }
            HostStructuredOutputKind::Result => {
                assert_eq!(
                    item.text, original_text,
                    "duration structured result identity"
                );
                item.text.clone_from(&success.value.text);
                results += 1;
            }
            HostStructuredOutputKind::Print | HostStructuredOutputKind::Message => {}
        }
    }
    assert_eq!((trace_index, results), (original_trace.len(), 1));
    comparison
}
