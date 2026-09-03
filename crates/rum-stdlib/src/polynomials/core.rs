use super::{
    candidates::{bigint_abs, bigint_gcd, bigint_lcm, linear_factor_candidates, rational_from_i64},
    expr_contains_pattern_language, parse_indeterminate_expr, *,
};

mod construction;
mod factors;
mod intersection;
mod term_order;
mod variables;
pub(super) use factors::parse_coefficient_list_monomials;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Polynomial {
    pub(super) terms: Vec<Term>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Term {
    pub(super) coefficient: Rational,
    pub(super) factors: Vec<Factor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Factor {
    pub(super) base: Expr,
    pub(super) exponent: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Monomial {
    pub(super) factors: Vec<Factor>,
}

impl Polynomial {
    pub(super) fn zero() -> Self {
        Self { terms: Vec::new() }
    }

    pub(super) fn one() -> Self {
        Self::constant(Rational::one())
    }

    pub(super) fn constant(value: Rational) -> Self {
        if value.is_zero() {
            Self::zero()
        } else {
            Self {
                terms: vec![Term::constant(value)],
            }
        }
    }

    pub(super) fn indeterminate(base: Expr) -> Self {
        Self {
            terms: vec![Term::indeterminate(base)],
        }
    }

    pub(super) fn from_expr(expr: &Expr) -> Option<Self> {
        if let Some(value) = expr_as_exact_rational(expr) {
            return Some(Self::constant(value));
        }

        match expr {
            Expr::Call { head, args, .. } if is_system_symbol_name(head, "Plus") => {
                let mut terms = Vec::new();

                for arg in args {
                    terms.extend(Self::from_expr(arg)?.terms);
                }

                Some(Self::from_normalized_terms(terms))
            }
            Expr::Call { head, args, .. } if is_system_symbol_name(head, "Times") => {
                let mut monomial_terms = Vec::new();
                let mut polynomial = None;

                for arg in args {
                    let factor = Self::from_expr(arg)?;
                    if polynomial.is_none() && factor.terms.len() == 1 {
                        monomial_terms.push(factor.terms.into_iter().next().expect("one term"));
                        continue;
                    }

                    let product = polynomial.take().unwrap_or_else(|| {
                        let product = Self::from_normalized_terms(vec![Term::multiply_many(
                            monomial_terms.iter(),
                        )]);
                        monomial_terms.clear();
                        product
                    });
                    polynomial = Some(product.multiply(&factor));
                }

                Some(polynomial.unwrap_or_else(|| {
                    Self::from_normalized_terms(vec![Term::multiply_many(monomial_terms.iter())])
                }))
            }
            Expr::Call { head, args, .. }
                if is_system_symbol_name(head, "Power") && args.len() == 2 =>
            {
                if let Some(exponent) = expr_as_non_negative_usize(&args[1]) {
                    return Some(Self::from_expr(&args[0])?.pow(exponent));
                }

                (!expr_contains_pattern_language(expr)).then(|| Self::indeterminate(expr.clone()))
            }
            _ => (!expr_contains_pattern_language(expr)).then(|| Self::indeterminate(expr.clone())),
        }
    }

    pub(super) fn add(mut self, rhs: Self) -> Self {
        self.terms.extend(rhs.terms);
        Self::from_normalized_terms(self.terms)
    }

    pub(super) fn multiply(&self, rhs: &Self) -> Self {
        if self.terms.is_empty() || rhs.terms.is_empty() {
            return Self::zero();
        }

        let mut terms = Vec::with_capacity(self.terms.len().saturating_mul(rhs.terms.len()));

        for lhs_term in &self.terms {
            for rhs_term in &rhs.terms {
                terms.push(lhs_term.multiply(rhs_term));
            }
        }

        Self::from_normalized_terms(terms)
    }

    pub(super) fn pow(&self, exponent: usize) -> Self {
        if exponent == 0 {
            return Self::one();
        }

        let mut result = Self::one();
        let mut factor = self.clone();
        let mut remaining = exponent;

        while remaining > 0 {
            if remaining % 2 == 1 {
                result = result.multiply(&factor);
            }

            remaining /= 2;

            if remaining > 0 {
                factor = factor.multiply(&factor);
            }
        }

        result
    }

    pub(super) fn scale(&self, scalar: &Rational) -> Self {
        if scalar.is_zero() || self.is_zero() {
            return Self::zero();
        }

        Self::from_order_preserving_terms(
            self.terms
                .iter()
                .map(|term| Term {
                    coefficient: term.coefficient.mul(scalar),
                    factors: term.factors.clone(),
                })
                .collect(),
        )
    }

    pub(super) fn coefficient_for_monomial_power(&self, target: &Monomial, power: usize) -> Self {
        Self::from_order_preserving_terms(
            self.terms
                .iter()
                .filter_map(|term| term.coefficient_after_exact_monomial_power(target, power))
                .collect(),
        )
    }

    pub(super) fn max_exact_power_for_monomial(&self, target: &Monomial) -> usize {
        self.terms
            .iter()
            .map(|term| target.exact_multiplicity_in_term(term))
            .max()
            .unwrap_or(0)
    }

    pub(super) fn coefficient_list_expr(
        &self,
        variables: &[Monomial],
        session: &Session,
    ) -> Result<Expr, EvalError> {
        let max_powers = variables
            .iter()
            .map(|variable| self.max_exact_power_for_monomial(variable))
            .collect::<Vec<_>>();

        self.coefficient_grid_expr(variables, &max_powers, session)
    }

    pub(super) fn coefficient_grid_expr(
        &self,
        variables: &[Monomial],
        max_powers: &[usize],
        session: &Session,
    ) -> Result<Expr, EvalError> {
        let Some((first, remaining_variables)) = variables.split_first() else {
            return self.to_expr(session);
        };

        let max_power = max_powers[0];
        let mut items = Vec::with_capacity(max_power + 1);

        for power in 0..=max_power {
            let coefficient = self.coefficient_for_monomial_power(first, power);
            let item = if remaining_variables.is_empty() {
                coefficient.to_expr(session)?
            } else {
                coefficient.coefficient_grid_expr(remaining_variables, &max_powers[1..], session)?
            };
            items.push(item);
        }

        Ok(Expr::call(system_expr("List"), items))
    }

    pub(super) fn subtract(&self, rhs: &Self) -> Self {
        self.clone().add(rhs.scale(&rational_from_i64(-1)))
    }

    pub(super) fn multiply_by_term(&self, term: &Term) -> Self {
        Self::from_order_preserving_terms(
            self.terms
                .iter()
                .map(|existing| existing.multiply(term))
                .collect(),
        )
    }

    pub(super) fn divide_by_scalar(&self, scalar: &Rational) -> Option<Self> {
        Some(self.scale(&scalar.reciprocal()?))
    }

    pub(super) fn common_monomial_factor(&self) -> Monomial {
        let Some((first, remaining)) = self.terms.split_first() else {
            return Monomial::one();
        };
        if first.factors.is_empty() {
            return Monomial::one();
        }

        let mut factors = first.factors.clone();
        let mut intersection = Vec::new();
        for term in remaining {
            intersection::intersect_normalized_factors_into(
                &factors,
                &term.factors,
                &mut intersection,
            );
            std::mem::swap(&mut factors, &mut intersection);
            if factors.is_empty() {
                return Monomial::one();
            }
        }

        Monomial { factors }
    }

    pub(super) fn divide_by_monomial(&self, monomial: &Monomial) -> Option<Self> {
        if monomial.is_one() {
            return Some(self.clone());
        }

        let terms = self
            .terms
            .iter()
            .map(|term| term.divide_by_monomial(monomial))
            .collect::<Option<Vec<_>>>()?;
        Some(Self::from_order_preserving_terms(terms))
    }

    pub(super) fn total_degree(&self) -> usize {
        self.terms.iter().map(Term::total_degree).max().unwrap_or(0)
    }

    pub(super) fn leading_term(&self) -> Option<&Term> {
        self.terms.iter().max_by(|lhs, rhs| lhs.compare_order(rhs))
    }

    pub(super) fn divide_exact_by(&self, divisor: &Self) -> Option<Self> {
        if divisor.is_zero() {
            return None;
        }

        let divisor_leading = divisor.leading_term()?.clone();
        let mut quotient_terms = Vec::new();
        let mut remainder = self.clone();
        let max_steps = self.terms.len().saturating_mul(divisor.terms.len()).max(1) * 32;

        for _ in 0..max_steps {
            if remainder.is_zero() {
                return Some(Self::from_normalized_terms(quotient_terms));
            }

            let remainder_leading = remainder.leading_term()?.clone();
            let quotient_term = remainder_leading.divide_by_term(&divisor_leading)?;
            quotient_terms.push(quotient_term.clone());
            remainder = remainder.subtract(&divisor.multiply_by_term(&quotient_term));
        }

        None
    }

    pub(super) fn rational_content(&self) -> Option<Rational> {
        if self.is_zero() {
            return Some(Rational::zero());
        }

        let common_denominator = self.terms.iter().fold(BigInt::from(1_u8), |current, term| {
            bigint_lcm(current, term.coefficient.denominator().clone())
        });

        let common_numerator = self
            .terms
            .iter()
            .map(|term| {
                term.coefficient.numerator().clone()
                    * (&common_denominator / term.coefficient.denominator())
            })
            .fold(BigInt::from(0_u8), |current, value| {
                if current == BigInt::from(0_u8) {
                    bigint_abs(&value)
                } else {
                    bigint_gcd(current, bigint_abs(&value))
                }
            });

        Rational::new(common_numerator, common_denominator)
    }

    pub(super) fn extract_factor_parts(&self) -> Option<(Rational, Monomial, Self)> {
        if self.is_zero() {
            return Some((Rational::zero(), Monomial::one(), Self::zero()));
        }

        let mut unit = self.rational_content()?;
        let mut primitive = self.divide_by_scalar(&unit)?;
        let common_monomial = primitive.common_monomial_factor();
        primitive = primitive.divide_by_monomial(&common_monomial)?;

        if let Some(leading) = primitive.leading_term() {
            if leading.coefficient.numerator() < &BigInt::from(0_u8) {
                let minus_one = rational_from_i64(-1);
                unit = unit.mul(&minus_one);
                primitive = primitive.scale(&minus_one);
            }
        }

        Some((unit, common_monomial, primitive))
    }

    pub(super) fn find_linear_factorization(&self) -> Option<(Self, Self)> {
        if self.total_degree() <= 1 {
            return None;
        }

        for candidate in linear_factor_candidates(self) {
            if let Some(quotient) = self.divide_exact_by(&candidate) {
                return Some((candidate, quotient));
            }
        }

        None
    }

    pub(super) fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    pub(super) fn as_constant_rational(&self) -> Option<Rational> {
        match self.terms.as_slice() {
            [] => Some(Rational::zero()),
            [term] if term.factors.is_empty() => Some(term.coefficient.clone()),
            _ => None,
        }
    }

    pub(super) fn is_one(&self) -> bool {
        self.as_constant_rational() == Some(Rational::one())
    }

    pub(super) fn constant_term_integer(&self) -> Option<BigInt> {
        let constant = self
            .terms
            .iter()
            .find(|term| term.factors.is_empty())
            .map(|term| term.coefficient.clone())
            .unwrap_or_else(Rational::zero);
        constant.as_integer().cloned()
    }

    pub(super) fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        if self.terms.is_empty() {
            return Ok(Expr::integer(0_i32.into()));
        }

        let mut term_exprs = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            term_exprs.push(term.to_expr(session)?);
        }

        session.combine_additive_args(&system_expr("Plus"), &term_exprs)
    }
}
