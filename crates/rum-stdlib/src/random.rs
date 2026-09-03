use rum_core::SharedArgs;
use rum_core::{BigInt, Expr};
use rum_runtime::kernel_api::{
    EntropyError, EvalError, Integer, Session, ToPrimitive, base64_encode, expr_as_exact_integer,
    expr_as_exact_rational, expr_as_string, is_system_symbol_name, list_items, system_expr,
};

mod password_policy;

const DEFAULT_RANDOM_ALPHABET: &str = crate::character_classes::ALPHANUMERIC;

pub(crate) fn eval_seed_random(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [seed] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("SeedRandom", args, 1));
    };
    let Some(seed) = expr_as_exact_integer(seed) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    session.seed_random_with_integer(&seed);
    Ok(system_expr("Null"))
}

pub(crate) fn eval_reseed_random(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !args.is_empty() {
        return Ok(session.invalid_call_with_exact_arg_count("ReseedRandom", args, 0));
    }
    match session.seed_random_from_entropy() {
        Ok(()) => Ok(system_expr("Null")),
        Err(error) => {
            push_entropy_error(session, "ReseedRandom", error.message());
            Ok(system_expr("$Failed"))
        }
    }
}

pub(crate) fn eval_random_integer(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [] => Ok(random_integer_between(
            session,
            &BigInt::from(0_u8),
            &BigInt::from(1_u8),
        )),
        [spec] => {
            let Some((min, max)) = random_integer_bounds(spec) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            Ok(random_integer_between(session, &min, &max))
        }
        [spec, count] => {
            let Some((min, max)) = random_integer_bounds(spec) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            if let Some(count) = parse_output_count(session, count)? {
                let values = (0..count)
                    .map(|_| random_integer_between(session, &min, &max))
                    .collect();
                Ok(list_expr(values))
            } else if let Some(dims) = parse_dims(session, count)? {
                Ok(build_random_array(session, &dims, &mut |s| {
                    random_integer_between(s, &min, &max)
                }))
            } else {
                Ok(Expr::call(head.clone(), args.to_vec()))
            }
        }
        _ => Ok(session.invalid_call_with_arg_range("RandomInteger", args, 0, 2)),
    }
}

pub(crate) fn eval_random_choice(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (list, count) = match args {
        [list] => (list, None),
        [list, count] => (list, Some(count)),
        _ => return Ok(session.invalid_call_with_arg_range("RandomChoice", args, 1, 2)),
    };
    if let Expr::Call {
        head: rule_head,
        args: rule_args,
    } = list
    {
        if is_system_symbol_name(rule_head, "Rule") && rule_args.len() == 2 {
            return eval_random_choice_weighted(
                session,
                head,
                args,
                &rule_args[0],
                &rule_args[1],
                count,
            );
        }
    }
    let Some(items) = list_items(list) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if items.is_empty() {
        session.push_message("RandomChoice", "empty", "expected a non-empty list");
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let Some(count) = count else {
        return Ok(items[random_usize_below(session, items.len())].clone());
    };
    if let Some(count) = parse_output_count(session, count)? {
        let choices = (0..count)
            .map(|_| items[random_usize_below(session, items.len())].clone())
            .collect();
        Ok(list_expr(choices))
    } else if let Some(dims) = parse_dims(session, count)? {
        let len = items.len();
        Ok(build_random_array(session, &dims, &mut |s| {
            items[random_usize_below(s, len)].clone()
        }))
    } else {
        Ok(Expr::call(head.clone(), args.to_vec()))
    }
}

fn eval_random_choice_weighted(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    weights_expr: &Expr,
    elements_expr: &Expr,
    count: Option<&Expr>,
) -> Result<Expr, EvalError> {
    let (Some(weight_items), Some(element_items)) =
        (list_items(weights_expr), list_items(elements_expr))
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if element_items.is_empty() || weight_items.len() != element_items.len() {
        session.push_message(
            "RandomChoice",
            "weights",
            "weight and element lists must have equal, nonzero length",
        );
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let mut weights = Vec::with_capacity(weight_items.len());
    for weight in weight_items {
        let Some(weight) = expr_as_exact_rational(weight) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if weight.numerator() < &BigInt::from(0_u8) {
            session.push_message("RandomChoice", "weights", "weights must be non-negative");
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        weights.push(weight);
    }

    let mut denom_lcm = BigInt::from(1_u8);
    for weight in &weights {
        denom_lcm = denom_lcm.lcm(weight.denominator());
    }
    let mut cum = Vec::with_capacity(weights.len());
    let mut total = BigInt::from(0_u8);
    for weight in &weights {
        let scaled = weight.numerator() * (&denom_lcm / weight.denominator());
        total += &scaled;
        cum.push(total.clone());
    }
    if total <= BigInt::from(0_u8) {
        session.push_message(
            "RandomChoice",
            "weights",
            "at least one weight must be positive",
        );
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let elements = element_items.to_vec();
    let mut draw = |s: &mut Session| -> Expr {
        let r = random_bigint_below(s, &total);
        let idx = cum.partition_point(|c| c <= &r);
        elements[idx].clone()
    };

    match count {
        None => Ok(draw(session)),
        Some(count) => {
            if let Some(count) = parse_output_count(session, count)? {
                let choices = (0..count).map(|_| draw(session)).collect();
                Ok(list_expr(choices))
            } else if let Some(dims) = parse_dims(session, count)? {
                Ok(build_random_array(session, &dims, &mut draw))
            } else {
                Ok(Expr::call(head.clone(), args.to_vec()))
            }
        }
    }
}

pub(crate) fn eval_random_sample(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (list, count) = match args {
        [list] => (list, None),
        [list, count] => (list, Some(count)),
        _ => return Ok(session.invalid_call_with_arg_range("RandomSample", args, 1, 2)),
    };
    let Some(items) = list_items(list) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let count = match count {
        Some(count) => {
            let Some(count) = parse_output_count(session, count)? else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            if count > items.len() {
                return Ok(session.invalid_call_with_message(
                    "RandomSample",
                    args,
                    "ksamp",
                    "sample count cannot exceed list length",
                ));
            }
            count
        }
        None => items.len(),
    };
    session.enforce_generated_element_limit_for_count(count)?;

    let mut sampled = items.to_vec();
    for index in 0..count {
        let swap_index = index + random_usize_below(session, sampled.len() - index);
        sampled.swap(index, swap_index);
    }
    sampled.truncate(count);
    Ok(list_expr(sampled))
}

pub(crate) fn eval_random_string(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("RandomString", args, 1, 2));
    }
    let Some((alphabet, count)) = parse_random_text_args(session, args)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut out = String::with_capacity(random_string_projected_bytes(&alphabet, count));
    for _ in 0..count {
        out.push(alphabet[random_usize_below(session, alphabet.len())]);
    }
    Ok(Expr::string(out))
}

pub(crate) fn eval_random_password(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() {
        return Ok(session.invalid_call_with_arg_form(
            "RandomPassword",
            args,
            "expected RandomPassword[n, opts...] or RandomPassword[alphabet, n, opts...]",
        ));
    }

    let no_option_call = args.len() == 1 || (args.len() == 2 && expr_as_string(&args[0]).is_some());
    if !no_option_call {
        return password_policy::eval_random_password_with_policy(session, head, args);
    }

    let Some((alphabet, count)) = parse_random_text_args(session, args)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut out = String::with_capacity(random_string_projected_bytes(&alphabet, count));
    for _ in 0..count {
        match entropy_usize_below(session, alphabet.len()) {
            Ok(index) => out.push(alphabet[index]),
            Err(error) => {
                push_entropy_error(session, "RandomPassword", error.message());
                return Ok(system_expr("$Failed"));
            }
        }
    }
    Ok(Expr::string(out))
}

pub(crate) fn eval_block_random(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [body] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("BlockRandom", args, 1));
    };

    session.evaluate_with_random_state_restored(body.clone())
}

pub(crate) fn eval_random_uuid(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !args.is_empty() {
        return Ok(session.invalid_call_with_exact_arg_count("RandomUuid", args, 0));
    }

    let mut bytes = [0_u8; 16];
    session.random_fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(Expr::string(uuid_string(&bytes)))
}

pub(crate) fn eval_uuid_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("UuidQ", args, 1));
    };

    Ok(bool_expr(expr_as_string(arg).is_some_and(uuid_q)))
}

pub(crate) fn eval_random_bytes(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [count] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("RandomBytes", args, 1));
    };
    let Some(count) = parse_byte_count(session, count)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut bytes = vec![0_u8; count];
    match session.fill_entropy(&mut bytes) {
        Ok(()) => Ok(byte_array_expr(&bytes)),
        Err(error) => {
            push_entropy_error(session, "RandomBytes", error.message());
            Ok(system_expr("$Failed"))
        }
    }
}

fn random_integer_bounds(spec: &Expr) -> Option<(BigInt, BigInt)> {
    if let Some(max) = expr_as_exact_integer(spec) {
        return (max >= BigInt::from(0_u8)).then_some((BigInt::from(0_u8), max));
    }

    let [min, max] = list_items(spec)? else {
        return None;
    };
    let min = expr_as_exact_integer(min)?;
    let max = expr_as_exact_integer(max)?;
    (min <= max).then_some((min, max))
}

fn random_integer_between(session: &mut Session, min: &BigInt, max: &BigInt) -> Expr {
    let width = max - min + BigInt::from(1_u8);
    Expr::integer(min + random_bigint_below(session, &width))
}

fn random_bigint_below(session: &mut Session, upper: &BigInt) -> BigInt {
    debug_assert!(upper > &BigInt::from(0_u8));
    if upper == &BigInt::from(1_u8) {
        return BigInt::from(0_u8);
    }

    let max = upper - BigInt::from(1_u8);
    let (_, max_bytes) = max.to_bytes_le();
    let bit_count = significant_bits(&max_bytes);
    let byte_count = bit_count.div_ceil(8);
    let high_mask = 0xff_u8 >> (byte_count * 8 - bit_count);

    loop {
        let mut bytes = vec![0_u8; byte_count];
        session.random_fill_bytes(&mut bytes);
        if let Some(last) = bytes.last_mut() {
            *last &= high_mask;
        }

        let value = bigint_from_unsigned_le_bytes(&bytes);
        if &value < upper {
            return value;
        }
    }
}

fn random_usize_below(session: &mut Session, upper: usize) -> usize {
    random_bigint_below(session, &BigInt::from(upper))
        .to_usize()
        .expect("usize upper bound produces usize random values")
}

fn entropy_usize_below(session: &Session, upper: usize) -> Result<usize, EntropyError> {
    debug_assert!(upper > 0);
    if upper == 1 {
        return Ok(0);
    }
    if upper <= 256 {
        return entropy_byte_below(session, upper);
    }

    let upper = BigInt::from(upper);
    let max = &upper - BigInt::from(1_u8);
    let (_, max_bytes) = max.to_bytes_le();
    let bit_count = significant_bits(&max_bytes);
    let byte_count = bit_count.div_ceil(8);
    let high_mask = 0xff_u8 >> (byte_count * 8 - bit_count);

    loop {
        let mut bytes = vec![0_u8; byte_count];
        session.fill_entropy(&mut bytes)?;
        if let Some(last) = bytes.last_mut() {
            *last &= high_mask;
        }

        let value = bigint_from_unsigned_le_bytes(&bytes);
        if value < upper {
            return Ok(value
                .to_usize()
                .expect("usize upper bound produces usize random values"));
        }
    }
}

fn entropy_byte_below(session: &Session, upper: usize) -> Result<usize, EntropyError> {
    let accept_bound = (256 / upper) * upper;
    loop {
        let mut byte = [0_u8; 1];
        session.fill_entropy(&mut byte)?;
        let value = usize::from(byte[0]);
        if value < accept_bound {
            return Ok(value % upper);
        }
    }
}

fn parse_random_text_args(
    session: &Session,
    args: &[Expr],
) -> Result<Option<(Vec<char>, usize)>, EvalError> {
    let (alphabet, count) = match args {
        [count] => (DEFAULT_RANDOM_ALPHABET, count),
        [alphabet, count] => {
            let Some(alphabet) = expr_as_string(alphabet) else {
                return Ok(None);
            };
            (alphabet, count)
        }
        _ => unreachable!("random text arity checked by caller"),
    };

    let alphabet = alphabet_chars(alphabet);
    let Some(alphabet) = alphabet else {
        return Ok(None);
    };
    let Some(count) = parse_output_count(session, count)? else {
        return Ok(None);
    };
    session.enforce_active_memory_limit_for_projected_bytes(random_string_projected_bytes(
        &alphabet, count,
    ))?;
    Ok(Some((alphabet, count)))
}

fn alphabet_chars(alphabet: &str) -> Option<Vec<char>> {
    let chars = alphabet.chars().collect::<Vec<_>>();
    (!chars.is_empty()).then_some(chars)
}

fn random_string_projected_bytes(alphabet: &[char], count: usize) -> usize {
    let max_char_bytes = alphabet.iter().map(|ch| ch.len_utf8()).max().unwrap_or(0);
    count.saturating_mul(max_char_bytes)
}

fn parse_output_count(session: &Session, expr: &Expr) -> Result<Option<usize>, EvalError> {
    let Some(count) = expr_as_exact_integer(expr) else {
        return Ok(None);
    };
    if count < BigInt::from(0_u8) {
        return Ok(None);
    }
    session.enforce_generated_element_limit_for_big_count(&count)?;
    Ok(Some(count.to_usize().expect(
        "generated element limit keeps count representable",
    )))
}

/// Parses a `{n1, n2, ...}` dimensions spec of non-negative exact integers.
fn parse_dims(session: &Session, expr: &Expr) -> Result<Option<Vec<usize>>, EvalError> {
    let Some(items) = list_items(expr) else {
        return Ok(None);
    };
    let mut product = BigInt::from(1_u8);
    let mut dims = Vec::with_capacity(items.len());
    for item in items {
        let Some(dim) = expr_as_exact_integer(item) else {
            return Ok(None);
        };
        if dim < BigInt::from(0_u8) {
            return Ok(None);
        }
        product *= &dim;
        // Bounds every prefix product, including huge leading dimensions
        // before a zero axis would make the total leaf count zero.
        session.enforce_generated_element_limit_for_big_count(&product)?;
        dims.push(dim.to_usize().unwrap_or(0));
    }
    Ok(Some(dims))
}

fn build_random_array(
    session: &mut Session,
    dims: &[usize],
    draw: &mut dyn FnMut(&mut Session) -> Expr,
) -> Expr {
    match dims.split_first() {
        None => draw(session),
        Some((&head, rest)) => {
            let items = (0..head)
                .map(|_| build_random_array(session, rest, draw))
                .collect();
            list_expr(items)
        }
    }
}

fn parse_byte_count(session: &Session, expr: &Expr) -> Result<Option<usize>, EvalError> {
    let Some(count) = expr_as_exact_integer(expr) else {
        return Ok(None);
    };
    if count < BigInt::from(0_u8) {
        return Ok(None);
    }
    session.enforce_generated_element_limit_for_big_count(&count)?;
    session.enforce_active_memory_limit_for_projected_bytes(
        count
            .to_usize()
            .expect("generated element limit keeps byte count representable"),
    )?;
    Ok(Some(count.to_usize().expect(
        "generated element limit keeps byte count representable",
    )))
}

fn significant_bits(bytes: &[u8]) -> usize {
    let Some(last) = bytes.last().copied() else {
        return 0;
    };
    (bytes.len() - 1) * 8 + (8 - last.leading_zeros() as usize)
}

fn bigint_from_unsigned_le_bytes(bytes: &[u8]) -> BigInt {
    let mut value = BigInt::from(0_u8);
    for byte in bytes.iter().rev() {
        value <<= 8_usize;
        value += BigInt::from(*byte);
    }
    value
}

fn push_entropy_error(session: &mut Session, symbol: &str, message: &str) {
    if !session.has_entropy_source() {
        session.emit_no_entropy_message();
        return;
    }

    session.push_message(
        symbol,
        "entropy",
        format!("could not read entropy: {message}"),
    );
}

fn byte_array_expr(bytes: &[u8]) -> Expr {
    Expr::call(
        system_expr("ByteArray"),
        vec![Expr::string(base64_encode(bytes))],
    )
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn bool_expr(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

fn uuid_string(bytes: &[u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(36);

    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            out.push('-');
        }
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }

    out
}

fn uuid_q(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }

    for (index, byte) in bytes.iter().copied().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }

    bytes[14] == b'4' && matches!(bytes[19], b'8' | b'9' | b'a' | b'b' | b'A' | b'B')
}
