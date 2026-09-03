mod resume_frames;
use resume_frames::*;

use super::matcher::{
    collect_string_cases, split_string_value, string_contains_q, string_count_matches,
    string_delete_matches, string_ends_q, string_free_q, string_match_q, string_position_expr,
    string_starts_q, trim_string_value,
};
use super::*;
use guarded::{
    collect_string_cases_values_with_guarded_rules_expr, collect_string_cases_with_guarded_rules,
    replace_string_value_with_guarded_rules, replace_string_values_with_guarded_rules_expr,
};
use rules::{
    collect_string_cases_values_with_rule_evaluation_expr,
    collect_string_cases_with_rule_evaluation, replace_string_value_with_rules,
    replace_string_values_with_rules_expr,
};
use rum_core::SharedArgs;

mod guarded;
mod rules;

pub(crate) fn eval_string_contains_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_string_pattern_predicate(session, head, args, "StringContainsQ", string_contains_q)
}

pub(crate) fn eval_string_starts_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_string_pattern_predicate(session, head, args, "StringStartsQ", string_starts_q)
}

pub(crate) fn eval_string_ends_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_string_pattern_predicate(session, head, args, "StringEndsQ", string_ends_q)
}

pub(crate) fn eval_string_position(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(2..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringPosition", args, 2, 4));
    }

    let Some(pattern) = parse_string_pattern(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let option_exprs =
        session.merged_option_exprs_for_system_symbol("StringPosition", &args[2..])?;
    let Some((max_results, config)) = parse_string_position_args(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(string_position_expr(value, &pattern, max_results, config));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| string_position_expr(&value, &pattern, max_results, config))
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_split(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringSplit", args, 1, 4));
    }

    let Some((delimiter_spec, limit, option_exprs)) = parse_string_split_args(args) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs =
        session.merged_option_exprs_for_system_symbol("StringSplit", option_exprs)?;
    let Some(config) = parse_string_match_config(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(string_list_expr(split_string_value(
            value,
            &delimiter_spec,
            limit,
            config,
        )));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    string_list_expr(split_string_value(&value, &delimiter_spec, limit, config))
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn parse_string_split_args(
    args: &[Expr],
) -> Option<(StringBoundarySpec, StringSplitLimit, &[Expr])> {
    let mut index = 1usize;

    let delimiter_spec = match args.get(index) {
        Some(expr) if is_string_split_option_expr(expr) => StringBoundarySpec::Whitespace,
        Some(expr) => {
            index += 1;
            parse_string_boundary_spec(expr)?
        }
        None => StringBoundarySpec::Whitespace,
    };

    let limit = match args.get(index) {
        Some(expr) if is_string_split_option_expr(expr) => StringSplitLimit::Unlimited,
        Some(expr) => {
            index += 1;
            parse_string_split_limit(expr)?
        }
        None => StringSplitLimit::Unlimited,
    };

    Some((delimiter_spec, limit, &args[index..]))
}

fn is_string_split_option_expr(expr: &Expr) -> bool {
    let Expr::Call { head, args, .. } = expr else {
        return false;
    };

    (is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        && args.len() == 2
}

pub(crate) fn eval_string_trim(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringTrim", args, 1, 2));
    }

    let trim_spec = match args.get(1) {
        Some(expr) => parse_string_boundary_spec(expr),
        None => Some(StringBoundarySpec::Whitespace),
    };
    let Some(trim_spec) = trim_spec else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(Expr::string(trim_string_value(value, &trim_spec)));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(string_list_expr(
            values
                .into_iter()
                .map(|value| trim_string_value(&value, &trim_spec))
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_replace(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() == 1 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    if !(2..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringReplace", args, 2, 4));
    }

    let Some(rules) = parse_exact_string_replacement_rules(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs =
        session.merged_option_exprs_for_system_symbol("StringReplace", &args[2..])?;
    let Some((max_replacements, config)) = parse_string_match_limit_and_options(&option_exprs)
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        let result = match &rules {
            ParsedExactStringReplacementRules::Unguarded(rules) => {
                replace_string_value_with_rules(session, value, rules, max_replacements, config)
            }
            ParsedExactStringReplacementRules::Guarded(rules) => {
                replace_string_value_with_guarded_rules(
                    session,
                    value,
                    rules,
                    max_replacements,
                    config,
                )
            }
        };
        return rum_runtime::resume_or_suspend!(
            session,
            result,
            ok(rendered) => { (|| {
                    Ok(Expr::string(rendered))
                })() },
            suspend => EvalStringReplaceSuspensionFrame,
        );
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return match &rules {
            ParsedExactStringReplacementRules::Unguarded(rules) => {
                replace_string_values_with_rules_expr(
                    session,
                    values,
                    rules,
                    max_replacements,
                    config,
                )
            }
            ParsedExactStringReplacementRules::Guarded(rules) => {
                replace_string_values_with_guarded_rules_expr(
                    session,
                    values,
                    rules,
                    max_replacements,
                    config,
                )
            }
        };
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_cases(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() == 1 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    if !(2..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("StringCases", args, 2, 4));
    }

    let Some(spec) = parse_exact_string_case_spec(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol("StringCases", &args[2..])?;
    let Some((max_matches, config)) = parse_string_cases_limit_and_options(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return match &spec {
            ParsedExactStringCaseSpec::Unguarded(spec) => {
                if matches!(spec, ExactStringCaseSpec::Rules(_)) {
                    let result = collect_string_cases_with_rule_evaluation(
                        session,
                        value,
                        spec,
                        max_matches,
                        config,
                    );
                    rum_runtime::resume_or_suspend!(
                        session,
                        result,
                        ok(items) => { (|| {
                                            Ok(list_expr(items))
                                        })() },
                        suspend => EvalStringCasesSuspensionFrame1,
                    )
                } else {
                    Ok(list_expr(collect_string_cases(
                        value,
                        spec,
                        max_matches,
                        config,
                    )))
                }
            }
            ParsedExactStringCaseSpec::Guarded(rules) => {
                let result = collect_string_cases_with_guarded_rules(
                    session,
                    value,
                    rules,
                    max_matches,
                    config,
                );
                rum_runtime::resume_or_suspend!(
                    session,
                    result,
                    ok(items) => { (|| {
                                    Ok(list_expr(items))
                                })() },
                    suspend => EvalStringCasesSuspensionFrame2,
                )
            }
        };
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return match &spec {
            ParsedExactStringCaseSpec::Unguarded(spec) => {
                if matches!(spec, ExactStringCaseSpec::Rules(_)) {
                    collect_string_cases_values_with_rule_evaluation_expr(
                        session,
                        values,
                        spec,
                        max_matches,
                        config,
                    )
                } else {
                    let mut outputs = Vec::with_capacity(values.len());
                    for value in values {
                        let items = collect_string_cases(&value, spec, max_matches, config);
                        outputs.push(list_expr(items));
                    }
                    Ok(list_expr(outputs))
                }
            }
            ParsedExactStringCaseSpec::Guarded(rules) => {
                collect_string_cases_values_with_guarded_rules_expr(
                    session,
                    values,
                    rules,
                    max_matches,
                    config,
                )
            }
        };
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_count(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() < 2 {
        return Ok(session.invalid_call_with_arg_range("StringCount", args, 2, 4));
    }

    let Some(patterns) = parse_string_patterns(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol("StringCount", &args[2..])?;
    let Some(config) = parse_string_count_options(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(Expr::integer(BigInt::from(
            string_count_matches(value, &patterns, config) as u64,
        )));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    Expr::integer(BigInt::from(
                        string_count_matches(&value, &patterns, config) as u64,
                    ))
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_delete(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() == 1 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    if args.len() < 2 {
        return Ok(session.invalid_call_with_arg_range("StringDelete", args, 1, 3));
    }

    let Some(patterns) = parse_string_patterns(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol("StringDelete", &args[2..])?;
    let Some(config) = parse_string_delete_options(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        return Ok(Expr::string(string_delete_matches(
            value, &patterns, config,
        )));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(string_list_expr(
            values
                .into_iter()
                .map(|value| string_delete_matches(&value, &patterns, config))
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_string_apply(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator] => Ok(string_unary_operator_form("StringApply", operator)),
        [operator, input] => {
            if let Some(value) = expr_as_string(input) {
                return string_apply_value(session, head, operator, value);
            }

            if let Some(values) = expr_as_string_list(input) {
                return resume_string_apply_values(session, head, operator, values);
            }

            Ok(Expr::call(head.clone(), args.to_vec()))
        }
        _ => Ok(session.invalid_call_with_arg_range("StringApply", args, 1, 2)),
    }
}

pub(crate) fn eval_string_match_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_string_patterns_predicate(session, head, args, "StringMatchQ", string_match_q)
}

pub(crate) fn eval_string_free_q(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    eval_string_patterns_predicate(session, head, args, "StringFreeQ", string_free_q)
}

fn eval_string_pattern_predicate(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    predicate: fn(&[char], &StringPattern, StringPatternMatchConfig) -> bool,
) -> Result<Expr, EvalError> {
    if args.len() == 1 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    if !(2..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range(symbol, args, 2, 3));
    }

    let Some(pattern) = parse_string_pattern(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol(symbol, &args[2..])?;
    let Some(config) = parse_string_match_config(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        let chars = value.chars().collect::<Vec<_>>();
        return Ok(bool_expr(predicate(&chars, &pattern, config)));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    let chars = value.chars().collect::<Vec<_>>();
                    bool_expr(predicate(&chars, &pattern, config))
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn eval_string_patterns_predicate(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    predicate: fn(&[char], &[StringPattern], StringPatternMatchConfig) -> bool,
) -> Result<Expr, EvalError> {
    if args.len() == 1 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    if !(2..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range(symbol, args, 2, 3));
    }

    let Some(patterns) = parse_string_patterns(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let option_exprs = session.merged_option_exprs_for_system_symbol(symbol, &args[2..])?;
    let Some(config) = parse_string_match_config(&option_exprs) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        let chars = value.chars().collect::<Vec<_>>();
        return Ok(bool_expr(predicate(&chars, &patterns, config)));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    let chars = value.chars().collect::<Vec<_>>();
                    bool_expr(predicate(&chars, &patterns, config))
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}
