use super::super::*;
use super::exact::expr_as_exact_integer;
use num_traits::ToPrimitive;

enum TotalLevel {
    Finite(usize),
    All,
}

impl Session {
    pub(super) fn eval_total(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let total = match args {
            [list] if list_args(list).is_some() => Some(build_total_levels(list, 1)),
            [list, spec] => build_total_with_spec(list, spec),
            _ => None,
        };

        let Some(total) = total else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.evaluate(total)
    }

    pub(super) fn eval_denominator(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Denominator", args, 1));
        }

        match &args[0] {
            Expr::Atom(Atom::Integer(_)) => Ok(Expr::integer(1_i32.into())),
            Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.denominator().clone())),
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(super) fn eval_numerator(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Numerator", args, 1));
        }

        match &args[0] {
            Expr::Atom(Atom::Integer(value)) => Ok(Expr::integer(value.to_bigint())),
            Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.numerator().clone())),
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }
}

fn build_total_with_spec(list: &Expr, spec: &Expr) -> Option<Expr> {
    if is_system_symbol_name(spec, "Infinity") {
        return Some(build_total_all_levels(list));
    }

    if let Some(level) = expr_as_positive_total_level(spec) {
        return Some(match level {
            TotalLevel::Finite(level) => build_total_levels(list, level),
            TotalLevel::All => build_total_all_levels(list),
        });
    }

    let level_items = list_args(spec)?;
    let [level] = level_items.as_slice() else {
        return None;
    };

    match expr_as_positive_total_level(level)? {
        TotalLevel::Finite(level) => build_total_at_level(list, level).map(|(expr, _)| expr),
        TotalLevel::All => None,
    }
}

fn build_total_levels(expr: &Expr, level: usize) -> Expr {
    let Some(items) = list_args(expr) else {
        return expr.clone();
    };

    if level <= 1 {
        return Expr::call_with_shared_args(system_expr("Plus"), items.clone());
    }

    Expr::call(
        system_expr("Plus"),
        items
            .iter()
            .map(|item| build_total_levels(item, level - 1))
            .collect(),
    )
}

fn build_total_all_levels(expr: &Expr) -> Expr {
    let Some(items) = list_args(expr) else {
        return expr.clone();
    };

    Expr::call(
        system_expr("Plus"),
        items.iter().map(build_total_all_levels).collect(),
    )
}

fn build_total_at_level(expr: &Expr, level: usize) -> Option<(Expr, Vec<usize>)> {
    let items = list_args(expr)?;

    if level <= 1 {
        return Some((
            Expr::call_with_shared_args(system_expr("Plus"), items.clone()),
            vec![items.len()],
        ));
    }

    let mut mapped = Vec::with_capacity(items.len());
    let mut expected_shape = None;

    for item in items.iter() {
        let (mapped_item, shape) = build_total_at_level(item, level - 1)?;
        if expected_shape
            .as_ref()
            .is_some_and(|expected| expected != &shape)
        {
            return None;
        }
        expected_shape.get_or_insert_with(|| shape.clone());
        mapped.push(mapped_item);
    }

    let mut shape = vec![items.len()];
    if let Some(child_shape) = expected_shape {
        shape.extend(child_shape);
    }

    Some((Expr::call(system_expr("List"), mapped), shape))
}

fn expr_as_positive_total_level(expr: &Expr) -> Option<TotalLevel> {
    let value = expr_as_exact_integer(expr)?;
    if value <= BigInt::from(0_u8) {
        return None;
    }

    Some(
        value
            .to_usize()
            .map(TotalLevel::Finite)
            .unwrap_or(TotalLevel::All),
    )
}

fn list_args(expr: &Expr) -> Option<&SharedArgs> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => Some(args),
        _ => None,
    }
}
