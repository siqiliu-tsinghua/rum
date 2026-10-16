mod resume_frames;

use super::super::*;
use super::incremental_mutation::DefinitionValueRollback;

#[cfg(test)]
use std::cell::RefCell;

impl Session {
    pub(in crate::runtime) fn try_set_positional_part_lvalue(
        &mut self,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let Expr::Call { head, args } = &lhs else {
            return Ok(None);
        };
        if !is_system_symbol_name(head, "Part") {
            return Ok(None);
        }

        let Some((owner, selectors)) = args.split_first() else {
            self.push_positional_part_assignment_argform(
                "expected Part assignment to have a symbol owner and at least one selector",
            );
            return Ok(Some(rhs));
        };
        let Some(owner) = expr_as_symbol(owner).copied() else {
            self.push_positional_part_assignment_argform(format!(
                "expected Part assignment owner to be a symbol, got {owner}",
            ));
            return Ok(Some(rhs));
        };
        if selectors.is_empty() {
            self.push_positional_part_assignment_argform(
                "expected Part assignment to have at least one selector",
            );
            return Ok(Some(rhs));
        }

        let evaluated_result = self.evaluate_part_selectors_owned(selectors.to_vec());
        let witness = slot_witness(&evaluated_result);
        let evaluated_selectors = match evaluated_result {
            Ok(evaluated_selectors) => evaluated_selectors,
            Err(EvalError::InputRequired { request }) => {
                return resume_frames::suspend_positional_part_selectors(
                    self, owner, rhs, witness, request,
                )
                .map(Some);
            }
            Err(error) => return Err(error),
        };
        self.finish_positional_part_assignment(owner, evaluated_selectors, rhs)
            .map(Some)
    }

    fn finish_positional_part_assignment(
        &mut self,
        owner: Symbol,
        evaluated_selectors: EvaluatedPartSelectors,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        // Selector evaluation can reassign, clear, protect, or remove the
        // owner. Re-fetch every target fact only after the final callback.
        if !self.ensure_symbol_not_protected_for_write("Set", &owner) {
            return Ok(rhs);
        }
        if self.removed_symbols.contains(&owner) {
            self.push_positional_part_assignment_argform(format!(
                "Part assignment owner {} has been removed",
                owner
            ));
            return Ok(rhs);
        }
        let Some(current) = self
            .definitions
            .get(&owner)
            .and_then(|definition| definition.own_value.as_ref())
        else {
            self.push_positional_part_assignment_argform(format!(
                "expected Part assignment owner {} to have a current List value",
                owner
            ));
            return Ok(rhs);
        };

        if let Some(selector) = evaluated_selectors
            .as_slice()
            .iter()
            .find(|selector| !is_positive_machine_integer_selector(selector))
        {
            self.push_positional_part_assignment_argform(format!(
                "expected a positive machine-sized integer Part selector, got {selector}",
            ));
            return Ok(rhs);
        }

        let Some(paths) =
            exact_position_paths_from_part_target_selector_exprs(current, &evaluated_selectors)
        else {
            self.push_positional_part_assignment_argform(
                "expected an exact positive integer Part path",
            );
            return Ok(rhs);
        };
        let [path] = paths.as_slice() else {
            let current_for_message = current.clone();
            if !self.push_existing_part_selection_failure_messages(
                &current_for_message,
                &evaluated_selectors,
            ) {
                self.push_positional_part_assignment_argform(
                    "the requested Part path does not exist",
                );
            }
            return Ok(rhs);
        };
        if path.len() != evaluated_selectors.len() {
            self.push_positional_part_assignment_argform(
                "expected one exact position for each Part selector",
            );
            return Ok(rhs);
        }
        if let Err(non_list) = plain_list_ancestors_at_exact_path(current, path) {
            self.push_positional_part_assignment_argform(format!(
                "expected a plain List at every level of the Part path, got {non_list}",
            ));
            return Ok(rhs);
        }

        if self.try_direct_positional_part_assignment(&owner, path, &rhs)? {
            return Ok(rhs);
        }

        // No evaluator call occurs from the structural rebuild through the
        // store. This generic COW-safe rebuild remains the fallback whenever
        // the direct route cannot prove an unobservable environment.
        let Some(current) = self
            .definitions
            .get(&owner)
            .and_then(|definition| definition.own_value.clone())
        else {
            self.push_positional_part_assignment_argform(format!(
                "expected Part assignment owner {} to have a current List value",
                owner
            ));
            return Ok(rhs);
        };
        let updated = replace_exact_path(current, path, rhs.clone());
        self.installed_system_own_values.remove(&owner);
        let definition = self.definitions.entry(owner).or_default();
        definition.own_value = Some(updated);
        definition.delayed_own_value = None;
        self.bump_evaluation_epoch_for_symbol(&owner);
        Ok(rhs)
    }

    pub(in crate::runtime) fn try_direct_positional_part_assignment(
        &mut self,
        target: &Symbol,
        path: &[isize],
        rhs: &Expr,
    ) -> Result<bool, EvalError> {
        if !self.incremental_append_environment_is_unobservable(target) {
            return Ok(false);
        }

        let Some(plan) = self.direct_positional_part_assignment_plan(target, path) else {
            return Ok(false);
        };

        if self.interrupt_armed {
            self.poll_host_interrupt_if_due()?;
        }
        if !self.ensure_symbol_not_protected_for_write("Set", target) {
            return Ok(true);
        }
        if self.removed_symbols.contains(target)
            || !self.direct_positional_part_assignment_plan_still_matches(target, &plan)
            || !self.direct_positional_part_assignment_cache_entries_are_evictable(&plan)
        {
            return Ok(false);
        }

        self.evict_call_stability_keys(&plan.cache_keys);
        let replacement = rhs.clone();
        {
            let Some(definition) = self.definitions.get_mut(target) else {
                return Ok(false);
            };
            let Some(mut value_guard) = DefinitionValueRollback::take(&mut definition.own_value)
            else {
                return Ok(false);
            };
            if !direct_positional_part_assignment_value_matches_plan(value_guard.value_mut(), &plan)
            {
                return Ok(false);
            }
            let Some(slot) =
                exact_plain_list_part_assignment_slot_mut(value_guard.value_mut(), &plan.path)
            else {
                return Ok(false);
            };

            let slot_guard = PartAssignmentSlotRollback::replace(slot, replacement);
            if take_positional_part_assignment_failure(target) {
                return Err(EvalError::Cancelled);
            }
            slot_guard.commit();
            value_guard.commit();
            definition.delayed_own_value = None;
        }

        self.installed_system_own_values.remove(target);
        self.bump_evaluation_epoch_for_symbol(target);
        Ok(true)
    }

    fn direct_positional_part_assignment_plan(
        &self,
        target: &Symbol,
        path: &[isize],
    ) -> Option<DirectPositionalPartAssignmentPlan> {
        let current = self
            .definitions
            .get(target)
            .and_then(|definition| definition.own_value.as_ref())?;
        direct_positional_part_assignment_plan(current, path)
    }

    fn direct_positional_part_assignment_plan_still_matches(
        &self,
        target: &Symbol,
        plan: &DirectPositionalPartAssignmentPlan,
    ) -> bool {
        self.definitions
            .get(target)
            .and_then(|definition| definition.own_value.as_ref())
            .is_some_and(|current| {
                direct_positional_part_assignment_value_matches_plan(current, plan)
            })
    }

    fn direct_positional_part_assignment_cache_entries_are_evictable(
        &self,
        plan: &DirectPositionalPartAssignmentPlan,
    ) -> bool {
        plan.cache_keys.iter().all(|key| {
            self.call_stability_cache
                .get(key)
                .is_none_or(|entry| CallStabilityCacheKey::new(&entry.head, &entry.args) == *key)
        })
    }

    #[cfg(test)]
    pub(in crate::runtime) fn fail_next_positional_part_assignment_after_slot_replacement(
        &mut self,
        target: Symbol,
    ) {
        POSITIONAL_PART_ASSIGNMENT_FAILURE_TARGET.with(|pending| {
            *pending.borrow_mut() = Some(target);
        });
    }

    fn push_positional_part_assignment_argform(&mut self, detail: impl Into<String>) {
        self.push_message("Set", "argform", detail);
    }
}

struct DirectPositionalPartAssignmentPlan {
    path: Vec<usize>,
    cache_keys: Vec<CallStabilityCacheKey>,
}

fn direct_positional_part_assignment_plan(
    root: &Expr,
    path: &[isize],
) -> Option<DirectPositionalPartAssignmentPlan> {
    if path.is_empty() {
        return None;
    }

    let mut selected = root;
    let mut normalized_path = Vec::with_capacity(path.len());
    let mut cache_keys = Vec::with_capacity(path.len());
    for &component in path {
        let Expr::Call { head, args } = selected else {
            return None;
        };
        if !is_system_symbol_name(head, heads::structure::LIST) {
            return None;
        }
        let index = usize::try_from(component.checked_sub(1)?).ok()?;
        selected = args.get(index)?;
        normalized_path.push(index);
        cache_keys.push(CallStabilityCacheKey::new(head, args));
    }

    Some(DirectPositionalPartAssignmentPlan {
        path: normalized_path,
        cache_keys,
    })
}

fn direct_positional_part_assignment_value_matches_plan(
    root: &Expr,
    plan: &DirectPositionalPartAssignmentPlan,
) -> bool {
    if plan.path.len() != plan.cache_keys.len() || plan.path.is_empty() {
        return false;
    }

    let mut selected = root;
    for (&index, &cache_key) in plan.path.iter().zip(&plan.cache_keys) {
        let Expr::Call { head, args } = selected else {
            return false;
        };
        if !is_system_symbol_name(head, heads::structure::LIST)
            || CallStabilityCacheKey::new(head, args) != cache_key
        {
            return false;
        }
        let Some(child) = args.get(index) else {
            return false;
        };
        selected = child;
    }

    true
}

fn exact_plain_list_part_assignment_slot_mut<'a>(
    root: &'a mut Expr,
    path: &[usize],
) -> Option<&'a mut Expr> {
    let mut selected = root;
    for (depth, &index) in path.iter().enumerate() {
        let Expr::Call { head, args } = selected else {
            return None;
        };
        if !is_system_symbol_name(head, heads::structure::LIST) {
            return None;
        }
        let child = args.make_mut().get_mut(index)?;
        if depth + 1 == path.len() {
            return Some(child);
        }
        selected = child;
    }

    None
}

struct PartAssignmentSlotRollback<'a> {
    slot: &'a mut Expr,
    previous: Expr,
    committed: bool,
}

impl<'a> PartAssignmentSlotRollback<'a> {
    fn replace(slot: &'a mut Expr, replacement: Expr) -> Self {
        let previous = std::mem::replace(slot, replacement);
        Self {
            slot,
            previous,
            committed: false,
        }
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for PartAssignmentSlotRollback<'_> {
    fn drop(&mut self) {
        if !self.committed {
            std::mem::swap(self.slot, &mut self.previous);
        }
    }
}

fn is_positive_machine_integer_selector(selector: &Expr) -> bool {
    expr_as_signed_isize(selector).is_some_and(|index| index > 0)
}

fn plain_list_ancestors_at_exact_path(expr: &Expr, path: &[isize]) -> Result<(), Expr> {
    let mut current = expr;
    for component in path {
        let Expr::Call { head, args } = current else {
            return Err(current.clone());
        };
        if !is_system_symbol_name(head, heads::structure::LIST) {
            return Err(current.clone());
        }

        let Some(index) = component
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
        else {
            return Err(current.clone());
        };
        let Some(child) = args.get(index) else {
            return Err(current.clone());
        };
        current = child;
    }

    Ok(())
}

#[cfg(test)]
thread_local! {
    static POSITIONAL_PART_ASSIGNMENT_FAILURE_TARGET: RefCell<Option<Symbol>> = const {
        RefCell::new(None)
    };
}

#[cfg(test)]
fn take_positional_part_assignment_failure(target: &Symbol) -> bool {
    POSITIONAL_PART_ASSIGNMENT_FAILURE_TARGET.with(|pending| {
        let mut pending = pending.borrow_mut();
        if pending
            .as_ref()
            .is_some_and(|candidate| candidate == target)
        {
            pending.take();
            true
        } else {
            false
        }
    })
}

#[cfg(not(test))]
#[inline]
fn take_positional_part_assignment_failure(_target: &Symbol) -> bool {
    false
}
