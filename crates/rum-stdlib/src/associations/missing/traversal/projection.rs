use super::*;
use rum_core::{IntValue, Rational};

pub(in super::super) fn projected_expr_memory_proxy_bytes(expr: &Expr) -> usize {
    projected_expr_memory_proxy_bytes_with_front_depth(expr, RECURSIVE_DELETE_MISSING_CALL_DEPTH)
}

pub(super) fn projected_expr_memory_proxy_bytes_with_front_depth(
    expr: &Expr,
    front_depth: usize,
) -> usize {
    match projected_expr_memory_proxy_bytes_bounded(expr, front_depth) {
        BoundedTraversal::Complete(bytes) => bytes,
        BoundedTraversal::Exhausted => projected_expr_memory_proxy_bytes_iterative(expr),
    }
}

fn projected_expr_memory_proxy_bytes_bounded(
    expr: &Expr,
    remaining_call_depth: usize,
) -> BoundedTraversal<usize> {
    match expr {
        Expr::Atom(Atom::Integer(IntValue::Small(_))) => {
            BoundedTraversal::Complete(std::mem::size_of::<Expr>())
        }
        Expr::Atom(Atom::Integer(IntValue::Big(value))) => BoundedTraversal::Complete(
            std::mem::size_of::<Expr>()
                .saturating_add(std::mem::size_of::<BigInt>())
                .saturating_add(projected_bigint_payload_bytes(value)),
        ),
        Expr::Atom(Atom::Rational(value)) => BoundedTraversal::Complete(
            std::mem::size_of::<Expr>()
                .saturating_add(std::mem::size_of::<Rational>())
                .saturating_add(projected_bigint_payload_bytes(value.numerator()))
                .saturating_add(projected_bigint_payload_bytes(value.denominator())),
        ),
        Expr::Atom(Atom::String(value)) => {
            BoundedTraversal::Complete(std::mem::size_of::<Expr>().saturating_add(value.len()))
        }
        Expr::Atom(Atom::Symbol(symbol)) => BoundedTraversal::Complete(
            std::mem::size_of::<Expr>()
                .saturating_add(symbol.context().len())
                .saturating_add(symbol.name().len()),
        ),
        Expr::Call { head, args } => {
            let Some(child_call_depth) = remaining_call_depth.checked_sub(1) else {
                return BoundedTraversal::Exhausted;
            };
            let BoundedTraversal::Complete(head_bytes) =
                projected_expr_memory_proxy_bytes_bounded(head, child_call_depth)
            else {
                return BoundedTraversal::Exhausted;
            };
            let mut total = std::mem::size_of::<Expr>().saturating_add(head_bytes);
            for arg in args {
                let BoundedTraversal::Complete(arg_bytes) =
                    projected_expr_memory_proxy_bytes_bounded(arg, child_call_depth)
                else {
                    return BoundedTraversal::Exhausted;
                };
                total = total.saturating_add(arg_bytes);
            }
            BoundedTraversal::Complete(total)
        }
    }
}

#[cold]
#[inline(never)]
fn projected_expr_memory_proxy_bytes_iterative(expr: &Expr) -> usize {
    let mut work = vec![expr];
    let mut total = 0usize;

    while let Some(expr) = work.pop() {
        match expr {
            Expr::Atom(Atom::Integer(IntValue::Small(_))) => {
                total = total.saturating_add(std::mem::size_of::<Expr>());
            }
            Expr::Atom(Atom::Integer(IntValue::Big(value))) => {
                total = total
                    .saturating_add(std::mem::size_of::<Expr>())
                    .saturating_add(std::mem::size_of::<BigInt>())
                    .saturating_add(projected_bigint_payload_bytes(value));
            }
            Expr::Atom(Atom::Rational(value)) => {
                total = total
                    .saturating_add(std::mem::size_of::<Expr>())
                    .saturating_add(std::mem::size_of::<Rational>())
                    .saturating_add(projected_bigint_payload_bytes(value.numerator()))
                    .saturating_add(projected_bigint_payload_bytes(value.denominator()));
            }
            Expr::Atom(Atom::String(value)) => {
                total = total
                    .saturating_add(std::mem::size_of::<Expr>())
                    .saturating_add(value.len());
            }
            Expr::Atom(Atom::Symbol(symbol)) => {
                total = total
                    .saturating_add(std::mem::size_of::<Expr>())
                    .saturating_add(symbol.context().len())
                    .saturating_add(symbol.name().len());
            }
            Expr::Call { head, args } => {
                total = total.saturating_add(std::mem::size_of::<Expr>());
                work.extend(args.iter().rev());
                work.push(head);
            }
        }
    }

    total
}

pub(super) fn projected_bigint_payload_bytes(value: &BigInt) -> usize {
    let bytes = u128::from(value.bits()).saturating_add(7) / 8;
    usize::try_from(bytes.saturating_add(1)).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system_call(name: &str, args: Vec<Expr>) -> Expr {
        Expr::call(system_expr(name), args)
    }

    fn rule(name: &str, value: Expr) -> Expr {
        system_call("Rule", vec![Expr::string(name), value])
    }

    fn list(items: Vec<Expr>) -> Expr {
        system_call("List", items)
    }

    fn association(items: Vec<Expr>) -> Expr {
        system_call("Association", items)
    }

    fn missing() -> Expr {
        system_call("Missing", Vec::new())
    }

    fn expected_projection(expr: &Expr) -> usize {
        let mut work = vec![expr];
        let mut total = 0usize;
        while let Some(expr) = work.pop() {
            total = total.saturating_add(match expr {
                Expr::Atom(Atom::Integer(IntValue::Small(_))) => std::mem::size_of::<Expr>(),
                Expr::Atom(Atom::Integer(IntValue::Big(value))) => std::mem::size_of::<Expr>()
                    .saturating_add(std::mem::size_of::<BigInt>())
                    .saturating_add(projected_bigint_payload_bytes(value)),
                Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
                    .saturating_add(std::mem::size_of::<Rational>())
                    .saturating_add(projected_bigint_payload_bytes(value.numerator()))
                    .saturating_add(projected_bigint_payload_bytes(value.denominator())),
                Expr::Atom(Atom::String(value)) => {
                    std::mem::size_of::<Expr>().saturating_add(value.len())
                }
                Expr::Atom(Atom::Symbol(symbol)) => std::mem::size_of::<Expr>()
                    .saturating_add(symbol.context().len())
                    .saturating_add(symbol.name().len()),
                Expr::Call { head, args } => {
                    work.extend(args.iter().rev());
                    work.push(head);
                    std::mem::size_of::<Expr>()
                }
            });
        }
        total
    }

    #[test]
    fn zero_front_heap_projection_preserves_the_exact_preflight_count() {
        let fixture = list(vec![
            list(vec![missing(), Expr::string("left-survivor")]),
            association(vec![
                rule("drop", missing()),
                rule("keep", Expr::string("right-survivor")),
            ]),
            Expr::string("root-survivor"),
        ]);

        assert_eq!(
            projected_expr_memory_proxy_bytes_with_front_depth(&fixture, 0),
            expected_projection(&fixture)
        );
    }
}
