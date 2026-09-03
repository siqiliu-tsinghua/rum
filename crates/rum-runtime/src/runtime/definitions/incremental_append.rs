use super::super::*;
use super::incremental_mutation::DefinitionValueRollback;

#[cfg(test)]
use std::cell::RefCell;

impl Session {
    pub(in crate::runtime) fn try_incremental_append_to_plain_list(
        &mut self,
        target_expr: &Expr,
        item: &Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let Some((target, selector_exprs)) = incremental_append_target(target_expr) else {
            return Ok(None);
        };
        if self.incremental_append_has_dynamic_own_value(&target)
            || self.removed_symbols.contains(&target)
        {
            return Ok(None);
        }
        let Some(plan) = self.incremental_append_plain_list_plan(&target, selector_exprs.as_ref())
        else {
            return Ok(None);
        };

        if !self.incremental_append_environment_is_unobservable(&target)
            || !self.list_has_default_runtime_semantics()
            || self.symbol_is_protected(&target)
            || is_system_symbol_name(item, "Nothing")
            || expr_contains_symbol(item, &target)
        {
            return Ok(None);
        }

        let (item_is_stable, item_bloom) = self.cached_expr_stability_with_bloom(item);
        if !item_is_stable {
            return Ok(None);
        }

        if self.interrupt_armed {
            self.poll_host_interrupt_if_due()?;
        }
        if self.symbol_is_protected(&target) {
            return Ok(None);
        }

        let Some(mut proof) = self.take_call_stability_proof_for_key(plan.leaf_cache_key) else {
            return Ok(None);
        };
        self.evict_call_stability_keys(&plan.ancestor_cache_keys);

        let (updated_head, updated_args, updated) = {
            let Some(definition) = self.definitions.get_mut(&target) else {
                return Ok(None);
            };
            let Some(mut value_guard) = DefinitionValueRollback::take(&mut definition.own_value)
            else {
                return Ok(None);
            };
            let Some((head, args)) = exact_plain_list_leaf_mut(value_guard.value_mut(), &plan.path)
            else {
                return Ok(None);
            };

            {
                let list_items = args.make_mut();
                if list_items.try_reserve(1).is_err() {
                    return Ok(None);
                }
                let append_guard = AppendedItemRollback::push(list_items, item.clone());
                if take_incremental_append_failure(&target) {
                    return Err(EvalError::Cancelled);
                }
                append_guard.commit();
            }

            let updated_head = head.clone();
            let updated_args = args.clone();
            let updated = Expr::call_shared(updated_head.clone(), updated_args.clone());
            value_guard.commit();
            (updated_head, updated_args, updated)
        };

        self.bump_evaluation_epoch_for_symbol(&target);
        proof.bloom.union_assign(item_bloom);
        self.reinsert_call_stability_proof(&updated_head, &updated_args, proof);

        Ok(Some(updated))
    }

    fn incremental_append_plain_list_plan(
        &self,
        target: &Symbol,
        selector_exprs: Option<&EvaluatedPartSelectors>,
    ) -> Option<IncrementalAppendPlan> {
        let current = self
            .definitions
            .get(target)
            .and_then(|definition| definition.own_value.as_ref())?;
        let Some(selector_exprs) = selector_exprs else {
            let Expr::Call { head, args, .. } = current else {
                return None;
            };
            return is_system_symbol_name(head, heads::structure::LIST).then(|| {
                IncrementalAppendPlan {
                    path: Vec::new(),
                    ancestor_cache_keys: Vec::new(),
                    leaf_cache_key: CallStabilityCacheKey::new(head, args),
                }
            });
        };

        let paths = exact_position_paths_from_part_target_selector_exprs(current, selector_exprs)?;
        let [path] = paths.as_slice() else {
            return None;
        };
        if path.len() != selector_exprs.len() {
            return None;
        }

        let mut selected = current;
        let mut normalized_path = Vec::with_capacity(path.len());
        let mut ancestor_cache_keys = Vec::with_capacity(path.len());
        for &component in path {
            let Expr::Call { head, args, .. } = selected else {
                return None;
            };
            if !is_system_symbol_name(head, heads::structure::LIST) {
                return None;
            }
            let index = usize::try_from(component.checked_sub(1)?).ok()?;
            selected = args.get(index)?;
            normalized_path.push(index);
            ancestor_cache_keys.push(CallStabilityCacheKey::new(head, args));
        }

        let Expr::Call { head, args, .. } = selected else {
            return None;
        };
        if !is_system_symbol_name(head, heads::structure::LIST) {
            return None;
        }

        Some(IncrementalAppendPlan {
            path: normalized_path,
            ancestor_cache_keys,
            leaf_cache_key: CallStabilityCacheKey::new(head, args),
        })
    }

    fn list_has_default_runtime_semantics(&self) -> bool {
        let list = Symbol::system(heads::structure::LIST);
        self.attribute_set_for_symbol(&list) == default_system_attribute_set(&list)
            && self
                .definitions
                .get(&list)
                .is_none_or(SymbolDefinition::is_empty)
            && !self.incremental_append_has_dynamic_own_value(&list)
            && !self.removed_symbols.contains(&list)
            && !self.builtins.contains_key(&list)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn fail_next_incremental_append_after_value_take(
        &mut self,
        target: Symbol,
    ) {
        INCREMENTAL_APPEND_FAILURE_TARGET.with(|pending| {
            *pending.borrow_mut() = Some(target);
        });
    }
}

struct IncrementalAppendPlan {
    path: Vec<usize>,
    ancestor_cache_keys: Vec<CallStabilityCacheKey>,
    leaf_cache_key: CallStabilityCacheKey,
}

fn incremental_append_target(
    target_expr: &Expr,
) -> Option<(Symbol, Option<EvaluatedPartSelectors>)> {
    if let Some(target) = expr_as_symbol(target_expr) {
        return Some((*target, None));
    }

    let Expr::Call { head, args, .. } = target_expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Part") || args.len() < 2 {
        return None;
    }

    let target = *expr_as_symbol(&args[0])?;
    let selectors = EvaluatedPartSelectors::from_stable_positive_part_args(args)?;
    Some((target, Some(selectors)))
}

fn exact_plain_list_leaf_mut<'a>(
    root: &'a mut Expr,
    path: &[usize],
) -> Option<(&'a SharedExpr, &'a mut SharedArgs)> {
    let mut selected = root;
    for &index in path {
        let Expr::Call { head, args, .. } = selected else {
            return None;
        };
        if !is_system_symbol_name(head, heads::structure::LIST) {
            return None;
        }
        selected = args.make_mut().get_mut(index)?;
    }

    let Expr::Call { head, args, .. } = selected else {
        return None;
    };
    is_system_symbol_name(head, heads::structure::LIST).then_some((head, args))
}

struct AppendedItemRollback<'a> {
    args: &'a mut Vec<Expr>,
    committed: bool,
}

impl<'a> AppendedItemRollback<'a> {
    fn push(args: &'a mut Vec<Expr>, item: Expr) -> Self {
        args.push(item);
        Self {
            args,
            committed: false,
        }
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for AppendedItemRollback<'_> {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.args.pop();
        }
    }
}

#[cfg(test)]
thread_local! {
    static INCREMENTAL_APPEND_FAILURE_TARGET: RefCell<Option<Symbol>> = const {
        RefCell::new(None)
    };
}

#[cfg(test)]
fn take_incremental_append_failure(target: &Symbol) -> bool {
    INCREMENTAL_APPEND_FAILURE_TARGET.with(|pending| {
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
fn take_incremental_append_failure(_target: &Symbol) -> bool {
    false
}
