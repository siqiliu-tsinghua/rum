use std::collections::{BTreeMap, BTreeSet};

use rum_core::{Atom, BigInt, Expr, SharedArgs, heads};
use rum_runtime::kernel_api::{
    EvalError, HeadsOptionClassification, Session, association_entries_from_expr,
    association_entry_forms_from_expr, classify_heads_option_expr, compare_expr_canonical,
    expr_as_exact_integer, expr_as_non_negative_usize, expr_as_signed_isize, expr_as_string,
    is_system_symbol_name, list_items, missing_key_absent_expr, render_association,
    render_association_entry_forms_preserving_heads, system_expr, truthiness_symbol,
};

mod collection_stats;
mod combinatorial_lists;
mod dedup;
mod flatten;
mod join;
mod list_lookup;
mod list_plumbing;
mod ordering_sets;
mod reordering;
mod resumable_merge_sort;
mod selectors;
mod sequence;
pub(crate) mod stable_groups;
mod surface;
mod suspended_state;

use self::suspended_state::{
    FrozenStructureSequence, FrozenStructureValue, PersistentStructureCounts,
    PersistentStructureSequence, PersistentStructureSet,
};

pub(crate) use self::{
    collection_stats::{
        eval_gather_by, eval_max_by, eval_maximal_by, eval_min_by, eval_minimal_by,
        eval_reverse_sort, eval_reverse_sort_by, eval_riffle, eval_take_largest_by,
        eval_take_smallest_by, eval_tally,
    },
    combinatorial_lists::{
        eval_partition, eval_permutations, eval_subsets, eval_tuples, next_permutation,
    },
    dedup::{eval_delete_duplicates, eval_delete_duplicates_by},
    flatten::eval_flatten,
    join::eval_join,
    list_lookup::{eval_binary_search, eval_duplicates, eval_select_positions},
    list_plumbing::{eval_constant_array, eval_differences, eval_pad_left, eval_pad_right},
    ordering_sets::{eval_complement, eval_intersection, eval_sort, eval_sort_by, eval_union},
    reordering::{eval_reverse, eval_transpose},
    resumable_merge_sort::{
        ComparatorResultPolicy, MergeCompletion, MergeSortItem, sort_with_comparator,
    },
    sequence::{
        eval_delete, eval_drop, eval_insert, eval_range, eval_take, eval_take_drop, eval_take_list,
    },
    surface::{
        eval_append, eval_construct, eval_depth, eval_first, eval_full_form, eval_head, eval_last,
        eval_leaf_count, eval_length, eval_most, eval_prepend, eval_rest, eval_symbol,
    },
};

use selectors::{AtomicPartSpec, PartSelector, apply_part_selectors};

fn invalid_normal_expression_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.push_message(
        heads::messages::GENERAL,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    );
    Expr::call(system_expr(symbol), args.to_vec())
}

fn eval_binary_predicate(
    session: &mut Session,
    predicate: &Expr,
    lhs: &Expr,
    rhs: &Expr,
) -> Result<Expr, EvalError> {
    session.evaluate(Expr::call(
        predicate.clone(),
        vec![lhs.clone(), rhs.clone()],
    ))
}

fn is_true_expr(expr: &Expr) -> bool {
    is_system_symbol_name(expr, heads::structure::TRUE)
}

fn is_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)
    )
}

fn structural_head_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
}

fn structural_length(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 0,
        Expr::Call { args, .. } => args.len(),
    }
}

fn structural_depth(expr: &Expr, include_heads: bool) -> usize {
    match expr {
        Expr::Atom(_) => 1,
        Expr::Call { head, args } => {
            if let Some(entries) = association_entries_from_expr(expr) {
                let head_depth = include_heads
                    .then(|| structural_depth(head, true))
                    .unwrap_or(0);
                let value_depth = entries
                    .iter()
                    .map(|(_, value)| structural_depth(value, include_heads))
                    .max()
                    .unwrap_or(0);
                return 1 + 1.max(head_depth).max(value_depth);
            }

            let head_depth = include_heads
                .then(|| structural_depth(head, true))
                .unwrap_or(0);
            let arg_depth = args
                .iter()
                .map(|arg| structural_depth(arg, include_heads))
                .max()
                .unwrap_or(0);
            1 + 1.max(head_depth).max(arg_depth)
        }
    }
}

fn structural_leaf_count(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 1,
        Expr::Call { head, args } => {
            structural_leaf_count(head) + args.iter().map(structural_leaf_count).sum::<usize>()
        }
    }
}

fn select_exact_part(expr: &Expr, index: isize) -> Option<Expr> {
    if index == 0 {
        return Some(structural_head_expr(expr));
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            let normalized = normalize_structural_arg_position(index, args.len())?;
            Some(args[normalized].clone())
        }
    }
}

fn exact_part_component(expr: &Expr, index: isize) -> Option<isize> {
    if index == 0 {
        return Some(0);
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            Some(normalize_structural_arg_position(index, args.len())? as isize + 1)
        }
    }
}

fn normalize_structural_arg_position(position: isize, len: usize) -> Option<usize> {
    if position == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if position > 0 {
        position - 1
    } else {
        len + position
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}
