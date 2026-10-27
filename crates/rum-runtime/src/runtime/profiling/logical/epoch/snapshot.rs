//! Snapshot-only open-chain accumulation; no per-context ancestry traversal.
use super::*;

impl Epoch {
    pub(in crate::runtime) fn assert_quiescent(&self) {
        let (pending_children, terminal_parents, open_records) = {
            let data = self.data.lock().unwrap();
            let mut counts = (0, 0, 0);
            for (_, record) in data.records.iter() {
                let state = record.path.state.lock().unwrap();
                counts.0 += state.children;
                counts.1 += usize::from(state.closed && !state.settled);
                counts.2 += usize::from(!state.closed);
            }
            counts
        };
        assert_eq!(
            (
                pending_children,
                terminal_parents,
                open_records,
                self.contexts.load(Ordering::Relaxed),
                self.faults.load(Ordering::Relaxed),
            ),
            (0, 0, 0, 0, 0),
            "logical teardown: pending children, retained terminal parents, open records, context leases, faults"
        );
    }

    pub(in crate::runtime) fn rows(&self) -> Vec<(AggregateKey, Counts)> {
        let data = self.data.lock().unwrap();
        let mut rows: std::collections::BTreeMap<_, _> =
            data.rows.iter().map(|(k, c)| (k, *c)).collect();
        let mut pending: std::collections::BTreeMap<_, _> = data
            .records
            .iter()
            .map(|(id, r)| (id, (r.path.clone(), r.path.state.lock().unwrap().totals)))
            .collect();
        for (record, nanos) in segments::running(self) {
            if let Some((_, totals)) = pending.get_mut(&record) {
                totals.nanos = totals.nanos.saturating_add(nanos);
                totals.self_nanos = totals.self_nanos.saturating_add(nanos);
            }
        }
        // Parent ids precede child ids. One reverse pass adds every open or
        // terminal-but-unsettled descendant exactly once, only at snapshot.
        while let Some((_, (path, totals))) = pending.pop_last() {
            if let Some(counts) = rows.get_mut(&path.key) {
                totals.report(counts);
            }
            if let Some(parent) = path.parent.as_deref() {
                if let Some((_, parent_totals)) = pending.get_mut(&parent.record) {
                    parent_totals.child(totals);
                }
            }
        }
        rows.into_iter().collect()
    }
}
