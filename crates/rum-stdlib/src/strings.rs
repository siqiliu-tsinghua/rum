use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::{BTreeMap, BTreeSet};

use rum_core::{Atom, BigInt, Expr, Rational, Symbol, heads};
pub(crate) use rum_runtime::kernel_api::render_input_like_expr;
use rum_runtime::kernel_api::{
    EvalError, Session, SymbolExprMap, ToStringFormat, association_entries_from_expr,
    base64_encode, call_items_with_head, evaluate_definition_rhs_guards_for_match,
    expr_as_exact_rational, expr_as_non_negative_usize, expr_as_signed_isize, expr_as_string,
    expr_as_string_list, expr_as_symbol, is_system_symbol_name, render_association,
    render_to_string_expr, split_definition_rhs_guards, substitute_lexical_bindings, system_expr,
    truthiness_symbol,
};
use rum_syntax::parse_expr;

mod encoding;
mod patterns;
mod scripting;
mod similarity;
mod suspended_state;
mod template;
mod text;
mod tokenize;

use self::suspended_state::{
    FrozenStringSequence, FrozenStringValue, PersistentStringSequence, PersistentStringText,
};

#[cfg(feature = "hash")]
pub(crate) use encoding::eval_hash;
pub(crate) use encoding::{eval_compress, eval_string_to_byte_array, eval_uncompress};
pub(crate) use patterns::{
    eval_string_apply, eval_string_cases, eval_string_contains_q, eval_string_count,
    eval_string_delete, eval_string_drop, eval_string_ends_q, eval_string_free_q,
    eval_string_match_q, eval_string_part, eval_string_position, eval_string_replace,
    eval_string_split, eval_string_starts_q, eval_string_take, eval_string_take_drop,
    eval_string_trim,
};
pub(crate) use scripting::{
    eval_align_columns, eval_file_base_name, eval_file_extension, eval_file_name_split,
    eval_html_escape, eval_html_unescape, eval_parse_args, eval_shell_quote, eval_shell_split,
    eval_string_dedent, eval_string_expand_tabs, eval_string_indent, eval_string_wrap,
    eval_url_decode, eval_url_encode, eval_url_parse,
};
pub(crate) use similarity::{
    eval_alphabet, eval_alphabetic_sort, eval_diff, eval_edit_distance, eval_hamming_distance,
    eval_longest_common_subsequence, eval_nearest, eval_pluralize, eval_sequence_alignment,
};
pub(crate) use template::{eval_string_form, eval_string_template, eval_template_apply};
pub(crate) use text::{
    eval_capitalize, eval_character_counts, eval_character_range, eval_characters,
    eval_decapitalize, eval_decimal_string, eval_digit_q, eval_letter_q, eval_lower_case_q,
    eval_remove_diacritics, eval_row, eval_string_join, eval_string_length, eval_string_lines,
    eval_string_pad_left, eval_string_pad_right, eval_string_repeat, eval_string_reverse,
    eval_string_riffle, eval_syntax_q, eval_text_string, eval_text_words, eval_to_character_code,
    eval_to_expression, eval_to_lower_case, eval_to_string, eval_to_upper_case, eval_upper_case_q,
};
pub(crate) use tokenize::{eval_rum_tokens, eval_tokenize_with};

fn bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

fn string_slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn parse_strict_option(option_exprs: Vec<Expr>) -> Result<Option<bool>, EvalError> {
    let mut strict = false;

    for expr in option_exprs {
        let Some((name, value)) = string_option_rule(&expr) else {
            return Ok(None);
        };
        match name {
            "Strict" => {
                let Some(value) = bool_option_value(value) else {
                    return Ok(None);
                };
                strict = value;
            }
            _ => return Ok(None),
        }
    }

    Ok(Some(strict))
}

fn is_strict_option_rule(expr: &Expr) -> bool {
    string_option_rule(expr)
        .map(|(name, _)| name == "Strict")
        .unwrap_or(false)
}

fn string_option_rule(expr: &Expr) -> Option<(&str, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }
    Some((string_option_name(&args[0])?, &args[1]))
}

fn string_option_name(expr: &Expr) -> Option<&str> {
    if let Some(name) = expr_as_string(expr) {
        return Some(name);
    }
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

fn bool_option_value(expr: &Expr) -> Option<bool> {
    if is_system_symbol_name(expr, "True") {
        Some(true)
    } else if is_system_symbol_name(expr, "False") {
        Some(false)
    } else {
        None
    }
}

fn string_unary_operator_form(symbol: &str, operator: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator.clone(), string_slot_expr()],
        )],
    )
}

#[derive(FrameRootTrace)]
struct StringApplyEvalState {
    head: Expr,
    operator: Expr,
    values: Vec<String>,
    list_output: bool,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedStringApplyEvalState {
    head: Expr,
    operator: Expr,
    values: FrozenStringSequence<String>,
    list_output: bool,
}

impl SuspendedStringApplyEvalState {
    fn freeze(state: StringApplyEvalState) -> Self {
        Self {
            head: state.head,
            operator: state.operator,
            values: FrozenStringSequence::freeze(state.values),
            list_output: state.list_output,
        }
    }
}

fn string_apply_value(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    value: &str,
) -> Result<Expr, EvalError> {
    resume_string_apply_from_index(
        session,
        StringApplyEvalState {
            head: head.clone(),
            operator: operator.clone(),
            values: vec![value.to_string()],
            list_output: false,
        },
        0,
        Vec::with_capacity(1),
    )
}

/// Applies the operator to one string per loop iteration instead of
/// recursing per string. On suspension the state, the index, and the
/// accumulated output move into the suspension frame by value — the old
/// per-string `state.clone()` plus `outputs.clone()` made a wide list
/// quadratic in its element count.
fn resume_string_apply_from_index(
    session: &mut Session,
    state: StringApplyEvalState,
    mut index: usize,
    mut outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(value) = state.values.get(index) else {
            return Ok(if state.list_output {
                list_expr(outputs)
            } else {
                outputs
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| Expr::call(state.head, Vec::new()))
            });
        };

        let applied = Expr::call(
            state.operator.clone(),
            vec![list_expr(
                value
                    .chars()
                    .map(|ch| Expr::string(ch.to_string()))
                    .collect(),
            )],
        );
        match session.evaluate(applied) {
            Ok(evaluated) => {
                push_string_apply_output(&state, index, &mut outputs, evaluated);
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let state = SuspendedStringApplyEvalState::freeze(state);
                                        let outputs = PersistentStringSequence::freeze(outputs);
                ResumeStringApplyFromIndexSuspensionFrame { state, index, outputs }
                },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn push_string_apply_output(
    state: &StringApplyEvalState,
    index: usize,
    outputs: &mut Vec<Expr>,
    evaluated: Expr,
) {
    let fallback = state
        .values
        .get(index)
        .map(|value| {
            Expr::call(
                state.head.clone(),
                vec![state.operator.clone(), Expr::string(value.clone())],
            )
        })
        .unwrap_or_else(|| Expr::call(state.head.clone(), Vec::new()));
    outputs.push(string_apply_output_expr(evaluated).unwrap_or(fallback));
}

fn resume_suspended_string_apply_from_index(
    session: &mut Session,
    state: SuspendedStringApplyEvalState,
    mut index: usize,
    mut outputs: PersistentStringSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(value) = state.values.get(index) else {
            let outputs = outputs.to_vec();
            return Ok(if state.list_output {
                list_expr(outputs)
            } else {
                outputs
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| Expr::call(state.head, Vec::new()))
            });
        };

        let applied = Expr::call(
            state.operator.clone(),
            vec![list_expr(
                value
                    .chars()
                    .map(|ch| Expr::string(ch.to_string()))
                    .collect(),
            )],
        );
        match session.evaluate(applied) {
            Ok(evaluated) => {
                outputs = outputs.appended(suspended_string_apply_output(&state, index, evaluated));
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedStringApplyFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_string_apply_with_value(
    session: &mut Session,
    state: SuspendedStringApplyEvalState,
    index: usize,
    outputs: PersistentStringSequence<Expr>,
    evaluated: Expr,
) -> Result<Expr, EvalError> {
    let output = suspended_string_apply_output(&state, index, evaluated);
    resume_suspended_string_apply_from_index(session, state, index + 1, outputs.appended(output))
}

fn suspended_string_apply_output(
    state: &SuspendedStringApplyEvalState,
    index: usize,
    evaluated: Expr,
) -> Expr {
    let fallback = state
        .values
        .get(index)
        .map(|value| {
            Expr::call(
                state.head.clone(),
                vec![state.operator.clone(), Expr::string(value.clone())],
            )
        })
        .unwrap_or_else(|| Expr::call(state.head.clone(), Vec::new()));
    string_apply_output_expr(evaluated).unwrap_or(fallback)
}

fn resume_string_apply_values(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    values: Vec<String>,
) -> Result<Expr, EvalError> {
    let capacity = values.len();
    resume_string_apply_from_index(
        session,
        StringApplyEvalState {
            head: head.clone(),
            operator: operator.clone(),
            values,
            list_output: true,
        },
        0,
        Vec::with_capacity(capacity),
    )
}

fn string_apply_output_expr(expr: Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::String(_)) => Some(expr),
        Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
            if args
                .iter()
                .all(|item| matches!(item, Expr::Atom(Atom::String(_))))
            {
                let joined = args
                    .into_iter()
                    .filter_map(|item| match item {
                        Expr::Atom(Atom::String(value)) => Some(value),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .concat();
                return Some(Expr::string(joined));
            }

            let converted = args
                .into_iter()
                .map(string_apply_output_expr)
                .collect::<Option<Vec<_>>>()?;
            Some(list_expr(converted))
        }
        _ => None,
    }
}

fn is_string_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn string_list_expr(items: Vec<String>) -> Expr {
    list_expr(items.into_iter().map(Expr::string).collect())
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<StringApplyEvalState>();
    assert_traces::<SuspendedStringApplyEvalState>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeStringApplyFromIndexSuspensionFrame {
    state: crate::strings::SuspendedStringApplyEvalState,
    index: usize,
    outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeStringApplyFromIndexSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(evaluated) => (|| {
                resume_suspended_string_apply_with_value(session, state, index, outputs, evaluated)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct ResumeSuspendedStringApplyFromIndexSuspensionFrame {
    state: crate::strings::SuspendedStringApplyEvalState,
    index: usize,
    outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedStringApplyFromIndexSuspensionFrame
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(evaluated) => (|| {
                resume_suspended_string_apply_with_value(session, state, index, outputs, evaluated)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

pub(crate) use encoding::eval_uncompress_with_options;

pub(crate) use scripting::eval_html_unescape_with_options;

pub(crate) use template::eval_template_apply_with_options;

pub(crate) use scripting::eval_parse_args_with_options;

pub(crate) use scripting::{
    eval_url_decode_with_options, eval_url_encode_with_options, eval_url_parse_with_options,
};

pub(crate) use scripting::{
    eval_align_columns_with_options, eval_string_indent_with_options, eval_string_wrap_with_options,
};

pub(crate) use patterns::{
    eval_string_cases_with_options, eval_string_contains_q_with_options,
    eval_string_count_with_options, eval_string_delete_with_options,
    eval_string_ends_q_with_options, eval_string_free_q_with_options,
    eval_string_match_q_with_options, eval_string_position_with_options,
    eval_string_replace_with_options, eval_string_split_with_options,
    eval_string_starts_q_with_options,
};
