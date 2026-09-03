use super::super::*;

#[cfg(test)]
use std::cell::RefCell;

impl Session {
    pub(in crate::runtime) fn association_lvalue_has_valid_direct_proof(
        &self,
        symbol: &Symbol,
    ) -> bool {
        self.association_lvalue_call_cache_key(symbol)
            .is_some_and(|key| self.has_valid_call_stability_proof_for_key(key))
    }

    fn association_lvalue_call_cache_key(&self, symbol: &Symbol) -> Option<CallStabilityCacheKey> {
        let current = self
            .definitions
            .get(symbol)
            .and_then(|definition| definition.own_value.as_ref())?;
        let Expr::Call { head, args, .. } = current else {
            return None;
        };
        is_system_symbol_name(head, "Association").then(|| CallStabilityCacheKey::new(head, args))
    }

    pub(in crate::runtime) fn try_direct_association_rule_update(
        &mut self,
        target: &Symbol,
        key: Expr,
        rhs: &Expr,
    ) -> Result<bool, EvalError> {
        if !self.direct_association_update_environment_is_unobservable(target) {
            return Ok(false);
        }

        let Some(plan) = self.direct_association_rule_plan(target, &key) else {
            return Ok(false);
        };

        let updated_rule = Expr::call(system_expr("Rule"), vec![key, rhs.clone()]);
        let (rule_is_stable, rule_bloom) = self.cached_expr_stability_with_bloom(&updated_rule);
        if !rule_is_stable {
            return Ok(false);
        }

        if self.interrupt_armed {
            self.poll_host_interrupt_if_due()?;
        }
        if self.symbol_is_protected(target) {
            return Ok(false);
        }

        let mut proof = match self.take_call_stability_proof_for_key(plan.cache_key) {
            Some(proof) => proof,
            None => {
                // The cached proof can be lost between the evaluator's
                // seeding and this write — a size-cap clear wipes the whole
                // cache, and without recovery the direct path would stay
                // disabled for good. Rerun the evaluator's own seeding check
                // on the current value: when the data is still stable this
                // re-seeds the exact entry, which the second take below then
                // consumes. A rewritten-dirty Bloom collision is the
                // conservative exception: stability can succeed without
                // caching a proof, and the write then falls back.
                let current = self
                    .definitions
                    .get(target)
                    .and_then(|definition| definition.own_value.clone());
                let Some(current) = current else {
                    return Ok(false);
                };
                let matches_plan = match &current {
                    Expr::Call { head, args, .. } => {
                        is_system_symbol_name(head, "Association")
                            && CallStabilityCacheKey::new(head, args) == plan.cache_key
                    }
                    _ => false,
                };
                if !matches_plan || !self.cached_expr_stability_with_bloom(&current).0 {
                    return Ok(false);
                }
                let Some(proof) = self.take_call_stability_proof_for_key(plan.cache_key) else {
                    return Ok(false);
                };
                proof
            }
        };

        let (updated_head, updated_args) = {
            let Some(definition) = self.definitions.get_mut(target) else {
                return Ok(false);
            };
            let Some(Expr::Call { head, args, .. }) = definition.own_value.as_mut() else {
                return Ok(false);
            };
            if !is_system_symbol_name(head, "Association")
                || CallStabilityCacheKey::new(head, args) != plan.cache_key
            {
                return Ok(false);
            }

            let entries = args.make_mut();
            if plan.position.is_none() && entries.try_reserve(1).is_err() {
                return Ok(false);
            }
            let Some(mutation_guard) =
                AssociationRuleMutationRollback::apply(entries, plan.position, updated_rule)
            else {
                return Ok(false);
            };
            if take_direct_association_update_failure(target) {
                return Err(EvalError::Cancelled);
            }
            mutation_guard.commit();

            let updated_head = head.clone();
            let updated_args = args.clone();
            definition.delayed_own_value = None;
            (updated_head, updated_args)
        };

        if let Some(write_index) = plan.write_index {
            // Re-attach with the appended key recorded in O(1), or the next
            // indexed probe reports a freshly appended key `Missing` and a
            // later write of that key appends a duplicate. Replacing an
            // existing key's value needs no maintenance: the index maps keys
            // to positions, and a replacement leaves both the key set and
            // every position unchanged. The appended key is read back from
            // the mutated tail rather than cloned ahead of the move.
            let appended = plan
                .position
                .is_none()
                .then(|| updated_args.last())
                .flatten()
                .and_then(flat_association_rule_parts)
                .map(|(key, _)| key);
            self.association_lookup_cache.reattach_write_index(
                &updated_args,
                write_index,
                appended,
            );
        }

        self.bump_evaluation_epoch_for_symbol(target);
        proof.bloom.union_assign(rule_bloom);
        self.reinsert_call_stability_proof(&updated_head, &updated_args, proof);
        Ok(true)
    }

    fn direct_association_rule_plan(
        &mut self,
        target: &Symbol,
        key: &Expr,
    ) -> Option<DirectAssociationRulePlan> {
        let current = self
            .definitions
            .get(target)
            .and_then(|definition| definition.own_value.as_ref())?;
        let Expr::Call { head, args, .. } = current else {
            return None;
        };
        if !is_system_symbol_name(head, "Association") {
            return None;
        }
        let cache_key = CallStabilityCacheKey::new(head, args);

        // Locate the key through the read side's index machinery where one
        // applies, holding the index OUT of the cache for the mutation
        // window: `Arc::make_mut` clones while any `Weak` — not only a
        // strong reference — is alive, so an attached index would turn the
        // write into an O(n) copy. The caller re-attaches the handle after
        // the mutation with any appended key recorded. An index exists only
        // after the same whole-list Rule/RuleDelayed validation the scan
        // performs, so an indexed position arrives with every entry already
        // validated — the property the mutation window depends on. The
        // write path builds on first use rather than waiting for a second
        // lookup: a write loop amortizes the build across the writes that
        // follow. The linear scan stays for the lengths the index does not
        // cover: at or below the index threshold, and past the retention
        // bound.
        let (write_index, position) = if args.len() <= ASSOCIATION_LOOKUP_INDEX_THRESHOLD
            || args.len() > ASSOCIATION_LOOKUP_CACHE_MAX_KEYS
        {
            (None, direct_association_scan_position(args, key)?)
        } else if let Some((write_index, position)) =
            self.association_lookup_cache.detach_write_index(args, key)
        {
            (Some(write_index), position)
        } else {
            if !args
                .iter()
                .all(|entry| flat_association_rule_parts(entry).is_some())
            {
                return None;
            }
            let built = self.association_lookup_cache.build_write_index(
                args,
                key,
                args.iter().map(|entry| {
                    flat_association_rule_parts(entry)
                        .expect("validated association rule entry")
                        .0
                }),
            );
            match built {
                Some((write_index, position)) => (Some(write_index), position),
                None => (None, direct_association_scan_position(args, key)?),
            }
        };

        Some(DirectAssociationRulePlan {
            cache_key,
            position,
            write_index,
        })
    }

    fn direct_association_update_environment_is_unobservable(&self, target: &Symbol) -> bool {
        !self.has_up_value_definitions
            && !self.tracing_enabled
            && self.monitor_scopes.is_empty()
            && self.current_time_deadline.is_none()
            && self.current_memory_limit.is_none()
            && self.eval_deadline.is_none()
            && self.pending_input_request.is_none()
            && self.pending_eval_request.is_none()
            && self.pending_suspension.is_none()
            && !self
                .dynamic_own_values
                .iter()
                .any(|frame| frame.symbol == *target)
            && !self.removed_symbols.contains(target)
            && !self.symbol_is_protected(target)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn fail_next_direct_association_update_after_mutation(
        &mut self,
        target: Symbol,
    ) {
        DIRECT_ASSOCIATION_UPDATE_FAILURE_TARGET.with(|pending| {
            *pending.borrow_mut() = Some(target);
        });
    }
}

struct DirectAssociationRulePlan {
    cache_key: CallStabilityCacheKey,
    position: Option<usize>,
    write_index: Option<AssociationWriteIndex>,
}

/// Full-list validation plus first-match position, `None` when any entry is
/// not a two-argument `Rule`/`RuleDelayed` (the caller then declines the
/// direct path). This is the scan the index replaces; it stays for the
/// lengths the index does not cover.
fn direct_association_scan_position(args: &SharedArgs, key: &Expr) -> Option<Option<usize>> {
    let mut position = None;
    for (index, entry) in args.iter().enumerate() {
        let Expr::Call {
            head: rule_head,
            args: rule_args,
            ..
        } = entry
        else {
            return None;
        };
        if !(is_system_symbol_name(rule_head, "Rule")
            || is_system_symbol_name(rule_head, "RuleDelayed"))
            || rule_args.len() != 2
        {
            return None;
        }
        if position.is_none() && rule_args[0] == *key {
            position = Some(index);
        }
    }

    Some(position)
}

enum AssociationRuleMutationUndo {
    Appended,
    Replaced { index: usize, previous: Expr },
}

struct AssociationRuleMutationRollback<'a> {
    entries: &'a mut Vec<Expr>,
    undo: AssociationRuleMutationUndo,
    committed: bool,
}

impl<'a> AssociationRuleMutationRollback<'a> {
    fn apply(
        entries: &'a mut Vec<Expr>,
        position: Option<usize>,
        updated_rule: Expr,
    ) -> Option<Self> {
        let undo = if let Some(index) = position {
            let slot = entries.get_mut(index)?;
            AssociationRuleMutationUndo::Replaced {
                index,
                previous: std::mem::replace(slot, updated_rule),
            }
        } else {
            entries.push(updated_rule);
            AssociationRuleMutationUndo::Appended
        };

        Some(Self {
            entries,
            undo,
            committed: false,
        })
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for AssociationRuleMutationRollback<'_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        match &mut self.undo {
            AssociationRuleMutationUndo::Appended => {
                let _ = self.entries.pop();
            }
            AssociationRuleMutationUndo::Replaced { index, previous } => {
                std::mem::swap(&mut self.entries[*index], previous);
            }
        }
    }
}

#[cfg(test)]
thread_local! {
    static DIRECT_ASSOCIATION_UPDATE_FAILURE_TARGET: RefCell<Option<Symbol>> = const {
        RefCell::new(None)
    };
}

#[cfg(test)]
fn take_direct_association_update_failure(target: &Symbol) -> bool {
    DIRECT_ASSOCIATION_UPDATE_FAILURE_TARGET.with(|pending| {
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
fn take_direct_association_update_failure(_target: &Symbol) -> bool {
    false
}
