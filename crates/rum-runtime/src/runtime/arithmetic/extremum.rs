use super::super::*;
use super::exact::expr_as_exact_rational;

impl Session {
    pub(super) fn eval_max(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_exact_extremum(head, args, true)
    }

    pub(super) fn eval_min(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_exact_extremum(head, args, false)
    }

    pub(super) fn eval_min_max(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let (min, max) = match arg {
            Expr::Call {
                head: list_head,
                args: items,
                ..
            } if is_system_symbol_name(list_head, "List") => {
                let mut operands = Vec::new();
                flatten_list_operands(items.as_ref(), &mut operands);
                match operands.as_slice() {
                    [] => {
                        let min_head = system_expr("Min");
                        let max_head = system_expr("Max");
                        (
                            self.eval_min(&min_head, &[])?,
                            self.eval_max(&max_head, &[])?,
                        )
                    }
                    [only] => ((*only).clone(), (*only).clone()),
                    _ => {
                        let min_head = system_expr("Min");
                        let max_head = system_expr("Max");
                        let orderless = AttributeSet::from_attribute(Attribute::Orderless);
                        let extrema_args = operands.into_iter().cloned().collect();
                        let extrema_args = self.normalize_call_args_by_attributes(
                            &min_head,
                            extrema_args,
                            orderless,
                        );
                        (
                            self.eval_min(&min_head, &extrema_args)?,
                            self.eval_max(&max_head, &extrema_args)?,
                        )
                    }
                }
            }
            _ => (arg.clone(), arg.clone()),
        };

        Ok(Expr::call(system_expr("List"), vec![min, max]))
    }

    fn eval_exact_extremum(
        &mut self,
        head: &Expr,
        args: &[Expr],
        select_max: bool,
    ) -> Result<Expr, EvalError> {
        let mut operands = Vec::new();
        flatten_list_operands(args, &mut operands);

        let Some(first) = operands.first().copied() else {
            return Ok(if select_max {
                Expr::call(
                    system_expr("Times"),
                    vec![Expr::integer((-1_i32).into()), system_expr("Infinity")],
                )
            } else {
                system_expr("Infinity")
            });
        };

        let Some(mut best_value) = expr_as_exact_rational(first) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let mut best_expr = first;

        for operand in operands.iter().skip(1).copied() {
            let Some(value) = expr_as_exact_rational(operand) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };

            let ordering = compare_exact_rational_values(&value, &best_value);
            if (select_max && ordering == std::cmp::Ordering::Greater)
                || (!select_max && ordering == std::cmp::Ordering::Less)
            {
                best_value = value;
                best_expr = operand;
            }
        }

        Ok(best_expr.clone())
    }
}

fn flatten_list_operands<'a>(args: &'a [Expr], operands: &mut Vec<&'a Expr>) {
    for arg in args {
        flatten_list_operand(arg, operands);
    }
}

fn flatten_list_operand<'a>(expr: &'a Expr, operands: &mut Vec<&'a Expr>) {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            for arg in args.iter() {
                flatten_list_operand(arg, operands);
            }
        }
        _ => operands.push(expr),
    }
}

fn compare_exact_rational_values(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}
