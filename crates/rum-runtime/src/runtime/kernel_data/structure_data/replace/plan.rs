use super::*;

impl ContinuationValue for Vec<ReplacePartTarget> {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationReplacePartTargets"),
            self.into_iter()
                .map(|target| {
                    Expr::call(
                        system_expr("List"),
                        vec![
                            replace_part_position_expr(&target.path),
                            target.rhs,
                            system_expr(if target.delayed { "True" } else { "False" }),
                        ],
                    )
                })
                .collect(),
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args }
                if is_system_symbol_name(&head, "InternalContinuationReplacePartTargets") =>
            {
                args.into_iter()
                    .map(|item| match item {
                        Expr::Call { head, args }
                            if is_system_symbol_name(&head, "List") && args.len() == 3 =>
                        {
                            let mut parts = args.into_iter();
                            let path_expr = parts
                                .next()
                                .expect("replace-part target payload stores a path");
                            let rhs = parts
                                .next()
                                .expect("replace-part target payload stores a rhs");
                            let delayed_expr = parts
                                .next()
                                .expect("replace-part target payload stores a delayed flag");

                            ReplacePartTarget {
                                path: replace_part_path_from_expr(path_expr),
                                rhs,
                                delayed: is_system_symbol_name(&delayed_expr, "True"),
                            }
                        }
                        other => {
                            panic!("unexpected continuation item for ReplacePartTarget: {other:?}")
                        }
                    })
                    .collect()
            }
            other => panic!("unexpected continuation payload for ReplacePartTarget: {other:?}"),
        }
    }
}

pub(super) fn build_replace_plan_from_targets(targets: Vec<ReplacePartTarget>) -> ReplacePlanNode {
    let mut root = ReplacePlanNode::default();
    for target in targets {
        insert_replace_plan_target(
            &mut root,
            &target.path,
            ReplacePlanAction::Immediate(target.rhs),
        );
    }

    root
}

pub(super) fn replace_part_rule_is_merge_safe(rule: &ReplacePartRule) -> bool {
    // Pattern matching can run user code in rule-major candidate order, while
    // Key selectors can observe keys changed by an earlier rule.
    rule.position_specs.iter().all(|spec| match spec {
        ReplacePartPositionSpec::Exact(selectors) => selectors
            .iter()
            .all(|selector| matches!(selector, PartSelector::Single(_))),
        ReplacePartPositionSpec::Pattern(_) => false,
    })
}

pub(super) fn merge_replace_part_rule_into_plan(
    expr: &Expr,
    plan: &mut ReplacePlanNode,
    rule: &ReplacePartRule,
) {
    let mut targets = Vec::new();
    for spec in &rule.position_specs {
        let ReplacePartPositionSpec::Exact(selectors) = spec else {
            unreachable!("only merge-safe ReplacePart rules reach the combined planner");
        };
        let mut paths = Vec::new();
        collect_planned_exact_paths(
            expr,
            Some(plan),
            selectors,
            rule.heads,
            &mut Vec::new(),
            &mut paths,
        );
        targets.extend(paths.into_iter().map(|path| ReplacePartTarget {
            path,
            rhs: rule.rhs.clone(),
            delayed: rule.delayed,
        }));
    }

    merge_replace_plans(plan, build_replace_plan_from_targets(targets));
}

fn merge_replace_plans(plan: &mut ReplacePlanNode, incoming: ReplacePlanNode) {
    if let Some(action) = incoming.action {
        plan.action = Some(action);
        plan.children.clear();
    }
    for (component, child) in incoming.children {
        merge_replace_plans(plan.children.entry(component).or_default(), child);
    }
}

fn collect_planned_exact_paths(
    expr: &Expr,
    plan: Option<&ReplacePlanNode>,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
    path: &mut Vec<isize>,
    paths: &mut Vec<Vec<isize>>,
) {
    let Some((selector, rest)) = selectors.split_first() else {
        paths.push(path.clone());
        return;
    };
    let expr = plan
        .and_then(|plan| plan.action.as_ref())
        .map(|ReplacePlanAction::Immediate(rhs)| rhs)
        .unwrap_or(expr);
    let mut visit = |component: isize, child: &Expr| {
        path.push(component);
        collect_planned_exact_paths(
            child,
            plan.and_then(|plan| plan.children.get(&component)),
            rest,
            heads,
            path,
            paths,
        );
        path.pop();
    };

    match selector {
        PartSelector::Single(AtomicPartSpec::Index(0)) => {
            if !matches!(heads, ReplacePartHeadsMode::Exclude) {
                if let Expr::Call { head, .. } = expr {
                    visit(0, head);
                }
            }
        }
        PartSelector::Single(AtomicPartSpec::Index(index)) => {
            if let Expr::Call { args, .. } = expr {
                if let Some(index) = normalize_structural_arg_position(*index, args.len()) {
                    visit(index as isize + 1, &args[index]);
                }
            }
        }
        PartSelector::Single(AtomicPartSpec::All) => {
            if let Expr::Call { head, args } = expr {
                if matches!(heads, ReplacePartHeadsMode::Include) {
                    visit(0, head);
                }
                for (index, arg) in args.iter().enumerate() {
                    visit(index as isize + 1, arg);
                }
            }
        }
        PartSelector::Single(AtomicPartSpec::Span(span)) => {
            if let Some(selected) = select_positioned_parts_for_span(expr, *span) {
                for (component, child) in &selected {
                    visit(*component, child);
                }
            }
        }
        PartSelector::Key(_) | PartSelector::Multi(_) => {
            unreachable!("non-structural selectors use sequential ReplacePart application")
        }
    }
}

pub(super) fn insert_replace_plan_target(
    node: &mut ReplacePlanNode,
    path: &[isize],
    action: ReplacePlanAction,
) {
    let Some((first, rest)) = path.split_first() else {
        node.action = Some(action);
        node.children.clear();
        return;
    };

    if node.action.is_none() {
        insert_replace_plan_target(node.children.entry(*first).or_default(), rest, action);
    }
}

pub(super) fn wrap_replace_part_pattern_position(pattern: Expr) -> Expr {
    if replace_part_pattern_is_explicit_list(&pattern) {
        pattern
    } else {
        Expr::call(system_expr("List"), vec![pattern])
    }
}

fn replace_part_pattern_is_explicit_list(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST) => true,
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::PATTERN) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[1])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::PATTERN_TEST) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::CONDITION) && args.len() == 2 =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::patterns::OPTIONAL) && !args.is_empty() =>
        {
            replace_part_pattern_is_explicit_list(&args[0])
        }
        Expr::Call { head, args } if is_system_symbol_name(head, heads::patterns::ALTERNATIVES) => {
            !args.is_empty() && args.iter().all(replace_part_pattern_is_explicit_list)
        }
        _ => false,
    }
}

pub(super) fn replace_part_pattern_includes_heads(
    pattern: &Expr,
    heads: ReplacePartHeadsMode,
) -> bool {
    match heads {
        ReplacePartHeadsMode::Include => true,
        ReplacePartHeadsMode::Exclude => false,
        ReplacePartHeadsMode::Automatic => replace_part_pattern_mentions_explicit_zero(pattern),
    }
}

fn replace_part_pattern_mentions_explicit_zero(expr: &Expr) -> bool {
    if expr_as_signed_isize(expr) == Some(0) {
        return true;
    }

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            replace_part_pattern_mentions_explicit_zero(head)
                || args.iter().any(replace_part_pattern_mentions_explicit_zero)
        }
    }
}

pub(super) fn collect_replace_part_candidate_paths(
    expr: &Expr,
    include_heads: bool,
) -> Vec<Vec<isize>> {
    let mut paths = Vec::new();
    collect_replace_part_candidate_paths_inner(expr, include_heads, Vec::new(), &mut paths);
    paths
}

pub(super) fn collect_replace_part_candidate_paths_inner(
    expr: &Expr,
    include_heads: bool,
    path: Vec<isize>,
    paths: &mut Vec<Vec<isize>>,
) {
    if let Expr::Call { head, args } = expr {
        if include_heads {
            let mut head_path = path.clone();
            head_path.push(0);
            collect_replace_part_candidate_paths_inner(head, include_heads, head_path, paths);
        }

        for (index, arg) in args.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(index as isize + 1);
            collect_replace_part_candidate_paths_inner(arg, include_heads, child_path, paths);
        }
    }

    paths.push(path);
}

pub(super) fn replace_part_position_expr(path: &[isize]) -> Expr {
    Expr::call(
        system_expr("List"),
        path.iter()
            .map(|component| Expr::integer(BigInt::from(*component)))
            .collect(),
    )
}

fn replace_part_path_from_expr(expr: Expr) -> Vec<isize> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(&head, heads::structure::LIST) => args
            .into_iter()
            .map(|component| {
                expr_as_signed_isize(&component)
                    .expect("replace-part continuation paths should store integers")
            })
            .collect(),
        other => panic!("replace-part continuation paths should use List expressions: {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_plan_resolves_duplicates_and_prefixes_during_insertion() {
        let plan = build_replace_plan_from_targets(vec![
            target(&[1, 1], 1),
            target(&[1], 2),
            target(&[1, 2], 3),
            target(&[2], 4),
            target(&[2], 5),
        ]);

        assert_eq!(plan.children.len(), 2);
        let first = &plan.children[&1];
        assert_eq!(first.action, Some(action(2)));
        assert!(first.children.is_empty());
        assert_eq!(plan.children[&2].action, Some(action(5)));
    }

    #[test]
    fn root_target_discards_existing_and_later_descendants() {
        let plan =
            build_replace_plan_from_targets(vec![target(&[1], 1), target(&[], 2), target(&[2], 3)]);

        assert_eq!(plan.action, Some(action(2)));
        assert!(plan.children.is_empty());
    }

    fn target(path: &[isize], rhs: i64) -> ReplacePartTarget {
        ReplacePartTarget {
            path: path.to_vec(),
            rhs: Expr::integer(BigInt::from(rhs)),
            delayed: false,
        }
    }

    fn action(rhs: i64) -> ReplacePlanAction {
        ReplacePlanAction::Immediate(Expr::integer(BigInt::from(rhs)))
    }
}
