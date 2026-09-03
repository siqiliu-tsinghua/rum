mod resume_frames;
use resume_frames::*;

use super::super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum HeldUpdateReturn {
    Updated,
    Previous(Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum HeldUpdateOperation {
    AddTo,
    SubtractFrom,
    TimesBy,
    DivideBy,
    ApplyTo,
    AppendTo,
    PrependTo,
    Increment,
    Decrement,
    PreIncrement,
    PreDecrement,
}

impl HeldUpdateOperation {
    fn name(self) -> &'static str {
        match self {
            Self::AddTo => "AddTo",
            Self::SubtractFrom => "SubtractFrom",
            Self::TimesBy => "TimesBy",
            Self::DivideBy => "DivideBy",
            Self::ApplyTo => "ApplyTo",
            Self::AppendTo => "AppendTo",
            Self::PrependTo => "PrependTo",
            Self::Increment => "Increment",
            Self::Decrement => "Decrement",
            Self::PreIncrement => "PreIncrement",
            Self::PreDecrement => "PreDecrement",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum HeldUpdateTarget {
    Symbol(Symbol),
    Part {
        symbol: Symbol,
        current_target: Expr,
        selection_shape: Expr,
        distribute_updates: bool,
        paths: Vec<Vec<isize>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
enum SuspendedHeldUpdateTarget {
    Symbol(Symbol),
    Part {
        symbol: Symbol,
        current_target: Expr,
        selection_shape: Expr,
        distribute_updates: bool,
        paths: Arc<[Arc<[isize]>]>,
    },
}

impl SuspendedHeldUpdateTarget {
    fn freeze(target: HeldUpdateTarget) -> Self {
        match target {
            HeldUpdateTarget::Symbol(symbol) => Self::Symbol(symbol),
            HeldUpdateTarget::Part {
                symbol,
                current_target,
                selection_shape,
                distribute_updates,
                paths,
            } => {
                let paths = paths
                    .into_iter()
                    .map(Arc::<[isize]>::from)
                    .collect::<Vec<_>>();
                Self::Part {
                    symbol,
                    current_target,
                    selection_shape,
                    distribute_updates,
                    paths: Arc::from(paths),
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum HeldUpdateCurrent {
    Ready {
        target: HeldUpdateTarget,
        current: Expr,
    },
    Immediate(Expr),
}

impl Session {
    pub(in crate::runtime) fn eval_add_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [_, _] = args else {
            if args.len() == 1 {
                return Ok(self.invalid_call("AddTo", args));
            }
            return Ok(self.invalid_call_with_exact_arg_count("AddTo", args, 2));
        };

        self.eval_held_update_operation(HeldUpdateOperation::AddTo, args)
    }

    pub(in crate::runtime) fn eval_subtract_from(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [_, _] = args else {
            if args.len() == 1 {
                return Ok(self.invalid_call("SubtractFrom", args));
            }
            return Ok(self.invalid_call_with_exact_arg_count("SubtractFrom", args, 2));
        };

        self.eval_held_update_operation(HeldUpdateOperation::SubtractFrom, args)
    }

    pub(in crate::runtime) fn eval_times_by(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [_, _] = args else {
            if args.len() == 1 {
                return Ok(self.invalid_call("TimesBy", args));
            }
            return Ok(self.invalid_call_with_exact_arg_count("TimesBy", args, 2));
        };

        self.eval_held_update_operation(HeldUpdateOperation::TimesBy, args)
    }

    pub(in crate::runtime) fn eval_divide_by(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [_, _] = args else {
            if args.len() == 1 {
                return Ok(self.invalid_call("DivideBy", args));
            }
            return Ok(self.invalid_call_with_exact_arg_count("DivideBy", args, 2));
        };

        self.eval_held_update_operation(HeldUpdateOperation::DivideBy, args)
    }

    pub(in crate::runtime) fn eval_apply_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [_, _] = args else {
            if args.len() == 1 {
                return Ok(self.invalid_call("ApplyTo", args));
            }
            return Ok(self.invalid_call_with_exact_arg_count("ApplyTo", args, 2));
        };

        self.eval_held_update_operation(HeldUpdateOperation::ApplyTo, args)
    }

    pub(in crate::runtime) fn eval_append_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target_expr, item] = args else {
            if args.len() == 1 {
                return Ok(self.invalid_call("AppendTo", args));
            }
            return Ok(self.invalid_call_with_exact_arg_count("AppendTo", args, 2));
        };

        if let Some(updated) = self.try_incremental_append_to_plain_list(target_expr, item)? {
            return Ok(updated);
        }

        self.eval_held_update_operation(HeldUpdateOperation::AppendTo, args)
    }

    pub(in crate::runtime) fn eval_prepend_to(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [_, _] = args else {
            if args.len() == 1 {
                return Ok(self.invalid_call("PrependTo", args));
            }
            return Ok(self.invalid_call_with_exact_arg_count("PrependTo", args, 2));
        };

        self.eval_held_update_operation(HeldUpdateOperation::PrependTo, args)
    }

    pub(in crate::runtime) fn eval_increment(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [_] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Increment", args, 1));
        };

        self.eval_held_update_operation(HeldUpdateOperation::Increment, args)
    }

    pub(in crate::runtime) fn eval_decrement(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [_target_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Decrement", args, 1));
        };
        self.eval_held_update_operation(HeldUpdateOperation::Decrement, args)
    }

    pub(in crate::runtime) fn eval_pre_increment(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [_target_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("PreIncrement", args, 1));
        };
        self.eval_held_update_operation(HeldUpdateOperation::PreIncrement, args)
    }

    pub(in crate::runtime) fn eval_pre_decrement(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [_target_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("PreDecrement", args, 1));
        };
        self.eval_held_update_operation(HeldUpdateOperation::PreDecrement, args)
    }

    fn eval_held_update_operation(
        &mut self,
        operation: HeldUpdateOperation,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let target_expr = &args[0];
        let (target, current) =
            match self.current_held_update_value(operation, target_expr, args)? {
                HeldUpdateCurrent::Ready { target, current } => (target, current),
                HeldUpdateCurrent::Immediate(value) => return Ok(value),
            };
        self.finish_held_update_operation(operation, target, current, args)
    }

    fn current_held_update_value(
        &mut self,
        operation: HeldUpdateOperation,
        target_expr: &Expr,
        args: &[Expr],
    ) -> Result<HeldUpdateCurrent, EvalError> {
        let operator = operation.name();
        if let Some(target) = self.normalize_symbol_target_expr(target_expr, true) {
            return Ok(self.current_symbol_held_update_value(operator, target, args));
        }

        if let Some((target, selector_exprs)) = normalize_part_update_target(target_expr) {
            let evaluated_result = self.evaluate_part_selectors_owned(selector_exprs);
            return crate::resume_or_suspend!(
                self,
                evaluated_result,
                ok(evaluated_selectors) => Ok(self.current_part_held_update_value(
                    operator,
                    target,
                    &evaluated_selectors,
                    args,
                )),
                suspend => FinishHeldPartSelectorEvaluationSuspensionFrame {
                    operation,
                    target,
                    args: SharedArgs::new(args.to_vec()),
                },
            );
        }

        self.push_held_update_rvalue_message(operator, target_expr);
        Ok(HeldUpdateCurrent::Immediate(Expr::call(
            system_expr(operator),
            args.to_vec(),
        )))
    }

    fn current_symbol_held_update_value(
        &mut self,
        operator: &'static str,
        target: Symbol,
        args: &[Expr],
    ) -> HeldUpdateCurrent {
        if !self.ensure_symbol_not_protected_for_write(operator, &target) {
            return HeldUpdateCurrent::Immediate(Expr::call(system_expr(operator), args.to_vec()));
        }

        let Some(current) = self
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.clone())
        else {
            self.push_held_update_rvalue_message(
                operator,
                &Expr::Atom(Atom::Symbol(target.clone())),
            );
            return HeldUpdateCurrent::Immediate(Expr::call(system_expr(operator), args.to_vec()));
        };

        HeldUpdateCurrent::Ready {
            target: HeldUpdateTarget::Symbol(target),
            current,
        }
    }

    fn current_part_held_update_value(
        &mut self,
        operator: &'static str,
        target: Symbol,
        selector_exprs: &EvaluatedPartSelectors,
        args: &[Expr],
    ) -> HeldUpdateCurrent {
        if !self.ensure_symbol_not_protected_for_write(operator, &target) {
            return HeldUpdateCurrent::Immediate(evaluated_part_held_update_call(
                operator,
                target,
                selector_exprs,
                args,
            ));
        }

        let Some(current_target) = self
            .definitions
            .get(&target)
            .and_then(|definition| definition.own_value.clone())
        else {
            self.push_held_update_rvalue_message(
                operator,
                &Expr::Atom(Atom::Symbol(target.clone())),
            );
            return HeldUpdateCurrent::Immediate(evaluated_part_held_update_call(
                operator,
                target,
                selector_exprs,
                args,
            ));
        };

        let Some(paths) =
            exact_position_paths_from_part_target_selector_exprs(&current_target, &selector_exprs)
        else {
            if let Some(offender) = invalid_part_target_selector_expr(&selector_exprs) {
                self.push_invalid_part_target_selector_messages(offender);
            } else {
                self.push_message(
                    operator,
                    "argform",
                    "expected an exact Part target as the first argument",
                );
            }
            return HeldUpdateCurrent::Immediate(evaluated_part_held_update_call(
                operator,
                target,
                selector_exprs,
                args,
            ));
        };

        let Some(current) =
            selected_expr_from_part_target_selector_exprs(self, &current_target, &selector_exprs)
        else {
            return HeldUpdateCurrent::Immediate(evaluated_part_held_update_call(
                operator,
                target,
                selector_exprs,
                args,
            ));
        };

        HeldUpdateCurrent::Ready {
            target: HeldUpdateTarget::Part {
                symbol: target,
                current_target,
                selection_shape: current.clone(),
                distribute_updates: part_selector_exprs_select_bulk(selector_exprs),
                paths,
            },
            current,
        }
    }

    fn finish_suspended_held_part_selector_evaluation(
        &mut self,
        operation: HeldUpdateOperation,
        target: Symbol,
        args: SharedArgs,
        evaluated_selectors: EvaluatedPartSelectors,
    ) -> Result<Expr, EvalError> {
        let operator = operation.name();
        match self.current_part_held_update_value(
            operator,
            target,
            &evaluated_selectors,
            args.as_ref(),
        ) {
            HeldUpdateCurrent::Ready { target, current } => {
                self.finish_held_update_operation(operation, target, current, args.as_ref())
            }
            HeldUpdateCurrent::Immediate(value) => Ok(value),
        }
    }

    fn finish_held_update_operation(
        &mut self,
        operation: HeldUpdateOperation,
        target: HeldUpdateTarget,
        current: Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let operator = operation.name();
        let (update, return_value) = match operation {
            HeldUpdateOperation::AddTo => (
                Expr::call(system_expr("Plus"), vec![current, args[1].clone()]),
                HeldUpdateReturn::Updated,
            ),
            HeldUpdateOperation::SubtractFrom => (
                Expr::call(system_expr("Subtract"), vec![current, args[1].clone()]),
                HeldUpdateReturn::Updated,
            ),
            HeldUpdateOperation::TimesBy => (
                Expr::call(system_expr("Times"), vec![current, args[1].clone()]),
                HeldUpdateReturn::Updated,
            ),
            HeldUpdateOperation::DivideBy => (
                Expr::call(system_expr("Divide"), vec![current, args[1].clone()]),
                HeldUpdateReturn::Updated,
            ),
            HeldUpdateOperation::ApplyTo => (
                Expr::call(args[1].clone(), vec![current]),
                HeldUpdateReturn::Updated,
            ),
            HeldUpdateOperation::AppendTo | HeldUpdateOperation::PrependTo => {
                let Expr::Call {
                    head,
                    args: current_args,
                    ..
                } = current
                else {
                    let held_call = Expr::call(system_expr(operator), args.to_vec());
                    self.push_message(
                        operator,
                        "normal",
                        format!("Nonatomic expression expected at position 1 in {held_call}."),
                    );
                    return Ok(held_call);
                };
                let mut current_args = Expr::unwrap_shared_args(current_args);
                if operation == HeldUpdateOperation::AppendTo {
                    current_args.push(args[1].clone());
                } else {
                    current_args.insert(0, args[1].clone());
                }
                (
                    Expr::call_shared(head, SharedArgs::new(current_args)),
                    HeldUpdateReturn::Updated,
                )
            }
            HeldUpdateOperation::Increment
            | HeldUpdateOperation::Decrement
            | HeldUpdateOperation::PreIncrement
            | HeldUpdateOperation::PreDecrement => {
                let delta = match operation {
                    HeldUpdateOperation::Increment | HeldUpdateOperation::PreIncrement => 1,
                    HeldUpdateOperation::Decrement | HeldUpdateOperation::PreDecrement => -1,
                    _ => unreachable!("matched increment operation"),
                };
                let return_value = match operation {
                    HeldUpdateOperation::Increment | HeldUpdateOperation::Decrement => {
                        HeldUpdateReturn::Previous(current.clone())
                    }
                    HeldUpdateOperation::PreIncrement | HeldUpdateOperation::PreDecrement => {
                        HeldUpdateReturn::Updated
                    }
                    _ => unreachable!("matched increment operation"),
                };
                (
                    Expr::call(
                        system_expr("Plus"),
                        vec![current, Expr::integer(BigInt::from(delta))],
                    ),
                    return_value,
                )
            }
        };

        self.finish_held_update(operator, target, update, return_value)
    }

    fn push_invalid_part_target_selector_messages(&mut self, offender: &Expr) {
        let detail = format!("The expression {offender} cannot be used as a part specification.");
        self.push_message("Part", "pkspec1", detail.clone());
        self.push_message("Set", "pkspec1", detail);
    }

    fn push_held_update_rvalue_message(&mut self, operator: &'static str, target: &Expr) {
        self.push_message(
            operator,
            "rvalue",
            format!("{target} is not a variable with a value, so its value cannot be changed."),
        );
    }

    fn finish_held_update(
        &mut self,
        operator: &'static str,
        target: HeldUpdateTarget,
        update: Expr,
        return_value: HeldUpdateReturn,
    ) -> Result<Expr, EvalError> {
        match self.evaluate(update) {
            Ok(updated) => self.store_held_update(operator, target, updated, return_value),
            Err(EvalError::InputRequired { request }) => {
                let target = SuspendedHeldUpdateTarget::freeze(target);
                crate::resume_or_suspend!(insert self, FinishHeldUpdateSuspensionFrame { operator, target, return_value });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    fn store_held_update(
        &mut self,
        operator: &'static str,
        target: HeldUpdateTarget,
        updated: Expr,
        return_value: HeldUpdateReturn,
    ) -> Result<Expr, EvalError> {
        let owner = match &target {
            HeldUpdateTarget::Symbol(symbol) | HeldUpdateTarget::Part { symbol, .. } => symbol,
        };
        if !self.ensure_symbol_not_protected_for_write(operator, owner) {
            let current = self
                .definitions
                .get(owner)
                .and_then(|definition| definition.own_value.clone())
                .unwrap_or_else(|| Expr::Atom(Atom::Symbol(owner.clone())));
            return Ok(current);
        }

        match target {
            HeldUpdateTarget::Symbol(symbol) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(updated.clone());
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }
            HeldUpdateTarget::Part {
                symbol,
                current_target,
                selection_shape,
                distribute_updates,
                paths,
            } => {
                let mut updated_target = current_target;
                let replacements = bulk_part_update_replacements(
                    &paths,
                    &selection_shape,
                    updated.clone(),
                    distribute_updates,
                );
                for (path, updated_part) in replacements {
                    updated_target = replace_exact_path(updated_target, &path, updated_part);
                }
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(updated_target);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }
        }

        Ok(match return_value {
            HeldUpdateReturn::Updated => updated,
            HeldUpdateReturn::Previous(value) => value,
        })
    }

    fn store_suspended_held_update(
        &mut self,
        operator: &'static str,
        target: SuspendedHeldUpdateTarget,
        updated: Expr,
        return_value: HeldUpdateReturn,
    ) -> Result<Expr, EvalError> {
        let owner = match &target {
            SuspendedHeldUpdateTarget::Symbol(symbol)
            | SuspendedHeldUpdateTarget::Part { symbol, .. } => symbol,
        };
        if !self.ensure_symbol_not_protected_for_write(operator, owner) {
            let current = self
                .definitions
                .get(owner)
                .and_then(|definition| definition.own_value.clone())
                .unwrap_or_else(|| Expr::Atom(Atom::Symbol(*owner)));
            return Ok(current);
        }

        match target {
            SuspendedHeldUpdateTarget::Symbol(symbol) => {
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(updated.clone());
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }
            SuspendedHeldUpdateTarget::Part {
                symbol,
                current_target,
                selection_shape,
                distribute_updates,
                paths,
            } => {
                let mut updated_target = current_target;
                let replacements = part_update_replacement_values(
                    paths.len(),
                    &selection_shape,
                    updated.clone(),
                    distribute_updates,
                );
                for (path, updated_part) in paths.iter().zip(replacements) {
                    updated_target =
                        replace_exact_path(updated_target, path.as_ref(), updated_part);
                }
                let definition = self.definitions.entry(symbol).or_default();
                definition.own_value = Some(updated_target);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&symbol);
            }
        }

        Ok(match return_value {
            HeldUpdateReturn::Updated => updated,
            HeldUpdateReturn::Previous(value) => value,
        })
    }
}

fn normalize_part_update_target(expr: &Expr) -> Option<(Symbol, Vec<Expr>)> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Part") || args.len() < 2 {
        return None;
    }

    let target = expr_as_symbol(&args[0])?.clone();
    Some((target, args[1..].to_vec()))
}

fn evaluated_part_held_update_call(
    operator: &'static str,
    target: Symbol,
    selector_exprs: &EvaluatedPartSelectors,
    args: &[Expr],
) -> Expr {
    let mut part_args = Vec::with_capacity(selector_exprs.len() + 1);
    part_args.push(Expr::Atom(Atom::Symbol(target)));
    part_args.extend_from_slice(selector_exprs.as_slice());

    let mut call_args = args.to_vec();
    call_args[0] = Expr::call(system_expr("Part"), part_args);
    Expr::call(system_expr(operator), call_args)
}

fn bulk_part_update_replacements<'a>(
    paths: &'a [Vec<isize>],
    selection_shape: &Expr,
    updated: Expr,
    distribute_updates: bool,
) -> Vec<(&'a [isize], Expr)> {
    let replacement_values =
        part_update_replacement_values(paths.len(), selection_shape, updated, distribute_updates);

    paths
        .iter()
        .zip(replacement_values)
        .map(|(path, value)| (path.as_slice(), value))
        .collect()
}

fn part_update_replacement_values(
    path_count: usize,
    selection_shape: &Expr,
    updated: Expr,
    distribute_updates: bool,
) -> Vec<Expr> {
    let shape_matched = distribute_updates
        .then(|| replacement_values_matching_selection_shape(selection_shape, &updated))
        .flatten()
        .filter(|values| values.len() == path_count);
    match shape_matched {
        Some(values) => values,
        None => match &updated {
            Expr::Call { head, args, .. }
                if path_count > 1
                    && is_system_symbol_name(head, "List")
                    && args.len() == path_count =>
            {
                args.to_vec()
            }
            _ => vec![updated; path_count],
        },
    }
}

fn part_selector_exprs_select_bulk(selector_exprs: &EvaluatedPartSelectors) -> bool {
    selector_exprs
        .as_slice()
        .iter()
        .any(part_selector_expr_selects_bulk)
}

fn part_selector_expr_selects_bulk(expr: &Expr) -> bool {
    if is_system_symbol_name(expr, "All") {
        return true;
    }

    matches!(
        expr,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "List") || is_system_symbol_name(head, "Span")
    )
}

fn replacement_values_matching_selection_shape(
    selection_shape: &Expr,
    updated: &Expr,
) -> Option<Vec<Expr>> {
    match selection_shape {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => {
            let Expr::Call {
                head: updated_head,
                args: updated_args,
                ..
            } = updated
            else {
                return None;
            };

            if !is_system_symbol_name(updated_head, "List") || args.len() != updated_args.len() {
                return None;
            }

            args.iter()
                .zip(updated_args)
                .map(|(selection_arg, updated_arg)| {
                    replacement_values_matching_selection_shape(selection_arg, updated_arg)
                })
                .collect::<Option<Vec<_>>>()
                .map(|values| values.into_iter().flatten().collect())
        }
        _ => Some(vec![updated.clone()]),
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<HeldUpdateOperation>();
    assert_traces::<HeldUpdateReturn>();
    assert_traces::<HeldUpdateTarget>();
    assert_traces::<SuspendedHeldUpdateTarget>();
};
