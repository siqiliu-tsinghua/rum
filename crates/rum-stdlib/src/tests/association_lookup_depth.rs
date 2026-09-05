use super::*;
use rum_core::Atom;
use rum_runtime::SessionRequestError;
use rum_runtime::kernel_api::system_expr;

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const KEY_SPEC_LIST_WRAPPERS: usize = TARGET_PHYSICAL_HEIGHT - 1;
const HIT_KEY: &str = "lookup-deep-hit-474";
const MISS_KEY: &str = "lookup-deep-miss-474";

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB Lookup certificate thread should spawn")
        .join();
    assert!(joined.is_ok(), "Lookup certificate {name:?} panicked");
}

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn list(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn rule(key: Expr, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![key, value])
}

fn association(entries: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Association"), entries)
}

fn lookup_call(data: Expr, key_spec: Expr, default: Option<Expr>, wrapper: Option<Expr>) -> Expr {
    let mut args = vec![data, key_spec];
    if let Some(default) = default {
        args.push(default);
    }
    if let Some(wrapper) = wrapper {
        assert_eq!(args.len(), 3, "a Lookup wrapper requires a default slot");
        args.push(wrapper);
    }
    Expr::call(system_expr("Lookup"), args)
}

fn evaluate_lookup(
    session: &mut Session,
    data: Expr,
    key_spec: Expr,
    default: Option<Expr>,
    wrapper: Option<Expr>,
) -> Expr {
    let response = session
        .eval_expr_request(lookup_call(data, key_spec, default, wrapper))
        .unwrap_or_else(|error| panic!("Lookup certificate should complete: {error:?}"));
    assert!(
        response.messages.is_empty(),
        "Lookup certificate should not emit messages"
    );
    response.value
}

fn nested_key_spec(mut key: Expr) -> Expr {
    for _ in 0..KEY_SPEC_LIST_WRAPPERS {
        key = list(vec![key]);
    }
    key
}

fn assert_raw_height_with_unique_deepest_key(expr: &Expr, expected_key: &str) {
    let mut work = vec![(expr, 1usize)];
    let mut max_height = 0usize;
    let mut sentinel_heights = Vec::new();

    while let Some((current, height)) = work.pop() {
        max_height = max_height.max(height);
        match current {
            Expr::Call { head, args } => {
                work.push((head, height + 1));
                work.extend(args.iter().map(|arg| (arg, height + 1)));
            }
            Expr::Atom(Atom::String(value)) if value == expected_key => {
                sentinel_heights.push(height);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_height, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(sentinel_heights, vec![TARGET_PHYSICAL_HEIGHT]);
}

fn assert_exact_expr_iterative(actual: &Expr, expected: &Expr) {
    let mut work = vec![(actual, expected)];
    while let Some((actual, expected)) = work.pop() {
        match (actual, expected) {
            (Expr::Atom(actual), Expr::Atom(expected)) => assert_eq!(actual, expected),
            (
                Expr::Call {
                    head: actual_head,
                    args: actual_args,
                },
                Expr::Call {
                    head: expected_head,
                    args: expected_args,
                },
            ) => {
                assert_eq!(actual_args.len(), expected_args.len());
                work.push((actual_head, expected_head));
                work.extend(actual_args.iter().zip(expected_args.iter()));
            }
            _ => panic!("Lookup result differs from its exact expected expression"),
        }
    }
}

fn drop_expr_iteratively(expr: Expr) {
    let mut work = vec![expr];
    while let Some(expr) = work.pop() {
        if let Some((head, args)) = expr.into_call_parts() {
            work.push(head);
            work.extend(args);
        }
    }
}

fn missing_key(key: Expr) -> Expr {
    Expr::call(system_expr("Missing"), vec![Expr::string("KeyAbsent"), key])
}

fn shallow_key_spec() -> Expr {
    list(vec![
        list(vec![
            list(vec![Expr::string("hit-left"), Expr::string("miss-left")]),
            list(vec![
                Expr::call(system_expr("Key"), vec![Expr::string("hit-right")]),
                Expr::string("miss-right"),
            ]),
        ]),
        list(vec![
            list(vec![Expr::string("hit-right"), Expr::string("miss-left")]),
            list(vec![Expr::string("hit-left"), Expr::string("miss-right")]),
        ]),
    ])
}

fn shallow_expected(mut leaf: impl FnMut(&str) -> Expr) -> Expr {
    list(vec![
        list(vec![
            list(vec![leaf("hit-left"), leaf("miss-left")]),
            list(vec![leaf("hit-right"), leaf("miss-right")]),
        ]),
        list(vec![
            list(vec![leaf("hit-right"), leaf("miss-left")]),
            list(vec![leaf("hit-left"), leaf("miss-right")]),
        ]),
    ])
}

#[test]
fn lookup_three_level_companion_pins_shape_hits_misses_default_and_wrapper() {
    let data = association(vec![
        rule(Expr::string("hit-left"), Expr::small_integer(11)),
        rule(Expr::string("hit-right"), Expr::small_integer(29)),
    ]);
    let wrapper = Expr::symbol("lookupShallowWrapper");
    let default = Expr::string("lookup-shallow-default");
    let mut session = core_session();

    let actual = evaluate_lookup(
        &mut session,
        data.clone(),
        shallow_key_spec(),
        Some(default.clone()),
        Some(wrapper.clone()),
    );
    let expected = shallow_expected(|key| match key {
        "hit-left" => Expr::call(wrapper.clone(), vec![Expr::small_integer(11)]),
        "hit-right" => Expr::call(wrapper.clone(), vec![Expr::small_integer(29)]),
        _ => default.clone(),
    });
    assert_eq!(actual, expected);

    let actual = evaluate_lookup(&mut session, data, shallow_key_spec(), None, None);
    let expected = shallow_expected(|key| match key {
        "hit-left" => Expr::small_integer(11),
        "hit-right" => Expr::small_integer(29),
        missing => missing_key(Expr::string(missing)),
    });
    assert_eq!(actual, expected);
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn lookup_hit_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("lookup-hit-depth-4097", || {
        let data = association(vec![rule(Expr::string(HIT_KEY), Expr::small_integer(474))]);
        let key_spec = nested_key_spec(Expr::string(HIT_KEY));
        assert_raw_height_with_unique_deepest_key(&key_spec, HIT_KEY);

        let wrapper = Expr::symbol("lookupDeepWrapper");
        let mut session = core_session();
        let actual = evaluate_lookup(
            &mut session,
            data,
            key_spec,
            Some(Expr::string("unused-default")),
            Some(wrapper.clone()),
        );
        let expected = nested_key_spec(Expr::call(wrapper, vec![Expr::small_integer(474)]));
        assert_exact_expr_iterative(&actual, &expected);
        drop_expr_iteratively(actual);
        drop_expr_iteratively(expected);
        println!("Lookup hit: exact result at raw key-spec height 4097 on a 2 MiB thread");
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn lookup_miss_and_resume_drop_complete_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("lookup-miss-depth-4097", || {
        let data = association(vec![rule(
            Expr::string("present-key"),
            Expr::small_integer(7),
        )]);

        let missing_spec = nested_key_spec(Expr::string(MISS_KEY));
        assert_raw_height_with_unique_deepest_key(&missing_spec, MISS_KEY);
        let mut session = core_session();
        let actual = evaluate_lookup(&mut session, data.clone(), missing_spec, None, None);
        let expected = nested_key_spec(missing_key(Expr::string(MISS_KEY)));
        assert_exact_expr_iterative(&actual, &expected);
        drop_expr_iteratively(actual);
        drop_expr_iteratively(expected);

        let resumed_spec = nested_key_spec(Expr::string(MISS_KEY));
        assert_raw_height_with_unique_deepest_key(&resumed_spec, MISS_KEY);
        let request = session
            .eval_expr_request(lookup_call(
                data,
                resumed_spec,
                Some(Expr::call(
                    system_expr("InputString"),
                    vec![Expr::string("lookup default: ")],
                )),
                None,
            ))
            .expect_err("deep missing Lookup should suspend for its default");
        match &request.error {
            SessionRequestError::InputRequired { request } => {
                assert_eq!(request.prompt, "lookup default: ");
            }
            other => panic!("expected a deep Lookup input request, got {other:?}"),
        }
        drop(request);

        let response = session
            .resume_eval_request_with_input("lookup-deep-default")
            .expect("deep missing Lookup should finish through borrowed rendering");
        assert!(response.messages.is_empty());
        let actual = response.value;
        let expected = nested_key_spec(Expr::string("lookup-deep-default"));
        assert_exact_expr_iterative(&actual, &expected);
        drop_expr_iteratively(actual);
        drop_expr_iteratively(expected);
        println!(
            "Lookup miss: exact Missing and resumed-default results at raw key-spec height 4097; deep prepared outcome dropped on a 2 MiB thread"
        );
    });
}
