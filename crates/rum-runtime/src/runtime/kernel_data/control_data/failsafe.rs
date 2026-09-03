use super::confirmation::{build_failsafe_failure, is_confirmation_failure_like};
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_failsafe(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(head.clone(), args.to_vec()))
    }
}

fn is_failsafe_failure_like(expr: &Expr) -> bool {
    is_confirmation_failure_like(expr)
}

pub(in crate::runtime) fn apply_callable_failsafe_wrapper_to_args(
    head: &Expr,
    args: &[Expr],
) -> Option<Expr> {
    let spec = parse_failsafe_wrapper(head)?;

    if spec.test.is_none() {
        return Some(
            args.iter()
                .find(|arg| is_failsafe_failure_like(arg))
                .cloned()
                .unwrap_or_else(|| Expr::call(spec.function.clone(), args.to_vec())),
        );
    }

    let test = spec.test?.clone();
    let success = Expr::call(spec.function.clone(), args.to_vec());
    let fallback = match spec.fail_function {
        Some(fail_function) => Expr::call(fail_function.clone(), args.to_vec()),
        None => build_failsafe_failure(spec.function.clone(), test.clone(), args.to_vec()),
    };

    Some(Expr::call(
        system_expr("If"),
        vec![
            Expr::call(system_expr("TrueQ"), vec![Expr::call(test, args.to_vec())]),
            success,
            fallback,
        ],
    ))
}

pub(in crate::runtime) fn is_callable_failsafe_wrapper(expr: &Expr) -> bool {
    parse_failsafe_wrapper(expr).is_some()
}

struct FailsafeWrapperSpec<'a> {
    function: &'a Expr,
    test: Option<&'a Expr>,
    fail_function: Option<&'a Expr>,
}

fn parse_failsafe_wrapper(expr: &Expr) -> Option<FailsafeWrapperSpec<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Failsafe") {
        return None;
    }

    match args.as_slice() {
        [function] => Some(FailsafeWrapperSpec {
            function,
            test: None,
            fail_function: None,
        }),
        [function, test] => Some(FailsafeWrapperSpec {
            function,
            test: Some(test),
            fail_function: None,
        }),
        [function, test, fail_function] => Some(FailsafeWrapperSpec {
            function,
            test: Some(test),
            fail_function: Some(fail_function),
        }),
        _ => None,
    }
}
