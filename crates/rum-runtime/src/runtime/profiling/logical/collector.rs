//! Process-wide stash of paused logical phases.
//!
//! The hot record gate must not use the session. A registered suspension owner
//! inserts the boundary; otherwise the cold publish path still owns it.
use std::sync::{Arc, Mutex, OnceLock};

use super::ProfileBoundary;

type DrainFn = fn(&Arc<ProfileBoundary>) -> bool;
type ResetFn = fn();
type DropFn = fn(&Arc<crate::runtime::RequestInterruptState>);

fn stash() -> &'static Mutex<Vec<Arc<ProfileBoundary>>> {
    static STASH: OnceLock<Mutex<Vec<Arc<ProfileBoundary>>>> = OnceLock::new();
    STASH.get_or_init(|| Mutex::new(Vec::new()))
}

fn drain_hook() -> &'static Mutex<Option<DrainFn>> {
    static HOOK: OnceLock<Mutex<Option<DrainFn>>> = OnceLock::new();
    HOOK.get_or_init(|| Mutex::new(None))
}

fn reset_hook() -> &'static Mutex<Option<ResetFn>> {
    static HOOK: OnceLock<Mutex<Option<ResetFn>>> = OnceLock::new();
    HOOK.get_or_init(|| Mutex::new(None))
}

fn drop_hook() -> &'static Mutex<Option<DropFn>> {
    static HOOK: OnceLock<Mutex<Option<DropFn>>> = OnceLock::new();
    HOOK.get_or_init(|| Mutex::new(None))
}

pub(in crate::runtime) fn set_hooks(drain: DrainFn, reset: ResetFn, drop_owner: DropFn) {
    *drain_hook().lock().unwrap() = Some(drain);
    *reset_hook().lock().unwrap() = Some(reset);
    *drop_hook().lock().unwrap() = Some(drop_owner);
}

pub(in crate::runtime) fn stash_boundary(boundary: Arc<ProfileBoundary>) {
    stash().lock().unwrap().push(boundary);
}

pub(in crate::runtime) fn take_stash() -> Vec<Arc<ProfileBoundary>> {
    std::mem::take(&mut *stash().lock().unwrap())
}

pub(in crate::runtime) fn restore_stash(mut keep: Vec<Arc<ProfileBoundary>>) {
    if keep.is_empty() {
        return;
    }
    let mut queued = stash().lock().unwrap();
    keep.append(&mut queued);
    *queued = keep;
}

/// Insert every stashed boundary the registered owner can accept.
/// Boundaries with no owner yet stay stashed for the cold publish.
pub(in crate::runtime) fn drain_attached() {
    drain_matching(|_| true);
}

pub(in crate::runtime) fn drain_attached_for(control: &Arc<crate::runtime::RequestInterruptState>) {
    drain_matching(|boundary| boundary.belongs_to(control));
}

fn drain_matching(matches: impl Fn(&Arc<ProfileBoundary>) -> bool) {
    let paused = take_stash();
    if paused.is_empty() {
        return;
    }
    let hook = *drain_hook().lock().unwrap();
    let mut keep = Vec::new();
    for boundary in paused {
        let attached = matches(&boundary) && hook.is_some_and(|attach| attach(&boundary));
        if !attached {
            keep.push(boundary);
        }
    }
    restore_stash(keep);
}

pub(in crate::runtime) fn clear_registrations() {
    if let Some(reset) = *reset_hook().lock().unwrap() {
        reset();
    }
}

pub(in crate::runtime) fn clear_stash() {
    drop(take_stash());
}

pub(in crate::runtime) fn drop_belonging(control: &Arc<crate::runtime::RequestInterruptState>) {
    let paused = take_stash();
    let mut keep = Vec::new();
    for boundary in paused {
        if boundary.belongs_to(control) {
            drop(boundary);
        } else {
            keep.push(boundary);
        }
    }
    restore_stash(keep);
    if let Some(drop_owner) = *drop_hook().lock().unwrap() {
        drop_owner(control);
    }
}
