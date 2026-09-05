use super::*;
use rum_core::Atom;
use rum_runtime::kernel_api::{exact_position_paths_from_expr, is_system_symbol_name, system_expr};

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const DEEP_OUTER_ASSOCIATION_LEVELS: usize = 2_047;
const RAW_COMPONENTS_PER_KEY_SELECTOR: usize = 2;
const ASSOCIATE_DEEPEST_SENTINEL: &str = "associate-to-part-deepest-470";
const KEY_DROP_DEEPEST_SENTINEL: &str = "key-drop-from-part-deepest-470";

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB held-association exact-path certificate thread should spawn")
        .join();
    assert!(
        joined.is_ok(),
        "held-association exact-path certificate {name:?} panicked"
    );
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

fn path_key(level: usize) -> Expr {
    Expr::string(format!("path-{level}"))
}

fn side_key(level: usize) -> Expr {
    Expr::string(format!("side-{level}"))
}

fn side_value(level: usize) -> Expr {
    Expr::small_integer(10_000 + level as i64)
}

fn key_selector(key: Expr) -> Expr {
    Expr::call(system_expr("Key"), vec![key])
}

fn path_keys(levels: usize) -> Vec<Expr> {
    (0..levels).map(path_key).collect()
}

fn nested_owner(levels: usize, mut target: Expr) -> Expr {
    for level in (0..levels).rev() {
        target = association(vec![
            (path_key(level), target),
            (side_key(level), side_value(level)),
        ]);
    }
    target
}

fn part_target(owner: Expr, selectors: Vec<Expr>) -> Expr {
    let mut args = Vec::with_capacity(selectors.len() + 1);
    args.push(owner);
    args.extend(selectors);
    Expr::call(system_expr("Part"), args)
}

fn evaluate_without_messages(session: &mut Session, expr: Expr, context: &str) -> Expr {
    let value = session
        .evaluate(expr)
        .unwrap_or_else(|error| panic!("{context} should evaluate: {error:?}"));
    let messages = session.take_messages();
    assert!(messages.is_empty(), "{context} emitted {messages:?}");
    value
}

fn bind_owner(session: &mut Session, owner: &Expr, value: Expr) {
    let _ = evaluate_without_messages(
        session,
        Expr::call(system_expr("Set"), vec![owner.clone(), value]),
        "held-association owner binding",
    );
}

fn read_owner(session: &mut Session, owner: &Expr) -> Expr {
    evaluate_without_messages(session, owner.clone(), "held-association owner readback")
}

fn take_rule(expr: Expr) -> (Expr, Expr) {
    let Expr::Call { head, args } = expr else {
        panic!("an exact association entry must be a Rule call")
    };
    assert!(is_system_symbol_name(&head, "Rule"));
    let mut args = Expr::unwrap_shared_args(args).into_iter();
    let key = args.next().expect("an exact Rule has a key");
    let value = args.next().expect("an exact Rule has a value");
    assert!(args.next().is_none(), "an exact Rule has two arguments");
    (key, value)
}

fn take_association_entries(expr: Expr) -> Vec<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        panic!("an exact nested owner level must be an Association call")
    };
    assert!(is_system_symbol_name(&head, "Association"));
    Expr::unwrap_shared_args(args)
        .into_iter()
        .map(take_rule)
        .collect()
}

fn assert_exact_nested_owner(mut actual: Expr, levels: usize, expected_target: Expr) {
    for level in 0..levels {
        let entries = take_association_entries(actual);
        let [(path_entry_key, next), (actual_side_key, actual_side_value)] =
            <[(Expr, Expr); 2]>::try_from(entries)
                .unwrap_or_else(|_| panic!("owner level {level} must have exactly two entries"));
        assert_eq!(
            path_entry_key,
            path_key(level),
            "wrong path key at level {level}"
        );
        assert_eq!(
            actual_side_key,
            side_key(level),
            "wrong side key at level {level}"
        );
        assert_eq!(
            actual_side_value,
            side_value(level),
            "wrong side value at level {level}"
        );
        actual = next;
    }
    assert_eq!(actual, expected_target);
}

fn assert_raw_height_with_unique_sentinel(expr: &Expr, sentinel: &str) {
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
            Expr::Atom(Atom::String(value)) if value == sentinel => {
                sentinel_heights.push(height);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_height, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(
        sentinel_heights,
        vec![TARGET_PHYSICAL_HEIGHT],
        "the fixture must have one sentinel on its unique deepest target path"
    );
}

fn assert_exact_key_selector_path(expr: &Expr, selectors: Vec<Expr>) {
    let selector_count = selectors.len();
    let actual = exact_position_paths_from_expr(expr, &list(selectors));
    let expected = vec![
        std::iter::repeat_n([1, 2], selector_count)
            .flatten()
            .collect::<Vec<_>>(),
    ];
    assert_eq!(actual, Some(expected));
}

#[test]
fn held_association_part_three_level_companion_pins_selectors_order_and_results() {
    let levels = 3;
    let original_target = association(vec![
        (Expr::string("keep"), Expr::small_integer(1)),
        (Expr::string("drop"), Expr::small_integer(2)),
    ]);
    let original = nested_owner(levels, original_target.clone());

    let bare_selectors = path_keys(levels);
    assert_exact_key_selector_path(&original, bare_selectors.clone());
    let explicit_selectors = path_keys(levels)
        .into_iter()
        .map(key_selector)
        .collect::<Vec<_>>();
    assert_exact_key_selector_path(&original, explicit_selectors.clone());

    let mut session = core_session();
    let associate_owner = Expr::symbol("shallowAssociateOwner470");
    bind_owner(&mut session, &associate_owner, original.clone());
    let associated_target = association(vec![
        (Expr::string("keep"), Expr::small_integer(1)),
        (Expr::string("drop"), Expr::small_integer(2)),
        (Expr::string("added"), Expr::small_integer(3)),
    ]);
    let expected_associated = nested_owner(levels, associated_target);
    let associated = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("AssociateTo"),
            vec![
                part_target(associate_owner.clone(), bare_selectors.clone()),
                rule(Expr::string("added"), Expr::small_integer(3)),
            ],
        ),
        "shallow AssociateTo Part target",
    );
    assert_eq!(associated, expected_associated);
    assert_eq!(
        read_owner(&mut session, &associate_owner),
        expected_associated
    );

    let drop_owner = Expr::symbol("shallowKeyDropOwner470");
    bind_owner(&mut session, &drop_owner, original.clone());
    let expected_dropped_target = association(vec![(Expr::string("keep"), Expr::small_integer(1))]);
    let dropped = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("KeyDropFrom"),
            vec![
                part_target(drop_owner.clone(), explicit_selectors),
                Expr::string("drop"),
            ],
        ),
        "shallow KeyDropFrom Part target",
    );
    assert_eq!(dropped, expected_dropped_target);
    assert_eq!(
        read_owner(&mut session, &drop_owner),
        nested_owner(levels, expected_dropped_target)
    );

    let rejected_owner = Expr::symbol("shallowRejectedKeyDropOwner470");
    bind_owner(&mut session, &rejected_owner, original.clone());
    let rejected_call = Expr::call(
        system_expr("KeyDropFrom"),
        vec![
            part_target(rejected_owner.clone(), bare_selectors),
            Expr::string("drop"),
        ],
    );
    let rejected = session
        .evaluate(rejected_call.clone())
        .expect("bare-string KeyDropFrom Part target should return in band");
    assert_eq!(rejected, rejected_call);
    assert!(
        session
            .take_messages()
            .iter()
            .any(|message| { message.symbol == "KeyDropFrom" && message.tag == "invak" })
    );
    assert_eq!(read_owner(&mut session, &rejected_owner), original);

    let multi_owner = Expr::symbol("shallowMultiAssociateOwner470");
    let multi_original = list(vec![
        association(vec![(Expr::string("branch"), Expr::small_integer(1))]),
        association(vec![(Expr::string("branch"), Expr::small_integer(2))]),
        association(vec![(Expr::string("branch"), Expr::small_integer(3))]),
    ]);
    let multi_selector = list(vec![
        Expr::small_integer(3),
        Expr::small_integer(1),
        Expr::small_integer(2),
    ]);
    bind_owner(&mut session, &multi_owner, multi_original);
    let multi_expected = list(vec![
        association(vec![
            (Expr::string("branch"), Expr::small_integer(1)),
            (Expr::string("seen"), Expr::small_integer(470)),
        ]),
        association(vec![
            (Expr::string("branch"), Expr::small_integer(2)),
            (Expr::string("seen"), Expr::small_integer(470)),
        ]),
        association(vec![
            (Expr::string("branch"), Expr::small_integer(3)),
            (Expr::string("seen"), Expr::small_integer(470)),
        ]),
    ]);
    let multi_result = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("AssociateTo"),
            vec![
                part_target(multi_owner.clone(), vec![multi_selector]),
                rule(Expr::string("seen"), Expr::small_integer(470)),
            ],
        ),
        "ordered multi-selection AssociateTo",
    );
    assert_eq!(multi_result, multi_expected);
    assert_eq!(read_owner(&mut session, &multi_owner), multi_expected);
}

#[test]
fn shared_exact_path_consumers_keep_exact_shallow_behavior() {
    let mut session = core_session();
    let input = Expr::call(
        Expr::symbol("outer470"),
        vec![
            Expr::call(
                Expr::symbol("left470"),
                vec![Expr::small_integer(10), Expr::small_integer(20)],
            ),
            Expr::call(Expr::symbol("right470"), vec![Expr::small_integer(30)]),
        ],
    );
    let path = list(vec![Expr::small_integer(1), Expr::small_integer(2)]);

    let part = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("Part"),
            vec![
                input.clone(),
                Expr::small_integer(1),
                Expr::small_integer(2),
            ],
        ),
        "Part shared exact-path control",
    );
    assert_eq!(part, Expr::small_integer(20));

    let replaced_part = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("ReplacePart"),
            vec![input.clone(), rule(path.clone(), Expr::small_integer(21))],
        ),
        "ReplacePart shared exact-path control",
    );
    let expected_replaced = Expr::call(
        Expr::symbol("outer470"),
        vec![
            Expr::call(
                Expr::symbol("left470"),
                vec![Expr::small_integer(10), Expr::small_integer(21)],
            ),
            Expr::call(Expr::symbol("right470"), vec![Expr::small_integer(30)]),
        ],
    );
    assert_eq!(replaced_part, expected_replaced);

    let head_replaced = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("ReplacePart"),
            vec![
                input.clone(),
                rule(
                    list(vec![Expr::small_integer(0)]),
                    Expr::symbol("newOuter470"),
                ),
            ],
        ),
        "ReplacePart head-selection control",
    );
    let expected_head_replaced = match input.clone() {
        Expr::Call { args, .. } => Expr::call_with_shared_args(Expr::symbol("newOuter470"), args),
        Expr::Atom(_) => unreachable!("the shared-consumer fixture is a call"),
    };
    assert_eq!(head_replaced, expected_head_replaced);

    let mapped = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("MapAt"),
            vec![Expr::symbol("mapped470"), input.clone(), path.clone()],
        ),
        "MapAt shared exact-path control",
    );
    let expected_mapped = Expr::call(
        Expr::symbol("outer470"),
        vec![
            Expr::call(
                Expr::symbol("left470"),
                vec![
                    Expr::small_integer(10),
                    Expr::call(Expr::symbol("mapped470"), vec![Expr::small_integer(20)]),
                ],
            ),
            Expr::call(Expr::symbol("right470"), vec![Expr::small_integer(30)]),
        ],
    );
    assert_eq!(mapped, expected_mapped);

    let replaced_at = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("ReplaceAt"),
            vec![
                input.clone(),
                rule(Expr::small_integer(20), Expr::small_integer(22)),
                path.clone(),
            ],
        ),
        "ReplaceAt shared exact-path control",
    );
    let expected_replaced_at = Expr::call(
        Expr::symbol("outer470"),
        vec![
            Expr::call(
                Expr::symbol("left470"),
                vec![Expr::small_integer(10), Expr::small_integer(22)],
            ),
            Expr::call(Expr::symbol("right470"), vec![Expr::small_integer(30)]),
        ],
    );
    assert_eq!(replaced_at, expected_replaced_at);

    let inserted = evaluate_without_messages(
        &mut session,
        Expr::call(
            system_expr("Insert"),
            vec![input.clone(), Expr::small_integer(15), path.clone()],
        ),
        "Insert shared exact-path control",
    );
    let expected_inserted = Expr::call(
        Expr::symbol("outer470"),
        vec![
            Expr::call(
                Expr::symbol("left470"),
                vec![
                    Expr::small_integer(10),
                    Expr::small_integer(15),
                    Expr::small_integer(20),
                ],
            ),
            Expr::call(Expr::symbol("right470"), vec![Expr::small_integer(30)]),
        ],
    );
    assert_eq!(inserted, expected_inserted);

    let deleted = evaluate_without_messages(
        &mut session,
        Expr::call(system_expr("Delete"), vec![input.clone(), path.clone()]),
        "Delete shared exact-path control",
    );
    let expected_deleted = Expr::call(
        Expr::symbol("outer470"),
        vec![
            Expr::call(Expr::symbol("left470"), vec![Expr::small_integer(10)]),
            Expr::call(Expr::symbol("right470"), vec![Expr::small_integer(30)]),
        ],
    );
    assert_eq!(deleted, expected_deleted);

    let extracted = evaluate_without_messages(
        &mut session,
        Expr::call(system_expr("Extract"), vec![input, path]),
        "Extract shared exact-path control",
    );
    assert_eq!(extracted, Expr::small_integer(20));
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn associate_to_part_target_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("associate-to-part-exact-path-height-4097", || {
        let mut session = core_session();
        let owner = Expr::symbol("deepAssociateOwner470");
        let original_target = association(vec![(
            Expr::string("seed"),
            Expr::string(ASSOCIATE_DEEPEST_SENTINEL),
        )]);
        let original = nested_owner(DEEP_OUTER_ASSOCIATION_LEVELS, original_target);
        assert_raw_height_with_unique_sentinel(&original, ASSOCIATE_DEEPEST_SENTINEL);

        let selectors = path_keys(DEEP_OUTER_ASSOCIATION_LEVELS);
        assert_exact_key_selector_path(&original, selectors.clone());
        assert_eq!(selectors.len() * RAW_COMPONENTS_PER_KEY_SELECTOR, 4_094);
        bind_owner(&mut session, &owner, original);

        let expected_target = association(vec![
            (
                Expr::string("seed"),
                Expr::string(ASSOCIATE_DEEPEST_SENTINEL),
            ),
            (Expr::string("added"), Expr::small_integer(470)),
        ]);
        let returned = evaluate_without_messages(
            &mut session,
            Expr::call(
                system_expr("AssociateTo"),
                vec![
                    part_target(owner.clone(), selectors),
                    rule(Expr::string("added"), Expr::small_integer(470)),
                ],
            ),
            "deep AssociateTo Part target",
        );
        assert_exact_nested_owner(
            returned,
            DEEP_OUTER_ASSOCIATION_LEVELS,
            expected_target.clone(),
        );
        assert_exact_nested_owner(
            read_owner(&mut session, &owner),
            DEEP_OUTER_ASSOCIATION_LEVELS,
            expected_target,
        );
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn key_drop_from_part_target_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("key-drop-from-part-exact-path-height-4097", || {
        let mut session = core_session();
        let owner = Expr::symbol("deepKeyDropOwner470");
        let original_target = association(vec![(
            Expr::string("drop"),
            Expr::string(KEY_DROP_DEEPEST_SENTINEL),
        )]);
        let original = nested_owner(DEEP_OUTER_ASSOCIATION_LEVELS, original_target);
        assert_raw_height_with_unique_sentinel(&original, KEY_DROP_DEEPEST_SENTINEL);

        let selectors = path_keys(DEEP_OUTER_ASSOCIATION_LEVELS)
            .into_iter()
            .map(key_selector)
            .collect::<Vec<_>>();
        assert_exact_key_selector_path(&original, selectors.clone());
        assert_eq!(selectors.len() * RAW_COMPONENTS_PER_KEY_SELECTOR, 4_094);
        bind_owner(&mut session, &owner, original);

        let expected_target = association(Vec::new());
        let returned = evaluate_without_messages(
            &mut session,
            Expr::call(
                system_expr("KeyDropFrom"),
                vec![part_target(owner.clone(), selectors), Expr::string("drop")],
            ),
            "deep KeyDropFrom Part target",
        );
        assert_eq!(returned, expected_target);
        assert_exact_nested_owner(
            read_owner(&mut session, &owner),
            DEEP_OUTER_ASSOCIATION_LEVELS,
            expected_target,
        );
    });
}
