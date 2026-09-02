use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::{IntValue, Rational, SharedArgs};

const DELAYED_DELETE_MISSING_WRAPPER_PROXY_BYTES: usize = 64 * std::mem::size_of::<Expr>();

pub(crate) fn eval_missing(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(Expr::call(head.clone(), args.to_vec()))
}

pub(crate) fn eval_delete_missing(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, mode) = match args {
        [expr] => (
            expr,
            DeleteMissingMode::Recursive(DeleteMissingDepth::Finite(1)),
        ),
        [expr, depth_expr] => {
            let Some(depth) = parse_delete_missing_positive_depth(depth_expr) else {
                return Ok(session.invalid_call_with_message(
                    "DeleteMissing",
                    args,
                    "arg2",
                    delete_missing_arg2_message(depth_expr),
                ));
            };
            (expr, DeleteMissingMode::Recursive(depth))
        }
        [expr, level_expr, scan_expr] => {
            let Some(level) = parse_delete_missing_positive_depth(level_expr) else {
                return Ok(session.invalid_call_with_message(
                    "DeleteMissing",
                    args,
                    "arg2",
                    delete_missing_arg2_message(level_expr),
                ));
            };
            let Some(scan) = parse_delete_missing_scan_depth(scan_expr) else {
                return Ok(session.invalid_call_with_message(
                    "DeleteMissing",
                    args,
                    "arg3",
                    delete_missing_arg3_message(scan_expr),
                ));
            };
            (expr, DeleteMissingMode::Target { level, scan })
        }
        _ => {
            return Ok(session.invalid_call_with_message(
                "DeleteMissing",
                args,
                "argb",
                delete_missing_arg_count_message(args.len()),
            ));
        }
    };

    enforce_delete_missing_memory_constraint(session, expr, mode)?;

    let Some(value) = delete_missing_from_keyed_data(session, expr, mode)? else {
        return Ok(session.invalid_call_with_message(
            "DeleteMissing",
            args,
            "invrp",
            invalid_association_or_list_message(expr),
        ));
    };

    Ok(value)
}

fn enforce_delete_missing_memory_constraint(
    session: &Session,
    expr: &Expr,
    mode: DeleteMissingMode,
) -> Result<(), EvalError> {
    if session.active_memory_limit().is_none() {
        return session.enforce_active_memory_limit_for_projected_bytes(0);
    }

    let keyed_input = list_items(expr).is_some()
        || matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, "Association"));
    if !keyed_input {
        return Ok(());
    }

    let wrapper_count = delete_missing_delayed_wrapper_count(expr, mode);
    let projected_bytes = projected_expr_memory_proxy_bytes(expr)
        .saturating_add(wrapper_count.saturating_mul(DELAYED_DELETE_MISSING_WRAPPER_PROXY_BYTES));
    session.enforce_active_memory_limit_for_projected_bytes(projected_bytes)
}

fn delete_missing_delayed_wrapper_count(expr: &Expr, mode: DeleteMissingMode) -> usize {
    match mode {
        DeleteMissingMode::Recursive(depth) => recursive_delayed_wrapper_count(expr, depth),
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Infinity,
            ..
        } => recursive_delayed_wrapper_count(expr, DeleteMissingDepth::Infinity),
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Finite(level),
            ..
        } => target_delayed_wrapper_count(expr, level),
    }
}

fn recursive_delayed_wrapper_count(expr: &Expr, depth: DeleteMissingDepth) -> usize {
    let Some(child_depth) = depth.descend() else {
        return 0;
    };

    if let Some(items) = list_items(expr) {
        return items.iter().fold(0usize, |total, item| {
            total.saturating_add(recursive_delayed_wrapper_count(item, child_depth))
        });
    }

    let Expr::Call { head, args } = expr else {
        return 0;
    };
    if !is_system_symbol_name(head, "Association") {
        return 0;
    }
    recursive_association_item_wrapper_count(args, child_depth)
}

fn recursive_association_item_wrapper_count(
    items: &[Expr],
    child_depth: DeleteMissingDepth,
) -> usize {
    items.iter().fold(0usize, |total, item| {
        total.saturating_add(match item {
            Expr::Call { head, args }
                if (is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                if is_system_symbol_name(head, "RuleDelayed") {
                    1
                } else {
                    recursive_delayed_wrapper_count(&args[1], child_depth)
                }
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Association")
                    || is_system_symbol_name(head, "List") =>
            {
                recursive_association_item_wrapper_count(args, child_depth)
            }
            _ => 0,
        })
    })
}

fn target_delayed_wrapper_count(expr: &Expr, level: usize) -> usize {
    if level <= 1 {
        return 0;
    }

    if let Some(items) = list_items(expr) {
        return items.iter().fold(0usize, |total, item| {
            total.saturating_add(target_delayed_wrapper_count(item, level - 1))
        });
    }

    let Expr::Call { head, args } = expr else {
        return 0;
    };
    if !is_system_symbol_name(head, "Association") {
        return 0;
    }
    target_association_item_wrapper_count(args, level - 1)
}

fn target_association_item_wrapper_count(items: &[Expr], child_level: usize) -> usize {
    items.iter().fold(0usize, |total, item| {
        total.saturating_add(match item {
            Expr::Call { head, args }
                if (is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed"))
                    && args.len() == 2 =>
            {
                if is_system_symbol_name(head, "RuleDelayed") {
                    1
                } else {
                    target_delayed_wrapper_count(&args[1], child_level)
                }
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Association")
                    || is_system_symbol_name(head, "List") =>
            {
                target_association_item_wrapper_count(args, child_level)
            }
            _ => 0,
        })
    })
}

fn projected_expr_memory_proxy_bytes(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(Atom::Integer(IntValue::Small(_))) => std::mem::size_of::<Expr>(),
        Expr::Atom(Atom::Integer(IntValue::Big(value))) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<BigInt>())
            .saturating_add(projected_bigint_payload_bytes(value)),
        Expr::Atom(Atom::Rational(value)) => std::mem::size_of::<Expr>()
            .saturating_add(std::mem::size_of::<Rational>())
            .saturating_add(projected_bigint_payload_bytes(value.numerator()))
            .saturating_add(projected_bigint_payload_bytes(value.denominator())),
        Expr::Atom(Atom::String(value)) => std::mem::size_of::<Expr>().saturating_add(value.len()),
        Expr::Atom(Atom::Symbol(symbol)) => std::mem::size_of::<Expr>()
            .saturating_add(symbol.context().len())
            .saturating_add(symbol.name().len()),
        Expr::Call { head, args } => args.iter().fold(
            std::mem::size_of::<Expr>().saturating_add(projected_expr_memory_proxy_bytes(head)),
            |total, arg| total.saturating_add(projected_expr_memory_proxy_bytes(arg)),
        ),
    }
}

fn projected_bigint_payload_bytes(value: &BigInt) -> usize {
    let bytes = u128::from(value.bits()).saturating_add(7) / 8;
    usize::try_from(bytes.saturating_add(1)).unwrap_or(usize::MAX)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum DeleteMissingMode {
    Recursive(DeleteMissingDepth),
    Target {
        level: DeleteMissingDepth,
        scan: DeleteMissingDepth,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum DeleteMissingDepth {
    Finite(usize),
    Infinity,
}

impl DeleteMissingDepth {
    fn descend(self) -> Option<Self> {
        match self {
            Self::Finite(depth) if depth > 1 => Some(Self::Finite(depth - 1)),
            Self::Finite(_) => None,
            Self::Infinity => Some(Self::Infinity),
        }
    }

    fn descend_scan(self) -> Option<Self> {
        match self {
            Self::Finite(depth) if depth > 0 => Some(Self::Finite(depth - 1)),
            Self::Finite(_) => None,
            Self::Infinity => Some(Self::Infinity),
        }
    }

    fn expr(self) -> Expr {
        match self {
            Self::Finite(depth) => Expr::integer(BigInt::from(depth)),
            Self::Infinity => system_expr("Infinity"),
        }
    }
}

fn parse_delete_missing_positive_depth(expr: &Expr) -> Option<DeleteMissingDepth> {
    if is_system_symbol_name(expr, "Infinity") || is_system_symbol_name(expr, "All") {
        return Some(DeleteMissingDepth::Infinity);
    }

    expr_as_non_negative_usize(expr)
        .filter(|depth| *depth > 0)
        .map(DeleteMissingDepth::Finite)
}

fn parse_delete_missing_scan_depth(expr: &Expr) -> Option<DeleteMissingDepth> {
    if is_system_symbol_name(expr, "Infinity") || is_system_symbol_name(expr, "All") {
        return Some(DeleteMissingDepth::Infinity);
    }

    expr_as_non_negative_usize(expr).map(DeleteMissingDepth::Finite)
}

fn delete_missing_arg2_message(expr: &Expr) -> String {
    format!("The second argument {expr} is expected to be a positive integer or Infinity.")
}

fn delete_missing_arg3_message(expr: &Expr) -> String {
    format!("The third argument {expr} is expected to be a non-negative integer or Infinity.")
}

fn delete_missing_arg_count_message(count: usize) -> String {
    format!("DeleteMissing called with {count} arguments; between 1 and 3 arguments are expected.")
}

fn delete_missing_from_keyed_data(
    session: &mut Session,
    expr: &Expr,
    mode: DeleteMissingMode,
) -> Result<Option<Expr>, EvalError> {
    match mode {
        DeleteMissingMode::Recursive(depth) => {
            Ok(delete_missing_from_keyed_data_recursive(expr, depth))
        }
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Infinity,
            ..
        } => Ok(delete_missing_from_keyed_data_recursive(
            expr,
            DeleteMissingDepth::Infinity,
        )),
        DeleteMissingMode::Target {
            level: DeleteMissingDepth::Finite(level),
            scan,
        } => delete_missing_at_target_level(session, expr, level, scan),
    }
}

fn delete_missing_at_target_level(
    session: &mut Session,
    expr: &Expr,
    level: usize,
    scan: DeleteMissingDepth,
) -> Result<Option<Expr>, EvalError> {
    if let Some(items) = list_items(expr) {
        return resume_delete_missing_list_at_target_level(
            session,
            items.to_vec(),
            level,
            scan,
            0,
            Vec::new(),
        )
        .map(Some);
    }

    let Some(entries) = association_entry_forms_from_expr(expr) else {
        return Ok(None);
    };
    resume_delete_missing_association_at_target_level(session, entries, level, scan, 0, Vec::new())
        .map(Some)
}

fn resume_delete_missing_list_at_target_level(
    session: &mut Session,
    items: Vec<Expr>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    mut kept: Vec<Expr>,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        return Ok(Expr::call(system_expr("List"), kept));
    };

    if level == 1 {
        if !rule_like_expr_has_missing_value(&item) && !contains_missing_within(&item, scan) {
            kept.push(item);
        }

        return resume_delete_missing_list_at_target_level(
            session,
            items,
            level,
            scan,
            index + 1,
            kept,
        );
    }

    if let Some(child_items) = list_items(&item) {
        let child_result = resume_delete_missing_list_at_target_level(
            session,
            child_items.to_vec(),
            level - 1,
            scan,
            0,
            Vec::new(),
        );
        return match child_result {
            Ok(child_value) => resume_delete_missing_list_after_child(
                session,
                items,
                level,
                scan,
                index,
                kept,
                child_value,
            ),
            Err(EvalError::InputRequired { request }) => {
                rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let items = FrozenAssociationSequence::freeze(items);
                                    let kept = PersistentAssociationSequence::freeze(kept);
                ResumeDeleteMissingListAtTargetLevelSuspensionFrame1 { items, level, scan, index, kept }
                },
                )
            }
            Err(error) => Err(error),
        };
    }

    if let Some(child_entries) = association_entry_forms_from_expr(&item) {
        let child_result = resume_delete_missing_association_at_target_level(
            session,
            child_entries,
            level - 1,
            scan,
            0,
            Vec::new(),
        );
        return match child_result {
            Ok(child_value) => resume_delete_missing_list_after_child(
                session,
                items,
                level,
                scan,
                index,
                kept,
                child_value,
            ),
            Err(EvalError::InputRequired { request }) => {
                rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let items = FrozenAssociationSequence::freeze(items);
                                    let kept = PersistentAssociationSequence::freeze(kept);
                ResumeDeleteMissingListAtTargetLevelSuspensionFrame2 { items, level, scan, index, kept }
                },
                )
            }
            Err(error) => Err(error),
        };
    }

    kept.push(item);
    resume_delete_missing_list_at_target_level(session, items, level, scan, index + 1, kept)
}

fn resume_delete_missing_list_after_child(
    session: &mut Session,
    items: Vec<Expr>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    mut kept: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    kept.push(value);
    resume_delete_missing_list_at_target_level(session, items, level, scan, index + 1, kept)
}

fn resume_suspended_delete_missing_list_at_target_level(
    session: &mut Session,
    items: FrozenAssociationSequence<Expr>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    kept: PersistentAssociationSequence<Expr>,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        return Ok(Expr::call(system_expr("List"), kept.to_vec()));
    };

    if level == 1 {
        let kept =
            if !rule_like_expr_has_missing_value(&item) && !contains_missing_within(&item, scan) {
                kept.appended(item)
            } else {
                kept
            };
        return resume_suspended_delete_missing_list_at_target_level(
            session,
            items,
            level,
            scan,
            index + 1,
            kept,
        );
    }

    let child_result = if let Some(child_items) = list_items(&item) {
        Some(resume_delete_missing_list_at_target_level(
            session,
            child_items.to_vec(),
            level - 1,
            scan,
            0,
            Vec::new(),
        ))
    } else {
        association_entry_forms_from_expr(&item).map(|child_entries| {
            resume_delete_missing_association_at_target_level(
                session,
                child_entries,
                level - 1,
                scan,
                0,
                Vec::new(),
            )
        })
    };

    let Some(child_result) = child_result else {
        return resume_suspended_delete_missing_list_at_target_level(
            session,
            items,
            level,
            scan,
            index + 1,
            kept.appended(item),
        );
    };

    match child_result {
        Ok(value) => resume_suspended_delete_missing_list_after_child(
            session, items, level, scan, index, kept, value,
        ),
        Err(EvalError::InputRequired { request }) => {
            rum_runtime::resume_or_suspend!(
                session,
                Err(EvalError::InputRequired { request }),
                ok(value) => Ok(value),
                suspend => ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrame { items, level, scan, index, kept },
            )
        }
        Err(error) => Err(error),
    }
}

fn resume_suspended_delete_missing_list_after_child(
    session: &mut Session,
    items: FrozenAssociationSequence<Expr>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    kept: PersistentAssociationSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    resume_suspended_delete_missing_list_at_target_level(
        session,
        items,
        level,
        scan,
        index + 1,
        kept.appended(value),
    )
}

fn resume_delete_missing_association_at_target_level(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    mut kept: Vec<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    let Some(mut entry) = entries.get(index).cloned() else {
        return Ok(render_association_entry_forms_preserving_heads(kept));
    };

    if is_missing_expr(&entry.value) {
        return resume_delete_missing_association_at_target_level(
            session,
            entries,
            level,
            scan,
            index + 1,
            kept,
        );
    }

    if level == 1 {
        let predicate_result = if entry.delayed {
            session.evaluate(entry.value.clone())
        } else {
            Ok(entry.value.clone())
        };
        return match predicate_result {
            Ok(predicate_value) => resume_delete_missing_association_after_predicate(
                session,
                entries,
                level,
                scan,
                index,
                kept,
                entry,
                predicate_value,
            ),
            Err(EvalError::InputRequired { request }) => {
                rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let entries = FrozenAssociationSequence::freeze(entries);
                                    let kept = PersistentAssociationSequence::freeze(kept);
                ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame1 { entries, level, scan, index, kept, entry }
                },
                )
            }
            Err(error) => Err(error),
        };
    }

    if entry.delayed {
        entry.value = delete_missing_delayed_wrapper(
            entry.value,
            DeleteMissingMode::Target {
                level: DeleteMissingDepth::Finite(level - 1),
                scan,
            },
        );
        kept.push(entry);
        return resume_delete_missing_association_at_target_level(
            session,
            entries,
            level,
            scan,
            index + 1,
            kept,
        );
    }

    if let Some(child_items) = list_items(&entry.value) {
        let child_result = resume_delete_missing_list_at_target_level(
            session,
            child_items.to_vec(),
            level - 1,
            scan,
            0,
            Vec::new(),
        );
        return match child_result {
            Ok(child_value) => resume_delete_missing_association_after_child(
                session,
                entries,
                level,
                scan,
                index,
                kept,
                entry,
                child_value,
            ),
            Err(EvalError::InputRequired { request }) => {
                rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let entries = FrozenAssociationSequence::freeze(entries);
                                    let kept = PersistentAssociationSequence::freeze(kept);
                ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame2 { entries, level, scan, index, kept, entry }
                },
                )
            }
            Err(error) => Err(error),
        };
    }

    if let Some(child_entries) = association_entry_forms_from_expr(&entry.value) {
        let child_result = resume_delete_missing_association_at_target_level(
            session,
            child_entries,
            level - 1,
            scan,
            0,
            Vec::new(),
        );
        return match child_result {
            Ok(child_value) => resume_delete_missing_association_after_child(
                session,
                entries,
                level,
                scan,
                index,
                kept,
                entry,
                child_value,
            ),
            Err(EvalError::InputRequired { request }) => {
                rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let entries = FrozenAssociationSequence::freeze(entries);
                                    let kept = PersistentAssociationSequence::freeze(kept);
                ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrame3 { entries, level, scan, index, kept, entry }
                },
                )
            }
            Err(error) => Err(error),
        };
    }

    kept.push(entry);
    resume_delete_missing_association_at_target_level(
        session,
        entries,
        level,
        scan,
        index + 1,
        kept,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_delete_missing_association_after_predicate(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    mut kept: Vec<AssociationEntryForm>,
    entry: AssociationEntryForm,
    predicate_value: Expr,
) -> Result<Expr, EvalError> {
    if !is_missing_expr(&predicate_value) && !contains_missing_within(&predicate_value, scan) {
        kept.push(entry);
    }

    resume_delete_missing_association_at_target_level(
        session,
        entries,
        level,
        scan,
        index + 1,
        kept,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_delete_missing_association_after_child(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    mut kept: Vec<AssociationEntryForm>,
    mut entry: AssociationEntryForm,
    value: Expr,
) -> Result<Expr, EvalError> {
    entry.value = value;
    kept.push(entry);
    resume_delete_missing_association_at_target_level(
        session,
        entries,
        level,
        scan,
        index + 1,
        kept,
    )
}

fn resume_suspended_delete_missing_association_at_target_level(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    kept: PersistentAssociationSequence<AssociationEntryForm>,
) -> Result<Expr, EvalError> {
    let Some(mut entry) = entries.get(index).cloned() else {
        return Ok(render_association_entry_forms_preserving_heads(
            kept.to_vec(),
        ));
    };

    if is_missing_expr(&entry.value) {
        return resume_suspended_delete_missing_association_at_target_level(
            session,
            entries,
            level,
            scan,
            index + 1,
            kept,
        );
    }

    if level == 1 {
        let predicate_result = if entry.delayed {
            session.evaluate(entry.value.clone())
        } else {
            Ok(entry.value.clone())
        };
        return match predicate_result {
            Ok(value) => resume_suspended_delete_missing_association_after_predicate(
                session, entries, level, scan, index, kept, entry, value,
            ),
            Err(EvalError::InputRequired { request }) => {
                rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame1 { entries, level, scan, index, kept, entry },
                )
            }
            Err(error) => Err(error),
        };
    }

    if entry.delayed {
        entry.value = delete_missing_delayed_wrapper(
            entry.value,
            DeleteMissingMode::Target {
                level: DeleteMissingDepth::Finite(level - 1),
                scan,
            },
        );
        return resume_suspended_delete_missing_association_at_target_level(
            session,
            entries,
            level,
            scan,
            index + 1,
            kept.appended(entry),
        );
    }

    let child_result = if let Some(child_items) = list_items(&entry.value) {
        Some(resume_delete_missing_list_at_target_level(
            session,
            child_items.to_vec(),
            level - 1,
            scan,
            0,
            Vec::new(),
        ))
    } else {
        association_entry_forms_from_expr(&entry.value).map(|child_entries| {
            resume_delete_missing_association_at_target_level(
                session,
                child_entries,
                level - 1,
                scan,
                0,
                Vec::new(),
            )
        })
    };

    let Some(child_result) = child_result else {
        return resume_suspended_delete_missing_association_at_target_level(
            session,
            entries,
            level,
            scan,
            index + 1,
            kept.appended(entry),
        );
    };

    match child_result {
        Ok(value) => resume_suspended_delete_missing_association_after_child(
            session, entries, level, scan, index, kept, entry, value,
        ),
        Err(EvalError::InputRequired { request }) => {
            rum_runtime::resume_or_suspend!(
                session,
                Err(EvalError::InputRequired { request }),
                ok(value) => Ok(value),
                suspend => ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrame2 { entries, level, scan, index, kept, entry },
            )
        }
        Err(error) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_delete_missing_association_after_predicate(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    kept: PersistentAssociationSequence<AssociationEntryForm>,
    entry: AssociationEntryForm,
    predicate_value: Expr,
) -> Result<Expr, EvalError> {
    let kept =
        if !is_missing_expr(&predicate_value) && !contains_missing_within(&predicate_value, scan) {
            kept.appended(entry)
        } else {
            kept
        };
    resume_suspended_delete_missing_association_at_target_level(
        session,
        entries,
        level,
        scan,
        index + 1,
        kept,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_delete_missing_association_after_child(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    level: usize,
    scan: DeleteMissingDepth,
    index: usize,
    kept: PersistentAssociationSequence<AssociationEntryForm>,
    mut entry: AssociationEntryForm,
    value: Expr,
) -> Result<Expr, EvalError> {
    entry.value = value;
    resume_suspended_delete_missing_association_at_target_level(
        session,
        entries,
        level,
        scan,
        index + 1,
        kept.appended(entry),
    )
}

fn delete_missing_from_keyed_data_recursive(
    expr: &Expr,
    depth: DeleteMissingDepth,
) -> Option<Expr> {
    if let Some(items) = list_items(expr) {
        return Some(delete_missing_from_list_recursive(items, depth));
    }

    let entries = association_entry_forms_from_expr(expr)?;
    Some(render_association_entry_forms_preserving_heads(
        delete_missing_from_association_entries_recursive(entries, depth),
    ))
}

fn delete_missing_from_list_recursive(items: &[Expr], depth: DeleteMissingDepth) -> Expr {
    Expr::call(
        system_expr("List"),
        items
            .iter()
            .filter_map(|item| delete_missing_list_item_recursive(item, depth))
            .collect(),
    )
}

fn delete_missing_list_item_recursive(item: &Expr, depth: DeleteMissingDepth) -> Option<Expr> {
    if is_missing_expr(item) || rule_like_expr_has_missing_value(item) {
        return None;
    }

    if let Some(child_depth) = depth.descend() {
        if list_items(item).is_some() || association_entry_forms_from_expr(item).is_some() {
            return delete_missing_from_keyed_data_recursive(item, child_depth);
        }
    }

    Some(item.clone())
}

fn delete_missing_from_association_entries_recursive(
    entries: Vec<AssociationEntryForm>,
    depth: DeleteMissingDepth,
) -> Vec<AssociationEntryForm> {
    entries
        .into_iter()
        .filter_map(|mut entry| {
            if is_missing_expr(&entry.value) {
                return None;
            }

            if !entry.delayed {
                if let Some(child_depth) = depth.descend() {
                    if let Some(value) =
                        delete_missing_from_keyed_data_recursive(&entry.value, child_depth)
                    {
                        entry.value = value;
                    }
                }
            } else if let Some(child_depth) = depth.descend() {
                entry.value = delete_missing_delayed_wrapper(
                    entry.value,
                    DeleteMissingMode::Recursive(child_depth),
                );
            }

            Some(entry)
        })
        .collect()
}

fn delete_missing_delayed_wrapper(value: Expr, mode: DeleteMissingMode) -> Expr {
    let local = system_expr("rum$deleteMissingValue");
    let cleanup = Expr::call(
        system_expr("DeleteMissing"),
        delete_missing_call_args(local.clone(), mode),
    );
    let should_cleanup = Expr::call(
        system_expr("Or"),
        vec![
            Expr::call(system_expr("ListQ"), vec![local.clone()]),
            Expr::call(system_expr("AssociationQ"), vec![local.clone()]),
        ],
    );

    Expr::call(
        system_expr("With"),
        vec![
            Expr::call(
                system_expr("List"),
                vec![Expr::call(system_expr("Rule"), vec![local.clone(), value])],
            ),
            Expr::call(system_expr("If"), vec![should_cleanup, cleanup, local]),
        ],
    )
}

fn delete_missing_call_args(expr: Expr, mode: DeleteMissingMode) -> Vec<Expr> {
    match mode {
        DeleteMissingMode::Recursive(depth) => vec![expr, depth.expr()],
        DeleteMissingMode::Target { level, scan } => vec![expr, level.expr(), scan.expr()],
    }
}

fn contains_missing_within(expr: &Expr, depth: DeleteMissingDepth) -> bool {
    if is_missing_expr(expr) {
        return true;
    }

    let Some(child_depth) = depth.descend_scan() else {
        return false;
    };

    let Expr::Call { args, .. } = expr else {
        return false;
    };

    args.iter()
        .any(|arg| contains_missing_within(arg, child_depth))
}

fn rule_like_expr_has_missing_value(expr: &Expr) -> bool {
    parse_association_entry_form(expr).is_some_and(|entry| is_missing_expr(&entry.value))
}
