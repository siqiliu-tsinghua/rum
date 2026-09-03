use rum_core::SharedArgs;
use rum_core::{Atom, BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    EvalError, Session, association_entries_from_expr, expr_as_non_negative_usize, expr_as_string,
    expr_as_symbol, is_system_symbol_name, system_expr,
};

mod decode;
mod encode;

const MAX_JSON_DEPTH: usize = 256;

pub(crate) fn eval_from_json(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [source] = args else {
        return Ok(fail_json(
            session,
            "FromJson",
            "json",
            "FromJson expects exactly one string argument.",
        ));
    };

    let Some(source) = expr_as_string(source) else {
        return Ok(fail_json(
            session,
            "FromJson",
            "json",
            "FromJson expects a JSON text string.",
        ));
    };

    match decode::from_json(source) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_json(
            session,
            "FromJson",
            error.message_tag(),
            error.message(),
        )),
    }
}

pub(crate) fn eval_to_json(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, indent) = match parse_to_json_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            return Ok(fail_json(session, "ToJson", "json", message));
        }
    };

    match encode::to_json(expr, indent) {
        Ok(json) => Ok(Expr::string(json)),
        Err(error) => Ok(fail_json(
            session,
            "ToJson",
            error.message_tag(),
            error.message(),
        )),
    }
}

fn parse_to_json_args(args: &[Expr]) -> Result<(&Expr, Option<usize>), String> {
    match args {
        [expr] => Ok((expr, None)),
        [expr, option] => {
            let Some((name, value)) = option_rule(option) else {
                return Err(
                    "ToJson expects the optional second argument to be \"Indent\" -> n."
                        .to_string(),
                );
            };
            if name != "Indent" {
                return Err(format!("Unknown ToJson option {name}."));
            }
            let Some(indent) = expr_as_non_negative_usize(value) else {
                return Err("ToJson option \"Indent\" must be a non-negative integer.".to_string());
            };
            Ok((expr, Some(indent)))
        }
        _ => {
            Err("ToJson expects one expression and an optional \"Indent\" -> n option.".to_string())
        }
    }
}

fn option_rule(expr: &Expr) -> Option<(&str, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    let name = expr_as_string(&args[0])
        .or_else(|| expr_as_symbol(&args[0]).map(|symbol| symbol.name()))?;
    Some((name, &args[1]))
}

fn fail_json(
    session: &mut Session,
    symbol: &str,
    tag: &'static str,
    message: impl Into<String>,
) -> Expr {
    session.push_message(symbol, tag, message);
    system_expr("$Failed")
}
