use super::super::*;

impl Session {
    pub(in crate::runtime) fn incremental_append_environment_is_unobservable(
        &self,
        target: &Symbol,
    ) -> bool {
        !self.has_up_value_definitions
            && !self.tracing_enabled
            && self.monitor_scopes.is_empty()
            && self.current_time_deadline.is_none()
            && self.current_memory_limit.is_none()
            && self.eval_deadline.is_none()
            && self.pending_input_request.is_none()
            && self.pending_request.is_none()
            && self.pending_suspension.is_none()
            && !self.incremental_append_has_dynamic_own_value(target)
    }

    pub(in crate::runtime) fn incremental_append_has_dynamic_own_value(
        &self,
        symbol: &Symbol,
    ) -> bool {
        self.dynamic_own_values
            .iter()
            .any(|frame| frame.symbol == *symbol)
    }
}

pub(super) struct DefinitionValueRollback<'a> {
    slot: &'a mut Option<Expr>,
    value: Option<Expr>,
}

impl<'a> DefinitionValueRollback<'a> {
    pub(super) fn take(slot: &'a mut Option<Expr>) -> Option<Self> {
        let value = slot.take()?;
        Some(Self {
            slot,
            value: Some(value),
        })
    }

    pub(super) fn value_mut(&mut self) -> &mut Expr {
        self.value
            .as_mut()
            .expect("an active definition rollback guard owns its value")
    }

    pub(super) fn commit(mut self) {
        *self.slot = self.value.take();
    }
}

impl Drop for DefinitionValueRollback<'_> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            *self.slot = Some(value);
        }
    }
}
