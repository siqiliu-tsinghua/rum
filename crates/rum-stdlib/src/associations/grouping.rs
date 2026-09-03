use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::HashMap;

use super::*;

mod association;
mod list;

use self::{association::*, list::*};

const GROUP_INDEX_THRESHOLD: usize = 16;

#[derive(FrameRootTrace)]
pub(super) struct IndexedGroups<T> {
    entries: Vec<(Expr, Vec<T>)>,
    positions: Option<HashMap<Expr, usize>>,
}

impl<T> IndexedGroups<T> {
    pub(super) fn from_entries(entries: Vec<(Expr, Vec<T>)>) -> Self {
        let positions =
            (entries.len() > GROUP_INDEX_THRESHOLD).then(|| Self::positions_from_entries(&entries));
        Self { entries, positions }
    }

    pub(super) fn push(&mut self, key: Expr, value: T) {
        if let Some(positions) = &mut self.positions {
            if let Some(index) = positions.get(&key).copied() {
                self.entries[index].1.push(value);
                return;
            }

            let index = self.entries.len();
            positions.insert(key.clone(), index);
            self.entries.push((key, vec![value]));
            return;
        }

        if let Some((_, values)) = self
            .entries
            .iter_mut()
            .find(|(existing, _)| existing == &key)
        {
            values.push(value);
            return;
        }

        self.entries.push((key, vec![value]));
        if self.entries.len() > GROUP_INDEX_THRESHOLD {
            self.positions = Some(Self::positions_from_entries(&self.entries));
        }
    }

    pub(super) fn collect(pairs: impl IntoIterator<Item = (Expr, T)>) -> Self {
        let mut remaining = pairs.into_iter();
        let mut entries: Vec<(Expr, Vec<T>)> = Vec::new();
        for (key, value) in remaining.by_ref() {
            if let Some((_, values)) = entries.iter_mut().find(|(existing, _)| existing == &key) {
                values.push(value);
            } else {
                entries.push((key, vec![value]));
                if entries.len() > GROUP_INDEX_THRESHOLD {
                    return Self::collect_indexed(entries, remaining);
                }
            }
        }

        Self {
            entries,
            positions: None,
        }
    }

    pub(super) fn into_entries(self) -> Vec<(Expr, Vec<T>)> {
        self.entries
    }

    fn positions_from_entries(entries: &[(Expr, Vec<T>)]) -> HashMap<Expr, usize> {
        entries
            .iter()
            .enumerate()
            .map(|(index, (key, _))| (key.clone(), index))
            .collect()
    }

    #[inline(never)]
    fn collect_indexed(
        mut entries: Vec<(Expr, Vec<T>)>,
        remaining: impl Iterator<Item = (Expr, T)>,
    ) -> Self {
        let mut positions = Self::positions_from_entries(&entries);
        for (key, value) in remaining {
            if let Some(index) = positions.get(&key).copied() {
                entries[index].1.push(value);
            } else {
                let index = entries.len();
                positions.insert(key.clone(), index);
                entries.push((key, vec![value]));
            }
        }

        Self {
            entries,
            positions: Some(positions),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
struct GroupBySpec {
    key_function: Expr,
    value_function: Option<Expr>,
}

fn group_by_spec(function: &Expr) -> GroupBySpec {
    match function {
        Expr::Call { head, args, .. }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            GroupBySpec {
                key_function: args[0].clone(),
                value_function: Some(args[1].clone()),
            }
        }
        _ => GroupBySpec {
            key_function: function.clone(),
            value_function: None,
        },
    }
}

fn group_by_specs(function: &Expr) -> Vec<GroupBySpec> {
    if let Some(items) = list_items(function) {
        return items.iter().map(group_by_spec).collect();
    }

    vec![group_by_spec(function)]
}

pub(crate) fn eval_group_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    match args_slice {
        [function] => Ok(keyed_operator_form_slot_first("GroupBy", function.clone())),
        [expr, function] => {
            let specs = group_by_specs(function);
            if let Some(entries) = association_entry_forms_from_expr(expr) {
                if specs.is_empty() {
                    return finish_association_group_by_leaf(
                        session,
                        head.clone(),
                        args.clone(),
                        None,
                        entries,
                    );
                }

                return resume_association_group_by_from_index(
                    session,
                    head.clone(),
                    args.clone(),
                    specs,
                    0,
                    None,
                    entries,
                    0,
                    Vec::new(),
                );
            }

            let Some(items) = list_items(expr) else {
                return Ok(session.invalid_call_with_message(
                    "GroupBy",
                    args_slice,
                    "list1",
                    invalid_keyed_collection_list_message(expr),
                ));
            };

            if specs.is_empty() {
                return finish_group_by_leaf(
                    session,
                    head.clone(),
                    args.clone(),
                    None,
                    items.to_vec(),
                );
            }

            resume_group_by_from_index(
                session,
                head.clone(),
                args.clone(),
                specs,
                0,
                None,
                items.to_vec(),
                0,
                Vec::new(),
            )
        }
        [expr, function, reducer] => {
            let specs = group_by_specs(function);
            if let Some(entries) = association_entry_forms_from_expr(expr) {
                if specs.is_empty() {
                    return finish_association_group_by_leaf(
                        session,
                        head.clone(),
                        args.clone(),
                        Some(reducer.clone()),
                        entries,
                    );
                }

                return resume_association_group_by_from_index(
                    session,
                    head.clone(),
                    args.clone(),
                    specs,
                    0,
                    Some(reducer.clone()),
                    entries,
                    0,
                    Vec::new(),
                );
            }

            let Some(items) = list_items(expr) else {
                return Ok(session.invalid_call_with_message(
                    "GroupBy",
                    args_slice,
                    "list1",
                    invalid_keyed_collection_list_message(expr),
                ));
            };

            if specs.is_empty() {
                return finish_group_by_leaf(
                    session,
                    head.clone(),
                    args.clone(),
                    Some(reducer.clone()),
                    items.to_vec(),
                );
            }

            resume_group_by_from_index(
                session,
                head.clone(),
                args.clone(),
                specs,
                0,
                Some(reducer.clone()),
                items.to_vec(),
                0,
                Vec::new(),
            )
        }
        _ => Ok(session.invalid_call_with_message(
            "GroupBy",
            args_slice,
            "argb",
            between_one_and_three_argument_count_message("GroupBy", args_slice.len()),
        )),
    }
}

fn between_one_and_three_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; between 1 and 3 arguments are expected.")
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<GroupBySpec>();
    assert_traces::<IndexedGroups<Expr>>();
};
