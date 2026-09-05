use super::*;

fn branch_fixture() -> Expr {
    Expr::call(
        Expr::symbol("exactPathRoot"),
        vec![
            Expr::call(
                Expr::symbol("leftBranch"),
                vec![Expr::string("left-witness")],
            ),
            Expr::call(
                Expr::symbol("rightBranch"),
                vec![Expr::string("right-witness")],
            ),
        ],
    )
}

fn exhausted<T>(traversal: BoundedTraversal<T>) -> T {
    match traversal {
        BoundedTraversal::Exhausted(value) => value,
        BoundedTraversal::Complete(_) => panic!("front zero must exercise the heap tail"),
    }
}

fn complete<T>(traversal: BoundedTraversal<T>) -> T {
    match traversal {
        BoundedTraversal::Complete(value) => value,
        BoundedTraversal::Exhausted(_) => panic!("front sixteen must complete this fixture"),
    }
}

#[test]
fn exact_expr_at_path_forced_front_matches_each_branched_witness() {
    let fixture = branch_fixture();
    for (path, expected) in [
        (&[1, 1][..], Expr::string("left-witness")),
        (&[2, 1][..], Expr::string("right-witness")),
    ] {
        let tail = exhausted(exact_expr_at_path_with_front_depth(&fixture, path, 0));
        let front = complete(exact_expr_at_path_with_front_depth(&fixture, path, 16));
        assert_eq!(tail, Some(expected.clone()));
        assert_eq!(front, Some(expected));
        assert_eq!(tail, front);
    }
}

#[test]
fn collect_exact_paths_for_position_spec_forced_front_preserves_branch_order() {
    let fixture = branch_fixture();
    let selectors = vec![
        PartSelector::Multi(vec![
            PartSelector::Multi(vec![PartSelector::Single(AtomicPartSpec::Index(2))]),
            PartSelector::Multi(vec![PartSelector::Single(AtomicPartSpec::Index(1))]),
        ]),
        PartSelector::Single(AtomicPartSpec::Index(1)),
    ];
    let expected = vec![vec![2, 1], vec![1, 1]];

    let selector_tail = exhausted(select_positioned_parts_for_selector_with_front_depth(
        &fixture,
        &selectors[0],
        ReplacePartHeadsMode::Automatic,
        0,
    ));
    let selector_front = complete(select_positioned_parts_for_selector_with_front_depth(
        &fixture,
        &selectors[0],
        ReplacePartHeadsMode::Automatic,
        16,
    ));
    assert_eq!(selector_tail, selector_front);
    assert_eq!(
        selector_tail
            .expect("the nested multi-selector must select both branches")
            .into_iter()
            .map(|(prefix, _)| prefix)
            .collect::<Vec<_>>(),
        vec![vec![2], vec![1]]
    );

    let tail = exhausted(collect_exact_paths_for_position_spec_with_front_depth(
        &fixture,
        &selectors,
        ReplacePartHeadsMode::Automatic,
        0,
    ));
    let front = complete(collect_exact_paths_for_position_spec_with_front_depth(
        &fixture,
        &selectors,
        ReplacePartHeadsMode::Automatic,
        16,
    ));
    assert_eq!(tail, expected);
    assert_eq!(front, expected);
    assert_eq!(tail, front);
    assert_eq!(
        exact_expr_at_path(&fixture, &tail[0]),
        Some(Expr::string("right-witness"))
    );
    assert_eq!(
        exact_expr_at_path(&fixture, &tail[1]),
        Some(Expr::string("left-witness"))
    );
}

#[test]
fn replace_exact_path_forced_front_matches_head_and_child_edges() {
    let fixture = branch_fixture();

    let expected_head = Expr::call(
        Expr::symbol("replacedRoot"),
        match fixture.clone() {
            Expr::Call { args, .. } => args.to_vec(),
            Expr::Atom(_) => unreachable!("the branch fixture is a call"),
        },
    );
    let tail_head = exhausted(replace_exact_path_with_front_depth(
        fixture.clone(),
        &[0],
        Expr::symbol("replacedRoot"),
        0,
    ));
    let front_head = complete(replace_exact_path_with_front_depth(
        fixture.clone(),
        &[0],
        Expr::symbol("replacedRoot"),
        16,
    ));
    assert_eq!(tail_head, expected_head);
    assert_eq!(front_head, expected_head);
    assert_eq!(tail_head, front_head);

    let expected_child = Expr::call(
        Expr::symbol("exactPathRoot"),
        vec![
            Expr::call(
                Expr::symbol("leftBranch"),
                vec![Expr::string("left-witness")],
            ),
            Expr::call(
                Expr::symbol("rightBranch"),
                vec![Expr::string("right-replaced")],
            ),
        ],
    );
    let tail_child = exhausted(replace_exact_path_with_front_depth(
        fixture.clone(),
        &[2, 1],
        Expr::string("right-replaced"),
        0,
    ));
    let front_child = complete(replace_exact_path_with_front_depth(
        fixture,
        &[2, 1],
        Expr::string("right-replaced"),
        16,
    ));
    assert_eq!(tail_child, expected_child);
    assert_eq!(front_child, expected_child);
    assert_eq!(tail_child, front_child);
}
