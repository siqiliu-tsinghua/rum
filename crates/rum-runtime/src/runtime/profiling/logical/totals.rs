//! Record-local accumulation and one-time child-to-parent settlement.
use super::*;

#[derive(Clone, Copy, Default)]
pub(super) struct Totals {
    pub nanos: u64,
    pub self_nanos: u64,
    pub pauses: u64,
    pub segments: u64,
    // Restoring a context used to start a segment for every ancestor. Preserve
    // that telemetry without restoring those ancestor timers or tokens.
    pub restored: u64,
}
impl Totals {
    pub fn child(&mut self, child: Self) {
        self.nanos = self.nanos.saturating_add(child.nanos);
        self.pauses = self.pauses.saturating_add(child.pauses);
        self.segments = self.segments.saturating_add(child.restored);
        self.restored = self.restored.saturating_add(child.restored);
    }
    pub fn report(self, counts: &mut Counts) {
        counts.nanos = counts.nanos.saturating_add(self.nanos);
        counts.self_nanos = counts.self_nanos.saturating_add(self.self_nanos);
        counts.pauses = counts.pauses.saturating_add(self.pauses);
        counts.segments = counts.segments.saturating_add(self.segments);
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActiveId {
    pub thread: std::thread::ThreadId,
    pub index: usize,
    pub segment: SegmentId,
}

#[derive(Default)]
pub(super) struct State {
    pub closed: bool,
    pub settled: bool,
    pub children: usize,
    pub active: Option<ActiveId>,
    pub totals: Totals,
}

impl RecordPath {
    pub(super) fn is_open(&self) -> bool {
        !self.state.lock().unwrap().closed
    }

    pub(super) fn interval(&self, nanos: u64, restored: bool) {
        let mut state = self.state.lock().unwrap();
        if state.closed {
            return;
        }
        state.totals.nanos = state.totals.nanos.saturating_add(nanos);
        state.totals.self_nanos = state.totals.self_nanos.saturating_add(nanos);
        state.totals.segments += 1;
        state.totals.restored += u64::from(restored);
    }

    pub(super) fn pause(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.closed {
            state.totals.pauses += 1;
        }
    }
}

pub(super) fn terminal(
    path: &RecordPath,
    rows: &mut index::Index<AggregateKey, Counts>,
    outcome: Result<Outcome, Retire>,
) -> bool {
    let mut state = path.state.lock().unwrap();
    if state.closed {
        return false;
    }
    state.closed = true;
    if let Some(counts) = rows.get_mut(path.key) {
        counts.finish(outcome);
    }
    true
}

/// Each record settles once. A parent that terminates before a child retains
/// its budgeted node until that child settles; terminal outcomes are immediate.
/// Cascading settlement is amortized over the records that were opened.
pub(super) fn settle(
    path: Arc<RecordPath>,
    rows: &mut index::Index<AggregateKey, Counts>,
    mut remove: impl FnMut(RecordId),
) {
    let mut next = Some(path);
    while let Some(path) = next {
        let totals = {
            let mut state = path.state.lock().unwrap();
            if !state.closed || state.children != 0 || state.settled {
                break;
            }
            state.settled = true;
            state.totals
        };
        if let Some(counts) = rows.get_mut(path.key) {
            totals.report(counts);
        }
        remove(path.record);
        next = path.parent.clone().into_inner();
        if let Some(parent) = &next {
            let mut state = parent.state.lock().unwrap();
            state.totals.child(totals);
            state.children -= 1;
        }
    }
}
