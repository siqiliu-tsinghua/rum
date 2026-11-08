//! A one-shot scheduling seam for the reset-after-copy proof; absent in production.
use std::cell::RefCell;

thread_local! {
    static AFTER_COPY: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);
}

pub(in crate::runtime) fn set_after_copy(action: Box<dyn FnOnce()>) {
    AFTER_COPY.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some(action);
    });
}

pub(super) fn after_copy() {
    let action = AFTER_COPY.with(|slot| slot.borrow_mut().take());
    if let Some(action) = action {
        action();
    }
}
