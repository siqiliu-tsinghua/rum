mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::{HashMap, HashSet};

const KEY_COLLECTION_INDEX_THRESHOLD: usize = 16;

fn key_collection_indices(
    collections: &[Vec<AssociationEntryForm>],
    lookup_count: usize,
) -> Option<Vec<HashMap<Expr, usize>>> {
    (lookup_count > KEY_COLLECTION_INDEX_THRESHOLD).then(|| {
        collections
            .iter()
            .map(|entries| {
                entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| (entry.key.clone(), index))
                    .collect()
            })
            .collect()
    })
}

fn key_collection_entry<'a>(
    entries: &'a [AssociationEntryForm],
    positions: Option<&HashMap<Expr, usize>>,
    key: &Expr,
) -> Option<&'a AssociationEntryForm> {
    if let Some(positions) = positions {
        return positions.get(key).map(|index| &entries[*index]);
    }
    entries.iter().find(|entry| entry.key == *key)
}

#[derive(FrameRootTrace)]
struct KeyUnionPlan {
    collections: Vec<Vec<AssociationEntryForm>>,
    indices: Option<Vec<HashMap<Expr, usize>>>,
    union_entries: Vec<AssociationEntryForm>,
}

impl KeyUnionPlan {
    fn new(collections: Vec<Vec<AssociationEntryForm>>) -> Self {
        let union_entries = key_union_entries(&collections);
        let indices = key_collection_indices(&collections, union_entries.len());
        Self {
            collections,
            indices,
            union_entries,
        }
    }

    fn missing_keys(&self) -> Vec<Expr> {
        self.union_entries
            .iter()
            .filter(|entry| {
                self.collections
                    .iter()
                    .enumerate()
                    .any(|(collection_index, entries)| {
                        key_collection_entry(
                            entries,
                            self.indices
                                .as_ref()
                                .map(|indices| &indices[collection_index]),
                            &entry.key,
                        )
                        .is_none()
                    })
            })
            .map(|entry| entry.key.clone())
            .collect()
    }
}

pub(crate) fn eval_key_union(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (collections_expr, fill_function) = match args {
        [collections_expr] => (collections_expr, None),
        [collections_expr, fill_function] => (collections_expr, Some(fill_function.clone())),
        _ => {
            return Ok(session.invalid_call_with_message(
                "KeyUnion",
                args,
                "argt",
                one_or_two_argument_count_message("KeyUnion", args.len()),
            ));
        }
    };

    let collections = match parse_keyed_entry_forms_list_expr_with_diagnostics(collections_expr) {
        Ok(collections) => collections,
        Err(KeyedEntriesListError::NotList) => {
            return Ok(session.invalid_call_with_message(
                "KeyUnion",
                args,
                "invar",
                invalid_keyed_collection_outer_list_message(collections_expr),
            ));
        }
        Err(KeyedEntriesListError::InvalidElement(expr)) => {
            return Ok(session.invalid_call_with_message(
                "KeyUnion",
                args,
                "invas",
                invalid_association_or_rule_message(&expr),
            ));
        }
        Err(KeyedEntriesListError::StackLimitExceeded) => {
            return Ok(session.invalid_call_with_message(
                "KeyUnion",
                args,
                "reslim",
                keyed_collection_stack_limit_message(),
            ));
        }
    };

    let plan = KeyUnionPlan::new(collections);

    match fill_function {
        None => Ok(render_key_union_result(&plan, &[])),
        Some(fill_function) => {
            let missing_keys = plan.missing_keys();
            resume_key_union_missing_values_from_index(
                session,
                fill_function,
                plan,
                missing_keys,
                0,
                Vec::new(),
            )
        }
    }
}

pub(crate) fn eval_key_intersection(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [collections_expr] = args else {
        return Ok(session.invalid_call_with_message(
            "KeyIntersection",
            args,
            "argx",
            exact_argument_count_message("KeyIntersection", args.len(), 1),
        ));
    };

    let collections = match parse_keyed_entry_forms_list_expr_with_diagnostics(collections_expr) {
        Ok(collections) => collections,
        Err(KeyedEntriesListError::NotList) => {
            return Ok(session.invalid_call_with_message(
                "KeyIntersection",
                args,
                "invar",
                invalid_keyed_collection_outer_list_message(collections_expr),
            ));
        }
        Err(KeyedEntriesListError::InvalidElement(expr)) => {
            return Ok(session.invalid_call_with_message(
                "KeyIntersection",
                args,
                "invas",
                invalid_association_or_rule_message(&expr),
            ));
        }
        Err(KeyedEntriesListError::StackLimitExceeded) => {
            return Ok(session.invalid_call_with_message(
                "KeyIntersection",
                args,
                "reslim",
                keyed_collection_stack_limit_message(),
            ));
        }
    };

    let Some(first) = collections.first() else {
        return Ok(Expr::call(system_expr("List"), vec![]));
    };
    let indices = key_collection_indices(&collections, first.len());

    let intersection_keys = first
        .iter()
        .filter(|entry| {
            collections[1..]
                .iter()
                .enumerate()
                .all(|(rest_index, entries)| {
                    key_collection_entry(
                        entries,
                        indices.as_ref().map(|indices| &indices[rest_index + 1]),
                        &entry.key,
                    )
                    .is_some()
                })
        })
        .map(|entry| entry.key.clone())
        .collect::<Vec<_>>();

    Ok(Expr::call(
        system_expr("List"),
        collections
            .iter()
            .enumerate()
            .map(|(collection_index, entries)| {
                render_association_entry_forms_preserving_heads(
                    intersection_keys
                        .iter()
                        .filter_map(|key| {
                            key_collection_entry(
                                entries,
                                indices.as_ref().map(|indices| &indices[collection_index]),
                                key,
                            )
                            .map(|entry| AssociationEntryForm {
                                key: key.clone(),
                                value: entry.value.clone(),
                                delayed: entry.delayed,
                            })
                        })
                        .collect(),
                )
            })
            .collect(),
    ))
}

pub(crate) fn eval_key_complement(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [collections_expr] = args else {
        if args.is_empty() {
            return Ok(session.invalid_call_with_message(
                "KeyComplement",
                args,
                "argx",
                exact_argument_count_message("KeyComplement", args.len(), 1),
            ));
        }

        let held_input = Expr::call(system_expr("KeyComplement"), args.to_vec());
        return Ok(match collect_option_names_from_tail(session, &args[1..]) {
            Some(option_names) => session.invalid_call_with_message(
                "KeyComplement",
                args,
                "optx",
                unknown_option_message(&option_names[0], &held_input),
            ),
            None => session.invalid_call_with_message(
                "KeyComplement",
                args,
                "nonopt",
                non_option_syntax_message(&args[args.len() - 1], 1, &held_input),
            ),
        });
    };

    let collections = match parse_keyed_entry_forms_list_expr_with_diagnostics(collections_expr) {
        Ok(collections) => collections,
        Err(KeyedEntriesListError::NotList) => {
            return Ok(session.invalid_call_with_message(
                "KeyComplement",
                args,
                "invar",
                invalid_keyed_collection_outer_list_message(collections_expr),
            ));
        }
        Err(KeyedEntriesListError::InvalidElement(expr)) => {
            return Ok(session.invalid_call_with_message(
                "KeyComplement",
                args,
                "invrl",
                invalid_association_or_rule_list_message(&expr),
            ));
        }
        Err(KeyedEntriesListError::StackLimitExceeded) => {
            return Ok(session.invalid_call_with_message(
                "KeyComplement",
                args,
                "reslim",
                keyed_collection_stack_limit_message(),
            ));
        }
    };

    let Some((first, rest)) = collections.split_first() else {
        return Ok(session.invalid_call_with_message(
            "KeyComplement",
            args,
            "empt",
            nonempty_list_message(collections_expr),
        ));
    };

    let excluded = rest
        .iter()
        .flat_map(|entries| entries.iter().map(|entry| entry.key.clone()))
        .collect::<std::collections::HashSet<_>>();

    Ok(render_association_entry_forms_preserving_heads(
        first
            .iter()
            .filter(|entry| !excluded.contains(&entry.key))
            .cloned()
            .collect(),
    ))
}

fn resume_key_union_missing_values_from_index(
    session: &mut Session,
    fill_function: Expr,
    plan: KeyUnionPlan,
    missing_keys: Vec<Expr>,
    mut index: usize,
    mut fill_values: Vec<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(key) = missing_keys.get(index).cloned() else {
            return Ok(render_key_union_result(&plan, &fill_values));
        };

        let fill_value = match session
            .evaluate(Expr::call(fill_function.clone(), vec![key.clone()]))
        {
            Ok(fill_value) => fill_value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let plan = FrozenAssociationValue::freeze(plan);
                                            let missing_keys = FrozenAssociationSequence::freeze(missing_keys);
                                            let fill_values = PersistentAssociationSequence::freeze(fill_values);
                ResumeKeyUnionMissingValuesFromIndexSuspensionFrame { fill_function, plan, missing_keys, index, fill_values, key }
                },
                );
            }
            Err(error) => return Err(error),
        };

        fill_values.push((key, fill_value));
        index += 1;
    }
}

fn resume_suspended_key_union_missing_values_from_index(
    session: &mut Session,
    fill_function: Expr,
    plan: FrozenAssociationValue<KeyUnionPlan>,
    missing_keys: FrozenAssociationSequence<Expr>,
    mut index: usize,
    mut fill_values: PersistentAssociationSequence<(Expr, Expr)>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(key) = missing_keys.get(index).cloned() else {
            return Ok(render_key_union_result(
                plan.as_ref(),
                &fill_values.to_vec(),
            ));
        };

        let fill_value = match session
            .evaluate(Expr::call(fill_function.clone(), vec![key.clone()]))
        {
            Ok(fill_value) => fill_value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedKeyUnionMissingValuesFromIndexSuspensionFrame { fill_function, plan, missing_keys, index, fill_values, key },
                );
            }
            Err(error) => return Err(error),
        };

        fill_values = fill_values.appended((key, fill_value));
        index += 1;
    }
}

fn resume_suspended_key_union_missing_value(
    session: &mut Session,
    fill_function: Expr,
    plan: FrozenAssociationValue<KeyUnionPlan>,
    missing_keys: FrozenAssociationSequence<Expr>,
    index: usize,
    fill_values: PersistentAssociationSequence<(Expr, Expr)>,
    key: Expr,
    fill_value: Expr,
) -> Result<Expr, EvalError> {
    resume_suspended_key_union_missing_values_from_index(
        session,
        fill_function,
        plan,
        missing_keys,
        index + 1,
        fill_values.appended((key, fill_value)),
    )
}

fn key_union_entries(collections: &[Vec<AssociationEntryForm>]) -> Vec<AssociationEntryForm> {
    let mut remaining = collections.iter().flat_map(|entries| entries.iter());
    let mut union_entries = Vec::new();
    for entry in remaining.by_ref() {
        if union_entries
            .iter()
            .any(|existing: &AssociationEntryForm| existing.key == entry.key)
        {
            continue;
        }
        union_entries.push(entry.clone());
        if union_entries.len() > KEY_COLLECTION_INDEX_THRESHOLD {
            return key_union_entries_indexed(union_entries, remaining);
        }
    }
    union_entries
}

#[inline(never)]
fn key_union_entries_indexed<'a>(
    mut union_entries: Vec<AssociationEntryForm>,
    remaining: impl Iterator<Item = &'a AssociationEntryForm>,
) -> Vec<AssociationEntryForm> {
    let mut seen = union_entries
        .iter()
        .map(|entry| entry.key.clone())
        .collect::<HashSet<_>>();
    for entry in remaining {
        if seen.insert(entry.key.clone()) {
            union_entries.push(entry.clone());
        }
    }
    union_entries
}

fn render_key_union_result(plan: &KeyUnionPlan, fill_values: &[(Expr, Expr)]) -> Expr {
    let fill_positions = (fill_values.len() > KEY_COLLECTION_INDEX_THRESHOLD).then(|| {
        fill_values
            .iter()
            .enumerate()
            .map(|(index, (key, _))| (key, index))
            .collect::<HashMap<_, _>>()
    });
    Expr::call(
        system_expr("List"),
        plan.collections
            .iter()
            .enumerate()
            .map(|(collection_index, entries)| {
                render_association_entry_forms_preserving_heads(
                    plan.union_entries
                        .iter()
                        .map(|union_entry| {
                            key_collection_entry(
                                entries,
                                plan.indices
                                    .as_ref()
                                    .map(|indices| &indices[collection_index]),
                                &union_entry.key,
                            )
                            .cloned()
                            .or_else(|| {
                                let fill_value = if let Some(positions) = &fill_positions {
                                    positions
                                        .get(&union_entry.key)
                                        .map(|position| &fill_values[*position].1)
                                } else {
                                    fill_values
                                        .iter()
                                        .find(|(fill_key, _)| fill_key == &union_entry.key)
                                        .map(|(_, value)| value)
                                };
                                fill_value.map(|value| AssociationEntryForm {
                                    key: union_entry.key.clone(),
                                    value: value.clone(),
                                    delayed: union_entry.delayed,
                                })
                            })
                            .unwrap_or_else(|| AssociationEntryForm {
                                key: union_entry.key.clone(),
                                value: Expr::call(
                                    system_expr("Missing"),
                                    vec![Expr::string("KeyAbsent"), union_entry.key.clone()],
                                ),
                                delayed: union_entry.delayed,
                            })
                        })
                        .collect(),
                )
            })
            .collect(),
    )
}

fn collect_option_names_from_tail(session: &Session, tail: &[Expr]) -> Option<Vec<Symbol>> {
    let mut names = Vec::new();
    for expr in tail {
        collect_option_names_from_expr(session, expr, &mut names)?;
    }
    Some(names)
}

fn collect_option_names_from_expr(
    session: &Session,
    expr: &Expr,
    names: &mut Vec<Symbol>,
) -> Option<()> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            for item in args {
                collect_option_names_from_expr(session, item, names)?;
            }
            Some(())
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            names.push(session.normalize_symbol_target_expr(&args[0], true)?);
            Some(())
        }
        _ => None,
    }
}

fn exact_argument_count_message(symbol: &str, count: usize, expected: usize) -> String {
    let count_noun = if count == 1 { "argument" } else { "arguments" };
    let expected_noun = if expected == 1 {
        "argument is"
    } else {
        "arguments are"
    };
    format!("{symbol} called with {count} {count_noun}; {expected} {expected_noun} expected.")
}

fn non_option_syntax_message(
    offender: &Expr,
    positional_count: usize,
    held_input: &Expr,
) -> String {
    format!(
        "Options expected (instead of {offender}) beyond position {positional_count} in {held_input}. An option must be a rule or a list of rules."
    )
}

fn unknown_option_message(option_name: &Symbol, held_input: &Expr) -> String {
    format!("Unknown option {option_name} in {held_input}.")
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<KeyUnionPlan>();
};
