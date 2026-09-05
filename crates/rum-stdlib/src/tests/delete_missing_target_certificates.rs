use super::*;
use crate::associations as association_builtins;
use rum_core::{Atom, SharedArgs};
use rum_runtime::kernel_api::{
    AssociationEntryForm, association_entry_forms_from_expr, list_items, system_expr,
};

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;
const TARGET_ASSOCIATION_LEVELS: usize = 2_047;
const TARGET_OUTER_ASSOCIATIONS: usize = TARGET_ASSOCIATION_LEVELS - 1;
const TARGET_WIDTH: usize = 64_000;

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() + Send + 'static) {
    let joined = std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB DeleteMissing target certificate thread should spawn")
        .join();
    assert!(
        joined.is_ok(),
        "DeleteMissing target certificate {name:?} panicked"
    );
}

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn rule(key: Expr, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![key, value])
}

fn association(entries: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Association"), entries)
}

fn missing() -> Expr {
    Expr::call(system_expr("Missing"), Vec::new())
}

fn target_chain(innermost: Expr) -> Expr {
    let mut current = innermost;
    for witness in 0..TARGET_OUTER_ASSOCIATIONS {
        current = association(vec![
            rule(Expr::string("child"), current),
            rule(Expr::string("witness"), Expr::small_integer(witness as i64)),
        ]);
    }
    current
}

fn scan_zero_fixture() -> Expr {
    Expr::call(
        system_expr("List"),
        vec![target_chain(association(vec![
            rule(Expr::string("drop"), missing()),
            rule(Expr::string("survivor"), Expr::string("scan-zero")),
        ]))],
    )
}

fn scan_all_fixture() -> Expr {
    target_chain(association(vec![
        rule(
            Expr::string("drop"),
            Expr::call(system_expr("nested"), vec![missing()]),
        ),
        rule(Expr::string("survivor"), Expr::string("scan-all")),
    ]))
}

fn assert_raw_physical_height(expr: &Expr) {
    let mut work = vec![(expr, 1usize)];
    let mut max_height = 0usize;
    let mut missing_heads = Vec::new();

    while let Some((current, height)) = work.pop() {
        max_height = max_height.max(height);
        match current {
            Expr::Call { head, args } => {
                work.push((head, height + 1));
                work.extend(args.iter().map(|arg| (arg, height + 1)));
            }
            Expr::Atom(Atom::Symbol(symbol))
                if symbol.context() == "System`" && symbol.name() == "Missing" =>
            {
                missing_heads.push(height);
            }
            Expr::Atom(_) => {}
        }
    }

    assert_eq!(max_height, TARGET_PHYSICAL_HEIGHT);
    assert_eq!(missing_heads, vec![TARGET_PHYSICAL_HEIGHT]);
}

fn eval_target(session: &mut Session, input: Expr, level: usize, scan: Expr) -> Expr {
    let head = system_expr("DeleteMissing");
    let args = SharedArgs::new(vec![input, Expr::small_integer(level as i64), scan]);
    association_builtins::eval_delete_missing(session, &head, &args)
        .expect("DeleteMissing target certificate should complete")
}

fn assert_exact_chain(mut current: Expr, survivor: &str) {
    for expected_witness in (0..TARGET_OUTER_ASSOCIATIONS).rev() {
        let entries = association_entry_forms_from_expr(&current)
            .expect("every target-chain wrapper must remain an association");
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].key, Expr::string("child"));
        assert!(!entries[0].delayed);
        assert_eq!(entries[1].key, Expr::string("witness"));
        assert_eq!(
            entries[1].value,
            Expr::small_integer(expected_witness as i64)
        );
        assert!(!entries[1].delayed);
        current = entries[0].value.clone();
    }

    let entries = association_entry_forms_from_expr(&current)
        .expect("the target-chain centre must remain an association");
    assert_eq!(
        entries,
        vec![AssociationEntryForm {
            key: Expr::string("survivor"),
            value: Expr::string(survivor),
            delayed: false,
        }]
    );
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn delete_missing_target_scan_zero_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("delete-missing-target-scan-zero-depth-4097", || {
        let mut session = core_session();
        let input = scan_zero_fixture();
        assert_raw_physical_height(&input);
        let result = eval_target(
            &mut session,
            input,
            TARGET_ASSOCIATION_LEVELS + 1,
            Expr::small_integer(0),
        );

        let items = list_items(&result).expect("scan-zero target root must remain a list");
        assert_eq!(items.len(), 1);
        assert_exact_chain(items[0].clone(), "scan-zero");
    });
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn delete_missing_target_scan_all_completes_at_physical_height_4097_on_two_mib() {
    run_on_two_mib_thread("delete-missing-target-scan-all-depth-4097", || {
        let mut session = core_session();
        let input = scan_all_fixture();
        assert_raw_physical_height(&input);
        let result = eval_target(
            &mut session,
            input,
            TARGET_ASSOCIATION_LEVELS,
            system_expr("All"),
        );
        assert_exact_chain(result, "scan-all");
    });
}

#[test]
#[ignore = "release-only width probe; run explicitly with --release"]
fn delete_missing_target_list_clears_64000_siblings_on_two_mib() {
    run_on_two_mib_thread("delete-missing-target-list-width-64000", || {
        let mut session = core_session();
        let survivor_positions = [0usize, TARGET_WIDTH / 2, TARGET_WIDTH - 1];
        let survivor_values = ["first", "middle", "last"];
        let mut items = Vec::with_capacity(TARGET_WIDTH);
        for index in 0..TARGET_WIDTH {
            let value = survivor_positions
                .iter()
                .position(|position| *position == index)
                .map(|position| Expr::string(survivor_values[position]))
                .unwrap_or_else(missing);
            items.push(value);
        }

        let result = eval_target(
            &mut session,
            Expr::call(system_expr("List"), items),
            1,
            Expr::small_integer(0),
        );
        assert_eq!(
            list_items(&result).unwrap(),
            &[
                Expr::string("first"),
                Expr::string("middle"),
                Expr::string("last"),
            ]
        );
    });
}

#[test]
#[ignore = "release-only width probe; run explicitly with --release"]
fn delete_missing_target_association_clears_64000_siblings_on_two_mib() {
    run_on_two_mib_thread("delete-missing-target-association-width-64000", || {
        let mut session = core_session();
        let survivor_positions = [0usize, TARGET_WIDTH / 2, TARGET_WIDTH - 1];
        let survivor_values = ["first", "middle", "last"];
        let mut entries = Vec::with_capacity(TARGET_WIDTH);
        for index in 0..TARGET_WIDTH {
            let value = survivor_positions
                .iter()
                .position(|position| *position == index)
                .map(|position| Expr::string(survivor_values[position]))
                .unwrap_or_else(missing);
            entries.push(rule(Expr::small_integer(index as i64), value));
        }

        let result = eval_target(
            &mut session,
            association(entries),
            1,
            Expr::small_integer(0),
        );
        let actual = association_entry_forms_from_expr(&result).unwrap();
        let expected = survivor_positions
            .into_iter()
            .zip(survivor_values)
            .map(|(position, value)| AssociationEntryForm {
                key: Expr::small_integer(position as i64),
                value: Expr::string(value),
                delayed: false,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    });
}

#[test]
fn delete_missing_target_shallow_companion_pins_level_and_scan() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            r#"{
                DeleteMissing[
                    {Association[
                        outerWitness -> 1,
                        branch -> Association[
                            rootDrop -> Missing[],
                            scanDrop -> nested[Missing[]],
                            survivor -> 2
                        ],
                        outerTail -> 3
                    ]},
                    3, 0
                ],
                DeleteMissing[
                    {Association[
                        outerWitness -> 1,
                        branch -> Association[
                            rootDrop -> Missing[],
                            scanDrop -> nested[Missing[]],
                            survivor -> 2
                        ],
                        outerTail -> 3
                    ]},
                    3, All
                ]
            }"#,
        )
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        concat!(
            "List[List[Association[Rule[outerWitness, 1], Rule[branch, Association[",
            "Rule[scanDrop, nested[Missing[]]], Rule[survivor, 2]]], ",
            "Rule[outerTail, 3]]], List[Association[Rule[outerWitness, 1], ",
            "Rule[branch, Association[Rule[survivor, 2]]], Rule[outerTail, 3]]]]"
        )
    );
}
