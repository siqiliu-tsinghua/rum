use super::*;
use crate::associations as association_builtins;
use rum_core::{Atom, SharedArgs};
use rum_runtime::kernel_api::{association_entry_forms_from_expr, list_items, system_expr};

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const LIST_WRAPPER_COUNT: usize = TARGET_PHYSICAL_HEIGHT - 3;
const TAKE_SENTINEL: &str = "key-take-deepest-472";
const DROP_SENTINEL: &str = "key-drop-deepest-472";

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB KeyTake/KeyDrop certificate thread should spawn")
        .join();
    assert!(
        joined.is_ok(),
        "KeyTake/KeyDrop certificate {name:?} panicked"
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

fn rule(key: Expr, value: Expr, delayed: bool) -> Expr {
    Expr::call(
        system_expr(if delayed { "RuleDelayed" } else { "Rule" }),
        vec![key, value],
    )
}

fn association(entries: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Association"), entries)
}

fn nested_list_chain(mut value: Expr) -> Expr {
    for _ in 0..LIST_WRAPPER_COUNT {
        value = list(vec![value]);
    }
    value
}

fn evaluate_key_take_drop(
    session: &mut Session,
    input: Expr,
    key_spec: Expr,
    keep_selected: bool,
) -> Expr {
    let name = if keep_selected { "KeyTake" } else { "KeyDrop" };
    let head = system_expr(name);
    let args = SharedArgs::new(vec![input, key_spec]);
    let value = if keep_selected {
        association_builtins::eval_key_take(session, &head, &args)
    } else {
        association_builtins::eval_key_drop(session, &head, &args)
    }
    .unwrap_or_else(|error| panic!("{name} certificate should complete: {error:?}"));
    assert!(
        session.take_messages().is_empty(),
        "{name} certificate should not emit messages"
    );
    value
}

fn assert_raw_height_with_unique_deepest_sentinel(expr: &Expr, sentinel: &str) {
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
    assert_eq!(sentinel_heights, vec![TARGET_PHYSICAL_HEIGHT]);
}

fn assert_exact_nested_list_result(mut current: Expr, expected_terminal: Expr) {
    for level in 0..LIST_WRAPPER_COUNT {
        let items = list_items(&current)
            .unwrap_or_else(|| panic!("KeyTake/KeyDrop result level {level} must be a List"));
        assert_eq!(
            items.len(),
            1,
            "KeyTake/KeyDrop result level {level} must have one child"
        );
        current = items[0].clone();
    }

    let actual_entries = association_entry_forms_from_expr(&current)
        .expect("KeyTake/KeyDrop terminal result must be an Association");
    let expected_entries = association_entry_forms_from_expr(&expected_terminal)
        .expect("expected KeyTake/KeyDrop terminal must be an Association");
    assert_eq!(actual_entries, expected_entries);
}

fn shallow_terminal(prefix: &str, association_form: bool) -> Expr {
    let entries = vec![
        rule(
            Expr::string("a"),
            Expr::string(format!("{prefix}-a")),
            false,
        ),
        rule(
            Expr::string("stay-left"),
            Expr::string(format!("{prefix}-stay-left")),
            true,
        ),
        rule(Expr::string("b"), Expr::string(format!("{prefix}-b")), true),
        rule(
            Expr::string("stay-right"),
            Expr::string(format!("{prefix}-stay-right")),
            false,
        ),
    ];
    if association_form {
        association(entries)
    } else {
        list(entries)
    }
}

fn expected_shallow_terminal(prefix: &str, keep_selected: bool) -> Expr {
    let entries = if keep_selected {
        vec![
            rule(Expr::string("b"), Expr::string(format!("{prefix}-b")), true),
            rule(
                Expr::string("a"),
                Expr::string(format!("{prefix}-a")),
                false,
            ),
        ]
    } else {
        vec![
            rule(
                Expr::string("stay-left"),
                Expr::string(format!("{prefix}-stay-left")),
                true,
            ),
            rule(
                Expr::string("stay-right"),
                Expr::string(format!("{prefix}-stay-right")),
                false,
            ),
        ]
    };
    association(entries)
}

fn three_level_tree(mut leaf: impl FnMut(&str, bool) -> Expr) -> Expr {
    list(vec![
        list(vec![
            list(vec![
                leaf("left-left-left", true),
                leaf("left-left-right", false),
            ]),
            list(vec![
                leaf("left-right-left", false),
                leaf("left-right-right", true),
            ]),
        ]),
        list(vec![
            list(vec![
                leaf("right-left-left", true),
                leaf("right-left-right", false),
            ]),
            list(vec![
                leaf("right-right-left", false),
                leaf("right-right-right", true),
            ]),
        ]),
    ])
}

#[test]
fn key_take_drop_three_level_companion_pins_wrappers_sibling_key_order_and_rule_heads() {
    let input = three_level_tree(shallow_terminal);
    let key_spec = list(vec![
        Expr::string("b"),
        Expr::string("a"),
        Expr::string("missing"),
        Expr::string("a"),
    ]);

    for keep_selected in [true, false] {
        let mut session = core_session();
        let actual =
            evaluate_key_take_drop(&mut session, input.clone(), key_spec.clone(), keep_selected);
        let expected =
            three_level_tree(|prefix, _| expected_shallow_terminal(prefix, keep_selected));
        assert_eq!(actual, expected);
    }
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn key_take_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("key-take-depth-4097", || {
        let mut session = core_session();
        let sentinel = Expr::string(TAKE_SENTINEL);
        let terminal = association(vec![
            rule(Expr::string("take"), sentinel.clone(), true),
            rule(Expr::string("side"), Expr::string("key-take-side"), false),
        ]);
        let input = nested_list_chain(terminal);
        assert_raw_height_with_unique_deepest_sentinel(&input, TAKE_SENTINEL);

        let actual =
            evaluate_key_take_drop(&mut session, input, list(vec![Expr::string("take")]), true);
        let expected_terminal = association(vec![rule(Expr::string("take"), sentinel, true)]);
        assert_exact_nested_list_result(actual, expected_terminal);
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn key_drop_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("key-drop-depth-4097", || {
        let mut session = core_session();
        let sentinel = Expr::string(DROP_SENTINEL);
        let terminal = association(vec![
            rule(
                Expr::string("drop"),
                Expr::string("key-drop-removed"),
                false,
            ),
            rule(Expr::string("keep"), sentinel.clone(), true),
        ]);
        let input = nested_list_chain(terminal);
        assert_raw_height_with_unique_deepest_sentinel(&input, DROP_SENTINEL);

        let actual =
            evaluate_key_take_drop(&mut session, input, list(vec![Expr::string("drop")]), false);
        let expected_terminal = association(vec![rule(Expr::string("keep"), sentinel, true)]);
        assert_exact_nested_list_result(actual, expected_terminal);
    });
}
