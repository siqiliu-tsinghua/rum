mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{
    FrameRootTrace, FrameRootVisitor, PreparedDefinitionLhs, PreparedDefinitionLhsContinuation,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeldAssociationPartMutationResult {
    UpdatedTarget,
    UpdatedPart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, rum_runtime::kernel_api::FrameRootTrace)]
enum AssociationPartMutationOperation {
    AssociateTo,
    KeyDropFrom,
}

impl AssociationPartMutationOperation {
    fn name(self) -> &'static str {
        match self {
            Self::AssociateTo => "AssociateTo",
            Self::KeyDropFrom => "KeyDropFrom",
        }
    }

    fn result(self) -> HeldAssociationPartMutationResult {
        match self {
            Self::AssociateTo => HeldAssociationPartMutationResult::UpdatedTarget,
            Self::KeyDropFrom => HeldAssociationPartMutationResult::UpdatedPart,
        }
    }

    fn mutate(self, expr: Expr, spec: Expr) -> Option<Expr> {
        match self {
            Self::AssociateTo => apply_associate_to_value(expr, spec),
            Self::KeyDropFrom => apply_key_drop_from_value(expr, spec),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExactPartSelectorFailure {
    PartW { part: Expr, target: Expr },
    PartD { part: Expr },
}

fn part_target_selectors_are_exact_leaf_path(selector_exprs: &EvaluatedPartSelectors) -> bool {
    selector_exprs.as_slice().iter().all(|expr| {
        expr_as_signed_isize(expr).is_some()
            || matches!(
                expr,
                Expr::Call { head, args, .. } if is_system_symbol_name(head, "Key") && args.len() == 1
            )
    })
}

fn part_selector_key_expr(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn missing_key_in_exact_part_selector_path(
    expr: &Expr,
    selector_exprs: &[Expr],
) -> Option<(Expr, Expr)> {
    let mut selected = expr.clone();
    for selector in selector_exprs {
        if let Some(index) = expr_as_signed_isize(selector) {
            selected = exact_expr_at_path(&selected, &[index])?;
            continue;
        }

        let key = part_selector_key_expr(selector)?;
        let entries = association_entry_forms_from_expr(&selected)?;
        if let Some(entry) = entries.into_iter().find(|entry| entry.key == key) {
            selected = entry.value;
        } else {
            return Some((key, selected));
        }
    }

    None
}

fn failed_exact_part_selector_path(
    expr: &Expr,
    selector_exprs: &[Expr],
) -> Option<ExactPartSelectorFailure> {
    let mut selected = expr.clone();
    for selector in selector_exprs {
        if let Some(index) = expr_as_signed_isize(selector) {
            if let Some(next) = exact_expr_at_path(&selected, &[index]) {
                selected = next;
                continue;
            }

            if index == 0 {
                return None;
            }

            return Some(match selected {
                Expr::Atom(_) => ExactPartSelectorFailure::PartD {
                    part: Expr::call(system_expr("Part"), vec![selected, selector.clone()]),
                },
                Expr::Call { .. } => ExactPartSelectorFailure::PartW {
                    part: selector.clone(),
                    target: selected,
                },
            });
        }

        let key = part_selector_key_expr(selector)?;
        let entries = association_entry_forms_from_expr(&selected)?;
        if let Some(entry) = entries.into_iter().find(|entry| entry.key == key) {
            selected = entry.value;
        } else {
            return None;
        }
    }

    None
}

fn partw_message(part: &Expr, target: &Expr) -> String {
    format!("Part {part} of {target} does not exist.")
}

fn partd_message(part: &Expr) -> String {
    format!("Part specification {part} is longer than depth of object.")
}

fn mutation_rvalue_message(target: &Expr) -> String {
    format!("{target} is not a variable with a value, so its value cannot be changed.")
}

fn is_non_symbol_atom(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(atom) if !matches!(atom, Atom::Symbol(_)))
}

pub(crate) fn eval_associate_to(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    let [target_expr, spec] = args_slice else {
        if args_slice.len() == 1 {
            return Ok(Expr::call(head.clone(), args_slice.to_vec()));
        }
        return Ok(session.invalid_call_with_message(
            "AssociateTo",
            args_slice,
            two_argument_mutation_arity_tag(args_slice.len()),
            exact_argument_count_message("AssociateTo", args_slice.len(), 2),
        ));
    };

    if parse_association_update_spec_forms(spec).is_none() {
        return Ok(session.invalid_call_with_message(
            "AssociateTo",
            args_slice,
            "invlb",
            invalid_association_update_spec_message(spec),
        ));
    }

    if let Some(target) = session.normalize_symbol_target_expr(target_expr, true) {
        return eval_held_association_mutation(
            session,
            "AssociateTo",
            target_expr.clone(),
            target,
            spec.clone(),
            apply_associate_to_value,
        );
    };

    if let Some((target, selector_exprs)) = normalize_part_mutation_target(session, target_expr) {
        return eval_held_association_part_mutation(
            session,
            AssociationPartMutationOperation::AssociateTo,
            target_expr.clone(),
            target,
            selector_exprs,
            spec.clone(),
        );
    }

    if is_non_symbol_atom(target_expr) {
        return Ok(session.invalid_call_with_message(
            "AssociateTo",
            args_slice,
            "rvalue",
            mutation_rvalue_message(target_expr),
        ));
    }

    session.prepare_definition_lhs_for_consumer(
        target_expr.clone(),
        PreparedDefinitionLhsContinuation::new(
            resume_associate_to_after_target_preparation,
            args.clone(),
        ),
    )
}

fn resume_associate_to_after_target_preparation(
    session: &mut Session,
    prepared: PreparedDefinitionLhs,
    call_args: SharedArgs,
) -> Result<Expr, EvalError> {
    if let Some(target_symbol) =
        session.protected_or_known_prepared_definition_target_symbol(&prepared)
    {
        session.push_message(
            "AssociateTo",
            "wrsym",
            format!("Symbol {target_symbol} is Protected."),
        );
        return Ok(Expr::call(
            system_expr("AssociateTo"),
            call_args.as_ref().to_vec(),
        ));
    }

    let target_result = session.evaluate_prepared_definition_lhs(&prepared);
    rum_runtime::resume_or_suspend!(
        session,
        target_result,
        ok(expr) => finish_associate_to_value(session, call_args, prepared, expr),
        suspend => EvalAssociateToSuspensionFrame {
            args: call_args,
            prepared,
        },
    )
}

fn finish_associate_to_value(
    session: &mut Session,
    call_args: SharedArgs,
    prepared: PreparedDefinitionLhs,
    expr: Expr,
) -> Result<Expr, EvalError> {
    let [target_expr, spec] = call_args.as_ref() else {
        unreachable!("AssociateTo preparation retains its two held arguments")
    };
    let Some(updated) = apply_associate_to_value(expr, spec.clone()) else {
        return Ok(session.invalid_call_with_message(
            "AssociateTo",
            call_args.as_ref(),
            "invak",
            invalid_association_message(&target_expr),
        ));
    };

    store_association_lvalue_update(session, prepared, updated.clone());
    Ok(updated)
}

fn apply_associate_to_value(expr: Expr, spec: Expr) -> Option<Expr> {
    let mut entries = association_entry_forms_from_expr(&expr)?;
    let updates = parse_association_update_spec_forms(&spec)?;

    for update in updates {
        entries.retain(|existing| existing.key != update.key);
        entries.push(update);
    }

    Some(render_association_entry_forms_preserving_heads(entries))
}

pub(crate) fn eval_key_drop_from(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args_slice = args.as_ref();
    let [target_expr, key_spec] = args_slice else {
        if args_slice.len() == 1 {
            return Ok(Expr::call(head.clone(), args_slice.to_vec()));
        }
        return Ok(session.invalid_call_with_message(
            "KeyDropFrom",
            args_slice,
            two_argument_mutation_arity_tag(args_slice.len()),
            exact_argument_count_message("KeyDropFrom", args_slice.len(), 2),
        ));
    };

    if let Some(target) = session.normalize_symbol_target_expr(target_expr, true) {
        return eval_held_association_mutation(
            session,
            "KeyDropFrom",
            target_expr.clone(),
            target,
            key_spec.clone(),
            apply_key_drop_from_value,
        );
    };

    if let Some((target, selector_exprs)) = normalize_part_mutation_target(session, target_expr) {
        return eval_held_association_part_mutation(
            session,
            AssociationPartMutationOperation::KeyDropFrom,
            target_expr.clone(),
            target,
            selector_exprs,
            key_spec.clone(),
        );
    }

    if association_entries_from_expr(target_expr).is_some() {
        let updated = apply_key_drop_from_value(target_expr.clone(), key_spec.clone())
            .expect("association literal should be droppable");
        if let Some(message) = protected_tag_write_message(target_expr) {
            session.push_message("Set", "write", message);
        }
        return Ok(updated);
    }

    if rule_list_entries_from_expr(target_expr).is_some()
        || parse_association_entry_form(target_expr).is_some()
    {
        return Ok(session.invalid_call_with_message(
            "KeyDropFrom",
            args_slice,
            "invak",
            invalid_association_message(target_expr),
        ));
    }

    if is_non_symbol_atom(target_expr) {
        return Ok(session.invalid_call_with_message(
            "KeyDropFrom",
            args_slice,
            "rvalue",
            mutation_rvalue_message(target_expr),
        ));
    }

    session.prepare_definition_lhs_for_consumer(
        target_expr.clone(),
        PreparedDefinitionLhsContinuation::new(
            resume_key_drop_from_after_target_preparation,
            args.clone(),
        ),
    )
}

fn resume_key_drop_from_after_target_preparation(
    session: &mut Session,
    prepared: PreparedDefinitionLhs,
    call_args: SharedArgs,
) -> Result<Expr, EvalError> {
    let target_result = session.evaluate_prepared_definition_lhs(&prepared);
    rum_runtime::resume_or_suspend!(
        session,
        target_result,
        ok(expr) => finish_key_drop_from_value(session, call_args, prepared, expr),
        suspend => EvalKeyDropFromSuspensionFrame {
            args: call_args,
            prepared,
        },
    )
}

fn finish_key_drop_from_value(
    session: &mut Session,
    call_args: SharedArgs,
    prepared: PreparedDefinitionLhs,
    expr: Expr,
) -> Result<Expr, EvalError> {
    let [target_expr, key_spec] = call_args.as_ref() else {
        unreachable!("KeyDropFrom preparation retains its two held arguments")
    };
    let Some(updated) = apply_key_drop_from_value(expr, key_spec.clone()) else {
        return Ok(session.invalid_call_with_message(
            "KeyDropFrom",
            call_args.as_ref(),
            "invak",
            invalid_association_message(&target_expr),
        ));
    };

    if let Some(message) = protected_prepared_definition_target_write_message(session, &prepared) {
        session.push_message("Set", "write", message);
    } else {
        store_association_lvalue_update(session, prepared, updated.clone());
    }
    Ok(updated)
}

fn apply_key_drop_from_value(expr: Expr, key_spec: Expr) -> Option<Expr> {
    let entries = association_entry_forms_from_expr(&expr)?;
    let selected_keys = parse_key_spec(&key_spec)?;

    const KEY_SELECT_INDEX_THRESHOLD: usize = 16;
    let kept = if entries.len() > KEY_SELECT_INDEX_THRESHOLD
        && selected_keys.len() > KEY_SELECT_INDEX_THRESHOLD
    {
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

    Some(render_association_entry_forms_preserving_heads(kept))
}

fn protected_prepared_definition_target_write_message(
    session: &Session,
    prepared: &PreparedDefinitionLhs,
) -> Option<String> {
    session.protected_or_known_prepared_definition_target_symbol(prepared)?;
    protected_tag_write_message(prepared.lhs())
}

fn store_association_lvalue_update(
    session: &mut Session,
    prepared: PreparedDefinitionLhs,
    updated: Expr,
) {
    session.store_prepared_definition_lhs_immediate(prepared, updated);
}

fn eval_held_association_mutation(
    session: &mut Session,
    operator: &'static str,
    target_expr: Expr,
    target: Symbol,
    spec: Expr,
    mutate: fn(Expr, Expr) -> Option<Expr>,
) -> Result<Expr, EvalError> {
    let held_call = vec![target_expr.clone(), spec.clone()];

    if !session.ensure_symbol_not_protected_for_write(operator, &target) {
        return Ok(Expr::call(system_expr(operator), held_call));
    }

    let Some(current) = session.definition_own_value(&target) else {
        session.push_message(
            operator,
            "blnoval",
            missing_immediate_value_message(&target, 1),
        );
        return Ok(Expr::call(system_expr(operator), held_call));
    };

    finish_held_association_mutation(
        session,
        operator,
        target_expr,
        target,
        held_call,
        spec,
        current,
        mutate,
    )
}

fn finish_held_association_mutation(
    session: &mut Session,
    operator: &'static str,
    target_expr: Expr,
    target: Symbol,
    held_call: Vec<Expr>,
    spec: Expr,
    current: Expr,
    mutate: fn(Expr, Expr) -> Option<Expr>,
) -> Result<Expr, EvalError> {
    let Some(updated) = mutate(current, spec) else {
        return Ok(session.invalid_call_with_message(
            operator,
            &held_call,
            "invak",
            invalid_association_message(&target_expr),
        ));
    };

    session.store_immediate_symbol_own_value(target, updated.clone());

    Ok(updated)
}

fn normalize_part_mutation_target(
    session: &Session,
    target_expr: &Expr,
) -> Option<(Symbol, Vec<Expr>)> {
    let Expr::Call { head, args, .. } = target_expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Part") || args.len() < 2 {
        return None;
    }

    let target = session.normalize_symbol_target_expr(&args[0], true)?;
    Some((target, args[1..].to_vec()))
}

fn eval_held_association_part_mutation(
    session: &mut Session,
    operation: AssociationPartMutationOperation,
    target_expr: Expr,
    target: Symbol,
    selector_exprs: Vec<Expr>,
    spec: Expr,
) -> Result<Expr, EvalError> {
    match session.evaluate_part_selectors_owned_for_builtin(selector_exprs) {
        EvaluatedPartSelectorsEvaluation::Complete(result) => {
            finish_held_association_part_mutation(
                session,
                operation,
                target_expr,
                target,
                result?,
                spec,
            )
        }
        EvaluatedPartSelectorsEvaluation::Suspended {
            error,
            continuation,
        } => {
            rum_runtime::resume_or_suspend!(
                insert session,
                FinishHeldAssociationPartSelectorEvaluationSuspensionFrame {
                    operation,
                    target_expr,
                    target,
                    spec,
                    continuation,
                },
            );
            Err(error)
        }
    }
}

fn finish_held_association_part_mutation(
    session: &mut Session,
    operation: AssociationPartMutationOperation,
    target_expr: Expr,
    target: Symbol,
    selector_exprs: EvaluatedPartSelectors,
    spec: Expr,
) -> Result<Expr, EvalError> {
    let operator = operation.name();
    let mut evaluated_target_args = Vec::with_capacity(selector_exprs.as_slice().len() + 1);
    evaluated_target_args.push(Expr::Atom(Atom::Symbol(target)));
    evaluated_target_args.extend_from_slice(selector_exprs.as_slice());
    let evaluated_target_expr = Expr::call(system_expr("Part"), evaluated_target_args);
    let held_call = vec![evaluated_target_expr, spec.clone()];

    if !session.ensure_symbol_not_protected_for_write(operator, &target) {
        return Ok(Expr::call(system_expr(operator), held_call));
    }

    let Some(current) = session.definition_own_value(&target) else {
        return Ok(invalid_missing_part_mutation_target(
            session,
            operation,
            &target_expr,
            &held_call,
        ));
    };

    let Some(paths) = exact_position_paths_from_part_selector_exprs(&current, &selector_exprs)
    else {
        if let Some(offender) = invalid_part_selector_expr(&selector_exprs) {
            let detail =
                format!("The expression {offender} cannot be used as a part specification.");
            session.push_message("Part", "pkspec1", detail.clone());
            session.push_message("Set", "pkspec1", detail);
            return Ok(Expr::call(system_expr(operator), held_call));
        }
        return Ok(invalid_missing_part_mutation_target(
            session,
            operation,
            &target_expr,
            &held_call,
        ));
    };

    if operation == AssociationPartMutationOperation::KeyDropFrom
        && !part_target_selectors_are_exact_leaf_path(&selector_exprs)
    {
        return Ok(session.invalid_call_with_message(
            operator,
            &held_call,
            "invak",
            invalid_association_message(&target_expr),
        ));
    }

    if paths.is_empty() {
        let part_failure = failed_exact_part_selector_path(&current, selector_exprs.as_slice());
        if operation == AssociationPartMutationOperation::AssociateTo {
            if let Some((key, target)) =
                missing_key_in_exact_part_selector_path(&current, selector_exprs.as_slice())
            {
                session.push_message(
                    "AssociateTo",
                    "keynx",
                    format!("Key {key} does not exist in {target}."),
                );
            } else if let Some(ExactPartSelectorFailure::PartW { part, target }) = part_failure {
                session.push_message("AssociateTo", "partw", partw_message(&part, &target));
            }
            return Ok(Expr::call(system_expr(operator), held_call));
        }

        if operation == AssociationPartMutationOperation::KeyDropFrom {
            match part_failure {
                Some(ExactPartSelectorFailure::PartW { part, target }) => {
                    session.push_message("Part", "partw", partw_message(&part, &target));
                }
                Some(ExactPartSelectorFailure::PartD { part }) => {
                    session.push_message("Part", "partd", partd_message(&part));
                }
                None => {}
            }
            return Ok(session.invalid_call_with_message(
                operator,
                &held_call,
                "invak",
                invalid_association_message(&target_expr),
            ));
        }
    }

    let mut replacements = Vec::with_capacity(paths.len());
    for path in paths {
        let Some(part) = exact_expr_at_path(&current, &path) else {
            return Ok(Expr::call(system_expr(operator), held_call));
        };
        let Some(updated_part) = operation.mutate(part.clone(), spec.clone()) else {
            let invalid_expr = match operation.result() {
                HeldAssociationPartMutationResult::UpdatedTarget => part,
                HeldAssociationPartMutationResult::UpdatedPart => target_expr,
            };
            return Ok(session.invalid_call_with_message(
                operator,
                &held_call,
                "invak",
                invalid_association_message(&invalid_expr),
            ));
        };
        replacements.push((path, updated_part));
    }

    let selected_result = replacements
        .first()
        .map(|(_, updated_part)| updated_part.clone())
        .unwrap_or_else(|| Expr::call(system_expr(operator), held_call.clone()));

    let mut updated_target = current;
    for (path, updated_part) in replacements {
        updated_target = replace_exact_path(updated_target, &path, updated_part);
    }

    if !session.ensure_symbol_not_protected_for_write(operator, &target) {
        return Ok(Expr::call(system_expr(operator), held_call));
    }

    session.store_immediate_symbol_own_value(target, updated_target.clone());

    let value = match operation.result() {
        HeldAssociationPartMutationResult::UpdatedTarget => updated_target,
        HeldAssociationPartMutationResult::UpdatedPart => selected_result,
    };
    Ok(value)
}

fn invalid_missing_part_mutation_target(
    session: &mut Session,
    operation: AssociationPartMutationOperation,
    target_expr: &Expr,
    held_call: &[Expr],
) -> Expr {
    match operation {
        AssociationPartMutationOperation::AssociateTo => {
            session.push_message("AssociateTo", "wrsym", "Symbol Part is Protected.");
            Expr::call(system_expr("AssociateTo"), held_call.to_vec())
        }
        AssociationPartMutationOperation::KeyDropFrom => session.invalid_call_with_message(
            "KeyDropFrom",
            held_call,
            "invak",
            invalid_association_message(target_expr),
        ),
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

fn two_argument_mutation_arity_tag(count: usize) -> &'static str {
    if count == 1 { "argr" } else { "argrx" }
}
