use super::*;
use rum_core::SharedArgs;

mod distance;
mod lexical;
mod nearest;
mod selection;

use self::distance::*;
use self::lexical::*;
use self::nearest::*;
use self::selection::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NearestResultSpec {
    BestTies,
    Count(usize),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NearestSelectionSpec {
    Results(NearestResultSpec),
    ResultsWithin {
        results: NearestResultSpec,
        max_distance: Expr,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NearestDistanceFunction {
    Automatic,
    EditDistance,
    HammingDistance,
}

pub(crate) fn eval_nearest(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if let [data] = args {
        return Ok(Expr::call(
            system_expr("Function"),
            vec![Expr::call(
                system_expr("Nearest"),
                vec![data.clone(), string_slot_expr()],
            )],
        ));
    }

    if let [data, option_expr] = args {
        if parse_nearest_options(std::slice::from_ref(option_expr)).is_some() {
            return Ok(Expr::call(
                system_expr("Function"),
                vec![Expr::call(
                    system_expr("Nearest"),
                    vec![data.clone(), string_slot_expr(), option_expr.clone()],
                )],
            ));
        }
    }

    if !(2..=4).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("Nearest", args, 1, 4));
    }

    let (result_spec, distance_function) = match args {
        [_, _] => (
            NearestSelectionSpec::Results(NearestResultSpec::BestTies),
            NearestDistanceFunction::Automatic,
        ),
        [_, _, third] => {
            if let Some(spec) = parse_nearest_selection_spec(third) {
                (spec, NearestDistanceFunction::Automatic)
            } else if let Some(options) = parse_nearest_options(std::slice::from_ref(third)) {
                (
                    NearestSelectionSpec::Results(NearestResultSpec::BestTies),
                    options.distance_function,
                )
            } else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
        }
        [_, _, third, fourth] => {
            let Some(spec) = parse_nearest_selection_spec(third) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            let Some(options) = parse_nearest_options(std::slice::from_ref(fourth)) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            (spec, options.distance_function)
        }
        _ => unreachable!(),
    };

    if let Some(target) = expr_as_string(&args[1]) {
        if let Some(value) = expr_as_string(&args[0]) {
            let Some(results) =
                nearest_strings(&[value.to_string()], target, result_spec, distance_function)
            else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            return Ok(string_list_expr(results));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            let Some(results) = nearest_strings(&values, target, result_spec, distance_function)
            else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            return Ok(string_list_expr(results));
        }

        if let Some((keys, values)) = parse_string_rule_dataset(&args[0]) {
            let Some(results) =
                nearest_values_for_strings(&keys, &values, target, result_spec, distance_function)
            else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            return Ok(list_expr(results));
        }
    }

    if let Some(target) = expr_as_exact_rational(&args[1]) {
        if distance_function != NearestDistanceFunction::Automatic {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        if let Some(value) = expr_as_exact_rational(&args[0]) {
            return Ok(list_expr(nearest_values_for_numbers(
                &[value],
                std::slice::from_ref(&args[0]),
                &target,
                result_spec,
            )));
        }
        if let Some((keys, values)) = parse_numeric_dataset(&args[0]) {
            return Ok(list_expr(nearest_values_for_numbers(
                &keys,
                &values,
                &target,
                result_spec,
            )));
        }
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_pluralize(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [word_expr] => {
            let Some(word) = expr_as_string(word_expr) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            Ok(Expr::string(pluralize_english_phrase(word)))
        }
        [word_expr, count_expr] => {
            let Some(word) = expr_as_string(word_expr) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            let Some(count) = expr_as_exact_rational(count_expr) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            if count == Rational::one() {
                Ok(Expr::string(word.to_string()))
            } else {
                Ok(Expr::string(pluralize_english_phrase(word)))
            }
        }
        _ => Ok(session.invalid_call_with_arg_range("Pluralize", args, 1, 2)),
    }
}

pub(crate) fn eval_alphabet(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let supported = match args {
        [] => true,
        [arg] => expr_as_string(arg).is_some_and(|name| name == "English"),
        _ => false,
    };

    if !supported {
        return Ok(if args.len() <= 1 {
            Expr::call(head.clone(), args.to_vec())
        } else {
            session.invalid_call_with_arg_range("Alphabet", args, 0, 1)
        });
    }

    Ok(string_list_expr(english_alphabet()))
}

pub(crate) fn eval_alphabetic_sort(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("AlphabeticSort", args, 1, 2));
    }

    if let Some(alphabet_name) = args.get(1).and_then(expr_as_string) {
        if alphabet_name != "English" {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
    } else if args.len() == 2 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let Some(values) = expr_as_string_list(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut values = values;
    values.sort_by_key(|value| alphabetic_sort_key(value));
    Ok(string_list_expr(values))
}

pub(crate) fn eval_edit_distance(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("EditDistance", args, 2));
    }

    if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
        let lhs = lhs.chars().collect::<Vec<_>>();
        let rhs = rhs.chars().collect::<Vec<_>>();
        return Ok(Expr::integer(BigInt::from(
            levenshtein_distance(&lhs, &rhs) as u64,
        )));
    }

    if let (Some(lhs), Some(rhs)) = (
        call_items_with_head(&args[0], &system_expr("List")),
        call_items_with_head(&args[1], &system_expr("List")),
    ) {
        return Ok(Expr::integer(BigInt::from(
            levenshtein_distance(lhs, rhs) as u64
        )));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_hamming_distance(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("HammingDistance", args, 2));
    }

    if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
        let lhs = lhs.chars().collect::<Vec<_>>();
        let rhs = rhs.chars().collect::<Vec<_>>();
        let Some(distance) = hamming_distance(&lhs, &rhs) else {
            return Ok(invalid_hamming_distance_length(session, head, args));
        };
        return Ok(Expr::integer(BigInt::from(distance as u64)));
    }

    if let (Some(lhs), Some(rhs)) = (
        call_items_with_head(&args[0], &system_expr("List")),
        call_items_with_head(&args[1], &system_expr("List")),
    ) {
        let Some(distance) = hamming_distance(lhs, rhs) else {
            return Ok(invalid_hamming_distance_length(session, head, args));
        };
        return Ok(Expr::integer(BigInt::from(distance as u64)));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn invalid_hamming_distance_length(session: &mut Session, head: &Expr, args: &[Expr]) -> Expr {
    let call = Expr::call(head.clone(), args.to_vec());
    let message = format!("HammingDistance expects inputs with the same length in {call}.");
    if !session
        .messages()
        .last()
        .map(|last| last.symbol == "HammingDistance" && last.tag == "len" && last.text == message)
        .unwrap_or(false)
    {
        session.push_message("HammingDistance", "len", message);
    }
    call
}

pub(crate) fn eval_longest_common_subsequence(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("LongestCommonSubsequence", args, 2));
    }

    if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
        let lhs = lhs.chars().collect::<Vec<_>>();
        let rhs = rhs.chars().collect::<Vec<_>>();
        let common = longest_common_contiguous_slice(&lhs, &rhs);
        return Ok(Expr::string(common.into_iter().collect::<String>()));
    }

    if let (Some(lhs), Some(rhs)) = (
        call_items_with_head(&args[0], &system_expr("List")),
        call_items_with_head(&args[1], &system_expr("List")),
    ) {
        return Ok(list_expr(longest_common_contiguous_slice(lhs, rhs)));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_sequence_alignment(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("SequenceAlignment", args, 2));
    }

    if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
        let lhs = lhs.chars().collect::<Vec<_>>();
        let rhs = rhs.chars().collect::<Vec<_>>();
        let alignment = global_alignment_blocks(&lhs, &rhs);
        return Ok(render_string_alignment_blocks(&alignment));
    }

    if let (Some(lhs), Some(rhs)) = (
        call_items_with_head(&args[0], &system_expr("List")),
        call_items_with_head(&args[1], &system_expr("List")),
    ) {
        let alignment = global_alignment_blocks(lhs, rhs);
        return Ok(render_expr_alignment_blocks(&alignment));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_diff(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("Diff", args, 2));
    }

    if let (Some(lhs), Some(rhs)) = (expr_as_string(&args[0]), expr_as_string(&args[1])) {
        enforce_string_diff_memory_constraint(session, lhs, rhs)?;
        let lhs = lhs.chars().collect::<Vec<_>>();
        let rhs = rhs.chars().collect::<Vec<_>>();
        let alignment = global_alignment_blocks(&lhs, &rhs);
        return Ok(render_diff_object_from_chars(&alignment));
    }

    if let (Some(lhs), Some(rhs)) = (
        call_items_with_head(&args[0], &system_expr("List")),
        call_items_with_head(&args[1], &system_expr("List")),
    ) {
        enforce_expr_diff_memory_constraint(session, lhs, rhs)?;
        let alignment = global_alignment_blocks(lhs, rhs);
        return Ok(render_diff_object_from_exprs(&alignment));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn enforce_string_diff_memory_constraint(
    session: &Session,
    lhs: &str,
    rhs: &str,
) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let lhs_len = lhs.chars().count();
    let rhs_len = rhs.chars().count();
    let payload_bytes = lhs.len().saturating_add(rhs.len());
    session.enforce_active_memory_limit_for_projected_bytes(diff_projected_bytes(
        lhs_len,
        rhs_len,
        payload_bytes,
        std::mem::size_of::<char>(),
        std::mem::size_of::<(Option<char>, Option<char>)>(),
    ))
}

fn enforce_expr_diff_memory_constraint(
    session: &Session,
    lhs: &[Expr],
    rhs: &[Expr],
) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let payload_bytes = lhs.iter().chain(rhs).fold(0usize, |total, item| {
        total.saturating_add(projected_expr_memory_proxy_bytes(item))
    });
    session.enforce_active_memory_limit_for_projected_bytes(diff_projected_bytes(
        lhs.len(),
        rhs.len(),
        payload_bytes,
        std::mem::size_of::<Expr>(),
        std::mem::size_of::<(Option<Expr>, Option<Expr>)>(),
    ))
}

fn diff_projected_bytes(
    lhs_len: usize,
    rhs_len: usize,
    result_payload_bytes: usize,
    item_bytes: usize,
    aligned_pair_bytes: usize,
) -> usize {
    let rows = lhs_len.saturating_add(1);
    let columns = rhs_len.saturating_add(1);
    let cells = rows.saturating_mul(columns);
    let matrix_bytes = cells
        .saturating_mul(
            std::mem::size_of::<isize>().saturating_add(std::mem::size_of::<AlignmentStep>()),
        )
        .saturating_add(
            rows.saturating_mul(
                std::mem::size_of::<Vec<isize>>()
                    .saturating_add(std::mem::size_of::<Vec<AlignmentStep>>()),
            ),
        );
    let aligned_len = lhs_len.saturating_add(rhs_len);
    let alignment_bytes = aligned_len
        .saturating_mul(aligned_pair_bytes)
        .saturating_add(aligned_len.saturating_mul(item_bytes))
        .saturating_add(aligned_len.saturating_mul(std::mem::size_of::<AlignmentBlock<Expr>>()));
    let result_wrapper_bytes = aligned_len
        .saturating_mul(12)
        .saturating_add(17)
        .saturating_mul(std::mem::size_of::<Expr>());

    matrix_bytes
        .saturating_add(alignment_bytes)
        .saturating_add(result_wrapper_bytes)
        .saturating_add(result_payload_bytes.saturating_mul(2))
}

fn projected_expr_memory_proxy_bytes(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(Atom::Integer(rum_core::IntValue::Small(_))) => std::mem::size_of::<Expr>(),
        Expr::Atom(Atom::Integer(rum_core::IntValue::Big(value))) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<BigInt>())
            .saturating_add(projected_bigint_payload_bytes(value)),
        Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<Rational>())
            .saturating_add(projected_bigint_payload_bytes(value.numerator()))
            .saturating_add(projected_bigint_payload_bytes(value.denominator())),
        Expr::Atom(Atom::String(value)) => std::mem::size_of::<Expr>().saturating_add(value.len()),
        Expr::Atom(Atom::Symbol(symbol)) => std::mem::size_of::<Expr>()
            .saturating_add(symbol.context().len())
            .saturating_add(symbol.name().len()),
        Expr::Call { head, args } => args.iter().fold(
            std::mem::size_of::<Expr>().saturating_add(projected_expr_memory_proxy_bytes(head)),
            |total, arg| total.saturating_add(projected_expr_memory_proxy_bytes(arg)),
        ),
    }
}

fn projected_bigint_payload_bytes(value: &BigInt) -> usize {
    let bytes = u128::from(value.bits()).saturating_add(7) / 8;
    usize::try_from(bytes.saturating_add(1)).unwrap_or(usize::MAX)
}
