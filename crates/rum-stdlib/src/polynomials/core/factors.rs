use super::*;

impl Term {
    pub(in crate::polynomials) fn constant(coefficient: Rational) -> Self {
        Self {
            coefficient,
            factors: Vec::new(),
        }
    }

    pub(in crate::polynomials) fn indeterminate(base: Expr) -> Self {
        Self {
            coefficient: Rational::one(),
            factors: vec![Factor { base, exponent: 1 }],
        }
    }

    pub(in crate::polynomials) fn multiply(&self, rhs: &Self) -> Self {
        Self {
            coefficient: self.coefficient.mul(&rhs.coefficient),
            factors: multiply_normalized_factors(&self.factors, &rhs.factors),
        }
    }

    pub(in crate::polynomials) fn multiply_many<'a>(
        terms: impl IntoIterator<Item = &'a Self>,
    ) -> Self {
        let mut coefficient = Rational::one();
        let mut factors = Vec::new();

        for term in terms {
            coefficient = coefficient.mul(&term.coefficient);
            factors.extend(term.factors.iter().cloned());
        }

        Self {
            coefficient,
            factors: normalize_factors(factors),
        }
    }

    pub(in crate::polynomials) fn total_degree(&self) -> usize {
        self.factors.iter().map(|factor| factor.exponent).sum()
    }

    pub(in crate::polynomials) fn exponent_for_factor(&self, base: &Expr) -> usize {
        self.factors
            .iter()
            .find(|factor| factor.base == *base)
            .map(|factor| factor.exponent)
            .unwrap_or(0)
    }

    pub(in crate::polynomials) fn divide_by_monomial(&self, monomial: &Monomial) -> Option<Self> {
        let mut quotient = self.clone();

        for factor in &monomial.factors {
            let existing = quotient
                .factors
                .iter_mut()
                .find(|existing| existing.base == factor.base)?;

            if existing.exponent < factor.exponent {
                return None;
            }

            existing.exponent -= factor.exponent;
        }

        Some(quotient.normalized())
    }

    pub(in crate::polynomials) fn divide_by_term(&self, divisor: &Self) -> Option<Self> {
        let mut quotient = self.divide_by_monomial(&Monomial {
            factors: divisor.factors.clone(),
        })?;
        quotient.coefficient = quotient.coefficient.div(&divisor.coefficient)?;
        Some(quotient.normalized())
    }

    pub(in crate::polynomials) fn coefficient_after_exact_monomial_power(
        &self,
        target: &Monomial,
        power: usize,
    ) -> Option<Self> {
        if target.is_one() || target.exact_multiplicity_in_term(self) != power {
            return None;
        }

        let mut remainder = self.clone();

        for target_factor in &target.factors {
            let exponent_to_remove = target_factor.exponent * power;

            if exponent_to_remove == 0 {
                continue;
            }

            let factor = remainder
                .factors
                .iter_mut()
                .find(|factor| factor.base == target_factor.base)?;
            factor.exponent -= exponent_to_remove;
        }

        Some(remainder.normalized())
    }

    pub(in crate::polynomials) fn normalized(mut self) -> Self {
        self.factors = normalize_factors(self.factors);
        self
    }

    pub(in crate::polynomials) fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        let mut factors = Vec::with_capacity(self.factors.len() + 1);

        if !self.coefficient.is_one() || self.factors.is_empty() {
            factors.push(Expr::rational(self.coefficient.clone()));
        }

        for factor in &self.factors {
            let factor_expr = if factor.exponent == 1 {
                factor.base.clone()
            } else {
                Expr::call(
                    system_expr("Power"),
                    vec![
                        factor.base.clone(),
                        Expr::integer(BigInt::from(u64::try_from(factor.exponent).unwrap())),
                    ],
                )
            };
            factors.push(factor_expr);
        }

        session.combine_multiplicative_args(&system_expr("Times"), &factors)
    }
}

impl Monomial {
    pub(in crate::polynomials) fn one() -> Self {
        Self {
            factors: Vec::new(),
        }
    }

    pub(in crate::polynomials) fn from_expr(expr: &Expr) -> Option<Self> {
        if let Some(value) = expr_as_exact_rational(expr) {
            return value.is_one().then(Self::one);
        }

        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
                let mut factors = Vec::new();

                for arg in args {
                    factors.extend(Self::from_expr(arg)?.factors);
                }

                Some(Self { factors }.normalized())
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Power") && args.len() == 2 =>
            {
                let exponent = expr_as_non_negative_usize(&args[1])?;

                if exponent == 0 {
                    return Some(Self::one());
                }

                Some(Self {
                    factors: vec![Factor {
                        base: parse_indeterminate_expr(&args[0])?,
                        exponent,
                    }],
                })
            }
            _ => Some(Self {
                factors: vec![Factor {
                    base: parse_indeterminate_expr(expr)?,
                    exponent: 1,
                }],
            }),
        }
    }

    pub(in crate::polynomials) fn from_indeterminate(expr: &Expr) -> Option<Self> {
        Some(Self {
            factors: vec![Factor {
                base: parse_indeterminate_expr(expr)?,
                exponent: 1,
            }],
        })
    }

    pub(in crate::polynomials) fn is_one(&self) -> bool {
        self.factors.is_empty()
    }

    pub(in crate::polynomials) fn exact_multiplicity_in_term(&self, term: &Term) -> usize {
        if self.factors.is_empty() {
            return 0;
        }

        self.factors
            .iter()
            .map(|factor| {
                let available = term.exponent_for_factor(&factor.base);
                if available < factor.exponent {
                    0
                } else {
                    available / factor.exponent
                }
            })
            .min()
            .unwrap_or(0)
    }

    pub(in crate::polynomials) fn normalized(mut self) -> Self {
        self.factors = normalize_factors(self.factors);
        self
    }

    pub(in crate::polynomials) fn to_polynomial_factors(&self) -> Vec<Polynomial> {
        let mut factors = Vec::new();

        for factor in &self.factors {
            for _ in 0..factor.exponent {
                factors.push(Polynomial::indeterminate(factor.base.clone()));
            }
        }

        factors
    }
}

fn normalize_factors(mut factors: Vec<Factor>) -> Vec<Factor> {
    factors.retain(|factor| factor.exponent > 0);
    factors.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.base, &rhs.base));

    let mut merged: Vec<Factor> = Vec::with_capacity(factors.len());
    for factor in factors {
        if let Some(previous) = merged.last_mut() {
            if previous.base == factor.base {
                previous.exponent += factor.exponent;
                continue;
            }
        }

        merged.push(factor);
    }

    merged
}

fn multiply_normalized_factors(lhs: &[Factor], rhs: &[Factor]) -> Vec<Factor> {
    let mut factors = Vec::with_capacity(lhs.len() + rhs.len());
    let mut lhs_index = 0;
    let mut rhs_index = 0;

    while let (Some(lhs_factor), Some(rhs_factor)) = (lhs.get(lhs_index), rhs.get(rhs_index)) {
        match compare_expr_canonical(&lhs_factor.base, &rhs_factor.base) {
            std::cmp::Ordering::Less => {
                push_merged_factor(&mut factors, lhs_factor);
                lhs_index += 1;
            }
            std::cmp::Ordering::Greater => {
                push_merged_factor(&mut factors, rhs_factor);
                rhs_index += 1;
            }
            std::cmp::Ordering::Equal => {
                // `sort_by` is stable, so the legacy concat-and-normalize path
                // keeps the entire lhs equivalence class before the rhs one.
                // Canonical-order equality is not necessarily structural Eq.
                push_merged_factor(&mut factors, lhs_factor);
                lhs_index += 1;
            }
        }
    }

    for factor in &lhs[lhs_index..] {
        push_merged_factor(&mut factors, factor);
    }
    for factor in &rhs[rhs_index..] {
        push_merged_factor(&mut factors, factor);
    }
    factors
}

fn push_merged_factor(factors: &mut Vec<Factor>, factor: &Factor) {
    if let Some(previous) = factors.last_mut() {
        if previous.base == factor.base {
            previous.exponent += factor.exponent;
            return;
        }
    }

    factors.push(factor.clone());
}

#[cfg(test)]
mod tests;

pub(in crate::polynomials) fn parse_coefficient_list_monomials(
    expr: &Expr,
) -> Option<Vec<Monomial>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            args.iter().map(Monomial::from_indeterminate).collect()
        }
        _ => Some(vec![Monomial::from_indeterminate(expr)?]),
    }
}
