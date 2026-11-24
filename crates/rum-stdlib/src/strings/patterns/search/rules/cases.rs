use super::*;

#[derive(FrameRootTrace)]
pub(super) struct StringCasesRuleEvalState {
    chars: Vec<char>,
    spec: ExactStringCaseSpec,
    max_matches: usize,
    config: StringCasesConfig,
    literal_matches: Option<Vec<(usize, usize)>>,
    literal_match_index: usize,
}

#[derive(FrameRootTrace)]
pub(super) struct StringCasesValuesEvalState {
    values: Vec<String>,
    spec: ExactStringCaseSpec,
    max_matches: usize,
    config: StringCasesConfig,
}

pub(in super::super) fn collect_string_cases_with_rule_evaluation(
    session: &mut Session,
    value: &str,
    spec: &ExactStringCaseSpec,
    max_matches: usize,
    config: StringCasesConfig,
) -> Result<Vec<Expr>, EvalError> {
    if max_matches == 0 {
        return Ok(Vec::new());
    }

    let chars = value.chars().collect::<Vec<_>>();
    let literal_matches = match spec {
        ExactStringCaseSpec::Rules(rules) => match rules.as_slice() {
            [rule] => linear_literal_match_ranges(
                &chars,
                &rule.pattern,
                config.match_config,
                matches!(config.overlaps, StringSearchOverlaps::Overlapping),
                max_matches,
            ),
            _ => None,
        },
        ExactStringCaseSpec::Patterns(_) => None,
    };

    (if rum_runtime::cm740_probe::driven() {
        resume_collect_string_cases_from_index_cm740_driven(
            session,
            StringCasesRuleEvalState {
                chars,
                spec: spec.clone(),
                max_matches,
                config,
                literal_matches,
                literal_match_index: 0,
            },
            0,
            Vec::new(),
        )
    } else {
        resume_collect_string_cases_from_index(
            session,
            StringCasesRuleEvalState {
                chars,
                spec: spec.clone(),
                max_matches,
                config,
                literal_matches,
                literal_match_index: 0,
            },
            0,
            Vec::new(),
        )
    })
}

pub(in super::super) fn collect_string_cases_values_with_rule_evaluation_expr(
    session: &mut Session,
    values: Vec<String>,
    spec: &ExactStringCaseSpec,
    max_matches: usize,
    config: StringCasesConfig,
) -> Result<Expr, EvalError> {
    let capacity = values.len();
    resume_collect_string_cases_values_from_index(
        session,
        StringCasesValuesEvalState {
            values,
            spec: spec.clone(),
            max_matches,
            config,
        },
        0,
        Vec::with_capacity(capacity),
    )
}

fn resume_collect_string_cases_from_index(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    mut index: usize,
    mut matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if state.literal_matches.is_some() {
        return resume_collect_literal_string_cases(session, state, matches);
    }

    while index <= state.chars.len() && matches.len() < state.max_matches {
        match &state.spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, match_state)) = best_pattern_match_at_with_bindings(
                    patterns,
                    &state.chars,
                    index,
                    state.config.match_config,
                ) {
                    matches.push(Expr::string(pattern_match_text(
                        &state.chars[index..match_state.end],
                    )));
                    let Some(next_index) = next_string_match_index(
                        index,
                        match_state.end,
                        state.chars.len(),
                        state.config.overlaps,
                    ) else {
                        break;
                    };
                    index = next_index;
                } else {
                    if index == state.chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((output_expr, end, bindings)) = rules.iter().find_map(|rule| {
                    best_pattern_match_state(
                        &rule.pattern,
                        &state.chars,
                        index,
                        state.config.match_config,
                    )
                    .map(|match_state| {
                        let output_expr = match &rule.output {
                            ExactStringCaseOutput::Immediate(expr)
                            | ExactStringCaseOutput::Delayed(expr) => expr.clone(),
                        };
                        (output_expr, match_state.end, match_state.bindings)
                    })
                }) {
                    let output_result =
                        evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
                    return drive_string_cases_output_result(
                        session,
                        state,
                        index,
                        end,
                        matches,
                        output_result,
                        None,
                    );
                }

                if index == state.chars.len() {
                    break;
                }
                index += 1;
            }
        }
    }

    Ok(matches)
}

#[cold]
#[inline(never)]
#[allow(dead_code)]
fn resume_collect_string_cases_from_index_cm740_driven(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    mut index: usize,
    mut matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if state.literal_matches.is_some() {
        return resume_collect_literal_string_cases_cm740_driven(session, state, matches);
    }

    while index <= state.chars.len() && matches.len() < state.max_matches {
        match &state.spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, match_state)) = best_pattern_match_at_with_bindings(
                    patterns,
                    &state.chars,
                    index,
                    state.config.match_config,
                ) {
                    matches.push(Expr::string(pattern_match_text(
                        &state.chars[index..match_state.end],
                    )));
                    let Some(next_index) = next_string_match_index(
                        index,
                        match_state.end,
                        state.chars.len(),
                        state.config.overlaps,
                    ) else {
                        break;
                    };
                    index = next_index;
                } else {
                    if index == state.chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((output_expr, end, bindings)) = rules.iter().find_map(|rule| {
                    best_pattern_match_state(
                        &rule.pattern,
                        &state.chars,
                        index,
                        state.config.match_config,
                    )
                    .map(|match_state| {
                        let output_expr = match &rule.output {
                            ExactStringCaseOutput::Immediate(expr)
                            | ExactStringCaseOutput::Delayed(expr) => expr.clone(),
                        };
                        (output_expr, match_state.end, match_state.bindings)
                    })
                }) {
                    let (output_result, handoff) =
                        rum_runtime::cm740_probe::q25::around_handoff("P2379", || {
                            evaluate_string_rule_output_expr_result(
                                session,
                                &output_expr,
                                &bindings,
                            )
                        });
                    return drive_string_cases_output_result(
                        session,
                        state,
                        index,
                        end,
                        matches,
                        output_result,
                        handoff,
                    );
                }

                if index == state.chars.len() {
                    break;
                }
                index += 1;
            }
        }
    }

    Ok(matches)
}

fn resume_collect_literal_string_cases(
    session: &mut Session,
    mut state: StringCasesRuleEvalState,
    matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if matches.len() >= state.max_matches {
        return Ok(matches);
    }

    let next_match = state
        .literal_matches
        .as_ref()
        .and_then(|ranges| ranges.get(state.literal_match_index))
        .copied();
    let Some((start, end)) = next_match else {
        return Ok(matches);
    };
    state.literal_match_index += 1;

    let ExactStringCaseSpec::Rules(rules) = &state.spec else {
        unreachable!("literal rule matching requires a rule specification");
    };
    let rule = rules
        .first()
        .expect("literal case matching requires one rule");
    let output_expr = match &rule.output {
        ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
            expr.clone()
        }
    };
    let output_result =
        evaluate_string_rule_output_expr_result(session, &output_expr, &SymbolExprMap::default());
    drive_string_cases_output_result(session, state, start, end, matches, output_result, None)
}

#[cold]
#[inline(never)]
#[allow(dead_code)]
fn resume_collect_literal_string_cases_cm740_driven(
    session: &mut Session,
    mut state: StringCasesRuleEvalState,
    matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if matches.len() >= state.max_matches {
        return Ok(matches);
    }

    let next_match = state
        .literal_matches
        .as_ref()
        .and_then(|ranges| ranges.get(state.literal_match_index))
        .copied();
    let Some((start, end)) = next_match else {
        return Ok(matches);
    };
    state.literal_match_index += 1;

    let ExactStringCaseSpec::Rules(rules) = &state.spec else {
        unreachable!("literal rule matching requires a rule specification");
    };
    let rule = rules
        .first()
        .expect("literal case matching requires one rule");
    let output_expr = match &rule.output {
        ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
            expr.clone()
        }
    };
    let output_result = rum_runtime::cm740_probe::around("P2381", || {
        evaluate_string_rule_output_expr_result(session, &output_expr, &SymbolExprMap::default())
    });
    drive_string_cases_output_result(session, state, start, end, matches, output_result, None)
}

fn drive_string_cases_output_result(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    index: usize,
    end: usize,
    matches: Vec<Expr>,
    output_result: Result<Expr, EvalError>,
    handoff: Option<rum_runtime::cm740_probe::q25::Handoff>,
) -> Result<Vec<Expr>, EvalError> {
    match output_result {
        Ok(output) => {
            drop(handoff);
            resume_collect_string_cases_with_output(session, state, index, end, matches, output)
        }
        Err(EvalError::InputRequired { request }) => {
            let literal_match_index = state.literal_match_index;
            let state = FrozenStringValue::freeze(state);
            let matches = PersistentStringSequence::freeze(matches);
            if let Some(handoff) = handoff {
                let frame = DriveStringCasesOutputResultSuspensionFrame {
                    state,
                    index,
                    end,
                    literal_match_index,
                    matches,
                };
                session.insert_suspension_frame_q25(frame, handoff);
                Err(EvalError::InputRequired { request })
            } else {
                rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(output_expr) => {
                        let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(output_expr);
                        (|| {
                            resume_suspended_collect_string_cases_with_output(
                                session,
                                state,
                                index,
                                end,
                                literal_match_index,
                                matches,
                                output,
                            )
                        })()
                    },
                    suspend => DriveStringCasesOutputResultSuspensionFrame {
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                    },
                )
            }
        }
        Err(error) => {
            drop(handoff);
            Err(error)
        }
    }
}

fn resume_suspended_collect_string_cases_from_index(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    mut index: usize,
    mut literal_match_index: usize,
    mut matches: PersistentStringSequence<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    let fixed = state.as_ref();
    if fixed.literal_matches.is_some() {
        if matches.len() >= fixed.max_matches {
            return Ok(matches.to_vec());
        }
        let next_match = fixed
            .literal_matches
            .as_ref()
            .and_then(|ranges| ranges.get(literal_match_index))
            .copied();
        let Some((start, end)) = next_match else {
            return Ok(matches.to_vec());
        };
        literal_match_index += 1;

        let ExactStringCaseSpec::Rules(rules) = &fixed.spec else {
            unreachable!("literal rule matching requires a rule specification");
        };
        let rule = rules
            .first()
            .expect("literal case matching requires one rule");
        let output_expr = match &rule.output {
            ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
                expr.clone()
            }
        };
        let output_result = evaluate_string_rule_output_expr_result(
            session,
            &output_expr,
            &SymbolExprMap::default(),
        );
        return drive_suspended_string_cases_output_result(
            session,
            state,
            start,
            end,
            literal_match_index,
            matches,
            output_result,
        );
    }

    while index <= fixed.chars.len() && matches.len() < fixed.max_matches {
        match &fixed.spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, match_state)) = best_pattern_match_at_with_bindings(
                    patterns,
                    &fixed.chars,
                    index,
                    fixed.config.match_config,
                ) {
                    matches = matches.appended(Expr::string(pattern_match_text(
                        &fixed.chars[index..match_state.end],
                    )));
                    let Some(next_index) = next_string_match_index(
                        index,
                        match_state.end,
                        fixed.chars.len(),
                        fixed.config.overlaps,
                    ) else {
                        break;
                    };
                    index = next_index;
                } else {
                    if index == fixed.chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((output_expr, end, bindings)) = rules.iter().find_map(|rule| {
                    best_pattern_match_state(
                        &rule.pattern,
                        &fixed.chars,
                        index,
                        fixed.config.match_config,
                    )
                    .map(|match_state| {
                        let output_expr = match &rule.output {
                            ExactStringCaseOutput::Immediate(expr)
                            | ExactStringCaseOutput::Delayed(expr) => expr.clone(),
                        };
                        (output_expr, match_state.end, match_state.bindings)
                    })
                }) {
                    let output_result =
                        evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
                    return drive_suspended_string_cases_output_result(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output_result,
                    );
                }

                if index == fixed.chars.len() {
                    break;
                }
                index += 1;
            }
        }
    }

    Ok(matches.to_vec())
}

#[cold]
#[inline(never)]
#[allow(dead_code)]
fn resume_suspended_collect_string_cases_from_index_cm740_driven(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    mut index: usize,
    mut literal_match_index: usize,
    mut matches: PersistentStringSequence<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    let fixed = state.as_ref();
    if fixed.literal_matches.is_some() {
        if matches.len() >= fixed.max_matches {
            return Ok(matches.to_vec());
        }
        let next_match = fixed
            .literal_matches
            .as_ref()
            .and_then(|ranges| ranges.get(literal_match_index))
            .copied();
        let Some((start, end)) = next_match else {
            return Ok(matches.to_vec());
        };
        literal_match_index += 1;

        let ExactStringCaseSpec::Rules(rules) = &fixed.spec else {
            unreachable!("literal rule matching requires a rule specification");
        };
        let rule = rules
            .first()
            .expect("literal case matching requires one rule");
        let output_expr = match &rule.output {
            ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
                expr.clone()
            }
        };
        let output_result = rum_runtime::cm740_probe::around("P2385", || {
            evaluate_string_rule_output_expr_result(
                session,
                &output_expr,
                &SymbolExprMap::default(),
            )
        });
        return drive_suspended_string_cases_output_result(
            session,
            state,
            start,
            end,
            literal_match_index,
            matches,
            output_result,
        );
    }

    while index <= fixed.chars.len() && matches.len() < fixed.max_matches {
        match &fixed.spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, match_state)) = best_pattern_match_at_with_bindings(
                    patterns,
                    &fixed.chars,
                    index,
                    fixed.config.match_config,
                ) {
                    matches = matches.appended(Expr::string(pattern_match_text(
                        &fixed.chars[index..match_state.end],
                    )));
                    let Some(next_index) = next_string_match_index(
                        index,
                        match_state.end,
                        fixed.chars.len(),
                        fixed.config.overlaps,
                    ) else {
                        break;
                    };
                    index = next_index;
                } else {
                    if index == fixed.chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((output_expr, end, bindings)) = rules.iter().find_map(|rule| {
                    best_pattern_match_state(
                        &rule.pattern,
                        &fixed.chars,
                        index,
                        fixed.config.match_config,
                    )
                    .map(|match_state| {
                        let output_expr = match &rule.output {
                            ExactStringCaseOutput::Immediate(expr)
                            | ExactStringCaseOutput::Delayed(expr) => expr.clone(),
                        };
                        (output_expr, match_state.end, match_state.bindings)
                    })
                }) {
                    let output_result = rum_runtime::cm740_probe::around("P2387", || {
                        evaluate_string_rule_output_expr_result(session, &output_expr, &bindings)
                    });
                    return drive_suspended_string_cases_output_result(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output_result,
                    );
                }

                if index == fixed.chars.len() {
                    break;
                }
                index += 1;
            }
        }
    }

    Ok(matches.to_vec())
}

#[allow(clippy::too_many_arguments)]
fn drive_suspended_string_cases_output_result(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    index: usize,
    end: usize,
    literal_match_index: usize,
    matches: PersistentStringSequence<Expr>,
    output_result: Result<Expr, EvalError>,
) -> Result<Vec<Expr>, EvalError> {
    match output_result {
        Ok(output) => resume_suspended_collect_string_cases_with_output(
            session,
            state,
            index,
            end,
            literal_match_index,
            matches,
            output,
        ),
        Err(EvalError::InputRequired { request }) => rum_runtime::resume_or_suspend!(
            session,
            Err(EvalError::InputRequired { request }),
            ok(output_expr) => { let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(output_expr); (|| {
                            resume_suspended_collect_string_cases_with_output(
                                session,
                                state,
                                index,
                                end,
                                literal_match_index,
                                matches,
                                output,
                            )
                        })() },
            suspend => DriveSuspendedStringCasesOutputResultSuspensionFrame { state, index, end, literal_match_index, matches },
        ),
        Err(error) => Err(error),
    }
}

pub(super) fn resume_suspended_collect_string_cases_with_output(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    index: usize,
    end: usize,
    literal_match_index: usize,
    matches: PersistentStringSequence<Expr>,
    output: Expr,
) -> Result<Vec<Expr>, EvalError> {
    if rum_runtime::cm740_probe::driven() {
        return resume_suspended_collect_string_cases_with_output_cm740_driven(
            session,
            state,
            index,
            end,
            literal_match_index,
            matches,
            output,
        );
    }

    let matches = matches.appended(output);
    let Some(next_index) = next_string_match_index(
        index,
        end,
        state.as_ref().chars.len(),
        state.as_ref().config.overlaps,
    ) else {
        return Ok(matches.to_vec());
    };

    resume_suspended_collect_string_cases_from_index(
        session,
        state,
        next_index,
        literal_match_index,
        matches,
    )
}

#[cold]
#[inline(never)]
#[allow(dead_code)]
pub(super) fn resume_suspended_collect_string_cases_with_output_cm740_driven(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    index: usize,
    end: usize,
    literal_match_index: usize,
    matches: PersistentStringSequence<Expr>,
    output: Expr,
) -> Result<Vec<Expr>, EvalError> {
    let matches = if rum_runtime::cm740_probe::q25::consume_append(
        "P2379",
        "OmitActualStringCaseAppend",
        &output,
    ) {
        rum_runtime::cm740_probe::hit("P2379", "OmitActualStringCaseAppend");
        matches
    } else if let Some(row) = ["P2381", "P2385", "P2387"]
        .into_iter()
        .find(|row| rum_runtime::cm740_probe::frame(row))
    {
        rum_runtime::cm740_probe::hit(row, "OmitActualStringCaseAppend");
        matches
    } else {
        matches.appended(output)
    };
    let Some(next_index) = next_string_match_index(
        index,
        end,
        state.as_ref().chars.len(),
        state.as_ref().config.overlaps,
    ) else {
        return Ok(matches.to_vec());
    };

    resume_suspended_collect_string_cases_from_index_cm740_driven(
        session,
        state,
        next_index,
        literal_match_index,
        matches,
    )
}

fn resume_collect_string_cases_with_output(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    index: usize,
    end: usize,
    mut matches: Vec<Expr>,
    output: Expr,
) -> Result<Vec<Expr>, EvalError> {
    matches.push(output);
    let Some(next_index) =
        next_string_match_index(index, end, state.chars.len(), state.config.overlaps)
    else {
        return Ok(matches);
    };

    (if rum_runtime::cm740_probe::driven() {
        resume_collect_string_cases_from_index_cm740_driven(session, state, next_index, matches)
    } else {
        resume_collect_string_cases_from_index(session, state, next_index, matches)
    })
}

/// Collects one string per loop iteration instead of recursing per string.
/// On suspension the state, the index, and the accumulated output move into
/// the suspension frame by value — the old per-string `state.clone()` plus
/// `outputs.clone()` made a wide list quadratic in its element count.
fn resume_collect_string_cases_values_from_index(
    session: &mut Session,
    state: StringCasesValuesEvalState,
    mut index: usize,
    mut outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(value) = state.values.get(index) else {
            return Ok(list_expr(outputs));
        };

        let result = collect_string_cases_with_rule_evaluation(
            session,
            value,
            &state.spec,
            state.max_matches,
            state.config,
        );
        match result {
            Ok(items) => {
                outputs.push(list_expr(items));
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                let state = FrozenStringValue::freeze(state);
                let outputs = PersistentStringSequence::freeze(outputs);
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(items) => { (|| {
                                        resume_suspended_collect_string_cases_values_with_value(
                                            session, state, index, outputs, items,
                                        )
                                    })() },
                    suspend => ResumeCollectStringCasesValuesFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_collect_string_cases_values_from_index(
    session: &mut Session,
    state: FrozenStringValue<StringCasesValuesEvalState>,
    mut index: usize,
    mut outputs: PersistentStringSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let fixed = state.as_ref();
        let Some(value) = fixed.values.get(index) else {
            return Ok(list_expr(outputs.to_vec()));
        };

        let result = collect_string_cases_with_rule_evaluation(
            session,
            value,
            &fixed.spec,
            fixed.max_matches,
            fixed.config,
        );
        match result {
            Ok(items) => {
                outputs = outputs.appended(list_expr(items));
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(items) => { (|| {
                                        resume_suspended_collect_string_cases_values_with_value(
                                            session, state, index, outputs, items,
                                        )
                                    })() },
                    suspend => ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

pub(super) fn resume_suspended_collect_string_cases_values_with_value(
    session: &mut Session,
    state: FrozenStringValue<StringCasesValuesEvalState>,
    index: usize,
    outputs: PersistentStringSequence<Expr>,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    resume_suspended_collect_string_cases_values_from_index(
        session,
        state,
        index + 1,
        outputs.appended(list_expr(items)),
    )
}
