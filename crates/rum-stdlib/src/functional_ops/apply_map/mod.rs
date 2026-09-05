use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::run_with_native_stack_guard;

mod normalize;
mod traversal;

use self::normalize::*;
use self::traversal::{
    apply_heads_with_config, collect_scan_parts_with_config, map_indexed_parts_with_config,
    map_parts_with_config, map_parts_with_mode,
};

pub(crate) fn eval_comap(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_comap_call(session, head, args) {
        CommonArgMapNormalization::OperatorForm(expr)
        | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
        CommonArgMapNormalization::Call {
            functions,
            common_expr,
            config,
        } => {
            let operator =
                function_body_operator(Expr::call(slot_expr(), vec![common_expr.clone()]));
            let (mapped, _) = map_parts_with_config(&functions, &operator, config);
            session.evaluate(mapped)
        }
    }
}

pub(crate) fn eval_comap_apply(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_comap_apply_call(session, head, args) {
        CommonArgMapNormalization::OperatorForm(expr)
        | CommonArgMapNormalization::Immediate(expr) => Ok(expr),
        CommonArgMapNormalization::Call {
            functions,
            common_expr,
            config,
        } => {
            let operator = function_body_operator(Expr::call(
                system_expr("Apply"),
                vec![slot_expr(), common_expr.clone()],
            ));
            let (mapped, _) = map_parts_with_config(&functions, &operator, config);
            session.evaluate(mapped)
        }
    }
}

pub(crate) fn eval_map_indexed(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_map_like_call(session, head, args, MapLikeFlavor::MapIndexed) {
        MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
            Ok(expr)
        }
        MapLikeNormalization::Call {
            operator,
            expr,
            config,
        } => Ok(map_indexed_parts_with_config(&expr, &operator, config).0),
    }
}

pub(crate) fn eval_map(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_map_like_call(session, head, args, MapLikeFlavor::Map) {
        MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
            Ok(expr)
        }
        MapLikeNormalization::Call {
            operator,
            expr,
            config,
        } => Ok(map_parts_with_config(&expr, &operator, config).0),
    }
}

pub(crate) fn eval_map_all(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_map_all_call(session, head, args) {
        MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
            Ok(expr)
        }
        MapLikeNormalization::Call {
            operator,
            expr,
            config,
        } => Ok(map_parts_with_mode(&expr, &operator, config, MapTraversalFlavor::MapAll).0),
    }
}

pub(crate) fn eval_scan(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_map_like_call(session, head, args, MapLikeFlavor::Scan) {
        MapLikeNormalization::OperatorForm(expr) | MapLikeNormalization::Immediate(expr) => {
            Ok(expr)
        }
        MapLikeNormalization::Call {
            operator,
            expr,
            config,
        } => {
            let parts = run_with_native_stack_guard(session, |depth| {
                collect_scan_parts_with_config(&expr, config, &depth)
            })?;
            evaluate_scan_parts(session, &operator, parts)
        }
    }
}

pub(crate) fn eval_apply(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_apply_call(session, head, args, ApplyFlavor::Apply) {
        ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => Ok(expr),
        ApplyNormalization::Call {
            operator,
            expr,
            config,
        } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::Apply).0),
    }
}

pub(crate) fn eval_map_apply(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match normalize_apply_call(session, head, args, ApplyFlavor::MapApply) {
        ApplyNormalization::OperatorForm(expr) | ApplyNormalization::Immediate(expr) => Ok(expr),
        ApplyNormalization::Call {
            operator,
            expr,
            config,
        } => Ok(apply_heads_with_config(&expr, &operator, config, ApplyFlavor::MapApply).0),
    }
}
