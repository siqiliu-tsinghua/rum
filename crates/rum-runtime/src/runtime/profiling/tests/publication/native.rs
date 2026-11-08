//! Independent observer: enum fields become decoded components, never key text.
use super::decode::{Key, Row, Snapshot, SymbolParts};
use crate::runtime::{
    Symbol,
    profiling::logical::{AggregateKey, Epoch, synchronous::EventKey},
};
use std::collections::BTreeMap;

pub(super) fn parts(s: Symbol) -> SymbolParts {
    SymbolParts(s.context().into(), s.name().into(), s.identity())
}

pub(super) fn observe(epoch: &Epoch) -> BTreeMap<Key, Row> {
    let mut result = BTreeMap::new();
    let mut module = [0_u64; 4];
    for (key, c) in epoch.rows() {
        let (decoded, vector) = match key {
            AggregateKey::Runtime { phase, source } => {
                if phase == "module.eval.total" {
                    for (sum, value) in
                        module
                            .iter_mut()
                            .zip([c.calls, c.open, c.abandoned, c.cancelled])
                    {
                        *sum += value;
                    }
                    if let Some(src) = source {
                        assert!(
                            result
                                .insert(
                                    Key::CacheSource("eval_module_by_source".into(), parts(src)),
                                    Row {
                                        vector: vec![c.calls],
                                        accounting: [c.open, c.abandoned, c.cancelled],
                                        telemetry: vec![],
                                        durations: vec![c.nanos, c.self_nanos],
                                    }
                                )
                                .is_none()
                        );
                    }
                }
                (
                    Key::Runtime(phase.into(), source.map(parts)),
                    vec![
                        c.calls,
                        c.ok,
                        c.err,
                        c.some,
                        c.none,
                        c.return_err,
                        c.throw_err,
                    ],
                )
            }
            AggregateKey::Builtin {
                symbol,
                source,
                arity,
            } => (
                Key::Builtin(parts(symbol), source.map(parts), arity),
                vec![c.calls, c.ok, c.err],
            ),
            AggregateKey::Definition {
                kind,
                symbol,
                index,
            } => (
                Key::Definition(kind.into(), parts(symbol), index),
                vec![c.calls, c.ok, c.err],
            ),
            AggregateKey::RuleRhs { .. } => panic!("no honest native RHS vector"),
        };
        let mut telemetry = vec![c.input_required, c.pauses, c.segments];
        telemetry.extend(c.abandonment_reasons);
        assert!(
            result
                .insert(
                    decoded,
                    Row {
                        vector,
                        accounting: [c.open, c.abandoned, c.cancelled],
                        telemetry,
                        durations: vec![c.nanos, c.self_nanos]
                    }
                )
                .is_none()
        );
    }
    assert!(
        result
            .insert(
                Key::Counter("eval_module".into()),
                Row {
                    vector: vec![module[0]],
                    accounting: [module[1], module[2], module[3]],
                    telemetry: vec![],
                    durations: vec![0, 0]
                }
            )
            .is_none()
    );
    for (key, v) in epoch.event_rows() {
        let (key, vector) = match key {
            EventKey::Runtime { phase, source } => (
                Key::Runtime(phase.into(), source.map(parts)),
                v.vector.to_vec(),
            ),
            EventKey::Rhs { phase, source } => (
                Key::Rhs(phase.into(), source.map(parts)),
                v.vector[..3].to_vec(),
            ),
            EventKey::Dispatch {
                kind,
                symbol,
                source,
            } => (
                Key::Dispatch(kind.into(), parts(symbol), source.map(parts)),
                v.vector[..6].to_vec(),
            ),
            EventKey::Counter(name) => (Key::Counter(name.into()), vec![v.vector[0]]),
            EventKey::Cache(name) => (Key::Cache(name.into()), vec![v.vector[0]]),
            EventKey::CacheSymbol { event, symbol } => (
                Key::CacheSymbol(event.into(), parts(symbol)),
                vec![v.vector[0]],
            ),
            EventKey::CacheSource { event, symbol } => (
                Key::CacheSource(event.into(), parts(symbol)),
                vec![v.vector[0]],
            ),
        };
        assert!(
            result
                .insert(
                    key,
                    Row {
                        vector,
                        accounting: [0; 3],
                        telemetry: vec![v.input_required],
                        durations: v.durations.to_vec()
                    }
                )
                .is_none()
        );
    }
    for key in [
        Key::Counter("definition_rhs_eval".into()),
        Key::Cache("EVALUATE_ENTRY_COUNT".into()),
        Key::Cache("DOWN_VALUE_SCAN_COUNT".into()),
        Key::Cache("DOWN_VALUE_APPLICATION_COUNT".into()),
        Key::Cache("SUB_VALUE_APPLICATION_COUNT".into()),
        Key::Cache("BUILTIN_APPLICATION_COUNT".into()),
        Key::Cache("EVAL_MODULE_ENTRY_COUNT".into()),
    ] {
        result.entry(key).or_insert(Row {
            vector: vec![0],
            accounting: [0; 3],
            telemetry: vec![],
            durations: vec![0, 0],
        });
    }
    result
}

pub(super) fn parity(snapshot: &Snapshot, before: &BTreeMap<Key, Row>, after: &BTreeMap<Key, Row>) {
    assert_eq!(snapshot.validity, "valid");
    assert_eq!(snapshot.faults, 0);
    assert_eq!(
        snapshot.rows.keys().collect::<Vec<_>>(),
        before.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    for (key, guest) in &snapshot.rows {
        let a = &before[key];
        let b = &after[key];
        assert_eq!(
            (&guest.vector, guest.accounting, &guest.telemetry),
            (&a.vector, a.accounting, &a.telemetry),
            "{key:?}"
        );
        assert_eq!(
            (&guest.vector, guest.accounting, &guest.telemetry),
            (&b.vector, b.accounting, &b.telemetry),
            "{key:?}"
        );
        assert_eq!(guest.durations.len(), a.durations.len());
        for ((value, low), high) in guest.durations.iter().zip(&a.durations).zip(&b.durations) {
            assert!(
                low <= value && value <= high,
                "{key:?}: before={a:?} guest={guest:?} after={b:?}"
            );
        }
    }
}
