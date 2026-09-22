use super::*;
use rum_core::IntValue;

/// Each incoming edge counts, including repeated SharedExpr/SharedArgs roots.
/// An explicit work stack makes this independent of the recursive ByteCount
/// implementation. ByteArray is an ordinary call containing encoded text.
pub(in crate::runtime) fn expression(root: &Expr) -> Bytes {
    let mut pending = vec![root];
    let mut total = Bytes::Exact(0);
    while let Some(expr) = pending.pop() {
        total = total.add(Bytes::Exact(size_of::<Expr>()));
        let payload = match expr {
            Expr::Atom(Atom::Integer(IntValue::Small(_))) => Bytes::Exact(0),
            Expr::Atom(Atom::Integer(IntValue::Big(value))) => {
                Bytes::Exact(size_of::<BigInt>()).add(integer_payload(value))
            }
            Expr::Atom(Atom::Rational(value)) => Bytes::Exact(size_of::<Rational>())
                .add(integer_payload(value.numerator()))
                .add(integer_payload(value.denominator())),
            Expr::Atom(Atom::String(value)) => Bytes::Exact(value.capacity()),
            Expr::Atom(Atom::Symbol(value)) => symbol_text(value),
            Expr::Call { head, args } => {
                pending.push(head.as_ref());
                pending.extend(args.iter());
                Bytes::Exact(args.capacity())
                    .release(Bytes::Exact(args.len()))
                    .multiply(size_of::<Expr>())
            }
        };
        total = total.add(payload);
    }
    total
}

fn integer_payload(value: &BigInt) -> Bytes {
    // Signed little-endian serialized length, at least one byte; limb capacity
    // and allocator headers are deliberately outside this historical proxy.
    Bytes::Exact(value.to_signed_bytes_le().len().max(1))
}

pub(super) fn shared_args(args: &SharedArgs) -> Bytes {
    Bytes::Exact(args.capacity())
        .release(Bytes::Exact(args.len()))
        .multiply(size_of::<Expr>())
        .add(sum(args.iter().map(expression)))
}

pub(super) fn symbol_text(value: &Symbol) -> Bytes {
    Bytes::Exact(value.context().len()).add(Bytes::Exact(value.name().len()))
}

pub(super) fn symbol(value: &Symbol) -> Bytes {
    Bytes::Exact(size_of::<Symbol>()).add(symbol_text(value))
}
