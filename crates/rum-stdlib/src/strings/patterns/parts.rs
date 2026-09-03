use super::*;
use rum_core::SharedArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StringTakeDropSpec {
    Count(isize),
    InclusiveRange(isize, isize),
}

pub(crate) fn eval_string_take(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("StringTake", args, 2));
    }

    let Some(value) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(spec) = parse_string_take_drop_spec(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let chars = value.chars().collect::<Vec<_>>();
    let taken = match spec {
        StringTakeDropSpec::Count(count) => take_chars_by_count(&chars, count),
        StringTakeDropSpec::InclusiveRange(start, end) => {
            let Some((start, end)) = normalize_clamped_inclusive_range(start, end, chars.len())
            else {
                return Ok(Expr::string(String::new()));
            };
            chars[start..=end].iter().collect::<String>()
        }
    };

    Ok(Expr::string(taken))
}

pub(crate) fn eval_string_drop(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("StringDrop", args, 2));
    }

    let Some(value) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(spec) = parse_string_take_drop_spec(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let chars = value.chars().collect::<Vec<_>>();
    let dropped = match spec {
        StringTakeDropSpec::Count(count) => drop_chars_by_count(&chars, count),
        StringTakeDropSpec::InclusiveRange(start, end) => {
            let Some((start, end)) = normalize_clamped_inclusive_range(start, end, chars.len())
            else {
                return Ok(Expr::string(value.to_string()));
            };

            let mut output = String::new();
            output.extend(chars[..start].iter());
            output.extend(chars[end + 1..].iter());
            output
        }
    };

    Ok(Expr::string(dropped))
}

pub(crate) fn eval_string_part(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("StringPart", args, 2));
    }

    let Some(value) = expr_as_string(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let chars = value.chars().collect::<Vec<_>>();
    let Some(specs) = parse_string_part_spec(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut output = String::new();
    for spec in specs {
        let Some(index) = normalize_exact_position(spec, chars.len()) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        output.push(chars[index]);
    }

    Ok(Expr::string(output))
}

pub(crate) fn eval_string_take_drop(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("StringTakeDrop", args, 2));
    }

    let Some(spec) = parse_string_take_drop_spec(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(value) = expr_as_string(&args[0]) {
        let (taken, dropped) = string_take_drop_value(value, spec);
        return Ok(list_expr(vec![Expr::string(taken), Expr::string(dropped)]));
    }

    if let Some(values) = expr_as_string_list(&args[0]) {
        return Ok(list_expr(
            values
                .into_iter()
                .map(|value| {
                    let (taken, dropped) = string_take_drop_value(&value, spec);
                    list_expr(vec![Expr::string(taken), Expr::string(dropped)])
                })
                .collect(),
        ));
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(super) fn parse_string_take_drop_spec(expr: &Expr) -> Option<StringTakeDropSpec> {
    if let Some(count) = expr_as_signed_isize(expr) {
        return Some(StringTakeDropSpec::Count(count));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") || args.len() != 2 {
        return None;
    }

    Some(StringTakeDropSpec::InclusiveRange(
        expr_as_signed_isize(&args[0])?,
        expr_as_signed_isize(&args[1])?,
    ))
}

pub(super) fn parse_string_part_spec(expr: &Expr) -> Option<Vec<isize>> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(vec![index]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(expr_as_signed_isize).collect()
}

fn normalize_exact_position(position: isize, len: usize) -> Option<usize> {
    if position == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position - 1
    } else {
        len + position
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}

fn normalize_clamped_endpoint(position: isize, len: usize) -> Option<usize> {
    if position == 0 || len == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position
    } else {
        len + position + 1
    };

    let clamped = normalized.clamp(1, len);
    Some((clamped - 1) as usize)
}

fn normalize_clamped_inclusive_range(
    start: isize,
    end: isize,
    len: usize,
) -> Option<(usize, usize)> {
    let start = normalize_clamped_endpoint(start, len)?;
    let end = normalize_clamped_endpoint(end, len)?;
    (start <= end).then_some((start, end))
}

fn take_chars_by_count(chars: &[char], count: isize) -> String {
    if count == 0 {
        return String::new();
    }

    let len = chars.len();
    let magnitude = count.unsigned_abs().min(len);

    if count > 0 {
        chars[..magnitude].iter().collect()
    } else {
        chars[len - magnitude..].iter().collect()
    }
}

fn drop_chars_by_count(chars: &[char], count: isize) -> String {
    if count == 0 {
        return chars.iter().collect();
    }

    let len = chars.len();
    let magnitude = count.unsigned_abs().min(len);

    if count > 0 {
        chars[magnitude..].iter().collect()
    } else {
        chars[..len - magnitude].iter().collect()
    }
}

fn string_take_drop_value(value: &str, spec: StringTakeDropSpec) -> (String, String) {
    let chars = value.chars().collect::<Vec<_>>();
    match spec {
        StringTakeDropSpec::Count(count) => (
            take_chars_by_count(&chars, count),
            drop_chars_by_count(&chars, count),
        ),
        StringTakeDropSpec::InclusiveRange(start, end) => {
            let Some((start, end)) = normalize_clamped_inclusive_range(start, end, chars.len())
            else {
                return (String::new(), value.to_string());
            };

            let taken = chars[start..=end].iter().collect::<String>();
            let mut dropped = String::new();
            dropped.extend(chars[..start].iter());
            dropped.extend(chars[end + 1..].iter());
            (taken, dropped)
        }
    }
}
