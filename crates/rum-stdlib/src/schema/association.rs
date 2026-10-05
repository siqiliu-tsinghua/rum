//! Association selection and ordered completion; never evaluates a child.
use super::format::{push_error, render, render_key_segment};
use super::parse::{parse_association_schema, parse_entry_spec};
use super::state::{
    AssociationWork, EntrySpec, SchemaAbort, SchemaDestination, SchemaError, SchemaOptions,
    SchemaPath, SchemaState, SchemaTask,
};
use rum_core::Expr;
use rum_runtime::kernel_api::{association_entries_from_expr, expr_as_string, render_association};
use std::collections::HashMap;

const ASSOCIATION_SCHEMA_INDEX_THRESHOLD: usize = 16;

pub(super) fn enter_association(
    state: &mut SchemaState,
    value: &Expr,
    schema: &Expr,
) -> Result<Option<AssociationWork>, SchemaAbort> {
    let schema_entries = parse_association_schema(schema, &state.path)?;
    let Some(value_entries) = association_entries_from_expr(value) else {
        push_error(
            &mut state.errors,
            &state.path,
            format!("expected Association, got {}", render(value)),
        );
        return Ok(None);
    };
    let matched = vec![false; value_entries.len()];
    let conformed_by_index = vec![None; value_entries.len()];
    let default_entries = Vec::new();
    let positions = (schema_entries.len() > ASSOCIATION_SCHEMA_INDEX_THRESHOLD
        && value_entries.len() > ASSOCIATION_SCHEMA_INDEX_THRESHOLD)
        .then(|| {
            value_entries
                .iter()
                .enumerate()
                .filter_map(|(index, (key, _))| {
                    expr_as_string(key).map(|key| (key.to_string(), index))
                })
                .collect::<HashMap<_, _>>()
        });
    Ok(Some(AssociationWork {
        schema_entries,
        value_entries,
        next_schema: 0,
        matched,
        conformed_by_index,
        default_entries,
        positions,
        path_len: state.path.rendered.len(),
        waiting: None,
    }))
}

pub(super) fn next_association(
    work: &mut AssociationWork,
    path: &mut SchemaPath,
    errors: &mut Vec<SchemaError>,
) -> Result<Option<SchemaTask>, SchemaAbort> {
    while let Some(schema_entry) = work.schema_entries.get(work.next_schema) {
        path.append_key(&schema_entry.key);
        let entry_spec = parse_entry_spec(&schema_entry.spec, path)?;
        work.next_schema += 1;
        let value_index = match &work.positions {
            Some(positions) => positions.get(schema_entry.key.as_str()).copied(),
            None => work
                .value_entries
                .iter()
                .position(|(key, _)| expr_as_string(key) == Some(schema_entry.key.as_str())),
        };
        if let Some(index) = value_index {
            work.matched[index] = true;
            work.waiting = Some(SchemaDestination::ExistingEntry { index });
            return Ok(Some(SchemaTask::Visit {
                value: work.value_entries[index].1.clone(),
                schema: entry_spec.validation_spec().clone(),
            }));
        }
        match entry_spec {
            EntrySpec::Required(_) => push_error(errors, path, "missing required key"),
            EntrySpec::Optional { default, .. } if default.is_none() => {}
            EntrySpec::Optional { spec, default } => {
                work.waiting = Some(SchemaDestination::DefaultEntry {
                    key: schema_entry.key.clone(),
                });
                return Ok(Some(SchemaTask::Default {
                    value: default.expect("default was checked above"),
                    schema: spec,
                }));
            }
        }
        path.truncate(work.path_len);
    }
    Ok(None)
}

pub(super) fn finish_association(
    mut work: AssociationWork,
    options: SchemaOptions,
    path: &mut SchemaPath,
    errors: &mut Vec<SchemaError>,
) -> Expr {
    for (index, (key, _)) in work.value_entries.iter().enumerate() {
        if !work.matched[index] && !options.allow_extra_keys {
            path.append_key(&render_key_segment(key));
            push_error(errors, path, "extra key is not allowed");
            path.truncate(work.path_len);
        }
    }
    drop(work.positions);
    let mut output_entries = Vec::new();
    for (index, (key, value)) in work.value_entries.into_iter().enumerate() {
        if work.matched[index] {
            let conformed = work.conformed_by_index[index]
                .take()
                .expect("matched schema value should have a conformed value");
            output_entries.push((key, conformed));
        } else if options.allow_extra_keys {
            output_entries.push((key, value));
        }
    }
    output_entries.extend(work.default_entries);
    render_association(output_entries)
}
