//! The native, synchronous, alias and default union, outside the epoch lock.
use super::super::{AggregateKey, Counts, Epoch};
use super::keys;
use std::collections::{BTreeMap, btree_map::Entry};

pub(super) struct Row {
    pub vector: Vec<u64>,
    pub accounting: [u64; 3],
    pub telemetry: Vec<u64>,
    pub durations: Vec<u64>,
}

impl Row {
    fn scalar(c: Counts, durations: [u64; 2]) -> Self {
        Self {
            vector: vec![c.calls],
            accounting: [c.open, c.abandoned, c.cancelled],
            telemetry: Vec::new(),
            durations: durations.to_vec(),
        }
    }

    fn zero() -> Self {
        Self::scalar(Counts::default(), [0, 0])
    }
}

fn insert(rows: &mut BTreeMap<String, Row>, key: String, row: Row, collision: &mut bool) {
    match rows.entry(key) {
        Entry::Vacant(entry) => {
            entry.insert(row);
        }
        Entry::Occupied(_) => {
            *collision = true;
        }
    }
}

pub(super) fn collect(epoch: &Epoch) -> BTreeMap<String, Row> {
    // Each copy releases its own lock. No cross-family atomicity is claimed.
    let native = epoch.rows();
    let events = epoch.events();
    let mut rows = BTreeMap::new();
    let mut aliases = BTreeMap::new();
    let mut module = Counts::default();
    let mut has_module = false;
    let mut collision = false;
    for (key, c) in native {
        if let AggregateKey::Runtime {
            phase: "module.eval.total",
            source,
        } = key
        {
            has_module = true;
            module.calls = module.calls.saturating_add(c.calls);
            module.open = module.open.saturating_add(c.open);
            module.abandoned = module.abandoned.saturating_add(c.abandoned);
            module.cancelled = module.cancelled.saturating_add(c.cancelled);
            if let Some(symbol) = source {
                insert(
                    &mut aliases,
                    format!(
                        "cache-source|eval_module_by_source|{}",
                        keys::symbol(symbol)
                    ),
                    Row::scalar(c, [c.nanos, c.self_nanos]),
                    &mut collision,
                );
            }
        }
        let vector = match key {
            AggregateKey::Runtime { .. } => vec![
                c.calls,
                c.ok,
                c.err,
                c.some,
                c.none,
                c.return_err,
                c.throw_err,
            ],
            AggregateKey::Builtin { .. } | AggregateKey::Definition { .. } => {
                vec![c.calls, c.ok, c.err]
            }
            AggregateKey::RuleRhs { .. } => {
                // Counts has no changed/unchanged fields. This invalid row
                // cannot certify a logical vector; never relabel ok/err.
                collision = true;
                vec![0, 0, 0]
            }
        };
        let mut telemetry = vec![c.input_required, c.pauses, c.segments];
        telemetry.extend(c.abandonment_reasons);
        insert(
            &mut rows,
            keys::native(key),
            Row {
                vector,
                accounting: [c.open, c.abandoned, c.cancelled],
                telemetry,
                durations: vec![c.nanos, c.self_nanos],
            },
            &mut collision,
        );
    }
    for (key, value) in events {
        let (key, width) = keys::event(key);
        insert(
            &mut rows,
            key,
            Row {
                vector: value.vector.into_iter().take(width).collect(),
                accounting: [0; 3],
                telemetry: vec![value.input_required],
                durations: value.durations.to_vec(),
            },
            &mut collision,
        );
    }
    if has_module {
        insert(
            &mut aliases,
            "counter|eval_module".into(),
            Row::scalar(module, [0, 0]),
            &mut collision,
        );
    }
    for (key, row) in aliases {
        insert(&mut rows, key, row, &mut collision);
    }
    for key in [
        "counter|definition_rhs_eval",
        "counter|eval_module",
        "cache|EVALUATE_ENTRY_COUNT",
        "cache|DOWN_VALUE_SCAN_COUNT",
        "cache|DOWN_VALUE_APPLICATION_COUNT",
        "cache|SUB_VALUE_APPLICATION_COUNT",
        "cache|BUILTIN_APPLICATION_COUNT",
        "cache|EVAL_MODULE_ENTRY_COUNT",
    ] {
        rows.entry(key.into()).or_insert_with(Row::zero);
    }
    if collision {
        epoch.fault();
    }
    rows
}
