use super::*;
use std::mem::size_of;

fn assert_expression(expr: &Expr, expected: usize) {
    assert_eq!(expression(expr), Bytes::Exact(expected));
    let actual = Session::empty()
        .eval_byte_count(&system_expr("ByteCount"), std::slice::from_ref(expr))
        .unwrap();
    assert_eq!(actual, Expr::integer(BigInt::from(expected)));
}

#[test]
fn empty_and_installed_sessions_match_without_sampling_the_peak() {
    for session in [Session::empty(), Session::new()] {
        let model = assert_model(&session);
        assert_eq!(model.session_inline, Bytes::Exact(size_of::<Session>()));
        assert_eq!(
            model.field("resource_limits"),
            Bytes::Exact(size_of::<ResourceLimits>())
        );
        let expected = 2 * size_of::<usize>() + size_of::<RequestInterruptState>();
        assert_eq!(
            model.field("request_interrupt_control"),
            Bytes::Exact(expected)
        );
    }
}

#[test]
fn expressions_charge_each_shared_occurrence_and_spare_slot() {
    let text = reserved("abc", 97);
    let string_bytes = size_of::<Expr>() + text.capacity();
    let mut args = Vec::with_capacity(5);
    args.push(Expr::string(text));
    let head = SharedExpr::new(Expr::small_integer(0));
    let shared_args = SharedArgs::new(args);
    let subtree = Expr::call_shared(head.clone(), shared_args.clone());
    let duplicate = Expr::call_shared(head.clone(), shared_args.clone());
    let Expr::Call {
        head: duplicate_head,
        args: duplicate_args,
    } = &duplicate
    else {
        unreachable!()
    };
    assert_eq!(head.ptr_key(), duplicate_head.ptr_key());
    assert_eq!(shared_args.ptr_key(), duplicate_args.ptr_key());
    let subtree_bytes =
        2 * size_of::<Expr>() + string_bytes + (shared_args.capacity() - 1) * size_of::<Expr>();
    assert_expression(&subtree, subtree_bytes);
    let tree = Expr::call(Expr::small_integer(1), vec![subtree.clone(), duplicate]);
    assert_expression(&tree, 2 * size_of::<Expr>() + 2 * subtree_bytes);

    let mut session = Session::empty();
    session
        .once_values
        .insert(String::from("a"), subtree.clone());
    session.once_values.insert(String::from("b"), subtree);
    assert_eq!(
        assert_model(&session).field("once_values"),
        Bytes::Exact(2 + 2 * subtree_bytes)
    );
}

#[test]
fn numeric_payloads_follow_signed_bytes_not_limb_capacity() {
    assert_expression(&Expr::small_integer(0), size_of::<Expr>());
    for (value, encoded_len) in [
        (BigInt::from(1u8) << 80usize, 11),
        (-(BigInt::from(1u8) << 80usize), 11),
    ] {
        assert_expression(
            &Expr::integer(value),
            size_of::<Expr>() + size_of::<BigInt>() + encoded_len,
        );
    }
    let value = Rational::new(BigInt::from(1), BigInt::from(2)).unwrap();
    assert_expression(
        &Expr::rational(value),
        size_of::<Expr>() + size_of::<Rational>() + 2,
    );
}

#[test]
fn byte_array_charges_encoded_capacity_for_valid_invalid_and_nonstring_payloads() {
    let head = system_expr("ByteArray");
    let head_bytes = size_of::<Expr>() + "System`".len() + "ByteArray".len();
    for text in ["", "AQID", "AQ==", "not-base64"] {
        let encoded = reserved(text, 83);
        let capacity = encoded.capacity();
        let mut args = Vec::with_capacity(4);
        args.push(Expr::string(encoded));
        let expr = Expr::call(head.clone(), args);
        assert_expression(
            &expr,
            size_of::<Expr>() + head_bytes + 4 * size_of::<Expr>() + capacity,
        );
    }
    assert_expression(
        &Expr::call(head, vec![Expr::small_integer(7)]),
        2 * size_of::<Expr>() + head_bytes,
    );
}

#[test]
fn container_capacity_and_legacy_length_terms_are_distinct() {
    let mut session = Session::empty();
    session.prints = Vec::with_capacity(7);
    session.prints.push(reserved("p", 101));
    session.prints.push(reserved("q", 103));
    session.trace_event_depths = Vec::with_capacity(9);
    session.trace_event_depths.push(1);
    session.declared_stubs.reserve(30);
    let symbol = Symbol::from_name("kept");
    let symbol_bytes = size_of::<Symbol>() + symbol.context().len() + symbol.name().len();
    session.declared_stubs.insert(symbol.clone());
    session.stable_symbol_cache.reserve(30);
    session.stable_symbol_cache.insert(
        symbol.clone(),
        SymbolStabilityCacheEntry {
            generation: 0,
            stable: true,
        },
    );
    let mut path = PathBuf::with_capacity(111);
    path.push("file.rum");
    session.load_stack = Vec::with_capacity(3);
    session.load_stack.push(path);
    let model = assert_model(&session);
    assert_eq!(
        model.field("prints"),
        Bytes::Exact(session.prints.capacity() * size_of::<String>() + 204)
    );
    assert_eq!(
        model.field("trace_event_depths"),
        Bytes::Exact(session.trace_event_depths.capacity() * size_of::<usize>())
    );
    assert_eq!(model.field("declared_stubs"), Bytes::Exact(symbol_bytes));
    assert_eq!(
        model.field("stable_symbol_cache"),
        Bytes::Exact(
            session.stable_symbol_cache.capacity()
                * size_of::<(Symbol, SymbolStabilityCacheEntry)>()
                + symbol.context().len()
                + symbol.name().len()
        )
    );
    assert_eq!(
        model.field("load_stack"),
        Bytes::Exact(session.load_stack.capacity() * size_of::<PathBuf>() + "file.rum".len())
    );
}

#[test]
fn exception_registry_and_exception_expression_payloads_are_exact() {
    let mut session = Session::empty();
    session
        .exception_types
        .insert(reserved("Child", 97), Some(reserved("Parent", 131)));
    session.exception_types.insert(reserved("Root", 89), None);
    let model = assert_model(&session);
    assert_eq!(
        model.field("exception_types"),
        Bytes::Exact(2 * size_of::<(String, Option<String>)>() + 97 + 131 + 89)
    );
    let exception = Expr::call(
        system_expr("Exception"),
        vec![
            Expr::string(reserved("Child", 47)),
            Expr::string(reserved("detail", 79)),
        ],
    );
    let bytes = 4 * size_of::<Expr>() + "System`Exception".len() + 47 + 79;
    assert_expression(&exception, bytes);
    session
        .once_values
        .insert(String::from("exception"), exception);
    assert_eq!(
        assert_model(&session).field("once_values"),
        Bytes::Exact("exception".len() + bytes)
    );
}
