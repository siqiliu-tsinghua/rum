//! Reap append deltas never revisit the already harvested prefix.
use super::super::memory::{
    render_monitor_scope_memory_proxy_bytes, render_reap_scope_memory_proxy_bytes,
};
use super::*;

impl Session {
    pub(in crate::runtime) fn charge_reap_occurrence(&mut self, expr: &Expr) {
        self.charge_memory_owner(MemoryOwner::ReapScopes, expr_memory_proxy_bytes(expr));
    }

    pub(in crate::runtime) fn push_accounted_reap_scope(&mut self, scope: ReapScope) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed {
            render_reap_scope_memory_proxy_bytes(&scope) - size_of::<ReapScope>()
        } else {
            0
        };
        let capacity = self.reap_scopes.capacity();
        self.reap_scopes.push(scope);
        if armed {
            self.charge_memory_owner(MemoryOwner::ReapScopes, payload);
            self.charge_memory_owner(
                MemoryOwner::ReapScopes,
                (self.reap_scopes.capacity() - capacity) * size_of::<ReapScope>(),
            );
        }
    }

    pub(in crate::runtime) fn pop_accounted_reap_scope(&mut self) -> Option<ReapScope> {
        let scope = self.reap_scopes.pop()?;
        if self.current_memory_limit.is_some() {
            self.release_memory_owner(
                MemoryOwner::ReapScopes,
                render_reap_scope_memory_proxy_bytes(&scope) - size_of::<ReapScope>(),
            );
        }
        Some(scope)
    }

    pub(in crate::runtime) fn push_accounted_monitor_scope(&mut self, scope: MonitorScope) {
        let armed = self.current_memory_limit.is_some();
        let payload = if armed {
            render_monitor_scope_memory_proxy_bytes(&scope) - size_of::<MonitorScope>()
        } else {
            0
        };
        let capacity = self.monitor_scopes.capacity();
        self.monitor_scopes.push(scope);
        if armed {
            self.charge_memory_owner(MemoryOwner::MonitorScopes, payload);
            self.charge_memory_owner(
                MemoryOwner::MonitorScopes,
                (self.monitor_scopes.capacity() - capacity) * size_of::<MonitorScope>(),
            );
        }
    }

    pub(in crate::runtime) fn pop_accounted_monitor_scope(&mut self) -> Option<MonitorScope> {
        let scope = self.monitor_scopes.pop()?;
        if self.current_memory_limit.is_some() {
            self.release_memory_owner(
                MemoryOwner::MonitorScopes,
                render_monitor_scope_memory_proxy_bytes(&scope) - size_of::<MonitorScope>(),
            );
        }
        Some(scope)
    }
}
