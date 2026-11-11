//! The diagnostic queue owns and completes the exact prepared-stage binding.
use super::*;

impl ProfileQueue {
    pub(in crate::runtime::suspension::core) fn bind_prepared_boundary(
        &mut self,
        boundary: &Arc<ProfileBoundary>,
    ) -> bool {
        let Some(epoch) = self.epoch.as_ref().filter(|e| e.is_valid()) else {
            return false;
        };
        if !Arc::ptr_eq(epoch, &boundary.epoch) {
            boundary.epoch.fault();
            return false;
        }
        let Some(consumer) = self.inserted_consumer.take() else {
            epoch.fault();
            return false;
        };
        let Some(arena) = self.arena.as_ref() else {
            return false;
        };
        let root = self.root.lock().unwrap();
        let mut data = arena.data.lock().unwrap();
        let valid = data.belongs(*root, consumer)
            && data
                .cell(consumer)
                .is_some_and(|cell| matches!(cell.kind, Kind::Guest));
        if valid {
            data.bind_prepared(consumer, Arc::clone(boundary));
        }
        drop(data);
        drop(root);
        if !valid {
            epoch.fault();
        }
        valid
    }

    pub(in crate::runtime::suspension::core) fn complete_prepared_boundary(
        &self,
        control: &Arc<crate::runtime::RequestInterruptState>,
    ) {
        let Some(epoch) = self.epoch.as_ref().filter(|e| e.is_valid()) else {
            return;
        };
        let Some(arena) = self.arena.as_ref() else {
            return;
        };
        // The driver keeps the exact consuming frame rooted until resume returns.
        // Record selection is this queue's first generational cell, never a scan.
        let boundary = {
            let root = self.root.lock().unwrap();
            let data = arena.data.lock().unwrap();
            data.first(*root)
                .and_then(|id| data.cell(id))
                .and_then(|cell| {
                    if let Kind::PreparedGuest(boundary) = &cell.kind {
                        Some(Arc::clone(boundary))
                    } else {
                        None
                    }
                })
        };
        let Some(boundary) = boundary else {
            epoch.fault();
            return;
        };
        if !Arc::ptr_eq(epoch, &boundary.epoch) || !boundary.belongs_to(control) {
            epoch.fault();
            return;
        }
        if !matches!(boundary.policy(), Some(Policy::OptionalExpr)) {
            epoch.fault();
            return;
        }
        boundary.complete(crate::runtime::profiling::logical::Outcome::None);
    }
}
