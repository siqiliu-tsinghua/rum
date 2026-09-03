use super::super::*;
use super::exact::*;
use super::group_index::{existing_group_index, group_index_worthwhile, record_group_index};
use num_traits::ToPrimitive;
use std::borrow::Cow;

impl Session {
    pub(super) fn eval_plus(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if let Some(value) = fast_integer_plus(args) {
            return Ok(value);
        }

        self.combine_additive_args(head, args)
    }

    pub(super) fn eval_times(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.combine_multiplicative_args(head, args)
    }

    pub(super) fn eval_power(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Power", args, 2));
        }

        if let Some(base) = expr_as_exact_rational(&args[0]) {
            let Some(exponent) = expr_as_signed_integer_value(&args[1]) else {
                self.enforce_symbolic_power_memory_constraint(head, args)?;
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };

            if exponent == BigInt::from(0_u8) && base.is_zero() {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }

            self.enforce_integer_work_limit_for_power(&base, &exponent)?;
            self.enforce_exact_power_memory_constraint(&base, &exponent)?;

            return Ok(base
                .pow(&exponent)
                .map(Expr::rational)
                .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())));
        }

        self.enforce_symbolic_power_memory_constraint(head, args)?;

        if let Some((base, inner_exponent)) = split_power_base_exp(&args[0]) {
            if let Some(outer_exponent) = expr_as_signed_integer_value(&args[1]) {
                let exponent = inner_exponent.mul(&Rational::from_integer(outer_exponent));
                return Ok(build_power_from_base_exp(base, exponent));
            }
        }

        if let Expr::Call {
            head: times_head,
            args: factors,
        } = &args[0]
        {
            if is_system_symbol_name(times_head, "Times")
                && expr_as_exact_rational(&args[1]).is_some()
            {
                let mut numeric = Rational::one();
                let mut symbolic = Vec::new();
                for factor in factors.iter() {
                    if let Some(value) = expr_as_exact_rational(factor) {
                        numeric = numeric.mul(&value);
                    } else {
                        symbolic.push(factor.clone());
                    }
                }

                let power_head = system_expr("Power");
                let times_head = system_expr("Times");
                let numeric_part = if numeric.is_one() {
                    Expr::integer(1_i32.into())
                } else {
                    self.eval_power(&power_head, &[Expr::rational(numeric), args[1].clone()])?
                };
                if expr_as_signed_integer_value(&args[1]).is_some() {
                    let mut parts = Vec::with_capacity(symbolic.len() + 1);
                    parts.push(numeric_part);
                    for factor in symbolic {
                        parts.push(self.eval_power(&power_head, &[factor, args[1].clone()])?);
                    }
                    return self.combine_multiplicative_args(&times_head, &parts);
                }

                let symbolic_part = match symbolic.as_slice() {
                    [] => Expr::integer(1_i32.into()),
                    [factor] => self.eval_power(&power_head, &[factor.clone(), args[1].clone()])?,
                    _ => Expr::call(
                        power_head,
                        vec![Expr::call(times_head.clone(), symbolic), args[1].clone()],
                    ),
                };
                return self
                    .combine_multiplicative_args(&times_head, &[numeric_part, symbolic_part]);
            }
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    fn enforce_integer_work_limit_for_power(
        &self,
        base: &Rational,
        exponent: &BigInt,
    ) -> Result<(), EvalError> {
        // Bound the size of the produced value BEFORE `base.pow(exponent)` builds
        // it: the result's numerator/denominator bit-lengths are at most
        // `|exponent| * (num_bits + den_bits)`. Everything below is O(1) on limb
        // counts with no heap allocation, so the guard is negligible next to the
        // exponentiation it protects (mirrors the Factorial pre-check).
        let num_bits = base.numerator().bits();
        let den_bits = base.denominator().bits();

        // Base 0, 1, or -1: the result is bounded regardless of the exponent.
        // (`den_bits <= 1` means denominator == 1; `num_bits <= 1` means
        // |numerator| <= 1.) A small denominator like `(1/2)` is NOT trivial —
        // its reciprocal power has an astronomical denominator — so it is not
        // skipped here.
        if den_bits <= 1 && num_bits <= 1 {
            return Ok(());
        }

        let limit = self.resource_limits.integer_work_limit;
        let budget_bits = (limit as u128).saturating_mul(64);
        let per_unit = num_bits as u128 + den_bits as u128;

        // If `|exponent|` does not fit in u128 the base magnitude is >= 2 here, so
        // the result is astronomically large: fail closed.
        let exceeds = match exponent.magnitude().to_u128() {
            Some(magnitude) => magnitude.saturating_mul(per_unit) > budget_bits,
            None => true,
        };
        if exceeds {
            return Err(EvalError::IntegerWorkLimitExceeded { limit });
        }

        Ok(())
    }

    fn enforce_exact_power_memory_constraint(
        &self,
        base: &Rational,
        exponent: &BigInt,
    ) -> Result<(), EvalError> {
        if self.active_memory_limit().is_none() {
            return self.enforce_active_memory_limit_for_projected_bytes(0);
        }

        let magnitude = exponent.magnitude().to_u128().unwrap_or(u128::MAX);
        let numerator_bits = projected_power_bit_length(base.numerator(), magnitude);
        let denominator_bits = projected_power_bit_length(base.denominator(), magnitude);
        let result_bytes = std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<Rational>())
            .saturating_add(projected_bit_length_bytes(numerator_bits))
            .saturating_add(projected_bit_length_bytes(denominator_bits));
        self.enforce_active_memory_limit_for_projected_bytes(result_bytes)
    }

    fn enforce_symbolic_power_memory_constraint(
        &self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<(), EvalError> {
        if self.active_memory_limit().is_none() {
            return self.enforce_active_memory_limit_for_projected_bytes(0);
        }

        let mut projected_bytes = projected_call_memory_proxy_bytes(head, args);
        if let Expr::Call {
            head: times_head,
            args: factors,
        } = &args[0]
        {
            if is_system_symbol_name(times_head, "Times")
                && expr_as_exact_rational(&args[1]).is_some()
            {
                let exponent_bytes = projected_expr_memory_proxy_bytes(&args[1]);
                let power_wrapper_bytes = std::mem::size_of::<Expr>()
                    .saturating_add(projected_expr_memory_proxy_bytes(&system_expr("Power")));
                projected_bytes = projected_bytes.saturating_add(
                    factors
                        .len()
                        .saturating_mul(exponent_bytes.saturating_add(power_wrapper_bytes)),
                );
            }
        }

        self.enforce_active_memory_limit_for_projected_bytes(projected_bytes)
    }

    pub(super) fn eval_subtract(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Subtract", args, 2));
        }

        let negated_rhs = self.combine_multiplicative_args(
            &system_expr("Times"),
            &[Expr::integer((-1_i32).into()), args[1].clone()],
        )?;
        self.combine_additive_args(&system_expr("Plus"), &[args[0].clone(), negated_rhs])
    }

    pub(super) fn eval_divide(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Divide", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (
            expr_as_exact_rational(&args[0]),
            expr_as_exact_rational(&args[1]),
        ) {
            return Ok(lhs
                .div(&rhs)
                .map(Expr::rational)
                .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())));
        }

        let reciprocal = self.eval_power(
            &system_expr("Power"),
            &[args[1].clone(), Expr::integer((-1_i32).into())],
        )?;
        self.combine_multiplicative_args(&system_expr("Times"), &[args[0].clone(), reciprocal])
    }

    /// Canonically combines arguments using the kernel's Plus semantics.
    pub fn combine_additive_args(&self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let mut numeric = Rational::zero();
        let mut combined = Vec::new();

        for arg in args {
            if let Some(value) = expr_as_exact_rational(arg) {
                numeric = numeric.add(&value);
            } else {
                combined.push(arg.clone());
            }
        }

        if let Some(rebuilt) = combine_simple_additive_terms(&combined) {
            if let Some(rebuilt) = rebuilt {
                combined = rebuilt;
            }
        } else if !combined.is_empty() {
            let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(combined.len());
            let mut group_index = None;
            let indexing_enabled = group_index_worthwhile(combined.len());
            let mut rebuild_combined = false;
            for term in &combined {
                let (coefficient, kernel) = split_coeff_kernel(term);
                if let Some(position) = existing_group_index(
                    &grouped,
                    &mut group_index,
                    kernel.as_ref(),
                    indexing_enabled,
                ) {
                    grouped[position].1 = grouped[position].1.add(&coefficient);
                    rebuild_combined = true;
                } else {
                    if split_coeff_kernel_requires_rebuild(term, &coefficient, kernel.as_ref()) {
                        rebuild_combined = true;
                    }
                    let position = grouped.len();
                    grouped.push((kernel.into_owned(), coefficient));
                    record_group_index(&grouped, &mut group_index, position);
                }
            }

            if rebuild_combined {
                combined = Vec::with_capacity(grouped.len() + usize::from(!numeric.is_zero()));
                for (kernel, coefficient) in grouped {
                    if coefficient.is_zero() {
                        continue;
                    }

                    if coefficient.is_one() {
                        combined.push(kernel);
                    } else {
                        let coefficient = Expr::rational(coefficient);
                        combined.push(match kernel {
                            Expr::Call { head, args } if is_system_symbol_name(&head, "Times") => {
                                let mut factors = Vec::with_capacity(args.len() + 1);
                                factors.push(coefficient);
                                factors.extend(args);
                                Expr::call(system_expr("Times"), factors)
                            }
                            kernel => Expr::call(system_expr("Times"), vec![coefficient, kernel]),
                        });
                    }
                }
            }
        }

        if !numeric.is_zero() {
            combined.push(Expr::rational(numeric));
        }

        let normalized = self.normalize_call_args_by_attributes(
            head,
            combined,
            self.attribute_set_for_head(head),
        );

        Ok(match normalized.as_slice() {
            [] => Expr::integer(0_i32.into()),
            [only] => only.clone(),
            _ => Expr::call(head.clone(), normalized),
        })
    }

    /// Canonically combines arguments using the kernel's Times semantics.
    pub fn combine_multiplicative_args(
        &self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let mut numeric = Rational::one();
        let mut combined = Vec::new();

        for arg in args {
            if let Some(value) = expr_as_exact_rational(arg) {
                numeric = numeric.mul(&value);

                if numeric.is_zero() {
                    return Ok(Expr::integer(0_i32.into()));
                }
            } else {
                combined.push(arg.clone());
            }
        }

        if let Some(rebuilt) = combine_simple_multiplicative_factors(&combined) {
            if let Some(rebuilt) = rebuilt {
                combined = rebuilt;
            }
        } else if !combined.is_empty() {
            let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(combined.len());
            let mut group_index = None;
            let indexing_enabled = group_index_worthwhile(combined.len());
            let mut rebuild_combined = false;
            for factor in &combined {
                let (base, exponent) = split_base_exp(factor);
                if let Some(position) = existing_group_index(
                    &grouped,
                    &mut group_index,
                    base.as_ref(),
                    indexing_enabled,
                ) {
                    grouped[position].1 = grouped[position].1.add(&exponent);
                    rebuild_combined = true;
                } else {
                    if split_base_exp_requires_rebuild(factor, base.as_ref(), &exponent) {
                        rebuild_combined = true;
                    }
                    let position = grouped.len();
                    grouped.push((base.into_owned(), exponent));
                    record_group_index(&grouped, &mut group_index, position);
                }
            }

            if rebuild_combined {
                combined = Vec::with_capacity(grouped.len() + usize::from(!numeric.is_one()));
                for (base, exponent) in grouped {
                    if exponent.is_zero() {
                        continue;
                    }

                    combined.push(build_power_from_base_exp(base, exponent));
                }
            }
        }

        if !numeric.is_one() || combined.is_empty() {
            combined.push(Expr::rational(numeric));
        }

        let normalized = self.normalize_call_args_by_attributes(
            head,
            combined,
            self.attribute_set_for_head(head),
        );

        Ok(match normalized.as_slice() {
            [] => Expr::integer(1_i32.into()),
            [only] => only.clone(),
            _ => Expr::call(head.clone(), normalized),
        })
    }
}

fn projected_call_memory_proxy_bytes(head: &Expr, args: &[Expr]) -> usize {
    args.iter().fold(
        std::mem::size_of::<Expr>().saturating_add(projected_expr_memory_proxy_bytes(head)),
        |total, arg| total.saturating_add(projected_expr_memory_proxy_bytes(arg)),
    )
}

fn projected_expr_memory_proxy_bytes(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(Atom::Integer(rum_core::IntValue::Small(_))) => std::mem::size_of::<Expr>(),
        Expr::Atom(Atom::Integer(rum_core::IntValue::Big(value))) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<BigInt>())
            .saturating_add(projected_bit_length_bytes(value.bits() as u128)),
        Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<Rational>())
            .saturating_add(projected_bit_length_bytes(value.numerator().bits() as u128))
            .saturating_add(projected_bit_length_bytes(
                value.denominator().bits() as u128
            )),
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

fn projected_bit_length_bytes(bits: u128) -> usize {
    let bytes = bits.saturating_add(7) / 8;
    usize::try_from(bytes.saturating_add(1)).unwrap_or(usize::MAX)
}

fn projected_power_bit_length(value: &BigInt, magnitude: u128) -> u128 {
    if magnitude == 0 {
        return 1;
    }

    let bits = value.bits();
    if bits <= 1 {
        return bits as u128;
    }

    // Powers of two have an exact result length. This matters for the common
    // `2^n` case: the generic `bits * n` upper bound is almost twice the real
    // payload. `bits()` and `trailing_zeros()` inspect existing limbs only.
    if value.trailing_zeros() == Some(bits - 1) {
        return (u128::from(bits - 1))
            .saturating_mul(magnitude)
            .saturating_add(1);
    }

    u128::from(bits).saturating_mul(magnitude)
}

fn fast_integer_plus(args: &[Expr]) -> Option<Expr> {
    let mut iter = args.iter();
    let Some(first) = iter.next() else {
        return Some(Expr::integer(0_i32.into()));
    };

    let mut total = expr_as_exact_integer(first)?;
    for arg in iter {
        total += expr_as_exact_integer(arg)?;
    }

    Some(Expr::integer(total))
}
fn build_power_from_base_exp(base: Expr, exponent: Rational) -> Expr {
    if exponent.is_zero() {
        return Expr::integer(1_i32.into());
    }

    if exponent.is_one() {
        base
    } else {
        Expr::call(system_expr("Power"), vec![base, Expr::rational(exponent)])
    }
}

fn split_base_exp(expr: &Expr) -> (Cow<'_, Expr>, Rational) {
    let Some((base, exponent)) = split_power_base_exp_ref(expr) else {
        return (Cow::Borrowed(expr), Rational::one());
    };

    if !exponent.is_integer() {
        return (Cow::Borrowed(base), exponent);
    }

    let (inner_base, inner_exponent) = split_base_exp(base);
    (inner_base, inner_exponent.mul(&exponent))
}

fn split_base_exp_requires_rebuild(factor: &Expr, base: &Expr, exponent: &Rational) -> bool {
    if exponent.is_zero() {
        return true;
    }

    if exponent.is_one() {
        return base != factor;
    }

    let Expr::Call { head, args } = factor else {
        return true;
    };
    if !is_system_symbol_name(head, "Power") || args.len() != 2 {
        return true;
    }

    let Some(original_exponent) = expr_as_exact_rational(&args[1]) else {
        return true;
    };

    &args[0] != base || &original_exponent != exponent
}

fn combine_simple_multiplicative_factors(args: &[Expr]) -> Option<Option<Vec<Expr>>> {
    if args.is_empty() {
        return Some(None);
    }

    let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(args.len());
    let mut group_index = None;
    let indexing_enabled = group_index_worthwhile(args.len());
    let mut rebuild_combined = false;
    for factor in args {
        let (base, exponent) = simple_base_exp(factor)?;
        if let Some(position) =
            existing_group_index(&grouped, &mut group_index, base, indexing_enabled)
        {
            grouped[position].1 = grouped[position].1.add(&exponent);
            rebuild_combined = true;
        } else {
            if split_base_exp_requires_rebuild(factor, base, &exponent) {
                rebuild_combined = true;
            }
            let position = grouped.len();
            grouped.push((base.clone(), exponent));
            record_group_index(&grouped, &mut group_index, position);
        }
    }

    if !rebuild_combined {
        return Some(None);
    }

    let mut combined = Vec::with_capacity(grouped.len());
    for (base, exponent) in grouped {
        if exponent.is_zero() {
            continue;
        }

        combined.push(build_power_from_base_exp(base, exponent));
    }

    Some(Some(combined))
}

fn simple_base_exp(expr: &Expr) -> Option<(&Expr, Rational)> {
    if matches!(expr, Expr::Atom(_)) {
        return Some((expr, Rational::one()));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Power") || args.len() != 2 {
        return None;
    }

    let Expr::Atom(_) = &args[0] else {
        return None;
    };
    if expr_as_exact_rational(&args[0]).is_some() {
        return None;
    }
    let exponent = expr_as_exact_rational(&args[1])?;
    Some((&args[0], exponent))
}

fn split_power_base_exp(expr: &Expr) -> Option<(Expr, Rational)> {
    let (base, exponent) = split_power_base_exp_ref(expr)?;
    Some((base.clone(), exponent))
}

fn split_power_base_exp_ref(expr: &Expr) -> Option<(&Expr, Rational)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Power") || args.len() != 2 {
        return None;
    }

    let exponent = expr_as_exact_rational(&args[1])?;
    if expr_as_exact_rational(&args[0]).is_some() {
        return None;
    }

    Some((&args[0], exponent))
}

fn split_coeff_kernel(expr: &Expr) -> (Rational, Cow<'_, Expr>) {
    let Expr::Call { head, args } = expr else {
        return (Rational::one(), Cow::Borrowed(expr));
    };
    if !is_system_symbol_name(head, "Times") {
        return (Rational::one(), Cow::Borrowed(expr));
    }

    let mut coefficient = Rational::one();
    let mut rest = Vec::new();
    for arg in args.iter() {
        if let Some(value) = expr_as_exact_rational(arg) {
            coefficient = coefficient.mul(&value);
        } else {
            rest.push(arg);
        }
    }

    let kernel = match rest.as_slice() {
        [] => Cow::Owned(Expr::integer(1_i32.into())),
        [only] => Cow::Borrowed(*only),
        _ => Cow::Owned(Expr::call(
            system_expr("Times"),
            rest.into_iter().cloned().collect(),
        )),
    };

    (coefficient, kernel)
}

fn combine_simple_additive_terms(args: &[Expr]) -> Option<Option<Vec<Expr>>> {
    if args.is_empty() {
        return Some(None);
    }

    let mut grouped: Vec<(Expr, Rational)> = Vec::with_capacity(args.len());
    let mut group_index = None;
    let indexing_enabled = group_index_worthwhile(args.len());
    let mut rebuild_combined = false;
    for term in args {
        let (coefficient, kernel) = simple_coeff_kernel(term)?;
        if let Some(position) =
            existing_group_index(&grouped, &mut group_index, kernel, indexing_enabled)
        {
            grouped[position].1 = grouped[position].1.add(&coefficient);
            rebuild_combined = true;
        } else {
            if split_coeff_kernel_requires_rebuild(term, &coefficient, kernel) {
                rebuild_combined = true;
            }
            let position = grouped.len();
            grouped.push((kernel.clone(), coefficient));
            record_group_index(&grouped, &mut group_index, position);
        }
    }

    if !rebuild_combined {
        return Some(None);
    }

    let mut combined = Vec::with_capacity(grouped.len());
    for (kernel, coefficient) in grouped {
        if coefficient.is_zero() {
            continue;
        }

        if coefficient.is_one() {
            combined.push(kernel);
        } else {
            combined.push(Expr::call(
                system_expr("Times"),
                vec![Expr::rational(coefficient), kernel],
            ));
        }
    }

    Some(Some(combined))
}

fn simple_coeff_kernel(expr: &Expr) -> Option<(Rational, &Expr)> {
    if matches!(expr, Expr::Atom(_)) {
        return Some((Rational::one(), expr));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Times") || args.len() != 2 {
        return None;
    }

    let coefficient = expr_as_exact_rational(&args[0])?;
    if !matches!(&args[1], Expr::Atom(_)) {
        return None;
    }

    Some((coefficient, &args[1]))
}

fn split_coeff_kernel_requires_rebuild(term: &Expr, coefficient: &Rational, kernel: &Expr) -> bool {
    if coefficient.is_zero() {
        return true;
    }

    if coefficient.is_one() {
        return kernel != term;
    }

    let Expr::Call { head, args } = term else {
        return true;
    };
    if !is_system_symbol_name(head, "Times") || args.is_empty() {
        return true;
    }

    let Some(first_coefficient) = expr_as_exact_rational(&args[0]) else {
        return true;
    };
    if &first_coefficient != coefficient {
        return true;
    }

    match kernel {
        Expr::Call {
            head: kernel_head,
            args: kernel_args,
        } if is_system_symbol_name(kernel_head, "Times") => args[1..] != kernel_args[..],
        _ => args.len() != 2 || &args[1] != kernel,
    }
}
fn expr_as_signed_integer_value(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.to_bigint()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") && args.len() == 2 => {
            if is_negative_one_expr(&args[0]) {
                expr_as_exact_integer(&args[1]).map(|value| -value)
            } else if is_negative_one_expr(&args[1]) {
                expr_as_exact_integer(&args[0]).map(|value| -value)
            } else {
                None
            }
        }
        _ => None,
    }
}
