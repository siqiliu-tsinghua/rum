mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod key_predicates;
mod key_take_drop;
mod keys;
mod lookup;

pub(crate) use self::key_predicates::{eval_key_free_q, eval_key_member_q};
pub(crate) use self::key_take_drop::{eval_key_drop, eval_key_take};
pub(crate) use self::keys::eval_keys;
pub(crate) use self::lookup::eval_lookup;
use self::lookup::{
    eval_lookup_with_evaluated_data, finish_eval_lookup, finish_suspended_prepared_lookup,
};

pub(crate) fn eval_association(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    let source_items = match args_slice {
        [
            Expr::Call {
                head: list_head,
                args: items,
            },
        ] if is_system_symbol_name(list_head, heads::structure::LIST) => items.clone(),
        _ => args.clone(),
    };

    let Some(entries) = parse_association_entry_forms_deep(&source_items) else {
        return Ok(Expr::call_with_shared_args(head.clone(), args.clone()));
    };

    resume_association_key_evaluation(session, entries, Vec::new(), 0)
}

fn resume_association_key_evaluation(
    session: &mut Session,
    entries: Vec<AssociationEntryForm>,
    mut evaluated_entries: Vec<AssociationEntryForm>,
    mut index: usize,
) -> Result<Expr, EvalError> {
    loop {
        if index >= entries.len() {
            return Ok(render_association_entry_forms_preserving_heads(
                evaluated_entries,
            ));
        }

        let mut entry = entries[index].clone();
        let key = match session.evaluate(entry.key.clone()) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let entries = FrozenAssociationSequence::freeze(entries);
                                        let evaluated_entries =
                                            PersistentAssociationSequence::freeze(evaluated_entries);
                ResumeAssociationKeyEvaluationSuspensionFrame { entries, evaluated_entries, index, entry }
                },
                );
            }
            Err(error) => return Err(error),
        };

        entry.key = key;
        evaluated_entries.push(entry);
        index += 1;
    }
}

fn resume_suspended_association_key_evaluation(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    mut evaluated_entries: PersistentAssociationSequence<AssociationEntryForm>,
    mut index: usize,
) -> Result<Expr, EvalError> {
    loop {
        let Some(mut entry) = entries.get(index).cloned() else {
            return Ok(render_association_entry_forms_preserving_heads(
                evaluated_entries.to_vec(),
            ));
        };

        let key = match session.evaluate(entry.key.clone()) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedAssociationKeyEvaluationSuspensionFrame { entries, evaluated_entries, index, entry },
                );
            }
            Err(error) => return Err(error),
        };

        entry.key = key;
        evaluated_entries = evaluated_entries.appended(entry);
        index += 1;
    }
}

fn resume_suspended_association_key_evaluation_with_key(
    session: &mut Session,
    entries: FrozenAssociationSequence<AssociationEntryForm>,
    evaluated_entries: PersistentAssociationSequence<AssociationEntryForm>,
    index: usize,
    mut entry: AssociationEntryForm,
    key: Expr,
) -> Result<Expr, EvalError> {
    entry.key = key;
    let evaluated_entries = evaluated_entries.appended(entry);
    resume_suspended_association_key_evaluation(session, entries, evaluated_entries, index + 1)
}

pub(crate) fn eval_association_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr] = args else {
        return Ok(session.invalid_call_with_message(
            "AssociationQ",
            args,
            "argx",
            exact_argument_count_message("AssociationQ", args.len(), 1),
        ));
    };

    Ok(predicate_bool_expr(
        association_entries_from_expr(expr).is_some(),
    ))
}

pub(crate) fn eval_key(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [key] = args else {
        return Ok(session.invalid_call_with_message(
            "Key",
            args,
            "argx",
            exact_argument_count_message("Key", args.len(), 1),
        ));
    };

    Ok(Expr::call(head.clone(), vec![key.clone()]))
}

pub(crate) fn eval_values(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_message(
            "Values",
            args,
            "argt",
            one_or_two_argument_count_message("Values", args.len()),
        ));
    };

    let wrapper = args.get(1);
    match values_result_for_expr(&args[0], wrapper) {
        Ok(value) => Ok(value),
        Err(invalid) => Ok(session.invalid_call_with_message(
            "Values",
            args,
            "invrl",
            invalid_association_or_rule_list_message(&invalid),
        )),
    }
}

pub(crate) fn eval_key_exists_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, key) = match args {
        [key] => return Ok(keyed_operator_form_slot_first("KeyExistsQ", key.clone())),
        [expr, key] => (expr, key),
        _ => {
            return Ok(session.invalid_call_with_message(
                "KeyExistsQ",
                args,
                "argt",
                one_or_two_argument_count_message("KeyExistsQ", args.len()),
            ));
        }
    };

    let Some(entries) = single_rule_or_keyed_entries_from_expr(expr) else {
        session.push_message(
            "KeyExistsQ",
            "invrl",
            invalid_association_or_rule_list_message(expr),
        );
        return Ok(predicate_bool_expr(false));
    };

    Ok(predicate_bool_expr(
        entries.iter().any(|(entry_key, _)| entry_key == key),
    ))
}

pub(crate) fn eval_normal(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if args.is_empty() || args.len() > 3 {
        return Ok(session.invalid_call_with_message(
            "Normal",
            args,
            "argb",
            normal_arg_count_message(args.len()),
        ));
    }

    if let [expr] = args {
        if crate::lazy::is_lazy_list(expr) {
            return Ok(crate::lazy::realize_all(session, expr)?
                .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())));
        }
    }

    match args {
        [expr] => Ok(normalize_association_expr(expr)),
        [expr, target] => {
            if normal_target_matches_association(target) {
                return Ok(normalize_association_expr(expr));
            }

            Ok(expr.clone())
        }
        _ => Ok(Expr::call(head.clone(), args.to_vec())),
    }
}

pub(crate) fn eval_key_value_pattern(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [spec] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let Some(normalized_spec) = normalize_key_value_pattern_spec(spec) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::call(head.clone(), vec![normalized_spec]))
}

fn normal_arg_count_message(count: usize) -> String {
    format!("Normal called with {count} arguments; between 1 and 3 arguments are expected.")
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

fn one_or_two_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; 1 or 2 arguments are expected.")
}

fn normalize_association_expr(expr: &Expr) -> Expr {
    if let Some(entries) = association_rule_exprs_preserving_heads(expr) {
        return Expr::call(system_expr("List"), entries);
    }

    match expr {
        Expr::Call { head, args } => Expr::call(
            (**head).clone(),
            args.iter().map(normalize_association_expr).collect(),
        ),
        Expr::Atom(_) => expr.clone(),
    }
}

fn normal_target_matches_association(target: &Expr) -> bool {
    is_system_symbol_name(target, "Association")
        || is_system_symbol_name(target, "All")
        || matches!(target, Expr::Atom(Atom::Integer(value)) if value.to_bigint() >= BigInt::from(0))
}

#[derive(Clone, Debug, FrameRootTrace)]
struct PreparedLookup {
    outcome: LookupOutcome,
    plan: LookupPlan,
}

#[derive(Clone, Debug, FrameRootTrace)]
enum LookupOutcome {
    Hit(Expr),
    Missing(Expr),
    List(Vec<Self>),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FrameRootTrace)]
struct LookupPlan {
    has_hit: bool,
    has_missing: bool,
}

fn wrap_keyed_result(value: Expr, wrapper: Option<&Expr>) -> Expr {
    match wrapper.cloned() {
        Some(head) => Expr::call(head, vec![value]),
        None => value,
    }
}

fn values_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    if let Some((_, value)) = parse_association_entry(expr) {
        return Ok(wrap_keyed_result(value, wrapper));
    }

    if let Some(keyed) = keyed_entries_from_expr(expr) {
        return Ok(Expr::call(
            system_expr("List"),
            keyed
                .entries
                .into_iter()
                .map(|(_, value)| wrap_keyed_result(value, wrapper))
                .collect(),
        ));
    }

    let Some(items) = list_items(expr) else {
        return Err(expr.clone());
    };

    let mut values = Vec::with_capacity(items.len());
    for item in items {
        match values_result_for_expr(item, wrapper) {
            Ok(value) => values.push(value),
            Err(_) => return Err(expr.clone()),
        }
    }

    Ok(Expr::call(system_expr("List"), values))
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<LookupOutcome>();
    assert_traces::<LookupPlan>();
    assert_traces::<PreparedLookup>();
};
