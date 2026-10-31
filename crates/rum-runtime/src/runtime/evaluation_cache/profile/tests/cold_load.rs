//! The cold-load `EVALUATION_CACHE_PROFILE_ACTIVE` initializer publishes only
//! over the uninitialized 0 (brief-635 amendment 2). `RumProfileReset[]`
//! stores 1 into it, and a first load must keep that 1.

use super::*;
use crate::runtime::profiling::lease;

#[test]
fn a_cold_load_keeps_the_active_flag_published_after_its_load() {
    let _lease = lease::lease();
    let held = EVALUATION_CACHE_PROFILE_ACTIVE.load(Ordering::Relaxed);
    let mut overwritten = Vec::new();
    for published in [1, 2] {
        EVALUATION_CACHE_PROFILE_ACTIVE.store(published, Ordering::Relaxed);
        let answered = initialize_evaluation_cache_profile_active();
        let after = EVALUATION_CACHE_PROFILE_ACTIVE.load(Ordering::Relaxed);
        EVALUATION_CACHE_PROFILE_ACTIVE.store(held, Ordering::Relaxed);
        if after != published || answered != (published == 1) {
            overwritten.push(format!(
                "published {published}, after {after}, answered {answered}"
            ));
        }
    }
    assert!(
        overwritten.is_empty(),
        "a cold load replaced a value published after its load: {overwritten:#?}"
    );
}
