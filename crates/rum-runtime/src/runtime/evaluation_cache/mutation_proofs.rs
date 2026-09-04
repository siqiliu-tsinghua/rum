use super::profile::*;
use super::stability::call_stability_cache_max_entries;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct CallStabilityProof {
    pub(in crate::runtime) bloom: StabilityBloom,
}

impl Session {
    pub(in crate::runtime) fn has_valid_call_stability_proof_for_key(
        &self,
        key: CallStabilityCacheKey,
    ) -> bool {
        let Some(entry) = self.call_stability_cache.get(&key) else {
            return false;
        };
        CallStabilityCacheKey::new(&entry.head, &entry.args) == key
            && self.call_stability_entry_outcome(entry, &entry.head, &entry.args)
                == CallStabilityProfileOutcome::Hit
            && entry.stable
    }

    pub(in crate::runtime) fn take_call_stability_proof(
        &mut self,
        head: &SharedExpr,
        args: &SharedArgs,
    ) -> Option<CallStabilityProof> {
        let key = CallStabilityCacheKey::new(head, args);
        let outcome = {
            let entry = self.call_stability_cache.get(&key)?;
            if !call_stability_entry_ptr_eq(entry, head, args) {
                return None;
            }
            self.call_stability_entry_outcome(entry, head, args)
        };

        let entry = self
            .call_stability_cache
            .remove(&key)
            .expect("the pointer-matched call-stability entry must still exist");
        if outcome == CallStabilityProfileOutcome::Hit && entry.stable {
            Some(CallStabilityProof { bloom: entry.bloom })
        } else {
            None
        }
    }

    pub(in crate::runtime) fn take_call_stability_proof_for_key(
        &mut self,
        key: CallStabilityCacheKey,
    ) -> Option<CallStabilityProof> {
        let outcome = {
            let entry = self.call_stability_cache.get(&key)?;
            if CallStabilityCacheKey::new(&entry.head, &entry.args) != key {
                return None;
            }
            self.call_stability_entry_outcome(entry, &entry.head, &entry.args)
        };

        let entry = self
            .call_stability_cache
            .remove(&key)
            .expect("the validated call-stability entry must still exist");
        if outcome == CallStabilityProfileOutcome::Hit && entry.stable {
            Some(CallStabilityProof { bloom: entry.bloom })
        } else {
            None
        }
    }

    pub(in crate::runtime) fn reinsert_call_stability_proof(
        &mut self,
        head: &SharedExpr,
        args: &SharedArgs,
        proof: CallStabilityProof,
    ) {
        if self.call_stability_cache.len() > call_stability_cache_max_entries() {
            self.clear_call_stability_cache_for_size_cap();
        }

        record_evaluation_cache_counter(&CALL_STABILITY_INSERT_COUNT, "call_stability_insertions");
        record_call_stability_source_head_insertion(head.as_ref());
        self.call_stability_cache.insert(
            CallStabilityCacheKey::new(head, args),
            CallStabilityCacheEntry {
                generation: self.call_stability_generation,
                head: head.clone(),
                args: args.clone(),
                stable: true,
                bloom: proof.bloom,
                dirty_snapshot: self.stability_dirty_bloom,
            },
        );
    }

    pub(in crate::runtime) fn evict_call_stability_path(
        &mut self,
        path: &[(&SharedExpr, &SharedArgs)],
    ) {
        for &(head, args) in path {
            let key = CallStabilityCacheKey::new(head, args);
            let pointer_matches = self
                .call_stability_cache
                .get(&key)
                .is_some_and(|entry| call_stability_entry_ptr_eq(entry, head, args));
            if pointer_matches {
                self.call_stability_cache.remove(&key);
            }
        }
    }

    pub(in crate::runtime) fn evict_call_stability_keys(&mut self, keys: &[CallStabilityCacheKey]) {
        for key in keys {
            let pointer_matches = self
                .call_stability_cache
                .get(key)
                .is_some_and(|entry| CallStabilityCacheKey::new(&entry.head, &entry.args) == *key);
            if pointer_matches {
                self.call_stability_cache.remove(key);
            }
        }
    }
}

fn call_stability_entry_ptr_eq(
    entry: &CallStabilityCacheEntry,
    head: &SharedExpr,
    args: &SharedArgs,
) -> bool {
    SharedExpr::ptr_eq(&entry.head, head) && SharedArgs::ptr_eq(&entry.args, args)
}
