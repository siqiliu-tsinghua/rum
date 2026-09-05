use super::*;
use rum_core::Atom;
use rum_runtime::kernel_api::{EvalError, association_entries_from_expr, system_expr};

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const ASSOCIATION_WRAPPER_COUNT: usize = 2_048;
const DEEPEST_SENTINEL: &str = "map-indexed-association-deepest-469";
const HEADS_SENTINEL: &str = "map-indexed-association-heads-469";

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB MapIndexed certificate thread should spawn")
        .join();
    assert!(joined.is_ok(), "MapIndexed certificate {name:?} panicked");
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

fn association(entries: Vec<(Expr, Expr)>) -> Expr {
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| rule(key, value))
            .collect(),
    )
}

fn single_key_association(value: Expr) -> Expr {
    association(vec![(Expr::string("k"), value)])
}

fn nested_association_chain(wrapper_count: usize, mut value: Expr) -> Expr {
    for _ in 0..wrapper_count {
        value = single_key_association(value);
    }
    value
}

fn map_indexed_call(operator: Expr, input: Expr, levels: Expr, include_heads: bool) -> Expr {
    let mut args = vec![operator, input, levels];
    if include_heads {
        args.push(rule(system_expr("Heads"), system_expr("True")));
    }
    Expr::call(system_expr("MapIndexed"), args)
}

fn position(path: impl IntoIterator<Item = isize>) -> Expr {
    list(
        path.into_iter()
            .map(|component| Expr::small_integer(component as i64))
            .collect(),
    )
}

fn mapped(operator: &Expr, value: Expr, path: impl IntoIterator<Item = isize>) -> Expr {
    Expr::call(operator.clone(), vec![value, position(path)])
}

fn assert_raw_height_with_unique_target(expr: &Expr, target: &str) {
    let mut work = vec![(expr, 1usize)];
    let mut max_height = 0usize;
    let mut target_heights = Vec::new();

    while let Some((current, height)) = work.pop() {
        max_height = max_height.max(height);
        match current {
            Expr::Call { head, args } => {
                work.push((head, height + 1));
                work.extend(args.iter().map(|arg| (arg, height + 1)));
            }
            Expr::Atom(Atom::String(value)) if value == target => {
                target_heights.push(height);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_height, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(
        target_heights,
        vec![TARGET_PHYSICAL_HEIGHT],
        "the fixture must have one target at its unique deepest value"
    );
}

fn descend_exact_single_key_chain(mut current: Expr, wrapper_count: usize) -> Expr {
    for level in 0..wrapper_count {
        let mut entries = association_entries_from_expr(&current)
            .unwrap_or_else(|| panic!("MapIndexed result level {level} must be an association"));
        assert_eq!(
            entries.len(),
            1,
            "MapIndexed result level {level} must have one entry"
        );
        let (key, value) = entries.pop().expect("one exact association entry exists");
        assert_eq!(key, Expr::string("k"), "wrong key at level {level}");
        current = value;
    }
    current
}

fn evaluate_map_indexed(session: &mut Session, call: Expr) -> Expr {
    let value = session
        .evaluate(call)
        .expect("MapIndexed certificate should evaluate");
    assert!(
        session.take_messages().is_empty(),
        "MapIndexed certificate should not emit messages"
    );
    value
}

#[test]
fn map_indexed_association_three_level_companion_pins_positions_and_levelspec() {
    let mut session = core_session();
    let operator = Expr::symbol("mapIndexedWitness");
    let input = association(vec![
        (
            Expr::string("root"),
            association(vec![
                (
                    Expr::string("child"),
                    association(vec![
                        (Expr::string("left"), Expr::small_integer(11)),
                        (Expr::string("right"), Expr::small_integer(12)),
                    ]),
                ),
                (Expr::string("middle-side"), Expr::small_integer(21)),
            ]),
        ),
        (Expr::string("root-side"), Expr::small_integer(31)),
    ]);
    let expected = association(vec![
        (
            Expr::string("root"),
            association(vec![
                (
                    Expr::string("child"),
                    association(vec![
                        (
                            Expr::string("left"),
                            mapped(&operator, Expr::small_integer(11), [1, 1, 1]),
                        ),
                        (
                            Expr::string("right"),
                            mapped(&operator, Expr::small_integer(12), [1, 1, 2]),
                        ),
                    ]),
                ),
                (Expr::string("middle-side"), Expr::small_integer(21)),
            ]),
        ),
        (Expr::string("root-side"), Expr::small_integer(31)),
    ]);

    let actual = evaluate_map_indexed(
        &mut session,
        map_indexed_call(operator, input, list(vec![Expr::small_integer(3)]), false),
    );
    assert_eq!(actual, expected);
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn map_indexed_association_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("map-indexed-association-depth-4097", || {
        let mut session = core_session();
        let operator = Expr::symbol("deepMapIndexedWitness");
        let input =
            nested_association_chain(ASSOCIATION_WRAPPER_COUNT, Expr::string(DEEPEST_SENTINEL));
        assert_raw_height_with_unique_target(&input, DEEPEST_SENTINEL);

        let actual = evaluate_map_indexed(
            &mut session,
            map_indexed_call(
                operator.clone(),
                input,
                list(vec![Expr::small_integer(-1)]),
                false,
            ),
        );
        let deepest = descend_exact_single_key_chain(actual, ASSOCIATION_WRAPPER_COUNT);
        let expected = mapped(
            &operator,
            Expr::string(DEEPEST_SENTINEL),
            std::iter::repeat_n(1, ASSOCIATION_WRAPPER_COUNT),
        );
        assert_eq!(deepest, expected);
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn map_indexed_association_heads_refuses_in_band_rather_than_terminating() {
    // A *completing* deep certificate is not achievable with `Heads -> True`,
    // and the reason is not this walker. The head-bearing centre is a call of
    // undefined symbols, so the evaluator descends it as genuine evaluation
    // levels - unlike an `Association`, which is a data constructor and costs
    // none. The result therefore exhausts the evaluator's own 1 MiB byte budget
    // long before the walker matters. Director-measured: this shape refuses in
    // band from physical height 515 upward, identically on the pre-conversion
    // build, so it is a pre-existing property of evaluating the result and not
    // a regression.
    //
    // The head edge itself is covered where it belongs - the in-module
    // forced-front falsifier runs with `include_heads: true` and compares the
    // heap tail against the bounded front. What this certificate pins is the
    // direction: the head-bearing shape must fail CLOSED, never terminate the
    // process.
    run_on_two_mib_thread("map-indexed-association-heads-depth-4097", || {
        let mut session = core_session();
        let operator = Expr::symbol("deepMapIndexedHeadsWitness");
        let centre = Expr::call(
            Expr::symbol("outerHead"),
            vec![Expr::call(
                Expr::symbol("innerHead"),
                vec![Expr::string(HEADS_SENTINEL)],
            )],
        );
        let input = nested_association_chain(ASSOCIATION_WRAPPER_COUNT - 1, centre);
        assert_raw_height_with_unique_target(&input, HEADS_SENTINEL);

        let outcome = session.evaluate(map_indexed_call(
            operator,
            input,
            list(vec![Expr::small_integer(ASSOCIATION_WRAPPER_COUNT as i64)]),
            true,
        ));
        assert!(
            matches!(outcome, Err(EvalError::StackLimitExceeded { .. })),
            "head-bearing MapIndexed must refuse in band, got {outcome:?}"
        );
    });
}
