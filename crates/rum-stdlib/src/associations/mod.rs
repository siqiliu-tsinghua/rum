use rum_core::{Atom, BigInt, Expr, Symbol, heads};
use rum_runtime::kernel_api::{
    AssociationEntryForm, EvalError, EvaluatedPartSelectors, NativeStackDepth, Session,
    association_entries_from_expr, association_entry_forms_from_expr, call_items_with_head,
    compare_expr_canonical, exact_expr_at_path, exact_position_paths_from_part_selector_exprs,
    expr_as_non_negative_usize, expr_as_signed_isize, invalid_part_selector_expr, is_missing_expr,
    is_system_symbol_name, list_items, missing_key_absent_expr, parse_association_entry_form,
    render_association, render_association_entry_forms_preserving_heads, replace_exact_path,
    run_with_native_stack_guard, system_expr, truthiness_symbol,
};

mod aggregate;
mod basic;
mod data;
mod grouping;
mod join_across;
mod key_reshape;
mod missing;
mod mutation;
mod resumable;
mod suspended_state;
mod transforms;

use self::data::*;
use self::suspended_state::{
    FrozenAssociationSequence, FrozenAssociationValue, PersistentAssociationGroups,
    PersistentAssociationSequence,
};

pub(crate) use self::{
    aggregate::{eval_catenate, eval_counts, eval_position_index},
    basic::{
        eval_association, eval_association_q, eval_key, eval_key_drop, eval_key_exists_q,
        eval_key_free_q, eval_key_member_q, eval_key_take, eval_key_value_pattern, eval_keys,
        eval_lookup, eval_normal, eval_values,
    },
    grouping::eval_group_by,
    join_across::eval_join_across,
    key_reshape::{
        eval_association_key_deflatten, eval_association_key_flatten, eval_key_group_by,
    },
    missing::{eval_delete_missing, eval_missing},
    mutation::{eval_associate_to, eval_key_drop_from},
    resumable::{
        eval_association_map, eval_counts_by, eval_key_select, eval_key_sort_by, eval_merge,
        eval_merge_nested, sort_key_entries_with_ordering,
    },
    transforms::{
        eval_association_thread, eval_key_complement, eval_key_intersection, eval_key_map,
        eval_key_sort, eval_key_union, eval_key_value_map,
    },
};
