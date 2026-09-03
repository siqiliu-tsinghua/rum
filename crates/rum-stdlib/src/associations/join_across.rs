use rum_core::SharedArgs;
use std::collections::HashMap;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JoinType {
    Inner,
    Left,
    Right,
    Outer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct JoinKeySpec {
    left: Vec<Expr>,
    right: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct JoinRow {
    expr: Expr,
    entries: Vec<AssociationEntryForm>,
}

const JOIN_KEY_INDEX_THRESHOLD: usize = 16;

enum JoinKeyPositions {
    One(usize),
    Many(Vec<usize>),
}

enum JoinKeyLookup<'a> {
    Linear(&'a [Expr]),
    Indexed {
        positions: HashMap<&'a Expr, JoinKeyPositions>,
        len: usize,
    },
}

impl<'a> JoinKeyLookup<'a> {
    fn new(keys: &'a [Expr]) -> Self {
        if keys.len() <= JOIN_KEY_INDEX_THRESHOLD {
            return Self::Linear(keys);
        }

        let mut positions = HashMap::with_capacity(keys.len());
        for (position, key) in keys.iter().enumerate() {
            match positions.entry(key) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(JoinKeyPositions::One(position));
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let positions = entry.get_mut();
                    match positions {
                        JoinKeyPositions::One(first) => {
                            *positions = JoinKeyPositions::Many(vec![*first, position]);
                        }
                        JoinKeyPositions::Many(positions) => positions.push(position),
                    }
                }
            }
        }

        Self::Indexed {
            positions,
            len: keys.len(),
        }
    }

    #[inline]
    fn values(&self, row: &JoinRow) -> Option<Vec<Expr>> {
        match self {
            Self::Linear(keys) => join_value_tuple_linear(row, keys),
            Self::Indexed { positions, len } => join_value_tuple_indexed(row, positions, *len),
        }
    }
}

pub(crate) fn eval_join_across(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (left_expr, right_expr, key_spec_expr, join_type_expr) = match args {
        [left, right, key_spec] => (left, right, key_spec, None),
        [left, right, key_spec, join_type] => (left, right, key_spec, Some(join_type)),
        _ => {
            return Ok(session.invalid_call_with_message(
                "JoinAcross",
                args,
                "argb",
                join_across_argument_count_message(args.len()),
            ));
        }
    };

    let Some(left_rows) = join_rows(left_expr) else {
        return Ok(session.invalid_call_with_message(
            "JoinAcross",
            args,
            "lists",
            invalid_join_rows_message(1, args),
        ));
    };
    let Some(right_rows) = join_rows(right_expr) else {
        return Ok(session.invalid_call_with_message(
            "JoinAcross",
            args,
            "lists",
            invalid_join_rows_message(2, args),
        ));
    };
    let Some(key_spec) = parse_join_key_spec(key_spec_expr) else {
        return Ok(session.invalid_call_with_message(
            "JoinAcross",
            args,
            "keys",
            invalid_join_key_spec_message(key_spec_expr),
        ));
    };
    let join_type = match join_type_expr {
        None => JoinType::Inner,
        Some(expr) => {
            let Some(join_type) = parse_join_type(expr) else {
                return Ok(session.invalid_call_with_message(
                    "JoinAcross",
                    args,
                    "jtype",
                    invalid_join_type_message(expr),
                ));
            };
            join_type
        }
    };

    let rows = match join_type {
        JoinType::Inner => join_left_major(&left_rows, &right_rows, &key_spec, false, false),
        JoinType::Left => join_left_major(&left_rows, &right_rows, &key_spec, true, false),
        JoinType::Right => join_right_major(&left_rows, &right_rows, &key_spec),
        JoinType::Outer => join_left_major(&left_rows, &right_rows, &key_spec, true, true),
    };
    Ok(Expr::call(system_expr(heads::structure::LIST), rows))
}

fn join_rows(expr: &Expr) -> Option<Vec<JoinRow>> {
    list_items(expr)?
        .iter()
        .map(|row| {
            Some(JoinRow {
                expr: row.clone(),
                entries: association_entry_forms_from_expr(row)?,
            })
        })
        .collect()
}

fn parse_join_key_spec(expr: &Expr) -> Option<JoinKeySpec> {
    let components = if let Some(items) = list_items(expr) {
        if items.is_empty() {
            return None;
        }
        items
    } else {
        std::slice::from_ref(expr)
    };

    let mut left = Vec::with_capacity(components.len());
    let mut right = Vec::with_capacity(components.len());
    for component in components {
        let (left_key, right_key) = parse_join_key_component(component)?;
        left.push(left_key);
        right.push(right_key);
    }
    Some(JoinKeySpec { left, right })
}

fn parse_join_key_component(expr: &Expr) -> Option<(Expr, Expr)> {
    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(head, "Rule") && args.len() == 2 {
            return Some((parse_join_key(&args[0])?, parse_join_key(&args[1])?));
        }
    }

    let key = parse_join_key(expr)?;
    Some((key.clone(), key))
}

fn parse_join_key(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::String(_)) => Some(expr.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn parse_join_type(expr: &Expr) -> Option<JoinType> {
    let Expr::Atom(Atom::String(value)) = expr else {
        return None;
    };

    match value.as_str() {
        "Inner" => Some(JoinType::Inner),
        "Left" => Some(JoinType::Left),
        "Right" => Some(JoinType::Right),
        "Outer" => Some(JoinType::Outer),
        _ => None,
    }
}

fn join_left_major(
    left_rows: &[JoinRow],
    right_rows: &[JoinRow],
    key_spec: &JoinKeySpec,
    include_unmatched_left: bool,
    append_unmatched_right: bool,
) -> Vec<Expr> {
    let left_key_lookup = JoinKeyLookup::new(&key_spec.left);
    let right_key_lookup = JoinKeyLookup::new(&key_spec.right);
    let right_index = index_rows(right_rows, &right_key_lookup);
    let mut matched_right = vec![false; right_rows.len()];
    let mut joined = Vec::new();

    for left_row in left_rows {
        let matching_right = left_key_lookup
            .values(left_row)
            .and_then(|join_values| right_index.get(&join_values));
        if let Some(right_indices) = matching_right {
            for &right_index in right_indices {
                joined.push(join_rows_as_association(left_row, &right_rows[right_index]));
                matched_right[right_index] = true;
            }
        } else if include_unmatched_left {
            joined.push(left_row.expr.clone());
        }
    }

    if append_unmatched_right {
        joined.extend(
            right_rows
                .iter()
                .zip(matched_right)
                .filter(|(_, matched)| !matched)
                .map(|(row, _)| row.expr.clone()),
        );
    }

    joined
}

fn join_right_major(
    left_rows: &[JoinRow],
    right_rows: &[JoinRow],
    key_spec: &JoinKeySpec,
) -> Vec<Expr> {
    let left_key_lookup = JoinKeyLookup::new(&key_spec.left);
    let right_key_lookup = JoinKeyLookup::new(&key_spec.right);
    let left_index = index_rows(left_rows, &left_key_lookup);
    let mut joined = Vec::new();

    for right_row in right_rows {
        let matching_left = right_key_lookup
            .values(right_row)
            .and_then(|join_values| left_index.get(&join_values));
        if let Some(left_indices) = matching_left {
            for &left_index in left_indices {
                joined.push(join_rows_as_association(&left_rows[left_index], right_row));
            }
        } else {
            joined.push(right_row.expr.clone());
        }
    }

    joined
}

fn index_rows(rows: &[JoinRow], keys: &JoinKeyLookup<'_>) -> HashMap<Vec<Expr>, Vec<usize>> {
    let mut index = HashMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        if let Some(join_values) = keys.values(row) {
            index
                .entry(join_values)
                .or_insert_with(Vec::new)
                .push(row_index);
        }
    }
    index
}

fn join_value_tuple_linear(row: &JoinRow, keys: &[Expr]) -> Option<Vec<Expr>> {
    keys.iter()
        .map(|key| {
            row.entries
                .iter()
                .find(|entry| &entry.key == key)
                .map(|entry| entry.value.clone())
        })
        .collect()
}

fn join_value_tuple_indexed(
    row: &JoinRow,
    positions: &HashMap<&Expr, JoinKeyPositions>,
    len: usize,
) -> Option<Vec<Expr>> {
    let mut values = vec![None; len];
    let mut remaining = len;

    for entry in &row.entries {
        let Some(positions) = positions.get(&entry.key) else {
            continue;
        };

        match positions {
            JoinKeyPositions::One(position) => {
                values[*position] = Some(entry.value.clone());
                remaining -= 1;
            }
            JoinKeyPositions::Many(positions) => {
                for &position in positions {
                    values[position] = Some(entry.value.clone());
                    remaining -= 1;
                }
            }
        }

        if remaining == 0 {
            break;
        }
    }

    values.into_iter().collect()
}

fn join_rows_as_association(left: &JoinRow, right: &JoinRow) -> Expr {
    let mut entries = Vec::with_capacity(left.entries.len() + right.entries.len());
    entries.extend(left.entries.iter().cloned());
    entries.extend(right.entries.iter().cloned());
    render_association_entry_forms_preserving_heads(entries)
}

fn join_across_argument_count_message(count: usize) -> String {
    format!("JoinAcross called with {count} arguments; 3 or 4 arguments are expected.")
}

fn invalid_join_rows_message(position: usize, args: &[Expr]) -> String {
    let call = Expr::call(system_expr("JoinAcross"), args.to_vec());
    format!("The argument at position {position} in {call} is not a valid list of Associations.")
}

fn invalid_join_key_spec_message(expr: &Expr) -> String {
    format!(
        "The argument {expr} is not a valid key specification: a key, a rule of keys, or a non-empty list of these."
    )
}

fn invalid_join_type_message(expr: &Expr) -> String {
    format!(
        "The argument {expr} is not a valid join type; \"Inner\", \"Left\", \"Right\", or \"Outer\" is expected."
    )
}
