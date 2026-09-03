mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::HashMap;

mod key_predicates;

pub(crate) use self::key_predicates::{eval_key_free_q, eval_key_member_q};

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

pub(crate) fn eval_lookup(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    if args_slice.is_empty() || args_slice.len() > 4 {
        return Ok(session.invalid_call_with_message(
            "Lookup",
            args_slice,
            "argb",
            lookup_arg_count_message(args_slice.len()),
        ));
    }

    if let [key] = args_slice {
        return Ok(keyed_operator_form_slot_first("Lookup", key.clone()));
    }

    let [expr, key, ..] = args_slice else {
        return Ok(Expr::call_with_shared_args(head.clone(), args.clone()));
    };

    let expr_result = session.evaluate(expr.clone());
    let expr = rum_runtime::resume_or_suspend!(
        session,
        expr_result,
        ok(value) => Ok(value),
        suspend => {
    let head = head.clone();
            let key = key.clone();
            let call_args = args.clone();
    EvalLookupSuspensionFrame { head, key, call_args }
    },
    )?;

    eval_lookup_with_evaluated_data(session, head.clone(), expr, key.clone(), args.clone())
}

fn eval_lookup_with_evaluated_data(
    session: &mut Session,
    head: Expr,
    expr: Expr,
    key: Expr,
    call_args: SharedArgs,
) -> Result<Expr, EvalError> {
    let key_result = session.evaluate(key);
    let key = rum_runtime::resume_or_suspend!(
        session,
        key_result,
        ok(value) => Ok(value),
        suspend => {
    let head = head.clone();
            let expr = expr.clone();
            let call_args = call_args.clone();
    EvalLookupWithEvaluatedDataSuspensionFrame { head, expr, call_args }
    },
    )?;

    finish_eval_lookup(session, head, expr, key, call_args)
}

fn finish_eval_lookup(
    session: &mut Session,
    head: Expr,
    expr: Expr,
    key: Expr,
    call_args: SharedArgs,
) -> Result<Expr, EvalError> {
    let default = call_args.get(2).cloned();
    let wrapper = call_args.get(3).cloned();
    let Some(lookup) = prepare_lookup(&expr, &key) else {
        session.push_message(
            "Lookup",
            "invrl",
            invalid_association_or_rule_list_message(&expr),
        );
        let mut args = vec![expr, key];
        args.extend(call_args.as_ref().iter().skip(2).cloned());
        return Ok(Expr::call(head, args));
    };

    let default = if lookup.plan.has_missing {
        if let Some(default) = default {
            match session.evaluate(default) {
                Ok(default) => Some(default),
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let lookup = FrozenAssociationValue::freeze(lookup);
                    FinishEvalLookupSuspensionFrame { lookup, wrapper }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        }
    } else {
        None
    };

    finish_prepared_lookup(session, lookup, default, wrapper)
}

fn finish_prepared_lookup(
    session: &mut Session,
    lookup: PreparedLookup,
    default: Option<Expr>,
    wrapper: Option<Expr>,
) -> Result<Expr, EvalError> {
    let wrapper = if lookup.plan.has_hit {
        if let Some(wrapper) = wrapper {
            match session.evaluate(wrapper) {
                Ok(wrapper) => Some(wrapper),
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => {
                    let lookup = FrozenAssociationValue::freeze(lookup);
                    FinishPreparedLookupSuspensionFrame { lookup, default }
                    },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(lookup.outcome.into_expr(default.as_ref(), wrapper.as_ref()))
}

fn finish_suspended_prepared_lookup(
    session: &mut Session,
    lookup: FrozenAssociationValue<PreparedLookup>,
    default: Option<Expr>,
    wrapper: Option<Expr>,
) -> Result<Expr, EvalError> {
    let wrapper = if lookup.as_ref().plan.has_hit {
        if let Some(wrapper) = wrapper {
            match session.evaluate(wrapper) {
                Ok(wrapper) => Some(wrapper),
                Err(EvalError::InputRequired { request }) => {
                    return rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(value) => Ok(value),
                        suspend => FinishSuspendedPreparedLookupSuspensionFrame { lookup, default },
                    );
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(lookup
        .as_ref()
        .outcome
        .to_expr(default.as_ref(), wrapper.as_ref()))
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

pub(crate) fn eval_keys(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=2).contains(&args.len()) {
        return Ok(session.invalid_call_with_message(
            "Keys",
            args,
            "argt",
            one_or_two_argument_count_message("Keys", args.len()),
        ));
    }

    let wrapper = args.get(1);
    match keys_result_for_expr(&args[0], wrapper) {
        Ok(value) => Ok(value),
        Err(invalid) => Ok(session.invalid_call_with_message(
            "Keys",
            args,
            "invrl",
            invalid_association_or_rule_list_message(&invalid),
        )),
    }
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

pub(crate) fn eval_key_take(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [key_spec] => Ok(keyed_operator_form_slot_first("KeyTake", key_spec.clone())),
        [expr, key_spec] => {
            match keyed_take_drop_result_for_expr("KeyTake", expr, key_spec, true) {
                Ok(value) => Ok(value),
                Err(error) => Ok(session.invalid_call_with_message(
                    "KeyTake",
                    args,
                    error.tag,
                    invalid_association_or_rule_list_message(&error.invalid),
                )),
            }
        }
        _ => Ok(session.invalid_call_with_message(
            "KeyTake",
            args,
            "argt",
            one_or_two_argument_count_message("KeyTake", args.len()),
        )),
    }
}

pub(crate) fn eval_key_drop(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [key_spec] => Ok(keyed_operator_form_slot_first("KeyDrop", key_spec.clone())),
        [expr, key_spec] => {
            match keyed_take_drop_result_for_expr("KeyDrop", expr, key_spec, false) {
                Ok(value) => Ok(value),
                Err(error) => Ok(session.invalid_call_with_message(
                    "KeyDrop",
                    args,
                    error.tag,
                    invalid_association_or_rule_list_message(&error.invalid),
                )),
            }
        }
        _ => Ok(session.invalid_call_with_message(
            "KeyDrop",
            args,
            "argt",
            one_or_two_argument_count_message("KeyDrop", args.len()),
        )),
    }
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

fn lookup_arg_count_message(count: usize) -> String {
    format!("Lookup called with {count} arguments; between 2 and 4 arguments are expected.")
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

const LOOKUP_INDEX_THRESHOLD: usize = 16;

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

impl LookupOutcome {
    fn into_expr(self, default: Option<&Expr>, wrapper: Option<&Expr>) -> Expr {
        match self {
            Self::Hit(value) => wrap_keyed_result(value, wrapper),
            Self::Missing(key) => default
                .cloned()
                .unwrap_or_else(|| missing_key_absent_expr(key)),
            Self::List(items) => Expr::call(
                system_expr("List"),
                items
                    .into_iter()
                    .map(|item| item.into_expr(default, wrapper))
                    .collect(),
            ),
        }
    }

    fn to_expr(&self, default: Option<&Expr>, wrapper: Option<&Expr>) -> Expr {
        match self {
            Self::Hit(value) => wrap_keyed_result(value.clone(), wrapper),
            Self::Missing(key) => default
                .cloned()
                .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
            Self::List(items) => Expr::call(
                system_expr("List"),
                items
                    .iter()
                    .map(|item| item.to_expr(default, wrapper))
                    .collect(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
struct PreparedLookupEntries {
    entries: Vec<(Expr, Expr)>,
    positions: Option<HashMap<Expr, usize>>,
}

impl PreparedLookupEntries {
    fn new(entries: Vec<(Expr, Expr)>, query_count: usize) -> Self {
        let positions = if entries.len() > LOOKUP_INDEX_THRESHOLD && query_count > 1 {
            let mut positions = HashMap::with_capacity(entries.len());
            for (index, (key, _)) in entries.iter().enumerate() {
                positions.entry(key.clone()).or_insert(index);
            }
            Some(positions)
        } else {
            None
        };
        Self { entries, positions }
    }

    fn value(&self, key: &Expr) -> Option<&Expr> {
        if let Some(positions) = &self.positions {
            return positions.get(key).map(|index| &self.entries[*index].1);
        }

        self.entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, value)| value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FrameRootTrace)]
struct LookupPlan {
    has_hit: bool,
    has_missing: bool,
}

impl LookupPlan {
    fn combine(self, other: Self) -> Self {
        Self {
            has_hit: self.has_hit || other.has_hit,
            has_missing: self.has_missing || other.has_missing,
        }
    }
}

fn prepare_lookup(data: &Expr, key_spec: &Expr) -> Option<PreparedLookup> {
    let query_count = lookup_query_count(key_spec);
    if let Some(entries) = single_rule_or_keyed_entries_from_expr(data) {
        return Some(lookup_entries_outcome(
            &PreparedLookupEntries::new(entries, query_count),
            key_spec,
        ));
    }

    let items = call_items_with_head(data, &system_expr("List"))?;
    let mut plan = LookupPlan::default();
    let mut outcomes = Vec::with_capacity(items.len());
    for item in items {
        let entries = keyed_entries_from_expr(item)?;
        let lookup = lookup_entries_outcome(
            &PreparedLookupEntries::new(entries.entries, query_count),
            key_spec,
        );
        plan = plan.combine(lookup.plan);
        outcomes.push(lookup.outcome);
    }
    Some(PreparedLookup {
        outcome: LookupOutcome::List(outcomes),
        plan,
    })
}

fn wrap_keyed_result(value: Expr, wrapper: Option<&Expr>) -> Expr {
    match wrapper.cloned() {
        Some(head) => Expr::call(head, vec![value]),
        None => value,
    }
}

fn lookup_query_count(key_spec: &Expr) -> usize {
    let Some(keys) = call_items_with_head(key_spec, &system_expr("List")) else {
        return 1;
    };

    keys.iter().fold(0, |count, key| {
        count.saturating_add(lookup_query_count(key))
    })
}

fn lookup_entries_outcome(entries: &PreparedLookupEntries, key_spec: &Expr) -> PreparedLookup {
    if let Some(keys) = call_items_with_head(key_spec, &system_expr("List")) {
        let mut plan = LookupPlan::default();
        let mut outcomes = Vec::with_capacity(keys.len());
        for key in keys {
            let lookup = lookup_entries_outcome(entries, key);
            plan = plan.combine(lookup.plan);
            outcomes.push(lookup.outcome);
        }
        return PreparedLookup {
            outcome: LookupOutcome::List(outcomes),
            plan,
        };
    }

    let lookup_key = lookup_key_expr(key_spec);
    match entries.value(lookup_key) {
        Some(value) => PreparedLookup {
            outcome: LookupOutcome::Hit(value.clone()),
            plan: LookupPlan {
                has_hit: true,
                has_missing: false,
            },
        },
        None => PreparedLookup {
            outcome: LookupOutcome::Missing(lookup_key.clone()),
            plan: LookupPlan {
                has_hit: false,
                has_missing: true,
            },
        },
    }
}

fn lookup_key_expr(key_spec: &Expr) -> &Expr {
    match key_spec {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            &args[0]
        }
        _ => key_spec,
    }
}

fn keys_result_for_expr(expr: &Expr, wrapper: Option<&Expr>) -> Result<Expr, Expr> {
    if let Some((key, _)) = parse_association_entry(expr) {
        return Ok(wrap_keyed_result(key, wrapper));
    }

    if let Some(keyed) = keyed_entries_from_expr(expr) {
        return Ok(Expr::call(
            system_expr("List"),
            keyed
                .entries
                .into_iter()
                .map(|(key, _)| wrap_keyed_result(key, wrapper))
                .collect(),
        ));
    }

    let Some(items) = list_items(expr) else {
        return Err(expr.clone());
    };

    Ok(Expr::call(
        system_expr("List"),
        items
            .iter()
            .map(|item| keys_result_for_expr(item, wrapper))
            .collect::<Result<Vec<_>, _>>()?,
    ))
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeyTakeDropInvalid {
    tag: &'static str,
    invalid: Expr,
}

fn keyed_take_drop_result_for_expr(
    symbol: &'static str,
    expr: &Expr,
    key_spec: &Expr,
    keep_selected: bool,
) -> Result<Expr, KeyTakeDropInvalid> {
    let selected_keys = parse_key_spec(key_spec).ok_or_else(|| KeyTakeDropInvalid {
        tag: "invrl",
        invalid: expr.clone(),
    })?;

    if list_items(expr).is_some_and(|items| items.is_empty()) {
        return Ok(Expr::call(system_expr(heads::structure::LIST), vec![]));
    }

    if let Some(entries) = single_rule_or_keyed_entry_forms_from_expr(expr) {
        const KEY_SELECT_INDEX_THRESHOLD: usize = 16;
        let indexed = entries.len() > KEY_SELECT_INDEX_THRESHOLD
            && selected_keys.len() > KEY_SELECT_INDEX_THRESHOLD;
        let filtered = if keep_selected {
            if indexed {
                // The result keeps selection order, so index the entries by
                // key and walk the selected keys. Overwriting inserts keep
                // the last entry for a duplicated key — what rendering's
                // canonicalization would pick — and `remove` emits a repeated
                // selection once while missing keys fall through silently.
                let mut by_key = HashMap::with_capacity(entries.len());
                for entry in &entries {
                    by_key.insert(entry.key.clone(), entry.clone());
                }
                let mut taken = Vec::new();
                for selected_key in &selected_keys {
                    if let Some(entry) = by_key.remove(selected_key) {
                        taken.push(entry);
                    }
                }
                taken
            } else {
                let mut taken = Vec::new();
                for selected_key in &selected_keys {
                    taken.extend(
                        entries
                            .iter()
                            .filter(|entry| &entry.key == selected_key)
                            .cloned(),
                    );
                }
                taken
            }
        } else if indexed {
            let selected = selected_keys
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>();
            entries
                .into_iter()
                .filter(|entry| !selected.contains(&entry.key))
                .collect()
        } else {
            entries
                .into_iter()
                .filter(|entry| {
                    !selected_keys
                        .iter()
                        .any(|candidate| candidate == &entry.key)
                })
                .collect()
        };

        return Ok(render_association_entry_forms_preserving_heads(filtered));
    }

    let Some(items) = list_items(expr) else {
        return Err(KeyTakeDropInvalid {
            tag: "invrl",
            invalid: expr.clone(),
        });
    };

    let mut results = Vec::with_capacity(items.len());
    for item in items {
        match keyed_take_drop_result_for_expr(symbol, item, key_spec, keep_selected) {
            Ok(value) => results.push(value),
            Err(error) => {
                return Err(KeyTakeDropInvalid {
                    tag: "invru",
                    invalid: if keep_selected {
                        expr.clone()
                    } else {
                        error.invalid
                    },
                });
            }
        }
    }

    Ok(Expr::call(system_expr("List"), results))
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<LookupOutcome>();
    assert_traces::<LookupPlan>();
    assert_traces::<PreparedLookup>();
};
