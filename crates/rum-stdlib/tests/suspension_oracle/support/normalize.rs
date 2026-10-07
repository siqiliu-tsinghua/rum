use rum_core::{Atom, Expr};

/// Normalize only a declared duration leaf after checking its numeric shape.
/// Rebuilding the path preserves all heads, siblings, order and typed values.
pub fn duration(value: &Expr, path: &[usize]) -> Expr {
    if let Some((&index, rest)) = path.split_first() {
        let Expr::Call { head, args } = value else {
            panic!("duration path crosses atom {value}")
        };
        let mut args = args.to_vec();
        args[index] = duration(&args[index], rest);
        Expr::call(head.as_ref().clone(), args)
    } else {
        assert!(
            matches!(value, Expr::Atom(Atom::Integer(_) | Atom::Rational(_))),
            "duration is not exact numeric: {value}"
        );
        assert!(
            !value.to_string().starts_with('-'),
            "negative elapsed duration"
        );
        Expr::small_integer(0)
    }
}
