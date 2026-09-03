use super::{
    core::Polynomial, factor_collections::cancel_common_factor_multisets,
    factorization::PolynomialFactorization, *,
};
pub(super) struct RationalFunction {
    numerator: Polynomial,
    denominator: Polynomial,
}

impl RationalFunction {
    pub(super) fn one() -> Self {
        Self::from_polynomial(Polynomial::one())
    }

    pub(super) fn from_polynomial(polynomial: Polynomial) -> Self {
        Self {
            numerator: polynomial,
            denominator: Polynomial::one(),
        }
    }

    pub(super) fn from_expr(expr: &Expr) -> Option<Self> {
        match expr {
            Expr::Atom(_) => Some(Self::from_polynomial(Polynomial::from_expr(expr)?)),
            Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => {
                Self::from_sum(args)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
                let mut result = Self::one();

                for arg in args {
                    result = result.multiply(Self::from_expr(arg)?)?;
                }

                Some(result)
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Power") && args.len() == 2 =>
            {
                let exponent = expr_as_exact_integer(&args[1])?;

                if exponent == BigInt::from(0_u8) {
                    let base = Self::from_expr(&args[0])?;
                    return (!base.numerator.is_zero()).then(Self::one);
                }

                if exponent > BigInt::from(0_u8) {
                    let exponent = exponent.to_string().parse::<usize>().ok()?;
                    return Self::from_expr(&args[0])?.pow(exponent);
                }

                let magnitude = (-exponent).to_string().parse::<usize>().ok()?;
                Self::from_expr(&args[0])?.reciprocal()?.pow(magnitude)
            }
            _ => Some(Self::from_polynomial(Polynomial::from_expr(expr)?)),
        }
    }

    fn from_sum(args: &[Expr]) -> Option<Self> {
        match args {
            [] => Some(Self::from_polynomial(Polynomial::zero())),
            [arg] => Self::from_expr(arg),
            _ => {
                let (left, right) = args.split_at(args.len() / 2);
                Self::from_sum(left)?.add(Self::from_sum(right)?)
            }
        }
    }

    pub(super) fn add(self, rhs: Self) -> Option<Self> {
        let numerator = self
            .numerator
            .multiply(&rhs.denominator)
            .add(rhs.numerator.multiply(&self.denominator));
        let denominator = self.denominator.multiply(&rhs.denominator);

        Self {
            numerator,
            denominator,
        }
        .normalized()
    }

    pub(super) fn multiply(self, rhs: Self) -> Option<Self> {
        Self {
            numerator: self.numerator.multiply(&rhs.numerator),
            denominator: self.denominator.multiply(&rhs.denominator),
        }
        .normalized()
    }

    pub(super) fn pow(self, exponent: usize) -> Option<Self> {
        if exponent == 0 {
            return (!self.numerator.is_zero()).then(Self::one);
        }

        Some(Self {
            numerator: self.numerator.pow(exponent),
            denominator: self.denominator.pow(exponent),
        })
    }

    pub(super) fn reciprocal(self) -> Option<Self> {
        (!self.numerator.is_zero())
            .then_some(Self {
                numerator: self.denominator,
                denominator: self.numerator,
            })
            .and_then(Self::normalized)
    }

    pub(super) fn normalized(self) -> Option<Self> {
        if self.denominator.is_zero() {
            return None;
        }

        if self.numerator.is_zero() {
            return Some(Self::from_polynomial(Polynomial::zero()));
        }

        let cancelled = self.cancel_common_factors()?;

        if let Some(scale) = cancelled.denominator.as_constant_rational() {
            return Some(Self::from_polynomial(
                cancelled.numerator.scale(&scale.reciprocal()?),
            ));
        }

        Some(cancelled)
    }

    pub(super) fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        let numerator = self.numerator.to_expr(session)?;

        if self.denominator.as_constant_rational() == Some(Rational::one()) {
            return Ok(numerator);
        }

        let denominator = self.denominator.to_expr(session)?;
        let reciprocal = Expr::call(
            system_expr("Power"),
            vec![denominator, Expr::integer(BigInt::from(-1_i32))],
        );

        session.combine_multiplicative_args(&system_expr("Times"), &[numerator, reciprocal])
    }

    pub(super) fn cancel_common_factors(self) -> Option<Self> {
        let numerator_factors = PolynomialFactorization::from_polynomial(self.numerator);
        let denominator_factors = PolynomialFactorization::from_polynomial(self.denominator);
        let (remaining_numerator, remaining_denominator) =
            cancel_common_factor_multisets(numerator_factors.factors, denominator_factors.factors);

        let unit = numerator_factors.unit.div(&denominator_factors.unit)?;

        Some(Self {
            numerator: PolynomialFactorization {
                unit,
                factors: remaining_numerator,
            }
            .to_polynomial(),
            denominator: PolynomialFactorization {
                unit: Rational::one(),
                factors: remaining_denominator,
            }
            .to_polynomial(),
        })
    }
}
