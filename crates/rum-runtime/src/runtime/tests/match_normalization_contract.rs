use super::*;

const PLAIN: &str = "normalizerPlain";
const FLAT: &str = "normalizerFlat";
const ORDERLESS: &str = "normalizerOrderless";
const FLAT_ORDERLESS: &str = "normalizerFlatOrderless";

struct CorpusCase {
    name: &'static str,
    input: Expr,
    changed: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct CallArcOrigin {
    head_input_paths: Vec<String>,
    args_input_paths: Vec<String>,
}

struct InputCall<'a> {
    path: String,
    head: &'a SharedExpr,
    args: &'a SharedArgs,
}

fn contract_session() -> Session {
    let mut session = Session::new();
    session.set_attribute_set_for_symbol(
        Symbol::from_name(FLAT),
        AttributeSet::from_attribute(Attribute::Flat),
    );
    session.set_attribute_set_for_symbol(
        Symbol::from_name(ORDERLESS),
        AttributeSet::from_attribute(Attribute::Orderless),
    );
    session.set_attribute_set_for_symbol(
        Symbol::from_name(FLAT_ORDERLESS),
        AttributeSet::from_attribute(Attribute::Flat).with(Attribute::Orderless),
    );

    // This makes Function[Orderless, 0] normalize to Function[0, Orderless].
    // The former is an explicit Function with no attributes; the latter is a
    // slot Function with Orderless, so a parent proves it consults the changed head.
    let function = Symbol::system("Function");
    let function_attributes = session
        .attribute_set_for_symbol(&function)
        .with(Attribute::Orderless);
    session.set_attribute_set_for_symbol(function, function_attributes);
    session
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::call(Expr::symbol(name), args)
}

fn flat(args: Vec<Expr>) -> Expr {
    call(FLAT, args)
}

fn orderless_change() -> Expr {
    call(
        ORDERLESS,
        vec![Expr::small_integer(2), Expr::small_integer(1)],
    )
}

fn inert_child(name: &str) -> Expr {
    call("normalizerLeaf", vec![Expr::symbol(name)])
}

fn changed_argument_root(index: usize) -> Expr {
    let mut args = vec![inert_child("a"), inert_child("b"), inert_child("c")];
    args[index] = orderless_change();
    call(PLAIN, args)
}

fn changed_argument_spine(depth: usize) -> Expr {
    assert!(depth > 0);
    let mut expr = orderless_change();
    for _ in 1..depth {
        expr = call(PLAIN, vec![expr]);
    }
    expr
}

fn unchanged_argument_spine(depth: usize) -> Expr {
    assert!(depth > 0);
    let mut expr = inert_child("argumentSpineLeaf");
    for _ in 1..depth {
        expr = call(PLAIN, vec![expr]);
    }
    expr
}

fn changed_head_spine(depth: usize) -> Expr {
    assert!(depth > 0);
    let mut expr = orderless_change();
    for _ in 1..depth {
        expr = Expr::call(expr, vec![Expr::symbol("spineArg")]);
    }
    expr
}

fn unchanged_head_spine(depth: usize) -> Expr {
    assert!(depth > 0);
    let mut expr = inert_child("headSpineLeaf");
    for _ in 1..depth {
        expr = Expr::call(expr, vec![Expr::symbol("spineArg")]);
    }
    expr
}

fn nested_flat_chain() -> Expr {
    flat(vec![
        Expr::small_integer(0),
        flat(vec![
            Expr::small_integer(1),
            flat(vec![
                Expr::small_integer(2),
                flat(vec![Expr::small_integer(3), Expr::small_integer(4)]),
            ]),
        ]),
        Expr::small_integer(5),
    ])
}

fn normalized_function_head_root() -> Expr {
    let head = Expr::call(
        system_expr("Function"),
        vec![system_expr("Orderless"), Expr::small_integer(0)],
    );
    Expr::call(
        head,
        vec![
            Expr::small_integer(3),
            Expr::small_integer(1),
            Expr::small_integer(2),
        ],
    )
}

fn corpus_case(name: &'static str, input: Expr, changed: bool) -> CorpusCase {
    CorpusCase {
        name,
        input,
        changed,
    }
}

fn corpus() -> Vec<CorpusCase> {
    vec![
        corpus_case("atom", Expr::small_integer(7), false),
        corpus_case("empty-call", call(PLAIN, vec![]), false),
        corpus_case("unary-call", call(PLAIN, vec![inert_child("unary")]), false),
        corpus_case(
            "wide-call",
            call(
                PLAIN,
                (0..12)
                    .map(|index| inert_child(&index.to_string()))
                    .collect(),
            ),
            false,
        ),
        corpus_case("changed-argument-first", changed_argument_root(0), true),
        corpus_case("changed-argument-middle", changed_argument_root(1), true),
        corpus_case("changed-argument-last", changed_argument_root(2), true),
        corpus_case(
            "unchanged-root",
            call(PLAIN, vec![inert_child("unchanged")]),
            false,
        ),
        corpus_case(
            "head-only-changed-root",
            Expr::call(orderless_change(), vec![inert_child("argument")]),
            true,
        ),
        corpus_case(
            "args-only-changed-root",
            call(PLAIN, vec![orderless_change()]),
            true,
        ),
        corpus_case(
            "both-changed-root",
            Expr::call(orderless_change(), vec![orderless_change()]),
            true,
        ),
        corpus_case(
            "flat",
            flat(vec![
                Expr::small_integer(0),
                flat(vec![Expr::small_integer(1), Expr::small_integer(2)]),
                Expr::small_integer(3),
            ]),
            true,
        ),
        corpus_case(
            "orderless",
            call(
                ORDERLESS,
                vec![
                    Expr::small_integer(3),
                    Expr::small_integer(1),
                    Expr::small_integer(2),
                ],
            ),
            true,
        ),
        corpus_case(
            "orderless-already-canonical",
            call(
                ORDERLESS,
                vec![Expr::small_integer(1), Expr::small_integer(2)],
            ),
            false,
        ),
        corpus_case(
            "flat-and-orderless",
            call(
                FLAT_ORDERLESS,
                vec![
                    Expr::small_integer(3),
                    call(
                        FLAT_ORDERLESS,
                        vec![Expr::small_integer(2), Expr::small_integer(1)],
                    ),
                    Expr::small_integer(0),
                ],
            ),
            true,
        ),
        corpus_case("nested-flat-chain", nested_flat_chain(), true),
        corpus_case(
            "normalized-function-head",
            normalized_function_head_root(),
            true,
        ),
        corpus_case("argument-spine-17", changed_argument_spine(17), true),
        corpus_case(
            "unchanged-argument-spine-17",
            unchanged_argument_spine(17),
            false,
        ),
        corpus_case("argument-spine-65", changed_argument_spine(65), true),
        corpus_case(
            "unchanged-argument-spine-65",
            unchanged_argument_spine(65),
            false,
        ),
        corpus_case("head-spine-17", changed_head_spine(17), true),
        corpus_case("unchanged-head-spine-17", unchanged_head_spine(17), false),
        corpus_case("head-spine-65", changed_head_spine(65), true),
        corpus_case("unchanged-head-spine-65", unchanged_head_spine(65), false),
    ]
}

// Frozen test-only snapshot of resolution.rs's recursive normalizer. It is an
// equivalence oracle for the coming rewrite, not a helper for production use or
// a second implementation to evolve independently.
fn frozen_normalize_match_expr(session: &Session, expr: &Expr) -> Expr {
    frozen_normalize_match_expr_changed(session, expr).unwrap_or_else(|| expr.clone())
}

fn frozen_normalize_match_expr_changed(session: &Session, expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(_) => None,
        Expr::Call {
            head: shared_head,
            args,
            ..
        } => {
            let head = shared_head.as_ref();
            let normalized_head = frozen_normalize_match_expr_changed(session, head);
            let head_for_attributes = normalized_head.as_ref().unwrap_or(head);
            let normalized_args =
                frozen_normalize_match_arg_list_changed(session, args, head_for_attributes);

            match (normalized_head, normalized_args) {
                (None, None) => None,
                (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
                (None, Some(args)) => Some(Expr::call_shared(
                    shared_head.clone(),
                    SharedArgs::new(args),
                )),
                (Some(head), Some(args)) => Some(Expr::call(head, args)),
            }
        }
    }
}

fn frozen_normalize_match_arg_list_changed(
    session: &Session,
    args: &SharedArgs,
    normalized_head: &Expr,
) -> Option<Vec<Expr>> {
    let mut normalized_args: Option<Vec<Expr>> = None;

    for (index, arg) in args.iter().enumerate() {
        if let Some(normalized_arg) = frozen_normalize_match_expr_changed(session, arg) {
            normalized_args
                .get_or_insert_with(|| args[..index].to_vec())
                .push(normalized_arg);
        } else if let Some(normalized_args) = &mut normalized_args {
            normalized_args.push(arg.clone());
        }
    }

    frozen_finish_match_arg_list(session, args, normalized_head, normalized_args)
}

fn frozen_finish_match_arg_list(
    session: &Session,
    args: &SharedArgs,
    normalized_head: &Expr,
    normalized_args: Option<Vec<Expr>>,
) -> Option<Vec<Expr>> {
    let attributes = session.attribute_set_for_head(normalized_head);
    if !attributes.contains(Attribute::Flat) && !attributes.contains(Attribute::Orderless) {
        return normalized_args;
    }

    let args_for_attribute_normalization = normalized_args.unwrap_or_else(|| args.to_vec());
    let attribute_normalized_args = session.normalize_call_args_by_attributes(
        normalized_head,
        args_for_attribute_normalization,
        attributes,
    );

    if attribute_normalized_args.as_slice() == args.as_ref() {
        None
    } else {
        Some(attribute_normalized_args)
    }
}

fn frozen_normalize_definition_identity(
    session: &Session,
    expr: &Expr,
    attributes: AttributeSet,
) -> Expr {
    if matches!(attributes.hold_attribute(), HoldAttribute::None) {
        return expr.clone();
    }

    let Expr::Call { head, args, .. } = expr else {
        return expr.clone();
    };
    let normalized_head = frozen_normalize_match_expr_changed(session, head);
    let mut normalized_args: Option<Vec<Expr>> = None;
    for (index, arg) in args.iter().enumerate() {
        if let Some(normalized_arg) = frozen_normalize_match_expr_changed(session, arg) {
            normalized_args
                .get_or_insert_with(|| args[..index].to_vec())
                .push(normalized_arg);
        } else if let Some(normalized_args) = &mut normalized_args {
            normalized_args.push(arg.clone());
        }
    }

    match (normalized_head, normalized_args) {
        (None, None) => expr.clone(),
        (Some(head), None) => Expr::call_with_shared_args(head, args.clone()),
        (None, Some(args)) => Expr::call_shared(head.clone(), SharedArgs::new(args)),
        (Some(head), Some(args)) => Expr::call(head, args),
    }
}

fn logical_expr_eq(lhs: &Expr, rhs: &Expr) -> bool {
    match (lhs, rhs) {
        (Expr::Atom(lhs), Expr::Atom(rhs)) => lhs == rhs,
        (
            Expr::Call {
                head: lhs_head,
                args: lhs_args,
                ..
            },
            Expr::Call {
                head: rhs_head,
                args: rhs_args,
                ..
            },
        ) => {
            logical_expr_eq(lhs_head, rhs_head)
                && lhs_args.len() == rhs_args.len()
                && lhs_args
                    .iter()
                    .zip(rhs_args.iter())
                    .all(|(lhs, rhs)| logical_expr_eq(lhs, rhs))
        }
        _ => false,
    }
}

fn logical_slice_eq(lhs: &[Expr], rhs: &[Expr]) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .zip(rhs.iter())
            .all(|(lhs, rhs)| logical_expr_eq(lhs, rhs))
}

fn collect_input_calls<'a>(expr: &'a Expr, path: String, calls: &mut Vec<InputCall<'a>>) {
    let Expr::Call { head, args, .. } = expr else {
        return;
    };
    calls.push(InputCall {
        path: path.clone(),
        head,
        args,
    });
    collect_input_calls(head, format!("{path}.h"), calls);
    for (index, arg) in args.iter().enumerate() {
        collect_input_calls(arg, format!("{path}.a{index}"), calls);
    }
}

fn collect_output_origins(expr: &Expr, inputs: &[InputCall<'_>], origins: &mut Vec<CallArcOrigin>) {
    let Expr::Call { head, args, .. } = expr else {
        return;
    };
    origins.push(CallArcOrigin {
        head_input_paths: inputs
            .iter()
            .filter(|input| SharedExpr::ptr_eq(head, input.head))
            .map(|input| input.path.clone())
            .collect(),
        args_input_paths: inputs
            .iter()
            .filter(|input| SharedArgs::ptr_eq(args, input.args))
            .map(|input| input.path.clone())
            .collect(),
    });
    collect_output_origins(head, inputs, origins);
    for arg in args {
        collect_output_origins(arg, inputs, origins);
    }
}

fn arc_origin_signature(input: &Expr, output: &Expr) -> Vec<CallArcOrigin> {
    let mut inputs = Vec::new();
    collect_input_calls(input, "root".to_string(), &mut inputs);
    let mut origins = Vec::new();
    collect_output_origins(output, &inputs, &mut origins);
    origins
}

fn call_parts(expr: &Expr) -> (&SharedExpr, &SharedArgs) {
    let Expr::Call { head, args, .. } = expr else {
        panic!("expected call, got {expr}");
    };
    (head, args)
}

fn assert_root_arc_origin(
    case: &str,
    input: &Expr,
    output: &Expr,
    head_from_input: bool,
    args_from_input: bool,
) {
    let (input_head, input_args) = call_parts(input);
    let (output_head, output_args) = call_parts(output);
    assert_eq!(
        SharedExpr::ptr_eq(input_head, output_head),
        head_from_input,
        "{case}: head Arc origin"
    );
    assert_eq!(
        SharedArgs::ptr_eq(input_args, output_args),
        args_from_input,
        "{case}: args Arc origin"
    );
}

fn assert_producer_matches_frozen_reference(
    case: &CorpusCase,
    producer: &str,
    production: &Option<Expr>,
    reference: &Option<Expr>,
) {
    assert_eq!(
        production.is_some(),
        reference.is_some(),
        "{} ({producer}): Option discriminant differs from frozen reference",
        case.name
    );
    let production_output = production.as_ref().unwrap_or(&case.input);
    let reference_output = reference.as_ref().unwrap_or(&case.input);
    assert!(
        logical_expr_eq(production_output, reference_output),
        "{} ({producer}): logical expression differs\nproduction: {}\n reference: {}",
        case.name,
        production_output,
        reference_output
    );
    assert_eq!(
        arc_origin_signature(&case.input, production_output),
        arc_origin_signature(&case.input, reference_output),
        "{} ({producer}): input Arc reuse differs",
        case.name
    );
}

#[test]
fn match_normalization_contract_corpus_matches_frozen_reference() {
    let session = contract_session();
    for case in corpus() {
        let production = session.normalize_match_expr_changed_for_test(&case.input);
        let reference = frozen_normalize_match_expr_changed(&session, &case.input);

        assert_eq!(
            production.is_some(),
            reference.is_some(),
            "{}: Option discriminant differs from frozen reference",
            case.name
        );
        assert_eq!(
            production.is_some(),
            case.changed,
            "{}: unexpected frozen Option contract",
            case.name
        );

        let production_output = session.normalize_match_expr(&case.input);
        let reference_output = frozen_normalize_match_expr(&session, &case.input);
        assert!(
            logical_expr_eq(&production_output, &reference_output),
            "{}: logical expression differs\nproduction: {}\n reference: {}",
            case.name,
            production_output,
            reference_output
        );
        assert_eq!(
            arc_origin_signature(&case.input, &production_output),
            arc_origin_signature(&case.input, &reference_output),
            "{}: input Arc reuse differs",
            case.name
        );

        for budget in [0, 1, 15, 16, 17] {
            let bounded =
                session.normalize_match_expr_changed_with_call_depth_for_test(&case.input, budget);
            assert_producer_matches_frozen_reference(
                &case,
                &format!("bounded entry at budget {budget}"),
                &bounded,
                &reference,
            );
        }

        let iterative = session.normalize_match_expr_changed_iterative_for_test(&case.input);
        assert_producer_matches_frozen_reference(
            &case,
            "iterative fallback",
            &iterative,
            &reference,
        );
    }
}

#[test]
fn match_normalization_contract_bounded_front_handles_exhaustion_locally() {
    let session = contract_session();
    let atom = Expr::small_integer(1);
    let atom_outcome = session.normalize_match_expr_changed_recursive_bounded_for_test(&atom, 0);
    assert!(atom_outcome.is_none());

    let one_call = unchanged_argument_spine(1);
    assert!(
        session
            .normalize_match_expr_changed_recursive_bounded_for_test(&one_call, 0)
            .is_none(),
        "a zero-budget call must complete unchanged through local fallback"
    );
    assert!(
        session
            .normalize_match_expr_changed_recursive_bounded_for_test(&one_call, 1)
            .is_none(),
        "one call must complete unchanged on a one-call budget"
    );

    let depth_cases = [
        ("changed argument", changed_argument_spine(17), true),
        ("unchanged argument", unchanged_argument_spine(17), false),
        ("changed head", changed_head_spine(17), true),
        ("unchanged head", unchanged_head_spine(17), false),
    ];
    for (name, input, changed) in depth_cases {
        let case = corpus_case(name, input, changed);
        let reference = frozen_normalize_match_expr_changed(&session, &case.input);
        let fallback =
            session.normalize_match_expr_changed_recursive_bounded_for_test(&case.input, 16);
        assert_eq!(fallback.is_some(), changed, "{name}: fallback outcome");
        assert_producer_matches_frozen_reference(
            &case,
            "depth 17 at a 16-call budget",
            &fallback,
            &reference,
        );

        let completed =
            session.normalize_match_expr_changed_recursive_bounded_for_test(&case.input, 17);
        assert_eq!(completed.is_some(), changed, "{name}: completed outcome");
        assert_producer_matches_frozen_reference(
            &case,
            "depth 17 at a 17-call budget",
            &completed,
            &reference,
        );
    }
}

#[test]
fn match_normalization_contract_rebuild_arms_preserve_root_arc_origins() {
    let session = contract_session();
    let cases = [
        (
            "unchanged",
            call(PLAIN, vec![inert_child("unchanged")]),
            false,
            true,
            true,
        ),
        (
            "head-only",
            Expr::call(orderless_change(), vec![inert_child("argument")]),
            true,
            false,
            true,
        ),
        (
            "args-only",
            call(PLAIN, vec![orderless_change()]),
            true,
            true,
            false,
        ),
        (
            "both",
            Expr::call(orderless_change(), vec![orderless_change()]),
            true,
            false,
            false,
        ),
    ];

    for (name, input, changed, head_from_input, args_from_input) in cases {
        let outcome = session.normalize_match_expr_changed_for_test(&input);
        assert_eq!(outcome.is_some(), changed, "{name}: Option discriminant");
        let output = outcome.as_ref().unwrap_or(&input);
        assert_root_arc_origin(name, &input, output, head_from_input, args_from_input);
    }
}

#[test]
fn match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs() {
    let session = contract_session();
    for changed_index in 0..3 {
        let input = changed_argument_root(changed_index);
        let output = session
            .normalize_match_expr_changed_for_test(&input)
            .expect("changed child must rebuild the root");
        let (_, input_args) = call_parts(&input);
        let (_, output_args) = call_parts(&output);

        for index in 0..3 {
            let (input_head, input_child_args) = call_parts(&input_args[index]);
            let (output_head, output_child_args) = call_parts(&output_args[index]);
            assert!(
                SharedExpr::ptr_eq(input_head, output_head),
                "changed index {changed_index}, child {index}: head Arc"
            );
            assert_eq!(
                SharedArgs::ptr_eq(input_child_args, output_child_args),
                index != changed_index,
                "changed index {changed_index}, child {index}: args Arc"
            );
        }
    }
}

#[test]
fn match_normalization_contract_definition_identity_hold_gate_and_four_arms() {
    let session = contract_session();
    let held = AttributeSet::from_hold(HoldAttribute::HoldAll);
    let cases = [
        (
            "hold-none-early-return",
            Expr::call(orderless_change(), vec![orderless_change()]),
            AttributeSet::empty(),
            true,
            true,
        ),
        (
            "held-unchanged",
            call(PLAIN, vec![inert_child("unchanged")]),
            held,
            true,
            true,
        ),
        (
            "held-head-only",
            Expr::call(orderless_change(), vec![inert_child("argument")]),
            held,
            false,
            true,
        ),
        (
            "held-args-only",
            call(PLAIN, vec![orderless_change()]),
            held,
            true,
            false,
        ),
        (
            "held-both",
            Expr::call(orderless_change(), vec![orderless_change()]),
            held,
            false,
            false,
        ),
    ];

    for (name, input, attributes, head_from_input, args_from_input) in cases {
        let production = session.normalize_definition_identity_for_matching(&input, attributes);
        let reference = frozen_normalize_definition_identity(&session, &input, attributes);
        assert!(
            logical_expr_eq(&production, &reference),
            "{name}: logical definition identity"
        );
        assert_eq!(
            arc_origin_signature(&input, &production),
            arc_origin_signature(&input, &reference),
            "{name}: definition input Arc reuse"
        );
        assert_root_arc_origin(name, &input, &production, head_from_input, args_from_input);
    }

    let atom = Expr::symbol("heldAtom");
    let production = session.normalize_definition_identity_for_matching(&atom, held);
    let reference = frozen_normalize_definition_identity(&session, &atom, held);
    assert!(logical_expr_eq(&production, &reference));
}

#[test]
fn match_normalization_contract_argument_finisher_collapses_equal_changed_prefix() {
    let session = contract_session();
    let input = call(
        ORDERLESS,
        vec![Expr::small_integer(1), Expr::small_integer(2)],
    );
    let (head, args) = call_parts(&input);
    let synthetic_changed_prefix = Some(vec![args[0].clone(), args[1].clone()]);

    assert!(synthetic_changed_prefix.is_some());
    assert!(
        frozen_finish_match_arg_list(&session, args, head, synthetic_changed_prefix).is_none(),
        "attribute equality must collapse a materialized changed prefix back to None"
    );
}

#[test]
fn match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened() {
    let session = contract_session();
    let input = nested_flat_chain();
    let (head, args) = call_parts(&input);
    let one_level = flatten_flat_call_args(head, args.to_vec());
    let expected_args = (0..=5).map(Expr::small_integer).collect::<Vec<_>>();
    assert!(
        !logical_slice_eq(&one_level, &expected_args),
        "one parent-first flatten must leave a nested Flat call"
    );

    let output = session
        .normalize_match_expr_changed_for_test(&input)
        .expect("nested Flat chain must change");
    let (_, output_args) = call_parts(&output);
    assert!(logical_slice_eq(output_args, &expected_args));
}

#[test]
fn match_normalization_contract_uses_normalized_function_head_attributes() {
    let session = contract_session();
    let input = normalized_function_head_root();
    let output = session
        .normalize_match_expr_changed_for_test(&input)
        .expect("Function head and parent arguments must both change");
    let expected_head = Expr::call(
        system_expr("Function"),
        vec![Expr::small_integer(0), system_expr("Orderless")],
    );
    let expected = Expr::call(
        expected_head,
        vec![
            Expr::small_integer(1),
            Expr::small_integer(2),
            Expr::small_integer(3),
        ],
    );
    assert!(
        logical_expr_eq(&output, &expected),
        "normalized Function head must supply Orderless to the parent\noutput: {output}"
    );
}
