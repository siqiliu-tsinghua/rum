mod resume_frames;
use resume_frames::*;

use super::super::super::matcher::best_pattern_match_state;
use super::super::rules::evaluate_string_rule_output_expr_result;
use super::*;
use rum_runtime::RuntimeInputRequest;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct GuardedStringReplaceRuleEvalState {
    chars: Vec<char>,
    rules: Vec<GuardedExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
}

#[derive(FrameRootTrace)]
struct GuardedStringReplaceValuesEvalState {
    values: Vec<String>,
    rules: Vec<GuardedExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
}

pub(in crate::strings::patterns::search) fn replace_string_value_with_guarded_rules(
    session: &mut Session,
    value: &str,
    rules: &[GuardedExactStringReplacementRule],
    max_replacements: usize,
    config: StringPatternMatchConfig,
) -> Result<String, EvalError> {
    debug_assert!(rules.iter().any(|rule| rule.guard.is_some()));
    if max_replacements == 0 {
        return Ok(value.to_string());
    }

    (if rum_runtime::cm740_probe::driven() {
        resume_replace_string_value_from_candidate_cm740_driven(
            session,
            GuardedStringReplaceRuleEvalState {
                chars: value.chars().collect(),
                rules: rules.to_vec(),
                max_replacements,
                config,
            },
            0,
            0,
            0,
            String::new(),
        )
    } else {
        resume_replace_string_value_from_candidate(
            session,
            GuardedStringReplaceRuleEvalState {
                chars: value.chars().collect(),
                rules: rules.to_vec(),
                max_replacements,
                config,
            },
            0,
            0,
            0,
            String::new(),
        )
    })
}

pub(in crate::strings::patterns::search) fn replace_string_values_with_guarded_rules_expr(
    session: &mut Session,
    values: Vec<String>,
    rules: &[GuardedExactStringReplacementRule],
    max_replacements: usize,
    config: StringPatternMatchConfig,
) -> Result<Expr, EvalError> {
    debug_assert!(rules.iter().any(|rule| rule.guard.is_some()));
    let capacity = values.len();
    resume_replace_string_values_from_index(
        session,
        GuardedStringReplaceValuesEvalState {
            values,
            rules: rules.to_vec(),
            max_replacements,
            config,
        },
        0,
        Vec::with_capacity(capacity),
    )
}

fn resume_replace_string_value_from_candidate(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    mut index: usize,
    replacements: usize,
    mut rule_index: usize,
    mut output: String,
) -> Result<String, EvalError> {
    loop {
        if replacements >= state.max_replacements {
            output.extend(state.chars[index..].iter());
            return Ok(output);
        }

        let Some(rule) = state.rules.get(rule_index) else {
            if index == state.chars.len() {
                return Ok(output);
            }
            output.push(state.chars[index]);
            index += 1;
            rule_index = 0;
            continue;
        };

        let Some(match_state) =
            best_pattern_match_state(&rule.rule.pattern, &state.chars, index, state.config)
        else {
            rule_index += 1;
            continue;
        };
        let end = match_state.end;
        let bindings = match_state.bindings;

        if let Some(guard) = &rule.guard {
            let guard_result = evaluate_string_rule_guard_result(
                session,
                guard,
                &state.chars,
                index,
                end,
                &bindings,
            );
            return match guard_result {
                Ok(accepted) => resume_replace_string_value_after_guard(
                    session,
                    state,
                    index,
                    end,
                    replacements,
                    rule_index,
                    output,
                    bindings,
                    accepted,
                ),
                Err(EvalError::InputRequired { request }) => {
                    suspend_replace_string_value_after_guard(
                        session,
                        state,
                        index,
                        end,
                        replacements,
                        rule_index,
                        output,
                        bindings,
                        request,
                    )
                }
                Err(error) => Err(error),
            };
        }

        return evaluate_replace_string_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            bindings,
            rule_index,
        );
    }
}

#[cold]
#[inline(never)]
#[allow(dead_code)]
fn resume_replace_string_value_from_candidate_cm740_driven(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    mut index: usize,
    replacements: usize,
    mut rule_index: usize,
    mut output: String,
) -> Result<String, EvalError> {
    loop {
        if replacements >= state.max_replacements {
            output.extend(state.chars[index..].iter());
            return Ok(output);
        }

        let Some(rule) = state.rules.get(rule_index) else {
            if index == state.chars.len() {
                return Ok(output);
            }
            output.push(state.chars[index]);
            index += 1;
            rule_index = 0;
            continue;
        };

        let Some(match_state) =
            best_pattern_match_state(&rule.rule.pattern, &state.chars, index, state.config)
        else {
            rule_index += 1;
            continue;
        };
        let end = match_state.end;
        let bindings = match_state.bindings;

        if let Some(guard) = &rule.guard {
            let guard_result = rum_runtime::cm740_probe::around("P2348", || {
                evaluate_string_rule_guard_result(
                    session,
                    guard,
                    &state.chars,
                    index,
                    end,
                    &bindings,
                )
            });
            return match guard_result {
                Ok(accepted) => resume_replace_string_value_after_guard_cm740_driven(
                    session,
                    state,
                    index,
                    end,
                    replacements,
                    rule_index,
                    output,
                    bindings,
                    accepted,
                ),
                Err(EvalError::InputRequired { request }) => {
                    suspend_replace_string_value_after_guard(
                        session,
                        state,
                        index,
                        end,
                        replacements,
                        rule_index,
                        output,
                        bindings,
                        request,
                    )
                }
                Err(error) => Err(error),
            };
        }

        return evaluate_replace_string_output_cm740_driven(
            session,
            state,
            index,
            end,
            replacements,
            output,
            bindings,
            rule_index,
        );
    }
}

#[allow(clippy::too_many_arguments)]
#[cold]
#[inline(never)]
fn suspend_replace_string_value_after_guard(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    rule_index: usize,
    output: String,
    bindings: SymbolExprMap,
    request: RuntimeInputRequest,
) -> Result<String, EvalError> {
    let state = FrozenStringValue::freeze(state);
    let output = PersistentStringText::freeze(output);
    let bindings = FrozenStringValue::freeze(bindings);
    rum_runtime::resume_or_suspend!(insert session, ResumeReplaceStringValueFromCandidateSuspensionFrame { state, index, end, replacements, rule_index, output, bindings });
    Err(EvalError::InputRequired { request })
}

#[allow(clippy::too_many_arguments)]
fn resume_replace_string_value_after_guard(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    rule_index: usize,
    output: String,
    bindings: SymbolExprMap,
    accepted: bool,
) -> Result<String, EvalError> {
    if !accepted {
        return resume_replace_string_value_from_candidate(
            session,
            state,
            index,
            replacements,
            rule_index + 1,
            output,
        );
    }

    evaluate_replace_string_output(
        session,
        state,
        index,
        end,
        replacements,
        output,
        bindings,
        rule_index,
    )
}

#[allow(clippy::too_many_arguments)]
#[cold]
#[inline(never)]
#[allow(dead_code)]
fn resume_replace_string_value_after_guard_cm740_driven(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    rule_index: usize,
    output: String,
    bindings: SymbolExprMap,
    accepted: bool,
) -> Result<String, EvalError> {
    if !accepted {
        return resume_replace_string_value_from_candidate_cm740_driven(
            session,
            state,
            index,
            replacements,
            rule_index + 1,
            output,
        );
    }

    evaluate_replace_string_output_cm740_driven(
        session,
        state,
        index,
        end,
        replacements,
        output,
        bindings,
        rule_index,
    )
}

#[allow(clippy::too_many_arguments)]
fn evaluate_replace_string_output(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    output: String,
    bindings: SymbolExprMap,
    rule_index: usize,
) -> Result<String, EvalError> {
    let rule = &state.rules[rule_index].rule;
    let output_expr = match &rule.replacement {
        ExactStringReplacementValue::Immediate(expr)
        | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
    };
    let output_result = evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);

    match output_result {
        Ok(rendered_expr) => resume_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            rendered_expr,
        ),
        Err(EvalError::InputRequired { request }) => {
            suspend_replace_string_output(session, state, index, end, replacements, output, request)
        }
        Err(error) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
#[cold]
#[inline(never)]
#[allow(dead_code)]
fn evaluate_replace_string_output_cm740_driven(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    output: String,
    bindings: SymbolExprMap,
    rule_index: usize,
) -> Result<String, EvalError> {
    let rule = &state.rules[rule_index].rule;
    let output_expr = match &rule.replacement {
        ExactStringReplacementValue::Immediate(expr)
        | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
    };
    let output_result = rum_runtime::cm740_probe::around("P2354", || {
        evaluate_string_rule_output_expr_result(session, &output_expr, &bindings)
    });

    match output_result {
        Ok(rendered_expr) => resume_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            rendered_expr,
        ),
        Err(EvalError::InputRequired { request }) => {
            suspend_replace_string_output(session, state, index, end, replacements, output, request)
        }
        Err(error) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
#[cold]
#[inline(never)]
fn suspend_replace_string_output(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    output: String,
    request: RuntimeInputRequest,
) -> Result<String, EvalError> {
    let state = FrozenStringValue::freeze(state);
    let output = PersistentStringText::freeze(output);
    rum_runtime::resume_or_suspend!(insert session, EvaluateReplaceStringOutputSuspensionFrame { state, index, end, replacements, output });
    Err(EvalError::InputRequired { request })
}

fn resume_suspended_replace_string_value_from_candidate(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceRuleEvalState>,
    mut index: usize,
    replacements: usize,
    mut rule_index: usize,
    mut output: PersistentStringText,
) -> Result<String, EvalError> {
    loop {
        let fixed = state.as_ref();
        if replacements >= fixed.max_replacements {
            return Ok(output
                .appended(fixed.chars[index..].iter().collect::<String>())
                .to_string());
        }

        let Some(rule) = fixed.rules.get(rule_index) else {
            if index == fixed.chars.len() {
                return Ok(output.to_string());
            }
            output = output.appended(fixed.chars[index].to_string());
            index += 1;
            rule_index = 0;
            continue;
        };

        let Some(match_state) =
            best_pattern_match_state(&rule.rule.pattern, &fixed.chars, index, fixed.config)
        else {
            rule_index += 1;
            continue;
        };
        let end = match_state.end;
        let bindings = match_state.bindings;

        if let Some(guard) = &rule.guard {
            let guard_result = evaluate_string_rule_guard_result(
                session,
                guard,
                &fixed.chars,
                index,
                end,
                &bindings,
            );
            return match guard_result {
                Ok(accepted) => resume_suspended_replace_string_value_after_guard_owned(
                    session,
                    state,
                    index,
                    end,
                    replacements,
                    rule_index,
                    output,
                    bindings,
                    accepted,
                ),
                Err(EvalError::InputRequired { request }) => {
                    let bindings = FrozenStringValue::freeze(bindings);
                    rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(accepted) => { (|| {
                                                resume_suspended_replace_string_value_after_guard(
                                                    session,
                                                    state,
                                                    index,
                                                    end,
                                                    replacements,
                                                    rule_index,
                                                    output,
                                                    bindings,
                                                    accepted,
                                                )
                                            })() },
                        suspend => ResumeSuspendedReplaceStringValueFromCandidateSuspensionFrame { state, index, end, replacements, rule_index, output, bindings },
                    )
                }
                Err(error) => Err(error),
            };
        }

        return evaluate_suspended_replace_string_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            &bindings,
            rule_index,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_replace_string_value_after_guard(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceRuleEvalState>,
    index: usize,
    end: usize,
    replacements: usize,
    rule_index: usize,
    output: PersistentStringText,
    bindings: FrozenStringValue<SymbolExprMap>,
    accepted: bool,
) -> Result<String, EvalError> {
    if !accepted {
        return resume_suspended_replace_string_value_from_candidate(
            session,
            state,
            index,
            replacements,
            rule_index + 1,
            output,
        );
    }
    evaluate_suspended_replace_string_output(
        session,
        state,
        index,
        end,
        replacements,
        output,
        bindings.as_ref(),
        rule_index,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_replace_string_value_after_guard_owned(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceRuleEvalState>,
    index: usize,
    end: usize,
    replacements: usize,
    rule_index: usize,
    output: PersistentStringText,
    bindings: SymbolExprMap,
    accepted: bool,
) -> Result<String, EvalError> {
    if !accepted {
        return resume_suspended_replace_string_value_from_candidate(
            session,
            state,
            index,
            replacements,
            rule_index + 1,
            output,
        );
    }
    evaluate_suspended_replace_string_output(
        session,
        state,
        index,
        end,
        replacements,
        output,
        &bindings,
        rule_index,
    )
}

#[allow(clippy::too_many_arguments)]
fn evaluate_suspended_replace_string_output(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceRuleEvalState>,
    index: usize,
    end: usize,
    replacements: usize,
    output: PersistentStringText,
    bindings: &SymbolExprMap,
    rule_index: usize,
) -> Result<String, EvalError> {
    let rule = &state.as_ref().rules[rule_index].rule;
    let output_expr = match &rule.replacement {
        ExactStringReplacementValue::Immediate(expr)
        | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
    };
    let output_result = evaluate_string_rule_output_expr_result(session, &output_expr, bindings);
    match output_result {
        Ok(rendered_expr) => resume_suspended_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            rendered_expr,
        ),
        Err(EvalError::InputRequired { request }) => rum_runtime::resume_or_suspend!(
            session,
            Err(EvalError::InputRequired { request }),
            ok(rendered_expr_expr) => { let rendered_expr = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(rendered_expr_expr); (|| {
                            resume_suspended_replace_string_value_with_output(
                                session,
                                state,
                                index,
                                end,
                                replacements,
                                output,
                                rendered_expr,
                            )
                        })() },
            suspend => EvaluateSuspendedReplaceStringOutputSuspensionFrame { state, index, end, replacements, output },
        ),
        Err(error) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_replace_string_value_with_output(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceRuleEvalState>,
    index: usize,
    end: usize,
    replacements: usize,
    output: PersistentStringText,
    rendered_expr: Expr,
) -> Result<String, EvalError> {
    if rum_runtime::cm740_probe::driven() {
        return resume_suspended_replace_string_value_with_output_cm740_driven(
            session,
            state,
            index,
            end,
            replacements,
            output,
            rendered_expr,
        );
    }

    let rendered = expr_as_string(&rendered_expr)
        .map(str::to_owned)
        .unwrap_or_else(|| rendered_expr.to_string());
    let mut output = output.appended(rendered);
    let fixed = state.as_ref();
    let next_index = if end == index {
        if index == fixed.chars.len() {
            return Ok(output.to_string());
        }
        output = output.appended(fixed.chars[index].to_string());
        index + 1
    } else {
        end
    };
    resume_suspended_replace_string_value_from_candidate(
        session,
        state,
        next_index,
        replacements + 1,
        0,
        output,
    )
}

#[allow(clippy::too_many_arguments)]
#[cold]
#[inline(never)]
#[allow(dead_code)]
fn resume_suspended_replace_string_value_with_output_cm740_driven(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceRuleEvalState>,
    index: usize,
    end: usize,
    replacements: usize,
    output: PersistentStringText,
    rendered_expr: Expr,
) -> Result<String, EvalError> {
    let rendered = expr_as_string(&rendered_expr)
        .map(str::to_owned)
        .unwrap_or_else(|| rendered_expr.to_string());
    let mut output = if let Some(row) = ["P2348", "P2354"]
        .into_iter()
        .find(|row| rum_runtime::cm740_probe::frame(row))
    {
        rum_runtime::cm740_probe::hit(row, "OmitActualReplacementAppend");
        output
    } else {
        output.appended(rendered)
    };
    let fixed = state.as_ref();
    let next_index = if end == index {
        if index == fixed.chars.len() {
            return Ok(output.to_string());
        }
        output = output.appended(fixed.chars[index].to_string());
        index + 1
    } else {
        end
    };
    resume_suspended_replace_string_value_from_candidate(
        session,
        state,
        next_index,
        replacements + 1,
        0,
        output,
    )
}

fn resume_replace_string_value_with_output(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    mut output: String,
    rendered_expr: Expr,
) -> Result<String, EvalError> {
    let rendered = expr_as_string(&rendered_expr)
        .map(str::to_owned)
        .unwrap_or_else(|| rendered_expr.to_string());
    output.push_str(&rendered);

    let next_index = if end == index {
        if index == state.chars.len() {
            return Ok(output);
        }
        output.push(state.chars[index]);
        index + 1
    } else {
        end
    };

    (if rum_runtime::cm740_probe::driven() {
        resume_replace_string_value_from_candidate_cm740_driven(
            session,
            state,
            next_index,
            replacements + 1,
            0,
            output,
        )
    } else {
        resume_replace_string_value_from_candidate(
            session,
            state,
            next_index,
            replacements + 1,
            0,
            output,
        )
    })
}

fn resume_replace_string_values_from_index(
    session: &mut Session,
    state: GuardedStringReplaceValuesEvalState,
    mut index: usize,
    mut rendered: Vec<String>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(value) = state.values.get(index) else {
            return Ok(string_list_expr(rendered));
        };

        let result = replace_string_value_with_guarded_rules(
            session,
            value,
            &state.rules,
            state.max_replacements,
            state.config,
        );
        match result {
            Ok(value) => {
                rendered.push(value);
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                let state = FrozenStringValue::freeze(state);
                let rendered = PersistentStringSequence::freeze(rendered);
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => { (|| {
                                        resume_suspended_replace_string_values_with_value(
                                            session, state, index, rendered, value,
                                        )
                                    })() },
                    suspend => ResumeReplaceStringValuesFromIndexSuspensionFrame { state, index, rendered },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_replace_string_values_from_index(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceValuesEvalState>,
    mut index: usize,
    mut rendered: PersistentStringSequence<String>,
) -> Result<Expr, EvalError> {
    loop {
        let fixed = state.as_ref();
        let Some(value) = fixed.values.get(index) else {
            return Ok(string_list_expr(rendered.to_vec()));
        };
        let result = replace_string_value_with_guarded_rules(
            session,
            value,
            &fixed.rules,
            fixed.max_replacements,
            fixed.config,
        );
        match result {
            Ok(value) => {
                rendered = rendered.appended(value);
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => { (|| {
                                        resume_suspended_replace_string_values_with_value(
                                            session, state, index, rendered, value,
                                        )
                                    })() },
                    suspend => ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrame { state, index, rendered },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_replace_string_values_with_value(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringReplaceValuesEvalState>,
    index: usize,
    rendered: PersistentStringSequence<String>,
    value: String,
) -> Result<Expr, EvalError> {
    resume_suspended_replace_string_values_from_index(
        session,
        state,
        index + 1,
        rendered.appended(value),
    )
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<GuardedStringReplaceRuleEvalState>();
    assert_traces::<GuardedStringReplaceValuesEvalState>();
};
