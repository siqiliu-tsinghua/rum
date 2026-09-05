use super::*;
use crate::associations as association_builtins;
use rum_core::{Atom, SharedArgs};
use rum_runtime::kernel_api::{association_entry_forms_from_expr, system_expr};

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const ASSOCIATION_LEVELS: usize = 2_048;
const DEEPEST_SENTINEL: &str = "association-key-reshape-deepest-468";

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB association key-reshape certificate thread should spawn")
        .join();
    assert!(
        joined.is_ok(),
        "association key-reshape certificate {name:?} panicked"
    );
}

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
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

fn list(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn eval_flatten(session: &mut Session, input: Expr) -> Expr {
    let head = system_expr("AssociationKeyFlatten");
    let args = SharedArgs::new(vec![input]);
    association_builtins::eval_association_key_flatten(session, &head, &args)
        .expect("AssociationKeyFlatten certificate should complete")
}

fn eval_deflatten(session: &mut Session, input: Expr) -> Expr {
    let head = system_expr("AssociationKeyDeflatten");
    let args = SharedArgs::new(vec![input]);
    association_builtins::eval_association_key_deflatten(session, &head, &args)
        .expect("AssociationKeyDeflatten certificate should complete")
}

fn deep_path() -> Vec<Expr> {
    (0..ASSOCIATION_LEVELS)
        .map(|level| Expr::string(format!("key-{level}")))
        .collect()
}

fn nested_association_chain(path: &[Expr], value: Expr, delayed_leaf: bool) -> Expr {
    let mut current = value;
    for (reverse_index, key) in path.iter().rev().enumerate() {
        current = association(vec![rule(
            key.clone(),
            current,
            delayed_leaf && reverse_index == 0,
        )]);
    }
    current
}

fn assert_raw_height_with_unique_deepest_sentinel(expr: &Expr) {
    let mut work = vec![(expr, 1usize)];
    let mut max_height = 0;
    let mut sentinel_heights = Vec::new();

    while let Some((current, height)) = work.pop() {
        max_height = max_height.max(height);
        match current {
            Expr::Call { head, args } => {
                work.push((head, height + 1));
                work.extend(args.iter().map(|arg| (arg, height + 1)));
            }
            Expr::Atom(Atom::String(value)) if value == DEEPEST_SENTINEL => {
                sentinel_heights.push(height);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_height, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(sentinel_heights, vec![TARGET_PHYSICAL_HEIGHT]);
}

fn assert_exact_nested_chain(
    mut current: Expr,
    path: &[Expr],
    expected_value: &Expr,
    delayed_leaf: bool,
) {
    for (level, expected_key) in path.iter().enumerate() {
        let mut entries = association_entry_forms_from_expr(&current)
            .unwrap_or_else(|| panic!("deflattened level {level} must be an association"));
        assert_eq!(
            entries.len(),
            1,
            "deflattened level {level} must have exactly one entry"
        );
        let entry = entries.pop().expect("one exact chain entry should exist");
        assert_eq!(&entry.key, expected_key, "wrong key at level {level}");
        assert_eq!(
            entry.delayed,
            delayed_leaf && level + 1 == path.len(),
            "wrong rule head at level {level}"
        );
        current = entry.value;
    }
    assert_eq!(&current, expected_value);
}

#[test]
fn association_key_flatten_three_level_companion_pins_exact_paths_and_rule_heads() {
    let mut session = core_session();
    let input = association(vec![
        rule(
            Expr::string("a"),
            association(vec![
                rule(
                    Expr::string("b"),
                    association(vec![
                        rule(Expr::string("c"), Expr::small_integer(1), false),
                        rule(Expr::string("d"), Expr::small_integer(2), true),
                    ]),
                    false,
                ),
                rule(Expr::string("side"), Expr::small_integer(3), false),
            ]),
            false,
        ),
        rule(Expr::string("top"), Expr::small_integer(4), false),
    ]);
    let expected = association(vec![
        rule(
            list(vec![
                Expr::string("a"),
                Expr::string("b"),
                Expr::string("c"),
            ]),
            Expr::small_integer(1),
            false,
        ),
        rule(
            list(vec![
                Expr::string("a"),
                Expr::string("b"),
                Expr::string("d"),
            ]),
            Expr::small_integer(2),
            true,
        ),
        rule(
            list(vec![Expr::string("a"), Expr::string("side")]),
            Expr::small_integer(3),
            false,
        ),
        rule(
            list(vec![Expr::string("top")]),
            Expr::small_integer(4),
            false,
        ),
    ]);

    assert_eq!(eval_flatten(&mut session, input), expected);
}

#[test]
fn association_key_deflatten_three_level_companion_pins_order_overwrite_and_conflicts() {
    let mut session = core_session();
    let path_abc = list(vec![
        Expr::string("a"),
        Expr::string("b"),
        Expr::string("c"),
    ]);
    let path_abd = list(vec![
        Expr::string("a"),
        Expr::string("b"),
        Expr::string("d"),
    ]);
    let path_aside = list(vec![Expr::string("a"), Expr::string("side")]);
    let path_top = list(vec![Expr::string("top")]);
    let valid = association(vec![
        rule(path_abc.clone(), Expr::small_integer(1), false),
        rule(path_abd, Expr::small_integer(2), true),
        rule(path_aside, Expr::small_integer(3), false),
        rule(path_top, Expr::small_integer(4), false),
        rule(path_abc.clone(), Expr::small_integer(9), true),
    ]);
    let expected = association(vec![
        rule(
            Expr::string("a"),
            association(vec![
                rule(
                    Expr::string("b"),
                    association(vec![
                        rule(Expr::string("c"), Expr::small_integer(9), true),
                        rule(Expr::string("d"), Expr::small_integer(2), true),
                    ]),
                    false,
                ),
                rule(Expr::string("side"), Expr::small_integer(3), false),
            ]),
            false,
        ),
        rule(Expr::string("top"), Expr::small_integer(4), false),
    ]);
    assert_eq!(eval_deflatten(&mut session, valid), expected);

    let branch_then_leaf = association(vec![
        rule(path_abc.clone(), Expr::small_integer(1), false),
        rule(
            list(vec![Expr::string("a"), Expr::string("b")]),
            Expr::small_integer(2),
            false,
        ),
    ]);
    let branch_then_leaf_call = Expr::call(
        system_expr("AssociationKeyDeflatten"),
        vec![branch_then_leaf.clone()],
    );
    assert_eq!(
        eval_deflatten(&mut session, branch_then_leaf),
        branch_then_leaf_call
    );

    let leaf_then_branch = association(vec![
        rule(
            list(vec![Expr::string("a"), Expr::string("b")]),
            Expr::small_integer(2),
            false,
        ),
        rule(path_abc, Expr::small_integer(1), false),
    ]);
    let leaf_then_branch_call = Expr::call(
        system_expr("AssociationKeyDeflatten"),
        vec![leaf_then_branch.clone()],
    );
    assert_eq!(
        eval_deflatten(&mut session, leaf_then_branch),
        leaf_then_branch_call
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn association_key_flatten_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("association-key-flatten-depth-4097", || {
        let mut session = core_session();
        let path = deep_path();
        let sentinel = Expr::string(DEEPEST_SENTINEL);
        let input = nested_association_chain(&path, sentinel.clone(), true);
        assert_raw_height_with_unique_deepest_sentinel(&input);

        let result = eval_flatten(&mut session, input);
        let expected = association(vec![rule(list(path), sentinel, true)]);
        assert_eq!(result, expected);
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn association_key_deflatten_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("association-key-deflatten-depth-4097", || {
        let mut session = core_session();
        let path = deep_path();
        let sentinel = Expr::string(DEEPEST_SENTINEL);
        let input = association(vec![rule(list(path.clone()), sentinel.clone(), true)]);

        let result = eval_deflatten(&mut session, input);
        assert_raw_height_with_unique_deepest_sentinel(&result);
        assert_exact_nested_chain(result, &path, &sentinel, true);
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn association_key_deflatten_flatten_round_trip_is_exact_at_physical_height_4097() {
    run_on_two_mib_thread("association-key-reshape-round-trip-depth-4097", || {
        let mut session = core_session();
        let path = deep_path();
        let mut side_path = path.clone();
        *side_path
            .last_mut()
            .expect("the deep round-trip path should not be empty") = Expr::string("side-leaf");
        let input = association(vec![
            rule(list(path), Expr::string(DEEPEST_SENTINEL), true),
            rule(list(side_path), Expr::string("round-trip-side"), false),
        ]);

        let deflattened = eval_deflatten(&mut session, input.clone());
        assert_raw_height_with_unique_deepest_sentinel(&deflattened);
        let round_trip = eval_flatten(&mut session, deflattened);
        assert_eq!(round_trip, input);
    });
}

/// The deflatten error path drops a partially built `NestedPathNodes`, whose
/// `Branch` arm is recursive and would otherwise drop recursively.
///
/// `path_from_key` rejects exactly one shape — an **empty list** key — so a
/// deep good path followed by `{}` is the only way to build a deep intermediate
/// tree and then abandon it. A non-list key is treated as a one-element path,
/// not an error, which is why a fixture built around one measures the ordinary
/// success path instead.
///
/// The depth is measured, not chosen: on this 2 MiB thread a recursive drop of
/// the intermediate tree survives 20 000 and terminates the process at 60 000.
const DROP_PATH_LEVELS: usize = 60_000;

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack() {
    run_on_two_mib_thread("association-key-deflatten-drop-path", || {
        let mut session = core_session();
        let deep = list(
            (0..DROP_PATH_LEVELS)
                .map(|level| Expr::string(format!("drop-key-{level}")))
                .collect(),
        );
        let input = association(vec![
            rule(deep, Expr::string("drop-path-value"), false),
            rule(list(Vec::new()), Expr::string("unreachable"), false),
        ]);

        let head = system_expr("AssociationKeyDeflatten");
        let args = SharedArgs::new(vec![input.clone()]);
        let outcome =
            association_builtins::eval_association_key_deflatten(&mut session, &head, &args)
                .expect("abandoning the partial tree must not fail the evaluation");

        // The empty-list key is refused, so the call returns unevaluated with
        // its argument intact - and the deep intermediate tree built before the
        // refusal has been discarded without recursing.
        assert_eq!(outcome, Expr::call(head, vec![input]));
    });
}
