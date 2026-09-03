use super::plan::insert_replace_plan_target;
use super::*;

pub(super) fn build_guarded_replace_plan_from_targets(
    targets: Vec<GuardedReplacePartTarget>,
    rules: &[ReplacePartRule],
) -> ReplacePlanNode {
    let mut root = ReplacePlanNode::default();
    for target in targets {
        if target.state != GuardedReplacePartTargetState::Accepted {
            continue;
        }

        let rule = &rules[target.rule_index];
        insert_replace_plan_target(
            &mut root,
            &target.path,
            ReplacePlanAction::Immediate(substitute_rule_bindings(&rule.rhs, &target.bindings)),
        );
    }

    root
}

pub(super) fn append_guarded_replace_part_exact_targets(
    expr: &Expr,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
    rule_index: usize,
    targets: &mut Vec<GuardedReplacePartTarget>,
) {
    append_guarded_replace_part_exact_targets_inner(
        expr,
        selectors,
        heads,
        rule_index,
        &mut Vec::new(),
        targets,
    );
}

fn append_guarded_replace_part_exact_targets_inner(
    expr: &Expr,
    selectors: &[PartSelector],
    heads: ReplacePartHeadsMode,
    rule_index: usize,
    path: &mut Vec<isize>,
    targets: &mut Vec<GuardedReplacePartTarget>,
) {
    let Some((selector, rest)) = selectors.split_first() else {
        push_guarded_replace_part_target(
            targets,
            GuardedReplacePartTarget {
                path: path.clone(),
                rule_index,
                bindings: SymbolExprMap::default(),
                option_scope: None,
                state: GuardedReplacePartTargetState::PendingGuard,
            },
        );
        return;
    };

    let mut visit = |components: &[isize], child: &Expr| {
        path.extend_from_slice(components);
        append_guarded_replace_part_exact_targets_inner(
            child, rest, heads, rule_index, path, targets,
        );
        path.truncate(path.len() - components.len());
    };

    match selector {
        PartSelector::Single(AtomicPartSpec::Index(0)) => {
            if !matches!(heads, ReplacePartHeadsMode::Exclude) {
                if let Expr::Call { head, .. } = expr {
                    visit(&[0], head);
                }
            }
        }
        PartSelector::Single(AtomicPartSpec::Index(index)) => {
            if let Expr::Call { args, .. } = expr {
                if let Some(index) = normalize_structural_arg_position(*index, args.len()) {
                    visit(&[index as isize + 1], &args[index]);
                }
            }
        }
        PartSelector::Single(AtomicPartSpec::All) => {
            if let Expr::Call { head, args } = expr {
                if matches!(heads, ReplacePartHeadsMode::Include) {
                    visit(&[0], head);
                }
                for (index, arg) in args.iter().enumerate() {
                    visit(&[index as isize + 1], arg);
                }
            }
        }
        PartSelector::Single(AtomicPartSpec::Span(span)) => {
            if let Some(selected) = select_positioned_parts_for_span(expr, *span) {
                for (component, child) in &selected {
                    visit(&[*component], child);
                }
            }
        }
        PartSelector::Key(key) => {
            if let Some((component, child)) = association_position_and_value_by_key(expr, key) {
                visit(&[component, 2], &child);
            }
        }
        PartSelector::Multi(_) => {
            unreachable!("ReplacePart parsing expands multi-selectors into position specs")
        }
    }
}

pub(super) fn append_guarded_replace_part_pattern_candidates(
    expr: &Expr,
    include_heads: bool,
    rule_index: usize,
    targets: &mut Vec<GuardedReplacePartTarget>,
) -> std::ops::Range<usize> {
    let start = targets.len();
    append_guarded_replace_part_pattern_candidates_inner(
        expr,
        include_heads,
        rule_index,
        Vec::new(),
        targets,
    );
    start..targets.len()
}

fn append_guarded_replace_part_pattern_candidates_inner(
    expr: &Expr,
    include_heads: bool,
    rule_index: usize,
    path: Vec<isize>,
    targets: &mut Vec<GuardedReplacePartTarget>,
) {
    if let Expr::Call { head, args } = expr {
        if include_heads {
            let mut head_path = path.clone();
            head_path.push(0);
            append_guarded_replace_part_pattern_candidates_inner(
                head,
                include_heads,
                rule_index,
                head_path,
                targets,
            );
        }

        for (index, arg) in args.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(index as isize + 1);
            append_guarded_replace_part_pattern_candidates_inner(
                arg,
                include_heads,
                rule_index,
                child_path,
                targets,
            );
        }
    }

    targets.push(GuardedReplacePartTarget {
        path,
        rule_index,
        bindings: SymbolExprMap::default(),
        option_scope: None,
        state: GuardedReplacePartTargetState::PatternCandidate,
    });
}

pub(super) fn activate_guarded_replace_part_pattern_target(
    targets: &mut [GuardedReplacePartTarget],
    index: usize,
    bindings: SymbolExprMap,
    option_scope: Option<OptionValueScope>,
) {
    let rule_index = targets[index].rule_index;
    let path = targets[index].path.clone();
    if let Some(previous) = targets[..index].iter_mut().rev().find(|target| {
        target.rule_index == rule_index
            && target.path == path
            && target.state == GuardedReplacePartTargetState::PendingGuard
    }) {
        previous.state = GuardedReplacePartTargetState::Inactive;
    }

    targets[index].bindings = bindings;
    targets[index].option_scope = option_scope;
    targets[index].state = GuardedReplacePartTargetState::PendingGuard;
}

fn push_guarded_replace_part_target(
    targets: &mut Vec<GuardedReplacePartTarget>,
    target: GuardedReplacePartTarget,
) {
    if let Some(previous) = targets.iter_mut().rev().find(|previous| {
        previous.rule_index == target.rule_index
            && previous.path == target.path
            && previous.state == GuardedReplacePartTargetState::PendingGuard
    }) {
        previous.state = GuardedReplacePartTargetState::Inactive;
    }
    targets.push(target);
}
