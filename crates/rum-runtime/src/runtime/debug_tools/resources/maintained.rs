//! The evaluation stack is maintained only while armed. All other session
//! owners remain walked at every checkpoint, and observation always walks.

use super::memory::expr_memory_proxy_bytes;
use super::*;
use std::mem::size_of;

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
    /// One complete session walk per arming, including roots retained before
    /// entry. Keep only this owner's term; never reuse a stale session total.
    pub(in crate::runtime) fn initialize_accounted_evaluation_stack(&mut self) {
        let bytes = stack_bytes(&self.evaluation_stack);
        self.accounting_route
            .as_mut()
            .expect("armed route")
            .evaluation_stack_bytes = bytes;
        self.memory_proxy_bytes_with_stack(bytes.unwrap_or(usize::MAX));
    }

    pub(super) fn checkpoint_memory_proxy_bytes(&self) -> Option<usize> {
        #[cfg(test)]
        self.assert_evaluation_stack_accounting();
        let bytes = self.accounting_route.as_ref()?.evaluation_stack_bytes?;
        Some(self.memory_proxy_bytes_with_stack(bytes))
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
        let bytes = &mut self
            .accounting_route
            .as_mut()
            .expect("armed route")
            .evaluation_stack_bytes;
        *bytes = bytes
            .and_then(|total| total.checked_add(payload))
            .and_then(|total| total.checked_add(capacity_bytes?));
        payload
    }

    pub(in crate::runtime) fn pop_accounted_evaluation_entry(&mut self, payload: usize) {
        self.evaluation_stack.pop();
        if self.current_memory_limit.is_some() {
            let bytes = &mut self
                .accounting_route
                .as_mut()
                .expect("armed route")
                .evaluation_stack_bytes;
            *bytes = bytes.and_then(|total| total.checked_sub(payload));
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
            self.accounting_route
                .as_mut()
                .expect("armed route")
                .evaluation_stack_bytes = stack_bytes(&entries);
        }
        std::mem::replace(&mut self.evaluation_stack, entries)
    }

    pub(in crate::runtime) fn clear_accounted_evaluation_stack(&mut self) {
        self.evaluation_stack.clear();
        if self.current_memory_limit.is_some() {
            self.accounting_route
                .as_mut()
                .expect("armed route")
                .evaluation_stack_bytes = self
                .evaluation_stack
                .capacity()
                .checked_mul(size_of::<EvaluationStackEntry>());
        }
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
