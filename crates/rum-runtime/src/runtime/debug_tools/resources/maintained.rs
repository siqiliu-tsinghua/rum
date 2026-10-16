//! Armed mutation boundaries maintain all Session owners and their total.
//! Cold entry seeds each owner once; explicit observation always walks.

use super::memory::expr_memory_proxy_bytes;
use super::*;
use std::mem::size_of;

mod diagnostics;
mod owners;
mod scopes;
pub(in crate::runtime) use owners::MemoryOwner;

fn stack_bytes(entries: &Vec<EvaluationStackEntry>) -> Option<usize> {
    entries
        .capacity()
        .checked_mul(size_of::<EvaluationStackEntry>())
        .and_then(|shell| {
            entries.iter().try_fold(shell, |total, entry| {
                total.checked_add(expr_memory_proxy_bytes(&entry.expr))
            })
        })
}

impl Session {
    /// Seed every owner once from actual retained state, including bookkeeping.
    /// An unarmed interval always invalidates the old total for the next entry.
    pub(in crate::runtime) fn initialize_accounted_session_memory(&mut self) {
        let mut total = Some(size_of::<Session>());
        for owner in MemoryOwner::ALL {
            let bytes = if matches!(owner, MemoryOwner::EvaluationStack) {
                stack_bytes(&self.evaluation_stack)
            } else {
                Some(self.walked_memory_owner_bytes(owner))
            };
            let route = self.accounting_route.as_mut().expect("armed route");
            if matches!(owner, MemoryOwner::EvaluationStack) {
                route.evaluation_stack_bytes = bytes;
            } else {
                route.owner_bytes[owner as usize] = bytes;
            }
            total = total.and_then(|total| total.checked_add(bytes?));
        }
        self.accounting_route
            .as_mut()
            .expect("armed route")
            .total_bytes = total;
        if let Some(total) = total {
            self.peak_memory_proxy_bytes
                .set(self.peak_memory_proxy_bytes.get().max(total));
        }
    }

    pub(super) fn checkpoint_memory_proxy_bytes(&self) -> Option<usize> {
        #[cfg(test)]
        self.assert_maintained_memory_accounting();
        let total = self.accounting_route.as_ref()?.total_bytes?;
        self.peak_memory_proxy_bytes
            .set(self.peak_memory_proxy_bytes.get().max(total));
        Some(total)
    }

    pub(in crate::runtime) fn charge_memory_owner(&mut self, owner: MemoryOwner, bytes: usize) {
        if self.current_memory_limit.is_some() {
            let route = self.accounting_route.as_mut().expect("armed route");
            let value = &mut route.owner_bytes[owner as usize];
            *value = value.and_then(|total| total.checked_add(bytes));
            route.total_bytes = route.total_bytes.and_then(|total| {
                (*value)?;
                total.checked_add(bytes)
            });
        }
    }

    pub(in crate::runtime) fn release_memory_owner(&mut self, owner: MemoryOwner, bytes: usize) {
        if self.current_memory_limit.is_some() {
            let route = self.accounting_route.as_mut().expect("armed route");
            let value = &mut route.owner_bytes[owner as usize];
            *value = value.and_then(|total| total.checked_sub(bytes));
            route.total_bytes = route.total_bytes.and_then(|total| {
                (*value)?;
                total.checked_sub(bytes)
            });
        }
    }

    /// Reconcile a replaced owner at its mutation boundary, before guest code
    /// can observe it. Arithmetic invalidity remains sticky until a cold arm.
    pub(in crate::runtime) fn refresh_memory_owner(&mut self, owner: MemoryOwner) {
        if self.current_memory_limit.is_some() && owner.maintained() {
            let prior = self
                .accounting_route
                .as_ref()
                .expect("armed route")
                .owner_bytes[owner as usize];
            if let Some(prior) = prior {
                let current = self.walked_memory_owner_bytes(owner);
                self.release_memory_owner(owner, prior);
                self.charge_memory_owner(owner, current);
            }
        }
    }

    pub(in crate::runtime) fn refresh_definition_memory(&mut self) {
        self.refresh_memory_owner(MemoryOwner::Definitions);
        self.refresh_memory_owner(MemoryOwner::InstalledSystemOwnValues);
        self.refresh_memory_owner(MemoryOwner::InstalledSystemDownValues);
    }

    #[cfg(test)]
    pub(in crate::runtime) fn assert_maintained_memory_accounting(&self) {
        use super::occurrence::{Bytes, session_breakdown};
        self.assert_evaluation_stack_accounting();
        let oracle = session_breakdown(self);
        for owner in MemoryOwner::ALL {
            if owner.maintained() && !matches!(owner, MemoryOwner::EvaluationStack) {
                assert_eq!(
                    self.accounting_route.as_ref().unwrap().owner_bytes[owner as usize]
                        .map_or(Bytes::Invalid, Bytes::Exact),
                    oracle.field(owner.name()),
                    "maintained {} contribution drifted",
                    owner.name()
                );
            }
        }
        let route = self.accounting_route.as_ref().unwrap();
        let total = MemoryOwner::ALL.into_iter().fold(
            Bytes::Exact(size_of::<Session>()),
            |total, owner| {
                let bytes = if matches!(owner, MemoryOwner::EvaluationStack) {
                    route.evaluation_stack_bytes
                } else if owner.maintained() {
                    route.owner_bytes[owner as usize]
                } else {
                    Some(self.walked_memory_owner_bytes(owner))
                };
                total.add(bytes.map_or(Bytes::Invalid, Bytes::Exact))
            },
        );
        assert_eq!(
            total,
            oracle.total(),
            "maintained plus remaining walked owners drifted"
        );
        assert_eq!(total, Bytes::Exact(self.current_memory_proxy_bytes()));
        assert_eq!(
            route.total_bytes.map_or(Bytes::Invalid, Bytes::Exact),
            oracle.total(),
            "maintained session total drifted"
        );
    }

    /// Charge the retained occurrence and actual vector capacity growth. The
    /// armed specialization saves the payload so release needs no second walk.
    pub(in crate::runtime) fn push_accounted_evaluation_entry(
        &mut self,
        entry: EvaluationStackEntry,
    ) -> usize {
        let payload = expr_memory_proxy_bytes(&entry.expr);
        let old_capacity = self.evaluation_stack.capacity();
        self.evaluation_stack.push(entry);
        let capacity_bytes = (self.evaluation_stack.capacity() - old_capacity)
            .checked_mul(size_of::<EvaluationStackEntry>());
        let bytes = self
            .accounting_route
            .as_ref()
            .expect("armed route")
            .evaluation_stack_bytes
            .and_then(|total| total.checked_add(payload))
            .and_then(|total| total.checked_add(capacity_bytes?));
        self.replace_accounted_stack_bytes(bytes);
        payload
    }

    pub(in crate::runtime) fn pop_accounted_evaluation_entry(&mut self, payload: usize) {
        self.evaluation_stack.pop();
        if self.current_memory_limit.is_some() {
            let bytes = self
                .accounting_route
                .as_ref()
                .expect("armed route")
                .evaluation_stack_bytes
                .and_then(|total| total.checked_sub(payload));
            self.replace_accounted_stack_bytes(bytes);
        }
    }

    /// An unarmed caller can unwind through a still-armed InputRequired exit.
    /// Its root was included in the cold-arm walk, so it must now be released.
    #[cold]
    pub(in crate::runtime) fn pop_cold_armed_evaluation_entry(&mut self) {
        let payload =
            expr_memory_proxy_bytes(&self.evaluation_stack.last().expect("active entry").expr);
        self.pop_accounted_evaluation_entry(payload);
    }

    /// Resume, abandonment and rollback transfer actual vector ownership. Only
    /// the incoming stack is reconciled, and only while a limit is active.
    pub(in crate::runtime) fn replace_accounted_evaluation_stack(
        &mut self,
        entries: Vec<EvaluationStackEntry>,
    ) -> Vec<EvaluationStackEntry> {
        if self.current_memory_limit.is_some() {
            self.replace_accounted_stack_bytes(stack_bytes(&entries));
        }
        std::mem::replace(&mut self.evaluation_stack, entries)
    }

    /// Release removed occurrences while retaining and charging the vector shell.
    #[cold]
    pub(in crate::runtime) fn truncate_accounted_evaluation_stack(&mut self, len: usize) {
        if self.current_memory_limit.is_some() {
            while self.evaluation_stack.len() > len {
                self.pop_cold_armed_evaluation_entry();
            }
        } else {
            self.evaluation_stack.truncate(len);
        }
    }

    pub(in crate::runtime) fn clear_accounted_evaluation_stack(&mut self) {
        self.evaluation_stack.clear();
        if self.current_memory_limit.is_some() {
            self.replace_accounted_stack_bytes(
                self.evaluation_stack
                    .capacity()
                    .checked_mul(size_of::<EvaluationStackEntry>()),
            );
        }
    }

    fn replace_accounted_stack_bytes(&mut self, bytes: Option<usize>) {
        let route = self.accounting_route.as_mut().expect("armed route");
        let previous = std::mem::replace(&mut route.evaluation_stack_bytes, bytes);
        route.total_bytes = route
            .total_bytes
            .and_then(|total| total.checked_sub(previous?)?.checked_add(bytes?));
    }

    #[cfg(test)]
    pub(in crate::runtime) fn assert_evaluation_stack_accounting(&self) {
        use super::occurrence::{Bytes, session_breakdown};
        assert!(
            self.current_memory_limit.is_some(),
            "only armed totals are maintained"
        );
        let maintained = self
            .accounting_route
            .as_ref()
            .and_then(|route| route.evaluation_stack_bytes)
            .map_or(Bytes::Invalid, Bytes::Exact);
        assert_eq!(
            maintained,
            session_breakdown(self).field("evaluation_stack"),
            "maintained evaluation-stack contribution drifted"
        );
        assert_eq!(
            maintained,
            Bytes::Exact(self.walked_evaluation_stack_bytes())
        );
    }
}
