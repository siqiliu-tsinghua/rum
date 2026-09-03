use rum_core::SharedArgs;
use rum_core::{Atom, BigInt, Expr};
use rum_runtime::kernel_api::{
    EvalError, Session, ThreadArgSpec, ThreadingDispatch, expr_as_exact_integer,
    is_system_symbol_name, system_expr,
};

pub(crate) fn eval_integer_length(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (value_expr, base_expr) = match args {
        [value] => (value, None),
        [value, base] => (value, Some(base)),
        _ => return Ok(session.invalid_call_with_arg_range("IntegerLength", args, 1, 2)),
    };

    let Some(value) = expr_as_exact_integer(value_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let base = match base_expr {
        Some(base_expr) => {
            match parse_integer_digits_base(session, "IntegerLength", args, base_expr) {
                Ok(Some(base)) => base,
                Ok(None) => {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
                Err(invalid) => return Ok(invalid),
            }
        }
        None => BigInt::from(10_u8),
    };

    Ok(Expr::integer(BigInt::from(integer_digit_length_abs(
        &value, &base,
    ))))
}

pub(crate) fn eval_integer_digits(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (value_expr, base_expr, length_expr) = match args {
        [value] => (value, None, None),
        [value, base] => (value, Some(base), None),
        [value, base, length] => (value, Some(base), Some(length)),
        _ => return Ok(session.invalid_call_with_arg_range("IntegerDigits", args, 1, 3)),
    };

    let Some(value) = expr_as_exact_integer(value_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let base = match base_expr {
        Some(base_expr) => {
            match parse_integer_digits_base(session, "IntegerDigits", args, base_expr) {
                Ok(Some(base)) => base,
                Ok(None) => {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
                Err(invalid) => return Ok(invalid),
            }
        }
        None => BigInt::from(10_u8),
    };
    let length = match length_expr {
        Some(length_expr) => {
            let length = match parse_integer_digits_length(session, args, length_expr) {
                Ok(Some(length)) => length,
                Ok(None) => return Ok(Expr::call(head.clone(), args.to_vec())),
                Err(invalid) => return Ok(invalid),
            };
            Some(length)
        }
        None => None,
    };

    let mut digits = integer_digits_abs(&value, &base);
    if let Some(length) = length {
        session.enforce_generated_element_limit_for_count(length)?;
        if digits.len() > length {
            digits = digits[digits.len() - length..].to_vec();
        } else if digits.len() < length {
            let mut padded = vec![BigInt::from(0_u8); length - digits.len()];
            padded.extend(digits);
            digits = padded;
        }
    } else {
        session.enforce_generated_element_limit_for_count(digits.len())?;
    }

    Ok(list_expr(digits.into_iter().map(Expr::integer).collect()))
}

pub(crate) fn eval_integer_reverse(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if let Some(threaded) = eval_threaded_builtin_args(session, head, args, ThreadArgSpec::All)? {
        return Ok(threaded);
    }

    let (value_expr, base_expr) = match args {
        [value] => (value, None),
        [value, base] => (value, Some(base)),
        _ => return Ok(session.invalid_call_with_arg_range("IntegerReverse", args, 1, 2)),
    };

    let Some(value) = expr_as_exact_integer(value_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let base = match base_expr {
        Some(base_expr) => {
            match parse_integer_digits_base(session, "IntegerReverse", args, base_expr) {
                Ok(Some(base)) => base,
                Ok(None) => {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
                Err(invalid) => return Ok(invalid),
            }
        }
        None => BigInt::from(10_u8),
    };

    let mut result = BigInt::from(0_u8);
    for digit in integer_digits_abs(&value, &base).into_iter().rev() {
        result = result * &base + digit;
    }

    Ok(Expr::integer(result))
}

pub(crate) fn eval_from_digits(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (digits_expr, base_expr) = match args {
        [digits] => (digits, None),
        [digits, base] => (digits, Some(base)),
        _ => return Ok(session.invalid_call_with_arg_range("FromDigits", args, 1, 2)),
    };

    let base = match base_expr {
        Some(base_expr) => {
            let Some(base) = expr_as_from_digits_base(base_expr) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            base
        }
        None => BigInt::from(10_u8),
    };

    if let Some(digits) = expr_as_integer_digit_list(digits_expr) {
        return Ok(Expr::integer(from_integer_digits(&digits, &base)));
    }

    if let Expr::Atom(Atom::String(value)) = digits_expr {
        if let Some(value) = from_digit_string(value, &base) {
            return Ok(Expr::integer(value));
        }
    }

    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_digit_count(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if let Some(threaded) =
        eval_threaded_builtin_args(session, head, args, ThreadArgSpec::Single(1))?
    {
        return Ok(threaded);
    }

    let (value_expr, base_expr, digit_expr) = match args {
        [value] => (value, None, None),
        [value, base] => (value, Some(base), None),
        [value, base, digit] => (value, Some(base), Some(digit)),
        _ => return Ok(session.invalid_call_with_arg_range("DigitCount", args, 1, 3)),
    };

    let Some(value) = expr_as_exact_integer(value_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let base = match base_expr {
        Some(base_expr) => {
            match parse_integer_digits_base(session, "DigitCount", args, base_expr) {
                Ok(Some(base)) => base,
                Ok(None) => {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
                Err(invalid) => return Ok(invalid),
            }
        }
        None => BigInt::from(10_u8),
    };
    let digits = integer_digits_abs(&value, &base);

    if let Some(digit_expr) = digit_expr {
        if let Some(query_digits) = expr_as_digit_count_query_list(digit_expr, &base) {
            session.enforce_generated_element_limit_for_count(query_digits.len())?;
            return Ok(list_expr(
                query_digits
                    .iter()
                    .map(|digit| Expr::integer(count_digit(&digits, digit)))
                    .collect(),
            ));
        }

        let Some(query_digit) = expr_as_digit_count_query(digit_expr, &base) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        return Ok(Expr::integer(count_digit(&digits, &query_digit)));
    }

    let Some(base_len) = positive_bigint_to_usize(&base) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    session.enforce_generated_element_limit_for_count(base_len)?;

    Ok(digit_count_vector_expr(&digits, base_len)
        .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
}

fn eval_threaded_builtin_args(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    arg_spec: ThreadArgSpec,
) -> Result<Option<Expr>, EvalError> {
    match session.thread_call_args(
        head,
        args,
        &system_expr("List"),
        &system_expr("List"),
        &arg_spec,
    )? {
        ThreadingDispatch::NotApplicable => Ok(None),
        ThreadingDispatch::Threaded(threaded) => session.evaluate(threaded).map(Some),
        ThreadingDispatch::Mismatch => Ok(Some(Expr::call(head.clone(), args.to_vec()))),
    }
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn parse_integer_digits_base(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    expr: &Expr,
) -> Result<Option<BigInt>, Expr> {
    let Some(base) = expr_as_exact_integer(expr) else {
        return Ok(None);
    };

    if base < BigInt::from(2_u8) {
        let call = Expr::call(system_expr(symbol), args.to_vec());
        return Err(session.invalid_call_with_message(
            symbol,
            args,
            "ibase",
            format!("Base {base} is not an integer greater than 1 in {call}."),
        ));
    }

    Ok(Some(base))
}

fn parse_integer_digits_length(
    session: &mut Session,
    args: &[Expr],
    expr: &Expr,
) -> Result<Option<usize>, Expr> {
    if let Some(length) = expr_as_exact_integer(expr) {
        if length < BigInt::from(0_u8) {
            let call = Expr::call(system_expr("IntegerDigits"), args.to_vec());
            return Err(session.invalid_call_with_message(
                "IntegerDigits",
                args,
                "intnn",
                format!("Length {length} is not a non-negative integer in {call}."),
            ));
        }
    }

    Ok(expr_as_non_negative_machine_integer(expr))
}

fn expr_as_from_digits_base(expr: &Expr) -> Option<BigInt> {
    let base = expr_as_exact_integer(expr)?;
    (base > BigInt::from(0_u8)).then_some(base)
}

fn expr_as_non_negative_machine_integer(expr: &Expr) -> Option<usize> {
    let value = expr_as_exact_integer(expr)?;
    if value < BigInt::from(0_u8) {
        return None;
    }
    value.to_string().parse::<usize>().ok()
}

fn positive_bigint_to_usize(value: &BigInt) -> Option<usize> {
    if value <= &BigInt::from(0_u8) {
        return None;
    }
    value.to_string().parse::<usize>().ok()
}

fn integer_digit_length_abs(value: &BigInt, base: &BigInt) -> usize {
    let zero = BigInt::from(0_u8);
    if value == &zero {
        return 0;
    }

    let mut remaining = bigint_abs(value);
    let mut length = 0_usize;
    while remaining > zero {
        remaining /= base;
        length += 1;
    }
    length
}

fn integer_digits_abs(value: &BigInt, base: &BigInt) -> Vec<BigInt> {
    let zero = BigInt::from(0_u8);
    if value == &zero {
        return vec![zero];
    }

    let mut remaining = bigint_abs(value);
    let mut digits = Vec::new();
    while remaining > zero {
        digits.push(&remaining % base);
        remaining /= base;
    }
    digits.reverse();
    digits
}

fn from_integer_digits(digits: &[BigInt], base: &BigInt) -> BigInt {
    let mut total = BigInt::from(0_u8);
    for digit in digits {
        total = total * base + digit;
    }
    total
}

fn expr_as_integer_digit_list(expr: &Expr) -> Option<Vec<BigInt>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(expr_as_exact_integer).collect()
}

fn from_digit_string(value: &str, base: &BigInt) -> Option<BigInt> {
    let base_usize = positive_bigint_to_usize(base)?;
    if base_usize > 36 {
        return None;
    }

    let mut total = BigInt::from(0_u8);
    for ch in value.chars() {
        let digit = ascii_digit_value(ch)?;
        if digit >= base_usize {
            return None;
        }
        total = total * base + BigInt::from(digit);
    }
    Some(total)
}

fn ascii_digit_value(ch: char) -> Option<usize> {
    match ch {
        '0'..='9' => Some(ch as usize - '0' as usize),
        'a'..='z' => Some(ch as usize - 'a' as usize + 10),
        'A'..='Z' => Some(ch as usize - 'A' as usize + 10),
        _ => None,
    }
}

fn expr_as_digit_count_query(expr: &Expr, base: &BigInt) -> Option<BigInt> {
    let digit = expr_as_exact_integer(expr)?;
    if digit < BigInt::from(0_u8) || digit >= *base {
        return None;
    }
    Some(digit)
}

fn expr_as_digit_count_query_list(expr: &Expr, base: &BigInt) -> Option<Vec<BigInt>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter()
        .map(|arg| expr_as_digit_count_query(arg, base))
        .collect()
}

fn count_digit(digits: &[BigInt], query_digit: &BigInt) -> BigInt {
    BigInt::from(digits.iter().filter(|digit| *digit == query_digit).count())
}

fn digit_count_vector_expr(digits: &[BigInt], base_len: usize) -> Option<Expr> {
    let mut counts = vec![BigInt::from(0_u8); base_len];
    for digit in digits {
        let index = digit_count_vector_index(digit, base_len)?;
        counts[index] += BigInt::from(1_u8);
    }

    Some(list_expr(counts.into_iter().map(Expr::integer).collect()))
}

fn digit_count_vector_index(digit: &BigInt, base_len: usize) -> Option<usize> {
    if digit == &BigInt::from(0_u8) {
        return Some(base_len.checked_sub(1)?);
    }

    let one_based = digit.to_string().parse::<usize>().ok()?;
    one_based.checked_sub(1).filter(|index| *index < base_len)
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}
