//! Process-wide stash of paused logical phases.
//!
//! The hot record gate must not use the session. A registered suspension owner
//! inserts the boundary; otherwise the cold publish path still owns it.
use std::sync::{Arc, Mutex, OnceLock};

use super::ProfileBoundary;

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
type DrainFn = fn(&crate::runtime::profiling::lease::Admitted, &Arc<ProfileBoundary>) -> bool;
#[cfg(not(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
)))]
type DrainFn = fn(&Arc<ProfileBoundary>) -> bool;
#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
type ResetFn = fn(&crate::runtime::profiling::lease::Admitted);
#[cfg(not(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
)))]
type ResetFn = fn();
#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
type DropFn =
    fn(&crate::runtime::profiling::lease::Admitted, &Arc<crate::runtime::RequestInterruptState>);
#[cfg(not(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
)))]
type DropFn = fn(&Arc<crate::runtime::RequestInterruptState>);

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
struct AdmittedBoundary<'a> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    access: &'a crate::runtime::profiling::lease::Admitted,
    boundary: Option<Arc<ProfileBoundary>>,
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl<'a> AdmittedBoundary<'a> {
    fn new(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        access: &'a crate::runtime::profiling::lease::Admitted,
        boundary: Arc<ProfileBoundary>,
    ) -> Self {
        Self {
            access,
            boundary: Some(boundary),
        }
    }

    fn into_arc(mut self) -> Arc<ProfileBoundary> {
        self.boundary
            .take()
            .expect("owned boundary is present until transfer")
    }

    fn release(self) {
        ProfileBoundary::release_admitted(self.access, self.into_arc());
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl std::ops::Deref for AdmittedBoundary<'_> {
    type Target = Arc<ProfileBoundary>;

    fn deref(&self) -> &Self::Target {
        self.boundary
            .as_ref()
            .expect("owned boundary is present until transfer")
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl Drop for AdmittedBoundary<'_> {
    fn drop(&mut self) {
        if let Some(boundary) = self.boundary.take() {
            ProfileBoundary::release_admitted(self.access, boundary);
        }
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
struct AdmittedBoundaries<'a> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    access: &'a crate::runtime::profiling::lease::Admitted,
    boundaries: Vec<Arc<ProfileBoundary>>,
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl<'a> AdmittedBoundaries<'a> {
    fn new(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        access: &'a crate::runtime::profiling::lease::Admitted,
        boundaries: Vec<Arc<ProfileBoundary>>,
    ) -> Self {
        Self { access, boundaries }
    }

    fn is_empty(&self) -> bool {
        self.boundaries.is_empty()
    }

    fn push(&mut self, boundary: Arc<ProfileBoundary>) {
        self.push_owned(AdmittedBoundary::new(self.access, boundary));
    }

    fn push_owned(&mut self, boundary: AdmittedBoundary<'a>) {
        self.boundaries.reserve(1);
        self.boundaries.push(boundary.into_arc());
    }

    fn append(&mut self, other: &mut Vec<Arc<ProfileBoundary>>) {
        self.boundaries.append(other);
    }

    fn into_vec(mut self) -> Vec<Arc<ProfileBoundary>> {
        std::mem::take(&mut self.boundaries)
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl Drop for AdmittedBoundaries<'_> {
    fn drop(&mut self) {
        // Vec order is also the disposal order; no allocation or payload lock
        // is acquired here before forwarding each existing permit.
        for boundary in self.boundaries.drain(..) {
            ProfileBoundary::release_admitted(self.access, boundary);
        }
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
struct AdmittedBoundaryIter<'a> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    access: &'a crate::runtime::profiling::lease::Admitted,
    boundaries: std::vec::IntoIter<Arc<ProfileBoundary>>,
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl<'a> IntoIterator for AdmittedBoundaries<'a> {
    type Item = AdmittedBoundary<'a>;
    type IntoIter = AdmittedBoundaryIter<'a>;

    fn into_iter(mut self) -> Self::IntoIter {
        AdmittedBoundaryIter {
            access: self.access,
            boundaries: std::mem::take(&mut self.boundaries).into_iter(),
        }
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl<'a> Iterator for AdmittedBoundaryIter<'a> {
    type Item = AdmittedBoundary<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.boundaries
            .next()
            .map(|boundary| AdmittedBoundary::new(self.access, boundary))
    }
}

#[cfg(any(
    test,
    feature = "test-builtin-memory-growth",
    feature = "test-continuation-slots"
))]
impl Drop for AdmittedBoundaryIter<'_> {
    fn drop(&mut self) {
        for boundary in self.boundaries.by_ref() {
            ProfileBoundary::release_admitted(self.access, boundary);
        }
    }
}

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
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    set_hooks_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        drain,
        reset,
        drop_owner,
    )
}

pub(in crate::runtime) fn set_hooks_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    drain: DrainFn,
    reset: ResetFn,
    drop_owner: DropFn,
) {
    *drain_hook().lock().unwrap() = Some(drain);
    *reset_hook().lock().unwrap() = Some(reset);
    *drop_hook().lock().unwrap() = Some(drop_owner);
}

pub(in crate::runtime) fn stash_boundary(boundary: Arc<ProfileBoundary>) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    stash_boundary_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        boundary,
    )
}

pub(in crate::runtime) fn stash_boundary_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    boundary: Arc<ProfileBoundary>,
) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let boundary = AdmittedBoundary::new(_access, boundary);
    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    stash().lock().unwrap().push(boundary);
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    {
        let mut queued = stash().lock().unwrap();
        queued.reserve(1);
        queued.push(boundary.into_arc());
    }
}

pub(in crate::runtime) fn take_stash() -> Vec<Arc<ProfileBoundary>> {
    take_selected(|_| true)
}

/// Remove selected owners in one locked step; all others stay in STASH.
pub(in crate::runtime) fn take_selected(
    predicate: impl Fn(&Arc<ProfileBoundary>) -> bool,
) -> Vec<Arc<ProfileBoundary>> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return Vec::new();
    };
    take_selected_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        predicate,
    )
}

pub(in crate::runtime) fn take_belonging(
    control: &Arc<crate::runtime::RequestInterruptState>,
) -> Vec<Arc<ProfileBoundary>> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return Vec::new();
    };
    take_selected_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        |boundary| {
            boundary.belongs_to_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                &access,
                control,
            )
        },
    )
}

pub(in crate::runtime) fn take_selected_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,

    predicate: impl Fn(&Arc<ProfileBoundary>) -> bool,
) -> Vec<Arc<ProfileBoundary>> {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let mut selected = AdmittedBoundaries::new(_access, Vec::new());
    let mut queued = stash().lock().unwrap();
    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    let mut selected = Vec::new();
    queued.retain(|boundary| {
        if predicate(boundary) {
            selected.push(Arc::clone(boundary));
            false
        } else {
            true
        }
    });
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let selected = selected.into_vec();
    selected
}

pub(in crate::runtime) fn restore_stash(keep: Vec<Arc<ProfileBoundary>>) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    restore_stash_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        keep,
    )
}

pub(in crate::runtime) fn restore_stash_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    mut keep: Vec<Arc<ProfileBoundary>>,
) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let mut keep = AdmittedBoundaries::new(_access, std::mem::take(&mut keep));
    if keep.is_empty() {
        return;
    }
    let mut queued = stash().lock().unwrap();
    keep.append(&mut queued);
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let keep = keep.into_vec();
    *queued = keep;
}

/// Insert every stashed boundary the registered owner can accept.
/// Boundaries with no owner yet stay stashed for the cold publish.
pub(in crate::runtime) fn drain_attached() {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    drain_attached_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

pub(in crate::runtime) fn drain_attached_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    drain_matching_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        |_| true,
    );
}

pub(in crate::runtime) fn drain_attached_for(control: &Arc<crate::runtime::RequestInterruptState>) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    drain_attached_for_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        control,
    )
}

pub(in crate::runtime) fn drain_attached_for_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    control: &Arc<crate::runtime::RequestInterruptState>,
) {
    drain_matching_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        |boundary| {
            boundary.belongs_to_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _access,
                control,
            )
        },
    );
}

fn drain_matching_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    matches: impl Fn(&Arc<ProfileBoundary>) -> bool,
) {
    let paused = take_selected_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        matches,
    );
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let paused = AdmittedBoundaries::new(_access, paused);
    if paused.is_empty() {
        return;
    }
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let mut keep = AdmittedBoundaries::new(_access, Vec::new());
    let hook = *drain_hook().lock().unwrap();
    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    let mut keep = Vec::new();
    for boundary in paused {
        let attached = hook.is_some_and(|attach| {
            attach(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _access,
                &boundary,
            )
        });
        if !attached {
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            keep.push_owned(boundary);
            #[cfg(not(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            )))]
            keep.push(boundary);
        } else {
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            boundary.release();
        }
    }
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let keep = keep.into_vec();
    restore_stash_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        keep,
    );
}

pub(in crate::runtime) fn clear_registrations() {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    clear_registrations_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

pub(in crate::runtime) fn clear_registrations_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let hook = *reset_hook().lock().unwrap();
    if let Some(reset) = hook {
        reset(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
        );
    }
}

pub(in crate::runtime) fn clear_stash() {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    clear_stash_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
    )
}

pub(in crate::runtime) fn clear_stash_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
) {
    let selected = take_selected_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        |_| true,
    );
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let selected = AdmittedBoundaries::new(_access, selected);
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    for boundary in selected {
        boundary.release();
    }
    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    drop(selected);
}

pub(in crate::runtime) fn drop_belonging(control: &Arc<crate::runtime::RequestInterruptState>) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(access) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    drop_belonging_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        &access,
        control,
    )
}

pub(in crate::runtime) fn drop_belonging_admitted(
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    _access: &crate::runtime::profiling::lease::Admitted,
    control: &Arc<crate::runtime::RequestInterruptState>,
) {
    let paused = take_selected_admitted(
        #[cfg(any(
            test,
            feature = "test-builtin-memory-growth",
            feature = "test-continuation-slots"
        ))]
        _access,
        |boundary| {
            boundary.belongs_to_admitted(
                #[cfg(any(
                    test,
                    feature = "test-builtin-memory-growth",
                    feature = "test-continuation-slots"
                ))]
                _access,
                control,
            )
        },
    );
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let paused = AdmittedBoundaries::new(_access, paused);
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    for boundary in paused {
        boundary.release();
    }
    #[cfg(not(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    )))]
    drop(paused);
    let hook = *drop_hook().lock().unwrap();
    if let Some(drop_owner) = hook {
        drop_owner(
            #[cfg(any(
                test,
                feature = "test-builtin-memory-growth",
                feature = "test-continuation-slots"
            ))]
            _access,
            control,
        );
    }
}
