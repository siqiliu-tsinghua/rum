use rum_core::SharedArgs;
use rum_core::{BigInt, Expr};
use rum_runtime::kernel_api::{AbandonmentContext, FrameAbandon, FrameAbandonAction};
use rum_runtime::kernel_api::{
    ArithmeticDataBuiltin, EvalError, Session, call_items_with_head, system_expr,
};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod stats;

pub(crate) use stats::{eval_commonest, eval_median, eval_quantile};

fn dispatch(
    session: &mut Session,
    builtin: ArithmeticDataBuiltin,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    session.eval_arithmetic_data_builtin(builtin, head, args)
}

pub(crate) fn eval_identity(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Identity, head, args)
}

pub(crate) fn eval_plus(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Plus, head, args)
}

pub(crate) fn eval_times(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Times, head, args)
}

pub(crate) fn eval_power(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Power, head, args)
}

pub(crate) fn eval_subtract(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Subtract, head, args)
}

pub(crate) fn eval_divide(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Divide, head, args)
}

pub(crate) fn eval_abs(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Abs, head, args)
}

pub(crate) fn eval_sign(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Sign, head, args)
}

pub(crate) fn eval_floor(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Floor, head, args)
}

pub(crate) fn eval_ceiling(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Ceiling, head, args)
}

pub(crate) fn eval_integer_part(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::IntegerPart, head, args)
}

pub(crate) fn eval_fractional_part(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::FractionalPart, head, args)
}

pub(crate) fn eval_minus(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Minus, head, args)
}

pub(crate) fn eval_round(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Round, head, args)
}

pub(crate) fn eval_factorial(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Factorial, head, args)
}

pub(crate) fn eval_factorial2(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Factorial2, head, args)
}

pub(crate) fn eval_total(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Total, head, args)
}

pub(crate) fn eval_mean(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Mean", args, 1));
    };

    let Some(items) = call_items_with_head(expr, &system_expr("List")) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if items.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Mean",
            args,
            "empty",
            "Mean cannot aggregate an empty list.",
        ));
    }

    let total = Expr::call(system_expr("Total"), vec![expr.clone()]);
    let length = Expr::integer(BigInt::from(items.len()));
    let quotient = Expr::call(system_expr("Divide"), vec![total, length]);
    let evaluated = session.evaluate(quotient);
    rum_runtime::resume_or_suspend!(
        session,
        evaluated,
        ok(value) => Ok(value),
        suspend => EvalMeanSuspensionFrame,
    )
}

pub(crate) fn eval_denominator(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Denominator, head, args)
}

pub(crate) fn eval_numerator(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Numerator, head, args)
}

pub(crate) fn eval_mod(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Mod, head, args)
}

pub(crate) fn eval_quotient(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Quotient, head, args)
}

pub(crate) fn eval_power_mod(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::PowerMod, head, args)
}

pub(crate) fn eval_gcd(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::GCD, head, args)
}

pub(crate) fn eval_lcm(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::LCM, head, args)
}

pub(crate) fn eval_max(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Max, head, args)
}

pub(crate) fn eval_min(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Min, head, args)
}

pub(crate) fn eval_min_max(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::MinMax, head, args)
}

#[derive(rum_runtime::kernel_api::FrameRootTrace, rum_runtime::kernel_api::PureFrameAbandon)]
struct EvalMeanSuspensionFrame;

impl rum_runtime::kernel_api::SuspensionFrame for EvalMeanSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
