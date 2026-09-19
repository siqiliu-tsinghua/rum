use super::*;
use crate::associations as association_builtins;
use rum_core::{Atom, SharedArgs};
use rum_runtime::kernel_api::{list_items, system_expr};

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const LIST_WRAPPER_COUNT: usize = TARGET_PHYSICAL_HEIGHT - 3;
const DEEPEST_VALUE: &str = "values-deepest-475";

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB Values certificate thread should spawn")
        .join();
    assert!(joined.is_ok(), "Values certificate {name:?} panicked");
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

fn evaluate_values(session: &mut Session, input: Expr, wrapper: Option<Expr>) -> Expr {
    let head = system_expr("Values");
    let mut args = vec![input];
    args.extend(wrapper);
    let args = SharedArgs::new(args);
    let value = association_builtins::eval_values(session, &head, &args)
        .unwrap_or_else(|error| panic!("Values certificate should complete: {error:?}"));
    assert!(
        session.take_messages().is_empty(),
        "Values certificate should not emit messages"
    );
    value
}

fn assert_raw_height_with_unique_deepest_value(expr: &Expr) {
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
            Expr::Atom(Atom::String(value)) if value == DEEPEST_VALUE => {
                sentinel_heights.push(height);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_height, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(sentinel_heights, vec![TARGET_PHYSICAL_HEIGHT]);
}

fn assert_exact_nested_list_result(actual: &Expr, expected_terminal: &Expr) {
    let mut current = actual;
    for level in 0..LIST_WRAPPER_COUNT {
        let items = list_items(current)
            .unwrap_or_else(|| panic!("Values result level {level} must be a List"));
        assert_eq!(
            items.len(),
            1,
            "Values result level {level} must have one child"
        );
        current = &items[0];
    }
    assert_eq!(current, expected_terminal);
}

fn shallow_terminal(prefix: &str, association_form: bool) -> Expr {
    let entries = vec![
        rule(
            Expr::string(format!("{prefix}-key-first")),
            Expr::string(format!("{prefix}-value-1")),
            false,
        ),
        rule(
            Expr::string(format!("{prefix}-key-second")),
            Expr::string(format!("{prefix}-value-2")),
            true,
        ),
    ];
    if association_form {
        association(entries)
    } else {
        list(entries)
    }
}

fn expected_shallow_terminal(prefix: &str, wrapper: &Expr) -> Expr {
    list(vec![
        Expr::call(
            wrapper.clone(),
            vec![Expr::string(format!("{prefix}-value-1"))],
        ),
        Expr::call(
            wrapper.clone(),
            vec![Expr::string(format!("{prefix}-value-2"))],
        ),
    ])
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

fn two_error_fixture() -> Expr {
    list(vec![
        list(vec![
            shallow_terminal("valid-before", true),
            list(vec![
                Expr::string("first-malformed-item"),
                shallow_terminal("unvisited-after-first", false),
            ]),
        ]),
        list(vec![
            Expr::string("second-malformed-item"),
            shallow_terminal("unvisited-after-second", true),
        ]),
    ])
}

#[test]
fn values_three_level_companion_pins_wrappers_sibling_value_order_and_outer_error() {
    let wrapper = system_expr("valuesShallowWrapper");
    let input = three_level_tree(shallow_terminal);
    let mut session = core_session();
    let actual = evaluate_values(&mut session, input, Some(wrapper.clone()));
    let expected = three_level_tree(|prefix, _| expected_shallow_terminal(prefix, &wrapper));
    assert_eq!(actual, expected);

    let input = two_error_fixture();
    let args = vec![input.clone(), wrapper];
    let mut session = core_session();
    let actual = association_builtins::eval_values(
        &mut session,
        &system_expr("Values"),
        &SharedArgs::new(args.clone()),
    )
    .expect("malformed Values companion should return an in-band diagnostic");
    assert_eq!(actual, Expr::call(system_expr("Values"), args));

    let messages = session.take_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].symbol, "Values");
    assert_eq!(messages[0].tag, "invrl");
    assert_eq!(
        messages[0].text,
        "The argument List[List[Association[Rule[\"valid-before-key-first\", \"valid-before-value-1\"], RuleDelayed[\"valid-before-key-second\", \"valid-before-value-2\"]], List[\"first-malformed-item\", List[Rule[\"unvisited-after-first-key-first\", \"unvisited-after-first-value-1\"], RuleDelayed[\"unvisited-after-first-key-second\", \"unvisited-after-first-value-2\"]]]], List[\"second-malformed-item\", Association[Rule[\"unvisited-after-second-key-first\", \"unvisited-after-second-value-1\"], RuleDelayed[\"unvisited-after-second-key-second\", \"unvisited-after-second-value-2\"]]]] is not a valid Association or a list of rules."
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn values_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("values-depth-4097", || {
        let mut session = core_session();
        let sentinel = Expr::string(DEEPEST_VALUE);
        let side_value = Expr::string("values-side-value-475");
        let terminal = list(vec![
            rule(Expr::string("values-deep-key"), sentinel.clone(), false),
            rule(Expr::string("values-side-key"), side_value.clone(), true),
        ]);
        let input = nested_list_chain(terminal);
        assert_raw_height_with_unique_deepest_value(&input);

        let wrapper = system_expr("valuesDeepWrapper");
        let actual = evaluate_values(&mut session, input, Some(wrapper.clone()));
        let expected_terminal = list(vec![
            Expr::call(wrapper.clone(), vec![sentinel]),
            Expr::call(wrapper, vec![side_value]),
        ]);
        assert_exact_nested_list_result(&actual, &expected_terminal);
    });
}
