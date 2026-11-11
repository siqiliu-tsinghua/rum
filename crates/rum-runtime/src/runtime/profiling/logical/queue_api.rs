//! Explicit enabled-only marker insertion, kept with the queue owner.
use crate::runtime::Session;
use std::sync::Arc;

impl Session {
    /// Sampling-position exception: a definition stage's own Option slot.
    /// The pending owner is borrowed continuously from validation to insertion.
    pub(in crate::runtime) fn logical_insert_definition_stage_before_consumer(
        &mut self,
        boundary: Arc<crate::runtime::profiling::logical::ProfileBoundary>,
    ) {
        let Some(owner) = self.pending_suspension.as_mut() else {
            boundary.epoch.fault();
            return;
        };
        if !boundary.belongs_to(&self.request_interrupt_control.borrow()) {
            boundary.epoch.fault();
            return;
        }
        owner.frames.profile_boundary_before_consumer(boundary);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
    }
    /// Enabled-only producer/consumer frontier, stable through queue insertion.
    #[allow(dead_code)] // Explicit insertion API for later phase adoption.
    pub(in crate::runtime) fn logical_frontier(
        &mut self,
    ) -> Option<crate::runtime::profiling::logical::Frontier> {
        let epoch = crate::runtime::profiling::logical::executing_epoch()?;
        let owner = self.pending_suspension.as_mut()?;
        owner.frames.enable_profile(epoch);
        Some(crate::runtime::profiling::logical::Frontier {
            generation: owner.generation,
            after: owner.frames.frontier(),
            control: Arc::clone(&self.request_interrupt_control.borrow()),
            request_generation: self.request_interrupt_control.borrow().logical_generation(),
        })
    }

    pub(in crate::runtime) fn logical_insert_boundary(
        &mut self,
        boundary: Arc<crate::runtime::profiling::logical::ProfileBoundary>,
        frontier: Option<crate::runtime::profiling::logical::Frontier>,
        after_return: bool,
    ) {
        debug_assert!(matches!(
            boundary.policy(),
            Some(
                crate::runtime::profiling::logical::Policy::Expr
                    | crate::runtime::profiling::logical::Policy::ApplicableExpr
                    | crate::runtime::profiling::logical::Policy::Incoming(_)
                    | crate::runtime::profiling::logical::Policy::OptionalExpr
                    | crate::runtime::profiling::logical::Policy::Bool
                    | crate::runtime::profiling::logical::Policy::Prepared
                    | crate::runtime::profiling::logical::Policy::Match
            )
        ));
        let context = self.resume_context();
        let Some(owner) = self.pending_suspension.as_mut() else {
            boundary.epoch.fault();
            return;
        };
        let control = self.request_interrupt_control.borrow();
        if !boundary.belongs_to(&control)
            || frontier.as_ref().is_some_and(|f| {
                f.generation != owner.generation
                    || !Arc::ptr_eq(&f.control, &control)
                    || f.request_generation != control.logical_generation()
            })
        {
            boundary.epoch.fault();
            return;
        }
        drop(control);
        owner.frames.enable_profile(Arc::clone(&boundary.epoch));
        if boundary.has_runtime_key("call.stage.prepare_arguments") {
            owner.frames.bind_prepared_boundary(&boundary);
        }
        let after = frontier
            .map(|f| f.after)
            .unwrap_or_else(|| owner.frames.frontier());
        owner
            .frames
            .profile_boundary(context, boundary, after, after_return);
        self.refresh_memory_owner(
            crate::runtime::debug_tools::resources::MemoryOwner::PendingSuspension,
        );
    }

    #[cfg(test)]
    pub(in crate::runtime) fn logical_frontier_after_consumer(
        &mut self,
    ) -> Option<crate::runtime::profiling::logical::Frontier> {
        let mut frontier = self.logical_frontier()?;
        frontier.after = Some(
            self.pending_suspension
                .as_ref()?
                .frames
                .consumer_frontier()?,
        );
        Some(frontier)
    }
}

impl Session {
    #[cold]
    #[inline(never)]
    pub(in crate::runtime) fn logical_complete_prepared_arguments(&self) {
        let Some(owner) = self.resuming_suspension.as_ref() else {
            return;
        };
        owner
            .frames
            .complete_prepared_boundary(&self.request_interrupt_control.borrow());
    }
}
