use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
mod resume_frames;
mod target;
mod traversal;
use resume_frames::*;
use target::*;
use traversal::*;

use super::*;
use rum_core::SharedArgs;

const DELAYED_DELETE_MISSING_WRAPPER_PROXY_BYTES: usize = 64 * std::mem::size_of::<Expr>();

pub(crate) fn eval_missing(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_delete_missing(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, mode) = match args {
        [expr] => (
            expr,
            DeleteMissingMode::Recursive(DeleteMissingDepth::Finite(1)),
        ),
        [expr, depth_expr] => {
            let Some(depth) = parse_delete_missing_positive_depth(depth_expr) else {
                return Ok(session.invalid_call_with_message(
                    "DeleteMissing",
                    args,
                    "arg2",
                    delete_missing_arg2_message(depth_expr),
                ));
            };
            (expr, DeleteMissingMode::Recursive(depth))
        }
        [expr, level_expr, scan_expr] => {
            let Some(level) = parse_delete_missing_positive_depth(level_expr) else {
                return Ok(session.invalid_call_with_message(
                    "DeleteMissing",
                    args,
                    "arg2",
                    delete_missing_arg2_message(level_expr),
                ));
            };
            let Some(scan) = parse_delete_missing_scan_depth(scan_expr) else {
                return Ok(session.invalid_call_with_message(
                    "DeleteMissing",
                    args,
                    "arg3",
                    delete_missing_arg3_message(scan_expr),
                ));
            };
            (expr, DeleteMissingMode::Target { level, scan })
        }
        _ => {
            return Ok(session.invalid_call_with_message(
                "DeleteMissing",
                args,
                "argb",
                delete_missing_arg_count_message(args.len()),
            ));
        }
    };

    enforce_delete_missing_memory_constraint(session, expr, mode)?;

    let Some(value) = delete_missing_from_keyed_data(session, expr, mode)? else {
        return Ok(session.invalid_call_with_message(
            "DeleteMissing",
            args,
            "invrp",
            invalid_association_or_list_message(expr),
        ));
    };

    Ok(value)
}

fn enforce_delete_missing_memory_constraint(
    session: &Session,
    expr: &Expr,
    mode: DeleteMissingMode,
) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let keyed_input = list_items(expr).is_some()
        || matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Association"));
    if !keyed_input {
        return Ok(());
    }

    let wrapper_count = delete_missing_delayed_wrapper_count(expr, mode);
    let projected_bytes = projected_expr_memory_proxy_bytes(expr)
        .saturating_add(wrapper_count.saturating_mul(DELAYED_DELETE_MISSING_WRAPPER_PROXY_BYTES));
    session.enforce_active_memory_limit_for_projected_bytes(projected_bytes)
}

fn delete_missing_delayed_wrapper_count(expr: &Expr, mode: DeleteMissingMode) -> usize {
    match mode {
        DeleteMissingMode::Recursive(depth) => recursive_delayed_wrapper_count(expr, depth),
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Infinity,
            ..
        } => recursive_delayed_wrapper_count(expr, DeleteMissingDepth::Infinity),
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Finite(level),
            ..
        } => target_delayed_wrapper_count(expr, level),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum DeleteMissingMode {
    Recursive(DeleteMissingDepth),
    Target {
        level: DeleteMissingDepth,
        scan: DeleteMissingDepth,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum DeleteMissingDepth {
    Finite(usize),
    Infinity,
}

impl DeleteMissingDepth {
    fn descend(self) -> Option<Self> {
        match self {
            Self::Finite(depth) if depth > 1 => Some(Self::Finite(depth - 1)),
            Self::Finite(_) => None,
            Self::Infinity => Some(Self::Infinity),
        }
    }

    fn descend_scan(self) -> Option<Self> {
        match self {
            Self::Finite(depth) if depth > 0 => Some(Self::Finite(depth - 1)),
            Self::Finite(_) => None,
            Self::Infinity => Some(Self::Infinity),
        }
    }

    fn expr(self) -> Expr {
        match self {
            Self::Finite(depth) => Expr::integer(BigInt::from(depth)),
            Self::Infinity => system_expr("Infinity"),
        }
    }
}

fn parse_delete_missing_positive_depth(expr: &Expr) -> Option<DeleteMissingDepth> {
    if is_system_symbol_name(expr, "Infinity") || is_system_symbol_name(expr, "All") {
        return Some(DeleteMissingDepth::Infinity);
    }

    expr_as_non_negative_usize(expr)
        .filter(|depth| *depth > 0)
        .map(DeleteMissingDepth::Finite)
}

fn parse_delete_missing_scan_depth(expr: &Expr) -> Option<DeleteMissingDepth> {
    if is_system_symbol_name(expr, "Infinity") || is_system_symbol_name(expr, "All") {
        return Some(DeleteMissingDepth::Infinity);
    }

    expr_as_non_negative_usize(expr).map(DeleteMissingDepth::Finite)
}

fn delete_missing_arg2_message(expr: &Expr) -> String {
    format!("The second argument {expr} is expected to be a positive integer or Infinity.")
}

fn delete_missing_arg3_message(expr: &Expr) -> String {
    format!("The third argument {expr} is expected to be a non-negative integer or Infinity.")
}

fn delete_missing_arg_count_message(count: usize) -> String {
    format!("DeleteMissing called with {count} arguments; between 1 and 3 arguments are expected.")
}

fn delete_missing_from_keyed_data(
    session: &mut Session,
    expr: &Expr,
    mode: DeleteMissingMode,
) -> Result<Option<Expr>, EvalError> {
    match mode {
        DeleteMissingMode::Recursive(depth) => {
            Ok(delete_missing_from_keyed_data_recursive(expr, depth))
        }
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Infinity,
            ..
        } => Ok(delete_missing_from_keyed_data_recursive(
            expr,
            DeleteMissingDepth::Infinity,
        )),
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Finite(level),
            scan,
        } => delete_missing_at_target_level(session, expr, level, scan),
    }
}

fn delete_missing_delayed_wrapper(value: Expr, mode: DeleteMissingMode) -> Expr {
    let local = system_expr("rum$deleteMissingValue");
    let cleanup = Expr::call(
        system_expr("DeleteMissing"),
        delete_missing_call_args(local.clone(), mode),
    );
    let should_cleanup = Expr::call(
        system_expr("Or"),
        vec![
            Expr::call(system_expr("ListQ"), vec![local.clone()]),
            Expr::call(system_expr("AssociationQ"), vec![local.clone()]),
        ],
    );

    Expr::call(
        system_expr("With"),
        vec![
            Expr::call(
                system_expr("List"),
                vec![Expr::call(system_expr("Rule"), vec![local.clone(), value])],
            ),
            Expr::call(system_expr("If"), vec![should_cleanup, cleanup, local]),
        ],
    )
}

fn delete_missing_call_args(expr: Expr, mode: DeleteMissingMode) -> Vec<Expr> {
    match mode {
        DeleteMissingMode::Recursive(depth) => vec![expr, depth.expr()],
        DeleteMissingMode::Target { level, scan } => vec![expr, level.expr(), scan.expr()],
    }
}

fn rule_like_expr_has_missing_value(expr: &Expr) -> bool {
    parse_association_entry_form(expr).is_some_and(|entry| is_missing_expr(&entry.value))
}
