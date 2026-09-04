use rum_core::{Expr, SharedArgs};

const RECURSIVE_CONTAINMENT_CALL_DEPTH: usize = 16;

pub(super) fn expr_contains_exact_subexpr(expr: &Expr, needle: &Expr) -> bool {
    expr_contains_exact_subexpr_recursive_bounded(expr, needle, RECURSIVE_CONTAINMENT_CALL_DEPTH)
}

fn expr_contains_exact_subexpr_recursive_bounded(
    expr: &Expr,
    needle: &Expr,
    remaining_call_depth: usize,
) -> bool {
    if expr == needle {
        return true;
    }

    match expr {
        Expr::Call { head, args } if remaining_call_depth == 0 => {
            call_children_contain_exact_subexpr_iterative(head, args, needle)
        }
        Expr::Call { head, args } => {
            let remaining_call_depth = remaining_call_depth
                .checked_sub(1)
                .expect("iterative fallback handles the bounded containment tail");
            expr_contains_exact_subexpr_recursive_bounded(head, needle, remaining_call_depth)
                || args.iter().any(|arg| {
                    expr_contains_exact_subexpr_recursive_bounded(arg, needle, remaining_call_depth)
                })
        }
        Expr::Atom(_) => false,
    }
}

fn call_children_contain_exact_subexpr_iterative(
    head: &Expr,
    args: &SharedArgs,
    needle: &Expr,
) -> bool {
    let mut work = Vec::with_capacity(args.len().saturating_add(1));
    work.extend(args.iter().rev());
    work.push(head);

    while let Some(expr) = work.pop() {
        if expr == needle {
            return true;
        }

        if let Expr::Call { head, args } = expr {
            work.extend(args.iter().rev());
            work.push(head);
        }
    }

    false
}
